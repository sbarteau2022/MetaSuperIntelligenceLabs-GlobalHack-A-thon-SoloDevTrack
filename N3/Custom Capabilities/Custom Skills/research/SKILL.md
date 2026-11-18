---
name: research-runbook
description: Handle evidence with per-source provenance, separated observation from proposal, and labeled synthesis.
user-invocable: true
depends_on: [runbook-plan-and-judges]
load_mode: eager
---

# Research Runbook — Evidence Handling

Date: 2026-10-30. Status: PROPOSED.

Objective: answer a question with traceable evidence where every claim carries a label and every source carries provenance.

## Procedure

1. **Question.** State the single research question and what decision it unblocks.
   - Judge: `nonempty` — question recorded; `contains` — the downstream decision named.
2. **Criteria.** Pre-register acceptance criteria (what evidence would settle the question) BEFORE gathering sources.
   - Judge: `file_exists` — criteria block dated before the first source row.
3. **Provenance row per source.** For each source record URL/version/license/claim in one row; no sourceless claims admitted.
   - Judge: `contains` — every claim cites a row; every row has all four fields (URL, version, license, claim).
4. **Separate observed vs proposed.** Mark each finding VERIFIED (measured/read), REPORTED (a source says so), INFERRED (reasoned — state where evidence ends), or PROPOSED (a plan, not a fact).
   - Judge: `contains` — zero unlabeled claims; INFERRED items state their evidence boundary.
5. **Synthesis with labels.** Write the answer as labeled claims only; corrections are appended entries, never silent edits.
   - Judge: `file_exists` — synthesis block; `contains` — every paragraph carries at least one claim label.

## Exit criteria

All criteria from step 2 are dispositioned (met/unmet with evidence), every claim labeled, every source row complete. Unlabeled claim = step failed.

## Failure path

Missing provenance or unlabeled claim → Strike 1: pause, re-approach by re-sourcing (different source or direct read, never re-asserting). Repeated gap on the same claim → Strike 2 per `02-two-strike-tactical-escalation.md`: escalate with the evidence gap named, dependents `skipped` until resolved.
