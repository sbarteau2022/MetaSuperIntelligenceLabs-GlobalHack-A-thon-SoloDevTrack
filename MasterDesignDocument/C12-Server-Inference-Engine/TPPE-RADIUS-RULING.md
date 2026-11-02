---
title: "TPPE Expanding-Radius Ruling"
document_type: "preregistration"
version: "1.0.0"
date: "2026-11-01"
status: "active"
---

# Decision being tested

TPPE must be a dynamic spiral-out whose radius expands with accumulated
revolutions and approaches the hyperbolic ideal boundary without crossing it.
A constant-radius corkscrew is invalid. This ruling decides the radial law; it
does not reopen the already frozen toroidal phase recurrence.

# Defect found in the prior specification

`TorodialPhasePositionalEncoder-Spec.md` defines
`r(w) = r_min + span*(0.35*w + 0.65*ln(1+w))`. The same paragraph labels the
law “sublinear-asymptotic” and states `r(w)/w -> 0.35*span`. Those statements
cannot both describe a sublinear function: sublinear growth requires the ratio
to approach zero. The specification’s own stated nonzero limit identifies an
asymptotically linear law with a diminishing logarithmic contribution.

This is a claim correction, not proof that the law is operationally inferior.
Do not bench it merely for being mislabeled. Bench it only if the matched
limits and real-model bakeoff below lose.

# Candidates

- `linear_log_v1`: the checked-in `0.35*w + 0.65*ln(1+w)` law.
- `log_v1`: `ln(1+w)`, scaled under the same admitted start/end radial budget.

## Matched endpoint for the first bakeoff

Freeze the comparison horizon at **one derived revolution** (`w_ref = 1`),
before observing any held-out outcome. Both arms start at the same `u_start`
and reach the same `u_end` at that horizon. Keep `linear_log_v1` at its
existing span; set the `log_v1` span to the linear-log growth at one turn
divided by `ln(2)`. This matches endpoints, not the full radial curves:
intermediate and later radii intentionally differ. Record the reference
horizon and both numeric spans in the run receipt. A later horizon or revised
span is a new comparison identity, not a silent update of this one.

## Candidate runtime identity for the isolated redo

The historical `tppe-reference-v1` manifest is a design reference, not an
identity for the current Rust Q/K execution. Before a fresh trace or positional
bakeoff, require a separate `tppe-layer3-candidate-v2` contract whose bytes
bind the current `tppe.rs` and Layer-3 trace producer source digests, position
drive constants, derived-turn rule, radial law and numeric defaults, and the
single-Layer-3 scope. The manifest's TPE digest must equal the SHA-256 of
those contract bytes. A source or numeric change invalidates the candidate
identity until its contract and manifest are regenerated. The candidate
contract is not a full-model or quality certification.

Stage-3 judging must derive the admitted manifest from the candidate manifest
whose verified input, TPPE version and configuration digest equal the trace
provenance. A mismatch fails before writing the verdict artifact. Write only
the run-ID-specific admitted manifest; never overwrite a generic `run-001`
alias or substitute the historical reference manifest for a new run.

Both are candidates only. Configuration and receipt digests must distinguish
them. No silent fallback or coefficient tuning after results is permitted.

# Coordinate ruling

The candidate laws act on nonnegative hyperbolic radial distance `u`, not
directly on Poincare-ball radius. Directly assigning either unbounded law to a
Poincare radius would cross the unit boundary and contradict the required
geometry. The persistent address therefore carries both:

- `u >= 0`, the geodesic radial coordinate produced by the selected law; and
- `rho = tanh(u / 2)`, the Poincare-ball radius, which must satisfy
  `0 <= rho < 1` with a strictly positive represented boundary margin.

The corresponding unit-hyperboloid lift is `x0 = cosh(u)` and
`||x_spatial|| = sinh(u)`, subject to a checked Lorentz invariant and finite
representation. A candidate is benched rather than clamped if `rho` rounds to
one, the margin becomes nonpositive, or the lift overflows. This resolves the
earlier ambiguity between an expanding spiral and an ideal-boundary-safe
stored coordinate without choosing the winning radial law.

# Controls and gates

Run both laws on the same real frozen-model trace, token tape, TPPE phase
recurrence, H3 chart, precision, and address workload. Record:

The current Layer-3 partial-model runner is a component comparison regardless
of tape length. A `heldout` filename or 100 predictions does not establish
independent corpus provenance or full-model behavior. Its outputs cannot
promote or bench an encoder; those decisions require verified source/split
provenance and an end-to-end frozen-model control.

1. strict radius increase whenever revolution state advances;
2. positive ideal-boundary margin and preserved Lorentz-sheet invariant;
3. finite state and replay-identical addresses after restart;
4. atom/trajectory retrieval quality and collision rate;
5. full-forward parity control, latency, memory, wall power, and temperature;
6. equal-budget angle-only and RoPE reference arms.

Stop and bench a candidate on invariant failure, boundary saturation,
non-finite state, replay drift, or a dominated quality/resource frontier after
the preregistered sweep. If neither candidate wins, TPPE radial gating stays
off and the full real-weight reference remains active.

# Transfer evidence

Monster’s periodic-phase, causal-streaming, deterministic-repeat, finite-state,
and unknown-carrier controls are retained as test questions only. Its measured
native continuation result failed `0/24`, so that untrained recurrence/readout
architecture remains benched. The current Rust server’s real-model trace and
observer-nonperturbation controls are the extraction baseline, but must be
rerun under the fresh TPPE and GlobalMind identities.

# Numerical hazards

For both radial laws, run a native high-radius boundary probe using the same
finite phase drives and an enlarged radial span. Record the last valid
hyperbolic radius, Poincare radius, represented margin, and the first checked
error. Require finite, monotone, on-domain states until rejection and strict
increase in both hyperbolic distance and represented Poincare radius when
derived revolutions advance. A rounded-equal represented radius is saturation
and must reject before admission, even if hyperbolic distance increases. No
clamping or accepted zero margin. This is a component stress test, not a
quality or energy result.

The Rust implementation must reject non-finite inputs, checked-arithmetic
overflow, nonpositive boundary margin, and any radius regression. It must not
clamp a boundary crossing into a passing receipt. Float saturation near the
ideal boundary is a failure requiring a revised radial budget or precision,
not an excuse to report equality as continued expansion.
