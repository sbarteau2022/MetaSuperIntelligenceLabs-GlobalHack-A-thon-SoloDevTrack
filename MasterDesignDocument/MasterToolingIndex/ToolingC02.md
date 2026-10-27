# ToolingC02 — Tooling Certification for SKL-02 (Systems Programming & Kernel Fusion)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-02` (`SkillsC1.md`) — Systems Programming & Kernel Fusion (Rust/MLX/Metal).
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS

**Necessity re-verified [VERIFIED — `C05/C12/C35` specs read directly]:** C05 Atlas/VP-tree, C12 kernel fusion/ILP scheduling/prefix cache, C35 Lorentz/Picard/Q32.31 interop have no executable path without kernel authorship. Gate 3 (unit) and Gate 8 (perf/build receipt) cannot pass on stubs. A1/A31–A35 are kernel measurements. Necessity holds.

**Gate mapping correction (Scheme B → dual citation, per C15.9):**

| Day gate (SkillsC1) | Artifact gates (C15.x) operated |
| --- | --- |
| G3 Unit + Integration | C15.1 (backend suite) + C15.2 (UI E2E subset where kernel telemetry surfaces) |
| G4 Integration + Contract | C15.2 + C15.4 (browser E2E where applicable) |
| G8 Performance + Build Receipt | C15.8 (bench/pressure) + C15.7 (receipt) |

**Ablations:** A1, A31–A35 — no delta.

**Conditions:**

1. SIMD/nightly `rustc` vs `cargo test --locked` (C15 Gate 2/3, stable pinned per ADR-23): nightly-only code paths must be feature-gated (`cfg`) with stable fallback, or Gate 2 reproducibility fails. Record the gate decision in Decision-Log-ADR.
2. `perf` is Linux; Apple Silicon profiling is `instruments`/`powermetrics` (C21.8 records power-state requirement). Drop `perf` or scope it to the H100 replica target only.
3. `tlaplus`/`apalache` in SKL-02's tool list duplicate SKL-01's tools — cross-skill reuse, not re-certification: SKL-02 CONSUMES TOOL-01.01–01.03 outputs (verified specs/VCs) but does not operate those tools. No double certification.
4. `cargo-fuzz` harnesses for Q32.31 overflow (FLAG-02: no overflow at x₀ ≥ 65536) are REQUIRED evidence for C35.4, not optional — A31/A32 preregs must include the fuzz corpus seed.

---

## Tool Specs

### TOOL-02.01 `rustc` + `cargo` — Pinned toolchain (stable + nightly-SIMD gate)

1. **Tool Name:** `rustc` / `cargo` (workspace, bench, pinned per `rust-toolchain.toml`, ADR-23)
2. **Tool Structure:** typed-JSON (build envelope: `toolchain_hash`, `feature_flags`, `profile {dev|release}`, artifact hashes out).
3. **Grammar Constraint Mask:** `rust-toolchain.toml` channel pin = the mask (unpinned toolchain rejected by Gate 2 pre-step); nightly features require `RUSTC_BOOTSTRAP`-free `cfg` gating — wrapper greps for ungated `#![feature]` and fails closed.
4. **Tool Parser Required:** NO (cargo JSON `--message-format=json` consumed against schema).
5. **Definition/Purpose:** Only build path admitted to Gates 2/3/8 evidence; every perf number binds to a toolchain hash in the build receipt (C15.7).
6. **Draft Code:** (Bash — receipted build)

```bash
rustc -Vv > toolchain.txt && sha256sum toolchain.txt >> "$RECEIPT"
cargo build --release --locked --message-format=json \
  | cargo-build-parser --receipt "$RECEIPT"  # unknown artifact hash => build FAILS
```

### TOOL-02.02 `mlx` + `metal-rs` — Metal shader path

1. **Tool Name:** `mlx` (Metal shaders) + `metal-rs`/`metal-cpp` (host bindings)
2. **Tool Structure:** typed-JSON (kernel launch envelope: `shader_hash`, `grid_dims`, `buffer_hashes`, `fence_receipt` out).
3. **Grammar Constraint Mask:** shader source hash pinned in manifest (cf. C28 compile-in manifest — shaders are code, manifest them); runtime-compiled shader strings rejected.
4. **Tool Parser Required:** YES — `metal-log-parser` (validation-layer + timing-log → typed: `compile_ms`, `dispatch_ms`, `oom_flag`).
5. **Definition/Purpose:** Executes C05/C12 fused kernels (Lorentz boost + VP search; TwisterAttention + projection; draft-verify pair per C12 fusion rules). All A1 numbers flow through this tool.
6. **Draft Code:** (Rust — launch record)

