# TPPE memory kernel — TODO (Commandment I)

## Deferred (with reason, next step)
- [ ] DEFERRED: full-model quality claim — held-out bakeoff (383 preds, Layer-3 component path) returned OPEN, promoted=none. Next: independent task corpus + full-model reference (owner's gate).
- [ ] DEFERRED: radial-winner selection — radial CI 0.0063 crosses zero (tie). Fixture label stays candidate; no silent pick.
- [ ] DEFERRED: Q/K forward-pass integration of TPPE (qwen36.rs Layer-3 boundary) — separate gate; this task is the memory layer only.
- [ ] DEFERRED: UI wiring (status pill) — untouched per roadmap; memory endpoints exist.
- [ ] DEFERRED: legacy session_memory_hash384_v2 archival decision (currently 0 points; keep read-only until owner rules).
- [ ] DEFERRED: TS 7.1 contract artifact for the encoder config — backend has no TS layer; constants are typed in Rust and recorded in REPORT.md. Next step: add a checked TS contract only if the backend grows a TS policy surface.

## Open risks
- Denominator-free design: TPPE carries no rank-5 readout, so the tpe_e2e seq-1 denom-pin regime does not apply (different kernel) — but rho saturation at large pos is real; epoch cap + persisted seq/state required (see tests).
- Cross-platform f64 transcendental drift: state equality judged at 1e-12 tolerance, same-host replay only unless re-receipted.
