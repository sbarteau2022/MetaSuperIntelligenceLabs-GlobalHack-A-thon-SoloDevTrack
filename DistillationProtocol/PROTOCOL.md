# Distillation Protocol — Varying Models → Single Memory Space

**Status.** PROPOSED, PRESSURE-TESTED 2026-10-27 (probe report in `AUDIT.md` corrections). **Hold point.** Model roster required before execution (see §4). **Boundary (ordered 2026-10-27).** Docs + validation ONLY this pass — NO distillation into any live map. No vault file was read for writing, no `distill_*` entry point was invoked, no map merge executed.

## 1. Purpose

Bring multiple teacher models (varying sizes/architectures) into one shared carved memory space without fine-tuning. Each model's knowledge is distilled into the same (H³ × T²)/Γ₁₂ map under a common coordinate scheme so retrieval is model-agnostic.

## 2. Steps (ordered, judge-gated)

1. **Register teacher** (model name, fingerprint, num_layers, quant dtype) in roster file.
2. **Load frozen weights** (read-only; no mutation). Verify fingerprint.
3. **Pillar 1** `distill_static_weight_modes` → mint weight-mode atoms (greedy top-k by retained energy if §AUDIT finding 1 adopted).
4. **Pillar 2** `distill_reasoning_chain` → mint reasoning arcs (0/1/2-cells) from trajectories if available.
5. **Novelty gate + MGS orthogonalization** per atom.
6. **Judge gate** per packet; mint only on pass; emit receipt.
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
