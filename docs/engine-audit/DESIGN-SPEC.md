# DESIGN-SPEC — unified Qwen3.6-MoE engine (Rust, Apple Silicon)

Written from the audits in this folder: 01 (Rust seat, the thing being replaced), 02 (oMLX), 03 (rapid-mlx), 04 (SGLang), 05 (swift-mlx, unverified), 06 (mlx-lm / mlx-vlm reference). Every "why" points at a doc or a file:line.

Status: **specification only**. Nothing in this document is built. Acceptance tests in §7 are not run.

## 0. Non-negotiables (from AGENTS.md and the brief)

- **No JSON and no Python across the model I/O boundary.** Requests, replies, caches, drafter state and receipts are typed little-endian binary frames (the existing `QNC1`/`QNR1`, `GTK2`/`GTO2` pattern, `native_token_wire.rs`, is the template). JSON is allowed only at load time for `config.json` and `tokenizer.json`, as the seat already does (`model_config.rs:1`, `tokenizer.rs:1`).
- **No mocks, no synthetic data as evidence.** Every gate runs on the real checkpoint `~/models/fused/Qwen3.6-MoE-Combined` and real prompts, and compares against the real reference (mlx-lm, doc 06) on the same machine. A synthetic pass is a FAIL.
- **Receipts for every claim.** Each stage emits a SHA-256-bound receipt (pattern: `serve_qwen.rs:77–88`).
- **Ports.** Bind loopback only. Do not use 8000 (SurrealDB), 8001, 8002 (oMLX), 8011, 4810, 6331 (Qdrant, reserved). The seat's default `8003` (`serve_qwen.rs:106`) is reserved for "next model server" by the standing port key; a port decision is Stewart's (§9).
- **Numerical contract** (required by AGENTS §2): every numerical routine documents analytical bounds, ε-tolerances and IEEE-754 hazards, verified by a machine CAS receipt (`sympy_eval` / `math-cas`) before ship. This spec lists the hazards in §6; the CAS receipts are a deliverable of phase P0.

## 1. What the engine is

One Rust crate, one state machine, one outer governor:

```
                 ┌─────────────────────── Meta-Automaton (governor) ───────────────────────┐
                 │  decides: propose | verify | commit | iterate(plan/research/execute) | stop │
                 │  owns: confidence Q32.31, prediction-error ledger, iteration budget       │
                 └───────────┬───────────────────────────────┬──────────────────────────────┘
                             │ proposals                     │ outcomes (receipts only)
              ┌──────────────▼───────────┐        ┌──────────▼────────────────────────────────┐
              │ proposer plugins          │        │ target model (exact Qwen3.6 forward)      │
              │  - MTP head (model-mtp)   │ ─────► │  - verify window of k+1 positions         │
              │  - (later) DFlash-style   │        │  - full-attn: paged KV (chain-hashed)     │
              │  - n-gram / none (control)│        │  - linear-attn: GDN state + checkpoints   │
              └───────────────────────────┘        │  - MoE: routed experts, renormalised      │
                                                   └──────────────────┬────────────────────────┘
                                                                      │ commit | rollback (atomic)
                                                   ┌──────────────────▼────────────────────────┐
                                                   │ cache tiers: RAM → SSD (typed safetensors │
                                                   │ blocks, index, budget)                    │
                                                   └───────────────────────────────────────────┘
```

Why one loop: oMLX and rapid-mlx both have a second decode path for MTP that bypasses the main batch loop (doc 02 §7.2, `scheduler.py:216–240, 9763`). That split is what the brief calls "two loops" and is the first thing to remove. Here the proposer, verifier, commit/rollback and governor are one `step()` (§3).

## 2. The model core (must match the reference before anything else)

Fix list from doc 01 §5, in order. Each has a reference anchor:

| # | Change | Reference | Shape/receipt |
|---|---|---|---|
| M1 | Read per-tensor `bits`/`group_size` from `config.json` `quantization` block; drop `optiq_metadata.json` | REF-QWEN35:371–381 | `mlp.gate`, `shared_expert_gate` 8-bit; experts/embed/attn 4-bit (doc 01 R4) |
| M2 | Assert every projection shape against the index at load | CKPT index (doc 01 R3) | `in_proj_qkv` 8192×2048, `in_proj_z` 4096×2048, `out_proj` 2048×4096, `q_proj` 8192×2048 (layers 3,7,…,39) |
| M3 | MoE: renormalise top-k weights | REF-NEXT:332; REF-QWEN35:52 | sum of routed probs = 1 per token |
| M4 | MoE: shared expert × `sigmoid(shared_expert_gate(x))` | REF-NEXT:339 | per-token scalar gate |
| M5 | Full attention: split `q_proj` into query and gate; `q_norm`, `k_norm`; `sigmoid(gate)` before `o_proj`; RoPE pairing per reference | REF-NEXT:82–151; SRT `qwen3_5.py:1312–1332, 1402` | layer oracle |
| M6 | Linear attention = GatedDeltaNet with checkpointed conv1d, `A_log`, `dt_bias`, `in_proj_a/b`, per-head matrix state, gated RMSNorm | REF-QWEN35:87–206; REF-GDN:19–44, 385–605 | layer oracle |
| M7 | Keep GDN recurrent state in **fp32** (checkpoint `mamba_ssm_dtype: float32`; oMLX sidecar default fp32, doc 02 §4 `config.py:125`) | config; `config.py:125` | dtype assertion |
| M8 | Exact causal softmax for full attention at every length. Geometric retrieval is kept only as an *optional* candidate index that must be proved equal to the exact set (§7 A6) | doc 01 §2.2 item 4 | equality with exact softmax |

