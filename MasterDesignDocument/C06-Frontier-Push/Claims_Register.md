# C06 — Claims Register

**Status.** OPEN register. **Date:** 2026-10-26.

[PROPOSED] Claims are append-only. Corrections supersede claims by entry; they never rewrite history (ADR-19).

| Claim ID    | Claim                                                                                                                                                                   | Label    | Evidence pointer                                                         | Status | Notes                                                                 |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------ | ------ | --------------------------------------------------------------------- |
| CC-001      | A Continuity Capsule separates an exact, hash-validated critical state from optional approximate supporting recall.                                                     | PROPOSED | `Design.md`, C06.17                                                      | OPEN   | Design novelty; no implementation evidence yet                        |
| CC-002      | The capsule preserves all canonical critical fields through two compactions and one restart.                                                                            | PROPOSED | `Pre-Registration.md`, A38/M1                                            | OPEN   | Requires A38 pass on two hardware targets                             |
| CC-003      | The capsule manifest and parent chain detect corruption and recover the latest valid parent without silent data loss.                                                   | PROPOSED | `Design.md`, C06.17; `Pre-Registration.md`, A38/M2 and M5                | OPEN   | Requires crash/torn-tail fixtures                                     |
| CC-004      | Capsule-backed recall emits a verifiable evidence path for every supporting atom.                                                                                       | PROPOSED | `Design.md`, C06.17; `Pre-Registration.md`, A38/M3                       | OPEN   | Requires ledger-root linkage test                                     |
| CC-005      | The capsule treatment reduces median prompt tokens by at least 50% versus the flat-transcript arm under the A38 paired workload.                                        | PROPOSED | `Pre-Registration.md`, A38/M4                                            | OPEN   | Performance claim; no measurement exists                              |
| CC-006      | The standard-transformer fallback consumes the same capsule view with no critical-state loss.                                                                           | PROPOSED | `Design.md`, C06.17; `Pre-Registration.md`, A38/M6                       | OPEN   | Must not require the optional Atlas adapter                           |
| CC-007      | Atlas telemetry and metrics contain no raw continuity-payload fields.                                                                                                   | PROPOSED | `Design.md`, C06.17; `Pre-Registration.md`, A38/M7                       | OPEN   | Canary scan required                                                  |
| CC-008      | Hyperbolic geometry is an optional accelerator for supporting recall and is not the correctness authority for exact continuity.                                         | INFERRED | `Design.md`, C06.17; C24 checkpoint contract; C05/C35 geometry contracts | OPEN   | Architectural inference; implementation evidence is not yet available |
| CC-009      | C06.17 can become required only after A38 passes, replicates on a second machine, and passes a necessity ablation.                                                      | PROPOSED | `Design.md`, C06.17/C06.18; `Pre-Registration.md`, A38 decision rule     | OPEN   | Implements the C06.13 promotion ladder                                |
| CORR-C06-01 | Exact trajectory reconstruction is replaced by exact critical-state recovery through the Continuity Capsule; geometric basin recovery remains optional and approximate. | PROPOSED | `Design.md`, correction entry CORR-C06-01                                | OPEN   | Correction to the C06.1 basin-recovery implication                    |
| CORR-C06-02 | A VP-tree traversal is not, by itself, a Merkle proof; capsule recall proofs must terminate in ledger receipt roots.                                                    | PROPOSED | `Design.md`, correction entry CORR-C06-02                                | OPEN   | Correction to the C06.3 wording                                       |
| CORR-C06-03 | C06 items require one-to-one ablation ownership; the legacy matrix entries that reuse A1/A4/A5/A7/A9/A11 are superseded by the corrected ownership table.               | PROPOSED | `Design.md`, correction entry CORR-C06-03                                | OPEN   | Prevents one ablation from earning several distinct claims            |

## Registration Evidence

| Registration | Canonical file               | SHA-256                                                                   | Filed date | Status                |
| ------------ | ---------------------------- | ------------------------------------------------------------------------- | ---------- | --------------------- |
| A38-v1       | `Pre-Registration-A38-v1.md` | `sha256:28716ac077059c248a0720565dde032cc4341212a0ec82ab8543b4155a89eea8` | 2026-10-26 | SUPERSEDED, PRESERVED |
| A38-v1.1     | `Pre-Registration.md`        | `sha256:0078d3995c920329f67b89b04b3ae8951763402a7eea59f0a7ae3d9ee8920004` | 2026-10-26 | FILED, CANONICAL      |

## Run Evidence

[VERIFIED by workspace inspection] No A38 run has occurred in this workspace. [VERIFIED] No C06 performance, continuity, privacy, or necessity claim is VERIFIED.
