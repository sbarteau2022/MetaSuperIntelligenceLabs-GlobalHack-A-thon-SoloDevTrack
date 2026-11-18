# MIRROR.md — what the toy mirrors, byte-for-byte, and where it stops

Date: 2026-11-02. Status: PROPOSED (rehearsal contract).

## 1. Why this exists

The backend is still landing. This toy is a **solo rehearsal**: same wire
shapes, same chain formulas, same gate rules — so scheduler/framework ideas
(HYPERBOLIC-SCHEDULER-FRAMEWORK) get ironed out against real code paths now,
and port to the backend later by diffing, not rewriting.

## 2. Byte-for-byte mirrors (port = copy + rename)

| Toy path | Backend path | Parity test |
|---|---|---|
| `mission::chain_hash` canonical form | `harness::mission::chain_hash` L306–321 | `chain_hash_matches_backend_canonical_form` |
| `ReceiptLog::append` / `verify_chain` | same, L328–406 | `receipt_log_mints_and_verifies_like_backend` |
| `Judge` 4 kinds + `run` | same, L204–263 | `judges_are_deterministic` (incl. golden `sha256("hello")`) |
| `Plan::validate` / `ready_packets` | same, L86–163 | `plan_validation_rejects_judgeless_and_cyclic` |
| `EnergyBudget` charge rule | same, L415–441 | energy assertion in `live_mission_passes_with_scripted_executor` |
| `WorkingSetCache` mass/leak/evict | same, L712–809 | `working_set_cache_mass_and_eviction` |
| `research_gate` | same, L639–680 | `research_gate_and_session_chain` |
| `CapabilityGate::admit` rule | `harness::runtime` L60–82 | `capability_gate_requires_judged_atom_and_receipt` |
| `Engine::judge_output` fail-closed | same, L154–174 | `engine_judge_output_needs_registered_judge` |
| `run_sequential` escalate/skip shape | same, L340–411 | `mission_fails_closed_without_live_executor`, `forged_prior_skips_nothing` |
| `run_sequential_live` retry/escalate | same, L417–541 | `live_mission_passes…`, `transport_error_retries_once_then_escalates` |
| Tool-arg JSON-object check, 64-hex digest check | same, L215–242 | covered in live tests |
| Energy formula `dt·1e-9 + tokens·0.0005` | same, L243–246 | asserted to 1e-9 |

## 3. Intentional divergences (same semantics, smaller scope)

- `SessionEventKind`: toy has 5 variants (no Fork/Resume — unneeded for rehearsal).
- `Engine`: toy adds `parallel_slots: Option<usize>` (None default) as the
  explicitly-wired concurrency knob for a future parallel rehearsal; the
  sequential paths never read it (backend `max_parallel` is unread — the
  toy keeps the honesty, names the knob).
- `ScriptExecutor`: scripted answers + `sha256(tool|args)` digests. NOT a
  model. The backend plugs oMLX/gateway/hypermind-MCP here.
- `mind`: toy Lorentz H3 (no T2 phase, no IDF, hashed token directions).
  Same distance/shell/VP/depth-gate math, toy-grade embedding. Never
  presented as recall-quality evidence — parity is structural, not empirical.
- No compliance, stores, HTTP/SSE, Merkle vault, carved views. Out of scope
  by design (CALIBRATION.md CONTRACT-FIRST).

## 4. Rehearsals the backend cannot run yet (the payoff)

1. Scripted PASS missions end-to-end (backend has no executor wired).
2. Transport-error → escalate path with a ledger.
3. Capability routing by problem-text geometry + gate admit/deny split.
4. Recall-vs-recompute: depth-k argmin, exact-vs-brute parity, tight-cluster
   narrow-k behavior, invalidation exactness, scan-parity counterexample.
5. `verify_external_chain`: the port gate — backend-minted receipt rows are
   re-verified here with independent arithmetic in `backend_receipt_parity.rs`.

## 5. Port checklist (when the backend is ready)

1. `cargo test` green here → run backend `tests/harness_core.rs` + `governance_core.rs` green.
2. Diff toy `mission.rs`/`runtime.rs` against backend `harness/`; port only
   semantic deltas (parallel lane, telemetry export, real executors).
3. Feed backend receipt rows into `verify_external_chain`; mismatch = stop,
   canonical form drifted, fix before porting anything else.
4. Promote rehearsed pieces (capability routing, invalidation edges) through
   the kernel-register necessity ablation, never by copy-paste.