Oracle: a Python run of mlx-lm 0.31.4 (the copy in oMLX, doc 06) on the same machine, same checkpoint, same prompt, dumping per-layer hidden states. The Rust path dumps the same. Compare with max-relative-error per layer. The oracle is run *offline by a script*, not by a server.

## 3. The unified step function

One function owns a whole token-generation round. Pseudocode in Rust-like form (not compiled; the signature is the contract):

```rust
// All types are typed structs; the wire form is the binary frame of §5.
fn step(state: &mut EngineState, req: &StepRequest) -> Result<StepReceipt, StepError> {
    // 0. Pre-state digest (binds everything that follows)
    let pre = state.digest();

    // 1. Governor chooses the action set for this round (§4)
    let plan = state.governor.plan(&state.ledger, req.budget);      // depth k ∈ [0, K_MAX]

    // 2. Proposal (proposer plugin, may be None = control)
    let draft: Vec<TokenId> = state.proposer.propose(&state.target_ctx, plan.k)?;

    // 3. Verify: ONE target forward over k+1 positions (last committed token + drafts)
    let verified = state.target.verify_window(&state.cache, &draft)?;  // per-position logits

    // 4. Accept: greedy → longest prefix where argmax == draft; sampling → rejection rule
    let accepted = accept(&draft, &verified, req.sampling)?;

    // 5. Atomic commit or rollback of ALL state touched in step 3
    //    - KV pages: truncate to accepted length (chain hash recomputed)
    //    - GDN state: deferred commit (only accepted positions advance the recurrent state)
    //    - MTP/drafter state: same rule
    state.commit_or_rollback(&accepted, &verified)?;

    // 6. Ledger update from INDEPENDENT receipts only (§4.2)
    state.ledger.record(&accepted, &draft, &verified, pre);

    // 7. Governor decision: emit | iterate(plan/research/execute) | stop (§4.3)
    let decision = state.governor.decide(&state.ledger, req.iteration)?;

    Ok(StepReceipt::new(pre, req, &draft, &accepted, decision, state.digest()))
}
```

Invariants of the step (each becomes a test in §7):
- **I1 Exactness.** Under greedy sampling the emitted token sequence is identical with the proposer on, off, or with any `k` (lossless speculation).
- **I2 Atomicity.** After step 5, state digest equals `digest(state after accepting exactly the accepted prefix from pre)`. Rejected drafts leave no trace in KV, GDN state, or ledger.
- **I3 Bounded depth.** `plan.k ≤ K_MAX`. Start `K_MAX = 8` to match oMLX's global ceiling (`model_settings.py:37`).
- **I4 One loop.** There is exactly one call site that advances the target state (the `verify_window` in step 3). A second decode path is a bug.

## 4. The Meta-Automaton (governor)

Definitions taken from the repo, not invented:
- **Governor** = the single deterministic decider over proposer plugins (`handoff §5`: "MTP head and speculative drafter are proposer plugins under one deterministic governor").
- **Confidence** = Q32.31 fixed-point, updated only from independent receipts (`handoff §5`: "update Q32.31 confidence/prediction error only from independent receipts").

### 4.1 Prediction-error minimisation (what is minimised, and by which knob)

- Per proposal round, define the **prediction error** as the first disagreement index: `e = k − accepted_len` over `k` proposed tokens (0 ≤ e ≤ k). Normalised: `ē = e / max(k,1)`.
- The governor keeps an exponentially weighted estimate of acceptance `â` (fixed-point Q32.31) and chooses `k` to maximise expected accepted tokens per unit verify cost: `argmax_k (k_eff(â,k)) − λ·cost(k)`, where `cost(k)` is measured from receipts, not assumed. This is a 1-D control problem; no learned model is required.
- Calibration: the governor also tracks `|conf − observed_acceptance|` per bucket; buckets with persistent miscalibration fall back to `k = 0` (control path). Minimisation target: the calibration error, subject to exactness (I1).
- **The brief's phrase "prediction-error minimisation" is operationalised as this acceptance/calibration control. A full free-energy formulation (doc of research-grade-harness) is out of scope for the first ship** and is marked PROPOSED in its source (`Research-Grade-Agentic-Harness-Design.md:99`).

