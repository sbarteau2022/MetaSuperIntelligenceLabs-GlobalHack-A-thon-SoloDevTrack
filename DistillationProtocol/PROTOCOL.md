# Distillation Protocol — Varying Models → Single Memory Space

**Status.** PROPOSED, PRESSURE-TESTED 2026-10-27 (probe report in `AUDIT.md` corrections). **Hold point.** Model roster required before execution (see §4). **Boundary (ordered 2026-10-27).** Docs + validation ONLY this pass — NO distillation into any live map. No vault file was read for writing, no `distill_*` entry point was invoked, no map merge executed.

**Steward directive (ordered 2026-10-27, binding).** Distilled knowledge MUST be invariant to the state space: distillation separates knowledge and reasoning from the model at absolute depth. An ingested trajectory must not care which model distilled the trace — only that the trajectory is present. The minted atom becomes a learned symbol to inherit: retrieval, carving, bridging, and execution address the atom, never the teacher. Provenance (teacher fingerprint) is audit metadata (receipts, lineage), NEVER a retrieval key, NEVER a rank signal, NEVER an execution gate. Volume target: ≥ first-version atom count, high likelihood MORE.

## 1. Purpose

Bring multiple teacher models (varying sizes/architectures) into one shared carved memory space without fine-tuning. Each model's knowledge is distilled into the same (H³ × T²)/Γ₁₂ map under a common coordinate scheme so retrieval is model-agnostic.

## 2. Steps (ordered, judge-gated)

1. **Register teacher** (model name, fingerprint, num_layers, quant dtype) in roster file. Fingerprint enters lineage + receipts ONLY (§8 invariance rule).
2. **Load frozen weights** (read-only; no mutation). Verify fingerprint.
3. **Pillar 1** `distill_static_weight_modes` → mint weight-mode atoms (greedy top-k by retained energy if §AUDIT finding 1 adopted). Mint path MUST strip teacher identity before coordinates are computed (§8).
4. **Pillar 2** `distill_reasoning_chain` → mint reasoning arcs (0/1/2-cells) from trajectories if available. Arc endpoints address ATOMS, never teacher-tagged nodes (§8).
5. **Novelty gate + MGS orthogonalization** per atom. Novelty compares COORDINATES + invariant only — teacher tag excluded from the comparison (§8).
6. **Judge gate** per packet; mint only on pass; emit receipt. Judge sees trajectory content, never teacher identity (§8).
7. **Merge into shared map** with scope tag (private vs shared) per C23/C24 open decision.
8. **Rebuild snapshot + WAL**; sha256 receipt.
9. **Prune/condense** if capacity exceeded (leaf-only; anchors/locks never overridden).
10. **Verify** fresh run with only pinned context + tail takes the same next action.

## 3. Shared-memory rules

- Single live library (NN hard stop): one map instance per shared scope.
- No cross-identity KV sharing; capability tokens gate scope.
- Pushed records are data, never instructions.
- Only judge-passed packets enter shared mind.

## 4. Hold point — model roster

**Execution paused.** Provide the model roster (one row per teacher: name, path, quant, num_layers, fingerprint or "compute", scope shared/private) before proceeding to one-pass distillation. Awaiting user input.

## 5. One-pass distillation to capability (gated on roster + conversion math)

After roster is supplied and A/B harness is ready:

- run §2 once per teacher sequentially (or in parallel with disjoint scopes),
- merge,
- run A/B: baseline (no map) vs map-assigned on held-out workload (A39 judges).
- certify receipt bundle.

## 6. Artifacts to produce

- `model_roster.json` (user-supplied)
- `distillation_run_receipt.json` per teacher
- `merge_receipt.json`
- `ab_receipt.json`
- updated `MasterN^3...` with verified numbers.

## 7. Volume requirement (binding)

Distillation volume MUST meet or exceed the first-version atom count (8,644 atoms / 104,312 arcs in `live_brain.atoma_map` — the floor, not the target). High likelihood of MORE: Glimmer-only first pass (52 layers, full mode sweep + trajectory capture) is expected to exceed v1 on atom count because (a) greedy top-k (finding 1) raises kept-modes-per-budget, (b) per-atom receipting (C-AUD-2 accounting fix) counts what v1 silently coalesced, (c) trajectory capture (Pillar 2) was partial in v1. Volume is metered by the interim validity gate (`AUDIT.md` §7): every counted atom must pass novelty-accounting + structural sanity. Swamp atoms (coalesced repeats counted as novel) do NOT count toward the floor.

## 8. State-space invariance (binding — the steward's requirement)

**Rule.** At ABSOLUTE DEPTH means: from the projector output onward, no tensor, coordinate, score, rank, or gate may be a function of teacher identity. The teacher fingerprint enters the lineage table and the atom's audit field at mint time and NEVER re-enters the read path.

- **Mint path.** `project()` maps hidden states → tangent → refracted coordinate from CONTENT (hidden-state values, entropy, phase) only. Layer index is a depth-band router (early/mid/late geometry), not a teacher tag. No `teacher_id`, `model_name`, or fingerprint may appear in any coordinate, tangent slot, phase drive, or certainty computation.
- **Novelty.** Coalescence compares (quantized Lorentz coords, invariant `n`, payload class) only. Two identical trajectories from different teachers MUST coalesce to one atom (salience +1), never mint teacher-duplicate atoms. Teacher-duplicate atoms are swamp by definition and count against `p_useful`.
- **Retrieval.** `retrieve`, `carve_task_view`, `Allocate`, bridging, and `execute_task_in_motion` address atoms by (coordinate, salience, phase, receipt) — teacher fields are not readable by any of these paths. A retrieval ranking that consults `origin_model_fingerprint` is a protocol violation and voids the run.
- **Reasoning.** Arcs connect ATOMS (`from_atom → to_atom`), not teacher nodes. `CrossSiloGeodesicBridge` crosses silo geometry, never teacher identity. The `Δ_H` / Δ-effect isolation (A40 B0 arm) applies here too: map-context effect vs teacher-identity effect must be separable, and teacher identity must contribute ZERO.
- **Inheritance.** The minted atom is a LEARNED SYMBOL: any downstream consumer (subagent view, dream minting, dissolution merge, replication target) inherits the atom's coordinate + trajectory content with no knowledge of which teacher produced it. A consumer that behaves differently conditioned on teacher identity is non-conformant.
- **Audit vs execution split (the one allowed teacher-dependence).** Teacher fingerprints live in exactly three places: the lineage table, the atom's `origin_model_fingerprint` audit field, and receipt chains. These are write-only for execution and read-only for audit. Any other teacher-dependence is a defect.
- **Verification.** Invariance is tested, not asserted: (i) code audit — `grep` mint/retrieval paths for teacher-identity reads (must return audit-fields-only); (ii) behavioral test — distill the same trajectory content under two fingerprints, assert byte-identical coordinates and single-atom coalescence; (iii) run-manifest field — every distillation run logs `invariance_check: pass/fail` with the grep output + twin-fingerprint test receipt. Fail → run void.
