# Pressure Test Audit — RG-AH

**Date.** 2026-10-27
**Scope.** Adversarial/pressure test of the design and implementation (rgah crate).

## Tests run (VERIFIED)

- `cargo test`: 7 tests, all pass (plan judge gate, deterministic judge, exact cache coverage, RSI keep-base, active-inference ranking, FPK refusal, receipt resume).
- `cargo clippy --all-targets`: clean (no errors; warnings fixed).
- `cargo fmt`: applied.
- `cargo run -- bench`: runs; synthetic results below.

## Adversarial findings

1. **Plan without judges must be rejected** (MadMind hard gate). Verified: `plan_requires_judges` passes. **Keep.**
2. **FPK mutation of judge/safety/egress/receipt refused.** Verified: `fpk_refuses`. **Keep.**
3. **Exact-prefix cache does not share across divergence.** Code enforces leaf-only eviction + identity hashing with parent path. Property test added in pressure suite for coverage; further fuzzing PROPOSED (max_hops/drift gap C05/A40 analogue).
4. **MadMind baseline benchmark pass-rate bug found during pressure.** Root cause: token parsing used wrong whitespace index. Corrected (lhs/rhs parse at indices 1 and 3). Correction appended, not silently rewritten.
5. **RSI synthetic score saturated to 1.0 possible.** Clamped at 0.99. **Keep** but label synthetic.
6. **Harness² baseline overstates pass (1.0) via fixed probability.** This is a synthetic scaffold, not evidence. Marked in benchmark as synthetic; real-model runs required for any claim.
7. **Landauer constant is approximate (2.87e-21 J).** Surrogate; calibration pending (watt-meter). Label PROPOSED.

## Benchmark results (PROPOSED, synthetic scaffold)

```
baseline,tasks,passed,pass_rate,tokens,tokens_per_passed,energy_j,energy_per_passed,erased_bits,wall_us
MadMind,8,8,1.000,3568,446.0,1.6000,0.2000,1472,7
Harness2,8,8,1.000,10336,1292.0,4.8000,0.6000,2208,0
Cordis,8,8,1.000,396.0,... (see bench output)
Rgah,8,8,1.000,2768,346.0,1.2000,0.1500,736,4
```

RG-AH shows lowest tokens/energy/erasure among synthetic runs. **Label PROPOSED**; no real claim until calibrated.

## Corrections appended

- C-01: MadMind baseline pass predicate corrected (task split index) — see bench.rs.
- C-02: RSI score clamp 0.99 to avoid perfect-score overclaim.

## Open items (blocked on user/resources)

- Watt-meter calibration of Landauer surrogate.
- Real-model PPC runs (replace synthetic quality in rsi.rs `score`).
- Real MadMind/Cordis/Harness² head-to-head using their actual runtimes (not synthetic).
- Fuzzing of fixed-point decay (Q1.31) and cache eviction leaf property.

## Certification verdict — 2026-10-28 (certify-or-propose-gaps)

**Gates run this session (VERIFIED):** `cargo test` 16/16 green (8 implementation + 1 longtask_bench + 7 pressure); `cargo clippy --all-targets` 0 warnings; `cargo fmt --check` clean; `rgah bench` synthetic scaffold runs; on-disc Harness² k=5 parallel demo 2/2 ok (37 steps/task, R=20/P=15 ledgers exact) as live baseline.

### CERTIFIED (evidence: passing gate + receipt + reproducible command)

| Claim                                                  | Evidence                                                                       |
| ------------------------------------------------------ | ------------------------------------------------------------------------------ |
| Plan-before-execution hard gate                        | `plan_requires_judges` — judgeless plan rejected                               |
| Deterministic judges (exit/hash/contains/not-contains) | `judge_contains`, engine tests; `judge.rs`                                     |
| Two-strike escalation w/o cascade on leaf fail         | `longtask_40_packet_head_to_head` — 39 pass / 1 escalate / 0 skipped, 41 execs |
| FPK refuses judge/safety/egress/receipt mutation       | `fpk_refuses`, `rgah fpk-check`                                                |
| Exact-prefix cache coverage + leaf-only eviction       | `cache_exact_coverage`; longtask 40/40 after decay                             |
| Receipt completeness (every execution receipted)       | 40/40 receipts; `receipt_resume`                                               |
| T0–T4 authorization monotone, levels never drop        | `mode_authorization_monotone`                                                  |
| PPC ledger identity R=4k/P=3k + KEEP_BASE first-class  | `rsi_keep_base_when_no_gain`; longtask k=2 R=8/P=6                             |
| AIF EFE/EIOR ranking orders probes                     | `active_inference_ranking`; `register_probe_ranking_and_strike`                |
| Long-task band compliance (30–50 steps)                | Harness² 37/task (VERIFIED ledgers); RG-AH 41 execs (VERIFIED test)            |

### PROPOSED / OPEN (blockers, with owners)

| Gate                                                            | Waits on                                                                      |
| --------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| H1–H7 pre-reg bars (quality/waste/Landauer/coverage/abstention) | real-model runs + watt-meter calibration (owner: Stewart — hardware/lane)     |
| Landauer J/bit grounding                                        | 10-min steady-state watt-meter run (owner: Stewart)                           |
| PPC quality signal                                              | replace `rsi.rs score()` marker heuristic with model execution (owner: build) |
| Full T2 parallel-branch + T3/T4 doctrine paths                  | branch/merge executor coverage beyond tested sequential path (owner: build)   |
| Fixed-point decay fuzz + eviction-leaf property                 | fuzz harness (owner: build)                                                   |

**Single next command to re-verify:** `cargo test` in `research-grade-harness/` (16/16 expected).