```rust
struct MetalLaunch { shader_hash: Hash, grid: (u32,u32,u32), bufs: Vec<Hash> }
fn dispatch(l: &MetalLaunch) -> Result<MetalReceipt> {
    let enc = queue.command_buffer(l)?; // fence bound at encode
    Ok(MetalReceipt { dispatch_ms: enc.commit_and_wait()?, oom: enc.oom_flag() })
}
```

### TOOL-02.03 `cargo-asm` — Instruction inspection

1. **Tool Name:** `cargo-asm` (assembly output for hot-path audit)
2. **Tool Structure:** typed-JSON (fn id in, `asm_hash` + `vec_width` + `spill_count` out).
3. **Grammar Constraint Mask:** audit-only gate (never blocks build; blocks PERF CLAIMS — no asm record, no "SIMD-accelerated" label per C03.2).
4. **Tool Parser Required:** YES — `asm-feature-parser` (detects NEON/SVE ops, counts spills, flags scalar fallback).
5. **Definition/Purpose:** Evidence for vectorization claims on Q32.31 Minkowski paths; feeds A1 waste-figure computation (C21).
6. **Draft Code:** (Bash — claim evidence)

```bash
cargo asm --rust mycrate::minkowski_dot > asm.txt
asm-feature-parser --in asm.txt --out asm_evidence.json  # vec_width=0 => claim "vectorized" REJECTED
```

### TOOL-02.04 `instruments` + `powermetrics` — Profiling + energy

1. **Tool Name:** `instruments` (time profiler) + `powermetrics` (energy sampler)
2. **Tool Structure:** typed-JSON (sample config in; `joules_per_token`, `joules_per_recall`, `peak_watts`, `power_state {mains|battery}`, `thermal_state` out — power/thermal MANDATORY per C21.8).
3. **Grammar Constraint Mask:** sampling window + workload ID bound at invocation (unbound sample rejected — unrepeatable measurement per C09.3).
4. **Tool Parser Required:** YES — `powermetrics-parser` (raw sampler lines → typed EnergyMetrics; drops warmup window).
5. **Definition/Purpose:** Sole admissible source for NF9–NF11 energy numbers; feeds C21 SLO table and A1 energy arm.
6. **Draft Code:** (Python — energy gate)

```python
def energy_gate(workload: str, secs: int) -> EnergyMetrics:
    raw = run(["powermetrics", "--samplers", "gpu_power,cpu_power,thermal",
               "-i", "100", "-n", str(secs * 10)])
    m = powermetrics_parser(raw, warmup_s=2)  # drop warmup
    assert m.power_state == "mains", "battery runs invalidate comparison (C21.8)"
    return m
```

### TOOL-02.05 `criterion` — Benchmark harness

1. **Tool Name:** `criterion` (statistical benchmarking with change detection)
2. **Tool Structure:** typed-JSON (bench id + baseline ref in; `mean`, `p99`, `effect_size`, `regression_flag` out).
3. **Grammar Constraint Mask:** baseline binding required (bench without baseline ref runs but its output is inadmissible as a claim — recorded, not cited).
4. **Tool Parser Required:** NO (criterion JSON consumed against result schema).
5. **Definition/Purpose:** Produces the A1 tok/s numbers + C06.15 A36 parent baselines; `regression_flag` wires into C09.8 re-validation triggers.
6. **Draft Code:** (Rust — bench with evidence binding)

```rust
criterion::criterion_group!(benches, vp_retrieve_bench);
fn vp_retrieve_bench(c: &mut Criterion) {
    let base = BenchBaseline::load("A1_baseline.json").expect("no baseline, no claim");
    c.bench_function("vp_retrieve", |b| b.iter(|| retrieve_5d(&Q, 10)))
     .with_baseline(base); // regression_flag => triggers C09.8 re-run list
}
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-02.01–02.05 | G3→C15.1, G4→C15.2/C15.4, G8→C15.7/C15.8 | A1, A31–A35, A36 parents | C05, C12, C21.8, C35 |
