# C06 — Failures

**Status.** OPEN ledger. **Date:** 2026-10-26.

[PROPOSED] Failures and corrections are retained as entries; they are not silently removed.

## F-C06-01 — Exact trajectory reconstruction was not justified by the stated basin/gradient-flow construction

- **Date.** 2026-10-26.
- **Finding.** [INFERRED from the C06.1 equations] Recovering a trajectory by gradient flow on distance to the union of trajectory neighborhoods does not, as written, guarantee recovery of trajectory order or distinguish a trajectory from another trajectory contained in the same union. The C06.1 claim of exact reconstruction after two compactions therefore lacks a sufficient proof premise.
- **Evidence.** [VERIFIED by reading `Design.md` C06.1] `B_γ` is defined as a union of balls and recovery is defined as gradient flow on `d(x, B_γ)`; no injective encoding or uniqueness argument is present.
- **Consequence.** [PROPOSED] Exact continuity moves to the hash-validated critical-state capsule (C06.17). Geometric basin recovery remains optional and approximate unless a separate proof and ablation are filed. Replacement is recorded as CORR-C06-01 in `Claims_Register.md` and `Design.md`; the original text remains above.

## F-C06-02 — The original verification matrix reused ablations across distinct mechanisms

- **Date.** 2026-10-26.
- **Finding.** [VERIFIED by reading the C06 verification matrix and C09 preregistrations] Distinct C06 claims were mapped to shared A1/A4/A5/A7/A9/A11 entries even though their hypotheses and judges differ. This conflicts with the C06.13 one-to-one ownership rule.
- **Evidence.** [VERIFIED] For example, C06.2 geometric forgetting was mapped to A1 although A34 is the registered geometric-forgetting ablation; C06.5 interference was mapped to A5 although A20 is the registered interference ablation; C06.12 was mapped to A4 although A21 is the registered brigade-weight ablation.
- **Consequence.** [PROPOSED] The legacy matrix is superseded by the corrected ownership table in CORR-C06-03. Existing A2, A20, A21, and A34 remain attached to their matching claims. Claims without a dedicated registration remain PROPOSED and cannot advance.

## F-C06-03 — C06.15 and C06.16 referenced A36/A37 without preregistration text

- **Date.** 2026-10-26.
- **Finding.** [VERIFIED by workspace inspection] C06.15 and C06.16 propose A36 and A37, but the shared prereg folder (`PreRegistrations/`, now pinned top-level) contained registration text only through A35 before this contribution.
- **Evidence.** [VERIFIED] `PreRegistrations/PRE-REGISTRATION-A1-A10.md` and `PreRegistrations/PRE-REGISTRATION-A11-A35.md` (now at repo-top `PreRegistrations/`) are the only shared preregistration files present.
- **Consequence.** [PROPOSED] A36 and A37 cannot earn claims until their hypotheses, baselines, judges, and hashes are filed. C06.17 is registered separately as A38 in this workstream.

## F-C06-04 — No experimental evidence exists for the frontier claims

- **Date.** 2026-10-26.
- **Finding.** [VERIFIED by workspace inspection] This workspace contains design artifacts but no A38 implementation, benchmark output, judge receipt, cross-hardware replication, or necessity result.
- **Evidence.** [VERIFIED] `Claims_Register.md` has no run-evidence pointer and A38 explicitly records that no execution occurred.
- **Consequence.** [VERIFIED] Every performance, correctness, privacy, and necessity assertion about the Continuity Capsule remains PROPOSED. The safe fallback remains authoritative until evidence is produced.

## F-C06-05 — Evidence ownership is not fully closed across all C06 items

- **Date.** 2026-10-26.
- **Finding.** [VERIFIED by reading C06, C09, and the shared preregistration files] C06.3, C06.4, C06.6, C06.7, C06.8, C06.9, C06.10, and C06.11 do not yet have dedicated, text-complete preregistrations.
- **Evidence.** [VERIFIED] The corrected ownership table in CORR-C06-03 marks these items `UNREGISTERED`; A36/A37 text is absent as recorded in F-C06-03.
- **Consequence.** [PROPOSED] The 10-day critical path must not depend on those items. They may remain research backlog or receive registrations before any run.

## F-C06-06 — A38-v1 left two judge-protocol details underspecified

- **Date.** 2026-10-26.
- **Finding.** [VERIFIED by adversarial review of the filed A38-v1] M1 referred to canonical field equality without defining how the agent response is parsed, and M3 did not state whether it applies to an Atlas-disabled fallback run.
- **Evidence.** [VERIFIED] A38-v1 is preserved verbatim at `Pre-Registration-A38-v1.md` with SHA-256 `28716ac077059c248a0720565dde032cc4341212a0ec82ab8543b4155a89eea8`; its M1/M3 rows contain those omissions.
- **Consequence.** [VERIFIED] No run occurred under v1. A38-v1.1 is the canonical pre-run registration with SHA-256 `0078d3995c920329f67b89b04b3ae8951763402a7eea59f0a7ae3d9ee8920004`; it defines strict-JSON parsing and M3 applicability without changing baselines, workload, thresholds, or decision rules.
