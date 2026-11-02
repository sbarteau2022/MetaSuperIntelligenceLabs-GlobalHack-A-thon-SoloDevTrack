---
title: "Queue 2 Adversarial Mathematical Audit"
document_type: "qualification"
date: "2026-11-02"
status: "blocking-corrections-required"
scope: "Read-only audit; no active agent implementation files changed"
---

# Verdict

Queue 2 is in progress, not complete. The real forward trace and retrieval
component work are useful evidence. The reported TPPE/RoPE/angle-only bench is
invalid and must not be used to bench or promote any positional encoder.

# Blocking findings

## Q2-M1 — Full-forward dominates by construction

`globalmind_tppe_qualify.rs::run_full_forward_control` assigns full-forward
`mean_phase_error = 0.0` and `wall_us = 0` instead of measuring its observed
runtime. The frontier test compares only phase error and wall time, so this row
necessarily weakly dominates every nonzero-error, nonzero-time candidate.
Although `state_bytes` is printed, it is not used in dominance. The conclusion
that all four cheap candidates are dominated is therefore tautological.

The Queue 2 report separately records the real dual-forward trace at 37.2 s;
that measurement is not used by the qualifier. A zero-time full-forward row is
not a valid resource control.

## Q2-M2 — The quality target is not observed model behavior

For TPPE, `predicted = state.phase_t2()`. The alleged observation is then
constructed as `predicted + actual_next_drive`. The error is therefore the
magnitude of the next drive, not error against an independently observed target.
The angle-only control uses the same construction. This explains why both
radial laws and angle-only report exactly the same value: the radial coordinate
cannot affect the metric.

No next-token likelihood, logits, hidden-state parity, retrieval quality, task
success, or independently observed phase enters this score. It cannot qualify
a positional encoder.

## Q2-M3 — Byte-mean drives have no validated semantic contract

`drives_from` averages raw envelope payload bytes and maps that scalar to fast
drive, half-sized slow drive, and a token-position revolution pulse. This is a
deterministic probe, but no derivation binds byte mean to the model's toroidal
phase dynamics, token position, attention behavior, or task objective. “Derived
from a real trace” does not make the derived quantity a valid target.

## Q2-M4 — The RoPE control is not RoPE

The control advances two scalar phases with hard-coded `+0.1` and `+0.05`.
There is no frequency schedule by dimension, complex pair rotation, query/key
application, attention score, or model execution. Label it `constant_phase`
if retained as a structural control; it cannot support a RoPE comparison.

## Q2-M5 — The geometry comparison is unmatched

The hyperbolic score receives periodic T2 distance, while the flat control uses
raw phase coordinates. Its test deliberately makes `2pi - 0.1` far from zero,
so the comparison changes both geometry and periodic representation at once.
Raw Lorentz ambient-coordinate distance also differs from a matched tangent or
Poincare Euclidean chart. The 200-atom result is a component contrast, not a
causal hyperbolic-versus-flat ablation.

The 200 atoms and five queries are deterministically generated rather than
drawn from the crystallized real trajectory. The report correctly withholds a
task-quality claim; it must also withhold a geometry winner.

## Q2-M6 — The active A1 score is explicitly a pseudo-logit proxy

The in-progress memory-injection test takes the last 64-value activation slice,
sets `target_class = target_token_id mod 64`, and applies softmax as if those
hidden values were vocabulary logits. They are not logits, and modulo 64 is not
the Qwen vocabulary target. This can show that prepended recalled context changes
internal state/KV, but it cannot measure next-token quality or memory value.

The trace producer already computes the real 248,320-dimensional logits for
observer parity but emits only their digest, not an admissible logits envelope.
A1 must either expose the real final logits through a bounded native binary
result or call an API returning them directly. Until then, label current A1
output `CAUSAL_STATE_CHANGE_ONLY`; do not compare pseudo-NLL or pseudo-hit rate
as task quality.

The active test also takes adjacent line windows from repository `history.txt`
as both memory and test material. That is not held-out task evidence and may
contain implementation/provenance text. Preserve it only as a wiring smoke.
The corrected ablation requires a preregistered external task corpus with
train/memory and held-out query/target partitions, contamination checks, and
enough independent examples for uncertainty intervals.

# Evidence retained

- Real frozen-model observer nonperturbation with bit-exact logits is useful.
- Both TPPE radial candidates preserve the implemented component invariants.
- Binary trace decoding and digest verification are useful boundary evidence.
- Hyperbolic and flat retrieval are deterministic under their current,
  different definitions.
- Recall injection causally changes the model's KV/hidden state if the active
  run finishes as designed; this alone does not establish benefit.
- No positional encoder or retrieval geometry has been validly promoted.

# Corrected preregistration

## Position-encoder ablation

Use the same real frozen model, token tape, weights, runtime, hardware state,
precision, warmup, and decode settings for every arm:

1. native model baseline with its actual positional encoder;
2. angle-only toroidal phase;
3. TPPE `linear_log_v1`;
4. TPPE `log_v1`;
5. full-forward reference.

Each encoder must be installed at the same actual query/key positional boundary
or none of the arms qualify. Measure held-out next-token NLL/perplexity,
task/judge quality, logit divergence from the full-forward reference, latency,
peak memory, and wall energy when available. Record real end-to-end wall time;
never assign a control zero cost. State bytes are a separate resource axis and
must participate in Pareto dominance if reported as evidence.

Structural phase/radius invariants are mandatory preconditions, not substitutes
for quality. A candidate is benched only after it is noncompetitive on the
preregistered quality/resource frontier with uncertainty intervals.

## Geometry ablation

Hold the candidate atoms, query set, candidate count, storage budget, semantic
projection, and T2 representation fixed. Compare:

1. H3 geodesic plus periodic T2;
2. a matched Euclidean tangent/Poincare chart plus the same periodic T2
   sin/cos representation;
3. vector-only semantic baseline.

Run on the real crystallized atoms plus held-out queries. Report recall@k,
nDCG/MRR or task success, hubness, collision rate, latency, and restart
determinism. Synthetic on-sheet probes remain component tests only.

# Required report correction

Amend `queue2-ablation-report.txt` so A4 reads `INVALIDATED_BY_AUDIT`, not
`COMPLETE, BENCH CONFIRMED`. Preserve the original numbers as historical raw
output, but explicitly state they do not measure encoder quality. A3 remains
`MEASURED_COMPONENT_ONLY` with no geometry decision.

# Stop rule

Do not advance positional-encoder or geometry conclusions into Queue 3. Other
Queue 2 implementation may continue on non-overlapping files. The full Queue 2
gate remains open until the corrected matched ablations run.
