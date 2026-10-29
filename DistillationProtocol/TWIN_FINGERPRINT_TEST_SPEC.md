# Twin-Fingerprint Invariance Test Spec (Theorem 5)
**Date.** 2026-10-28. **Status.** SPEC (prescribed, not yet run).

## Claim under test
Distill identical trajectory *content* under two distinct teacher fingerprints `fp_A`, `fp_B`. Assert:
1. Projected coordinates are byte-identical (cosine distance 0.0) for every step.
2. Atoms coalesce to a single atom (salience +1), not two teacher-duplicate atoms.
3. Retrieval ranking is identical whether keyed by coordinate or by any map-facing key (teacher fields not consulted).

## Procedure
- Same hidden-state + next-op-prob sequence, run through `distill_reasoning_chain` twice with differing `teacher_fingerprint`.
- Capture `atom_sequence`, coordinates, role sequences.
- Assert equality of coordinates and single-atom coalescence across runs.
- Run identity-invariant grep over mint/retrieval paths (PROTOCOL §8): no `teacher_id`/`model_name`/`fingerprint` read outside lineage+audit+receipts.

## Pass condition
All three assertions pass AND grep returns audit-fields-only. Fail → run void (PROTOCOL §8).
