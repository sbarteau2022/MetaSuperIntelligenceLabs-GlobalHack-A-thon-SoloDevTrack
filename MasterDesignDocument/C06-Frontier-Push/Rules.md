# Rules

These rules apply to this workstream/category and inherit the first-principles backbone (C03).

1. **Append-only truth:** events/workflows/ledgers reject update/delete; corrections are appended, never rewritten.
2. **Verifiable execution:** mutating actions run in holes, produce Merkle receipt + judge verdict, and are approval-gated where required.
3. **Evidence over claims:** no metric without workload/hardware/baseline; no badge without a receipt.
4. **Least privilege:** tools are data; risky execution stays behind seatbelt + approval; secrets by slot, never literal.
5. **Determinism:** resolve v2 validates tool refs deterministically; receipts are hash-derived.
6. **Cascade memory:** Org Policy → User → Project Rules → Session Auto; tight context budget.
7. **Security/compliance before build:** threat model, role matrix, audit log, erasure plan precede feature code.
8. **No component ships as required** unless a necessity ablation passes; otherwise optional/off-by-default with measured cost (C13.1).
9. **Telemetry privacy:** counts/ids/latency only; never args, results, or text.
10. **Claim labeling:** VERIFIED / REPORTED / INFERRED / PROPOSED with evidence; corrections are entries, not edits.

## Category-specific scope

- This `Rules.md` applies to `C06-Frontier-Push` and its subcategories.
- Deep-dive details live in `Design.md`.
- Contributor additions (Muse, 2026-10-26):

11. **Promotion ladder:** no C06 item becomes required without passing every rung (prereg → pass → replicate → necessity); skipping a rung disqualifies the claim.
12. **Interference statement:** every Atlas-touching item states eviction / weight-copy / privacy effects (C06.14) or it cannot advance past PROPOSED.
13. **Fusion rule:** an item fusing two parents (e.g. C06.15) must beat both parents on the shared workload or be withdrawn.
14. **Exactness boundary:** every C06 mechanism must identify which state is exact, which state is approximate, the authoritative store for each, and the recovery path when integrity verification fails.
15. **Geometry is not correctness by default:** exact continuity state must recover without ACFT, hyperbolic retrieval, or self-tuned curvature; geometry may accelerate supporting recall only.
16. **Sprint sufficiency:** during the 10-day sprint, a new C06 mechanism enters the critical path only when it strengthens the selected demonstration thesis or is already required by another frozen contract; research-only mechanisms remain off the critical path.
17. **Evidence ownership:** each C06 claim maps one-to-one to a text-complete preregistration and judge. Legacy reused mappings are superseded only by an appended correction entry.