### 4.2 Ledger rule (what may update confidence)

- Only inputs that carry a receipt may update the ledger: a target-verify receipt (§5), a tool/recall receipt, or an exact-KV hit/miss receipt. Model self-assessment text is never an input.
- Each ledger entry stores `pre` digest, the proposal, the accepted prefix, and the verify receipt id. Replays must reproduce the ledger digest.

### 4.3 Dynamic iteration (no-answer ⇒ plan / research / execute)

Repo definitions (`research-grade-harness/MATH-INVARIANTS.md:54–62`): no-answer paths mandate iteration; retry, blind escalate and silent stall are refused structurally. The engine adopts:
- Decision enum with **no `Stall` variant**: `Emit | Plan | Research | Execute | FailClosed`.
- `Option<IterationMandate>` return semantics: `None` means no mandate, never a retry (`MATH-INVARIANTS.md` table, harness `mandate_iteration`).
- **Budget** `max_iterations = 3` per request, then `FailClosed` (`MATH-INVARIANTS.md`, vault `hold_and_iterate`).
- Trigger for no-answer: no accepted token in a round **and** `e = k` for `N` consecutive rounds, or verify receipt missing/invalid. Threshold `N` is a parameter; default 2.

### 4.4 Homogeneity (one update law)

- Requirement (`MATH-INVARIANTS.md:11–22`): the same recurrence/update law is used on every substrate (CPU reference, Metal kernel, any fused path). Concretely: the GDN recurrence (`gated_delta.py:385`, `:557`) and the Q32.31 ledger update have one Rust definition each; GPU kernels are *ports* verified against it, not re-derivations.
- Gate: per-substrate state digest after N real steps must match the CPU reference (bitwise for integer ledger; ε-bounded for f32 GDN state, with ε declared in §6).

### 4.5 Isotropic suppression (no preferred direction survives)

- Requirement (`MATH-INVARIANTS.md:29–38`): when candidate continuations tie on score, none is preferred by position, by proposer identity, or by memory address. Suppression functions take **no source argument**.
- Concretely: ordering of tied candidates is by a canonical key (lexicographic on token ids), and proposer identity never enters the comparator. Gate: permuting the proposer list does not change the decision (A8).

## 5. Wire frames (typed, binary, receipted)

Extend the existing scheme; do not introduce JSON.

| Frame | Request | Reply | Receipt |
|---|---|---|---|
| `QNC2` complete | max_new, n, ids | `QNR2`: out ids, per-step records | SHA-256 over request ‖ reply body |
| `QST1` step | state handle, k-policy, sampling | `QSR1`: accepted, decision, digests | pre/post state digests in reply |
| `QVF1` verify (internal, not public) | window ids | per-position top-k logits digest | included in StepReceipt |
| `GTK2`/`GTO2`, `GDT2`/`GDR2` | as today | as today | as today (`native_token_wire.rs`) |

Fix in the same change: `add_bos` must be honoured or rejected, not ignored (doc 01 §3, `serve_qwen.rs:208`).

## 6. Numerical contract (to verify with CAS before ship)

| Routine | Hazard (IEEE-754 f32 unless stated) | Bound / ε to declare | CAS receipt needed |
|---|---|---|---|
| `compute_g` = exp(−exp(A_log)·softplus(a+dt_bias)) | overflow of `exp(A_log)` for large `A_log`; softplus overflow for large `a+dt_bias` | use stable softplus; clamp `exp(A_log)`; ε for `g∈(0,1]` | sympy: limits of softplus and exp composition |
| L2 norm of q,k (`normalize_qk`) | division by ≈0 norm | `eps = 1e-6` (REF-GDN:33–44) | bound on relative error |
| softmax over candidates (`ops.rs:65–78`) | `exp` of large differences; sum underflow to 0 | max-subtraction already present; ε on sum | bound on relative error of max-subtracted softmax |
| RMSNorm (`ops.rs:45–63`) | `mean(x²)` overflow for |x|>1e19 in f32 | none for real activations; assert finite | magnitude bound from logs |
| GDN state recurrence | drift over T steps; fp32 required (M7) | per-step relative error ε_step; test on T=1k | error propagation bound over T |
| Expert sum `Σ p_i·down_i` | cancellation when probs ~ equal | renormalised probs (M3) | — |
| Dequant `q·scale+bias` (`optiq.rs:115`) | bf16 scale/bias rounding | parity to dequant reference (R2 tol 1e-3) | — |

