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

- This `Rules.md` applies to `C21-Performance-Budgets-Energy` and its subcategories.
- Deep-dive details live in `Design.md`.
- Contributor additions (Muse, 2026-10-26):

11. **Normalization limits:** no cross-architecture or cross-model rescaling; energy never normalized (C21.8).
12. **SLO provenance:** targets are PROPOSED until baselined; qwen36 evidence files carry TEMPLATE status until populated (C21.9).
