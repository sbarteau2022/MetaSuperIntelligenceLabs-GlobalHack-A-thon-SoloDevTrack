# Runner + receipt patch (A39) — staged, NOT applied

Do not apply while Stewart's 1:37pm control run is active. Apply after it
finishes, then `cargo check --release --bin ablation_runner` on the Mac
(no rustc is available in this VM, so the Rust code below is uncompiled
here; the Python mirror of the judge is tested, the Rust port mirrors it
test-for-test, including the sha256 known vector).

## 1. Receipt fields — continuous_active_mind.rs, AgenticTaskCompletionReceipt

Add, with defaults so existing constructors compile (add `..Default` or
explicit zeros at the two dummy-receipt sites, runtime_model.rs ~1097 and
~1670, and at the `decoded_output_text: String::new()` site ~475):

```rust
pub attempt_count: usize,            // 1 normally; 2 when fallback re-executes
pub backtrack_count: usize,          // backtrack_and_reiterate calls in this task
pub prompt_op_count: usize,          // ingress_ops.len() — OPS, not BPE tokens
pub generated_token_count: usize,    // expanded_tokens.len() — TRUE token count
pub generated_op_count: usize,       // emitted_ops.len() — kept, never called tokens
pub gate_rejection_count: Option<usize>, // Some(n) only if gate counters are wired;
                                      // None = unavailable, never 0-filled
pub peak_rss_bytes_sampled: Option<u64>, // runner-side ps RSS max, see §3
pub warm_up: bool,                   // true for warm-up executions only
```

Populate in `execute_task_in_motion` return sites and in the subagent
path (~2791): `attempt_count = if fallback_to_parent { 2 } else { 1 }`.
`generated_token_count` / `generated_op_count` are set wherever
`expanded_tokens` / `emitted_ops` are set today (runtime_model.rs ~627).

### What is NOT added, and why
- `tool_call_count` is not added. This transformer has no external tool
  calls. The countable analogues are gate evaluations and backtracks,
  named as such. A zero-filled tool_call_count would be a fake metric.
- `peak_vram_bytes` is not added. On unified Metal there is no separate
  discrete-VRAM counter. J5 is redefined in the amendment as sampled RSS
  plus, separately, existing metal kernel telemetry (`bytes_saved_from_heap_alloc`).

### Gate counters (engine TODO, same PR, not optional for J2-gate)
`predict_gate.rs` / `TrajectoryMatchGate` chassis must expose a rejection
counter per episode (rejections + BoundaryHandoffGate escalations). Until
that counter exists, `gate_rejection_count` stays `None` and the analysis
reports "gate count unavailable". Do not backfill it with
`superposition_interventions`: interventions are holds inside the
superposition engine, not gate rejections, and conflating them is how the
old retries metric was contaminated.

## 2. Runner — orchestrator/src/bin/ablation_runner.rs

### a) Remove warm-up from measured runs
Add flags `--no-warmup`, `--vault-copy <path>`, `--no-persist`.
Default for A39: `--no-warmup` on. If warm-up is ever enabled for
debugging, its executions are written to a separate file with
`warm_up: true` and are never merged into the measured file.
The current code runs `execute_active_transformer_task(..., 128, true)`
as a warm-up with `learn_enabled=true` before every measured task — that
is the warm-up seeding that drove up intervention counts and crystallized
task trajectories into the map. Delete that call for A39 runs; do not
just stop counting it, because its map side-effects are the contamination.

### b) Replace the judge (lines ~299 control, ~400 treatment)
Replace, in both arms:
```rust
receipt.decoded_output_text.to_lowercase().contains(&expected.to_lowercase())
```
with a call into `judge.rs` using the task's preregistered judge from the
frozen corpus (the corpus struct gains a `judge` field: check, target,
case_sensitive). Store the verdict, never recompute it in analysis
without checking agreement:
```rust
let verdict = judge::judge_task(&task, &decoded_text); // Pass|Fail|Error
let success = verdict == Verdict::Pass && !fallback_to_parent;
```
`fallback_to_parent` comes from the subagent receipt (treatment) and is
`false` for control. See the amendment for why fallback can never pass.

### c) AblationResult — replace the snippet with the full receipt
```rust
pub struct AblationResult {
    // existing fields, except:
    //   tokens_generated -> generated_token_count (expanded_tokens.len())
    //   retries          -> retry_count ((attempt_count - 1) + backtrack_count)
    pub judge_version: &'static str,       // "deterministic-judge-1.0"
    pub judge_mode: String,                // "rubric" for A39 tasks
    pub judge_verdict: String,             // "pass"|"fail"|"error"
    pub judge_checks: Vec<CheckResult>,    // serialized
    pub decoded_output_text: String,       // FULL text, not 200 chars
    pub decoded_sha256: String,
    pub fallback_to_parent: bool,
    pub warm_up: bool,                     // always false in a measured file
    pub attempt_count: usize,
    pub backtrack_count: usize,
    pub prompt_op_count: usize,
    pub generated_token_count: usize,
    pub generated_op_count: usize,
    pub superposition_interventions: usize, // kept, labelled, NOT retries
    pub gate_rejection_count: Option<usize>,
    pub peak_rss_bytes_sampled: Option<u64>,
    pub vault_map_hash_before: String,
    pub vault_map_hash_after: String,
}
```
`output_snippet` is deleted. A run file without full decoded text cannot
be re-judged, and a run that cannot be re-judged cannot earn a claim.

## 3. Peak RSS sampler (std only, macOS)
In the runner, before and after each task, run
`/bin/ps -o rss= -p <own pid>`, parse KB, keep the max across the task.
Record it as `peak_rss_bytes_sampled` (KB * 1024). Label it "sampled at
task boundaries" in the report — it is not a continuous peak, and the
report must not call it VRAM.

## 4. Verify after applying
1. `cargo check --release --bin ablation_runner` — 0 errors.
2. `cargo test --release --bin ablation_runner` if tests are added, or
   at minimum the `judge.rs` unit tests pass (`cargo test judge`).
3. Run the probe (probe_corpus.py) on a vault copy, not the live vault.
4. Only then run A39 on the frozen corpus.