No numeric claim in this spec is counted as verified until its CAS receipt exists. This is a P0 deliverable.

## 7. Acceptance tests and the receipts each produces

All tests use the real checkpoint and real prompts. Prompts: a fixed set of 3 real prompts (chosen in P0 and frozen; stored as token-id files with digests).

| ID | Test | Pass condition | Receipt |
|---|---|---|---|
| A1 | Layer oracle, per layer, linear & full | max rel. err ≤ declared ε (P0 value) | per-layer error table + both hidden-state digests |
| A2 | Shape/bits load assertions (M1, M2) | all tensors match index shapes; bits match config | load report with digests of each shard header |
| A3 | MoE router (M3, M4) | per-token prob sum = 1 ± ε; shared gate applied | per-layer sums and gate values |
| A4 | End-to-end greedy parity vs mlx-lm | identical token ids for 3 prompts × 64 new tokens | token-id digests from both engines |
| A5 | Speculation lossless (I1) | greedy output identical with proposer on/off and for k ∈ {1,2,4,8} | digests per k |
| A6 | Exact attention at all lengths (M8) | exact softmax equals reference at lengths 1, 64, 65, 512, 2048 | per-length error + digest |
| A7 | Atomic rollback (I2) | state digest after reject equals digest of accepted-prefix replay | paired digests |
| A8 | Isotropic suppression | permuting proposers leaves decisions unchanged; 4 tied branches all survive | decision digests for each permutation |
| A9 | Homogeneity across substrates | CPU vs Metal state digests equal (integer) / ε-equal (f32) after N steps | digests + ε table |
| A10 | Dynamic iteration bound | never more than 3 iterations; no `Stall` reachable; FailClosed on budget exhaustion | iteration trace per request |
| A11 | Ledger from receipts only | ledger digest reproducible from receipt chain; no update from non-receipt input | replay digest |
| A12 | Cache restore (paged KV + GDN checkpoint + SSD) | restored state digest equals pre-eviction digest; continuation tokens equal uninterrupted run | digests |
| A13 | Wire | malformed/truncated frames rejected; receipts verify; no JSON in frames | fuzz-style corpus with pass/fail table |
| A14 | Timing (warm and cold, tokens/s, s/token) | reported with raw logs on this machine; no claim without it | timed run logs |

Existing receipts to keep: R2 (`expert_matvec` parity), the wire unit tests (`native_token_wire.rs:165–221`).

## 8. Order of work (phases with gates)

| Phase | Work | Gate (must pass to enter next) |
|---|---|---|
| P0 | Freeze prompts; build oracle script (mlx-lm, offline); CAS receipts for §6; declare ε values | A1 baseline table exists; CAS receipts exist |
| P1 | M1, M2 (bits, shapes), M3, M4 | A2, A3 pass |
| P2 | M6 GatedDeltaNet, M7 fp32 | A1 linear layers pass |
| P3 | M5, M8 full attention (exact) | A1 full layers pass; A6 pass |
| P4 | End-to-end forward, no caches beyond in-memory | A4 pass |
| P5 | Cache tiers: paged KV, GDN checkpoints, SSD | A7, A12 pass |
| P6 | Proposer plugin trait; MTP head loaded from `model-mtp.safetensors` (doc 01 R5); verify_window; atomic commit | A5 pass (lossless) |
| P7 | Governor: ledger, prediction-error control, iteration budget, decision enum | A8, A10, A11 pass |
| P8 | Homogeneity ports (any Metal kernel) under CPU reference | A9 pass |
| P9 | Wire: `QNC2`, `QST1`, receipts; loopback serve on an approved port | A13 pass |
| P10 | Hardware timing and comparison vs oMLX/rapid-mlx on the same prompts (external numbers are vendor claims until reproduced) | A14 logs |

Do not start P6 before A4 passes: speculation on a wrong model proves nothing. Do not start P8 before A5 passes: a kernel that is fast and wrong is not an optimisation.

## 9. Decisions needed from Stewart

1. **Port.** The seat's default `8003` conflicts with the standing key ("next model server"). Choose a port, or confirm `8003` for this seat. No port is touched by this spec.
2. **Reference oracle.** Use mlx-lm 0.31.4 from oMLX (the copy audited here), or pin a specific upstream commit?
3. **Prompt set.** Approve the three frozen prompts before P0 freezes them.
4. **Scope of the governor.** "Prediction-error minimisation" is specified above as acceptance/calibration control. Confirm, or specify the exact objective.

## 10. What is explicitly out of scope for the first ship

- Batched multi-request serving (the seat is sequential by design, `serve_qwen.rs:3`).
- Swift and SGLang backends (docs 04, 05).
- Any claim of Qwen3.6 quality beyond A4 parity with mlx-lm.
