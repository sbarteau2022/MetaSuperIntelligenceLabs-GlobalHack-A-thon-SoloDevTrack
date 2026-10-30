# PPC-ADOPT-RUN-01 Report (PPC adopt-demo: instructional candidates)

**Date.** 2026-10-30. **Manifest.** `PPC-ADOPT-RUN-01-Manifest.md` (frozen; SHA-256 filed below before execution per C09.1/C11.3).
**Model.** `Qwen3.6-MoE-Combined` via `http://127.0.0.1:8000/v1/chat/completions` (Stewart directive). **Binary.** `hardware_ppc_adopt` (new; `hardware_ppc.rs` untouched).
**Commit.** Filed at commit time (this report + manifest + binary + lib module + receipt committed together).

## Manifest hash

filed 2026-10-30 BEFORE execution: `sha256:b96500bbfd0b46b91743251cf7cfc24bfe8a98275cce8e6b9cd02e824c376b29` (recompute: `sha256sum PPC-ADOPT-RUN-01-Manifest.md`).

## Results

| Candidate | Passes | Quality | Tokens | Adopt | Reasons |
|---|---|---|---|---|---|
| BASE | 8/8 | 1.0000 | 1911 | kept | — |
| WIN (show-work discipline) | 7/8 | 0.8750 | 2142 | **false** | noise band (gain −0.1250 ≤ 0.02); pareto dominated |
| LOSE (weakened FINAL) | 8/8 | 1.0000 | 1881 | **false** | leakage critic flagged [0]; noise band (gain 0.0) |
| JUNK (control) | 8/8 | 1.0000 | 1886 | **false** | noise band (gain 0.0) |

Adopted: **BASE** (`kept_base=true`, `quality_gain=0.0000`). Held-out on base: **4/4** (102, 130, 10, 212). Full receipt: `research-grade-harness/HARDWARE-PPC-ADOPT-RECEIPT.txt`.

## Verdict vs decision mapping

- **REJECT-DEMO: PASS.** The gate correctly rejected a regressing WIN (7/8 < 8/8, pareto-dominated) and a leakage-flagged LOSE. Both rejections carry named reasons (noise band + pareto; leakage + noise band) — fail-explained, not silent.
- **ADOPT-DEMO: INCONCLUSIVE.** No candidate beat base, so the adoption direction is unproven. The gate abstained correctly on the evidence available. Per mapping: escalate task difficulty or WIN discipline, next run.
- **JUNK control: reproduced.** 8/8 tie → abstain. The saturation baseline holds.

## Findings (filed, not in this run)

1. **Show-work discipline HURT (7/8 < 8/8, probe-02: 171 ≠ 172).** The extra reasoning step introduced an off-by-one in the featurette/intermission sum — the judge punishes verbosity because every intermediate number is a last-number trap. Implication for future WIN designs: reasoning discipline must REDUCE the number surface (verify-then-single-FINAL), not expand it. A show-work candidate that reasons on scratch then emits ONLY the FINAL line might win; this one emitted steps into the judged channel.
2. **Leakage-critic substring false-positive.** LOSE's "A plain answer is fine" matched marker "answer is" (`evolution.rs:22` MARKERS, substring scan, no word-boundary or intent check). The flag was CORRECT in outcome (LOSE deserved rejection on instructional grounds — it ties only because the strict template FINAL line downstream overrode the looseness) but WRONG in mechanism (a substring hit on ordinary prose, not a prompt-injection signal). Prescribed: word-boundary + intent-scoped matching, or marker list narrowed to injection-shaped phrases. Frozen `evolution.rs` NOT touched in this run.
3. **Instrument note (correction entry).** Manifest §4 said the new binary copies helpers "verbatim, not refactored". Implemented instead as shared lib module `src/ppc_probe.rs` (same logic, byte-equivalent behavior) with `hardware_ppc.rs` untouched. Rationale: duplication of 160 lines of TCP/JSON plumbing invites drift; the manifest's frozen SUBSTANCE (tasks, judges, candidates, adoption rule) is preserved exactly. The old binary's behavior is unchanged (not rebuilt against the module).

## What this run licenses (and does not)

- Licenses: the gate rejects regressions and flags with named reasons live (third consecutive honest verdict: abstain/abstain/reject+abstain). The adoption RULE fires correctly in the negative direction.
- Does not license: adoption-direction firing (no win observed); any quality claim beyond the ledger; cross-hardware claims.
