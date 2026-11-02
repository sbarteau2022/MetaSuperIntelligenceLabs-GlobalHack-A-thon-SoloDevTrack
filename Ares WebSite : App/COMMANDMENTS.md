# The Ten Commandments — Systems-Enforced Rules

Version: 2026-11-02. Status: BINDING. These are gates, not guidance.
Enforcement: `scripts/verify/gate.sh` (run via `npm run gate` or pre-commit).
A failing gate stops the lane and emits a why-receipt (see §Receipts).
No commandment may be bypassed by an agent or by code. Waivers require a
dated, scoped, expiry entry in `FAILURES.md` BEFORE proceeding (never §6
of the global law).

## I. Follow through on stated tasks

- Every task stated in a plan, PR description, or agent message is tracked in `TODO.md`.
- A task marked DONE requires an evidence pointer (commit sha / file:line / checker output). No evidence = stays OPEN.
- Deferred work is declared upfront in `TODO.md` with a reason and an owner; silent dropping is a violation.

## II. Never change evaluation conditions to flatter results

- Evaluation scripts, thresholds, inputs, and ground truth are frozen per experiment run.
- Any change to evaluation conditions invalidates prior results and must be recorded as a new run with a new run id.
- Detectable via git diff on `scripts/eval/` and `tests/`; gate checks for uncommitted changes there.

## III. Verify literature — no hallucinated citations

- Every citation in `docs/report.tex` or code comments must resolve to a real source (DOI/URL) stored in `docs/citations.bib`.
- Gate runs `scripts/verify/check_citations.sh` which fails on unknown keys.

## IV. Don't stop to ask whether to continue when the plan has more work

- Process rule enforced by runbooks: a pending checklist item is executed unless only Stewart can clear its blocker.
- Agent-level: no "should I continue?" prompt mid-batch; blocker declared or item completed.

## V. Make it work — investigate failures

- A failing approach is investigated (logs, minimal repro) before being declared unworkable.
- Failure classification recorded in `FAILURES.md` with evidenced cause, never "doesn't work."

## VI. Change exactly one variable per experiment

- `experiments/` entries declare exactly one changed variable (`changed_var`) in their manifest.
- Gate `scripts/verify/check_experiment_manifest.sh` rejects manifests with 0 or >1.

## VII. Staged evaluation

- Every experiment has a `sanity` stage (< 5 min) before any `full` stage.
- Results of sanity must be recorded; an idea is not discarded on a single toy failure (must be tried with a fix).

## VIII. Compute oracle / theoretical upper bound first

- `docs/report.tex` Section 1 must state the oracle/upper bound before any method.
- Gate checks that the oracle subsection exists and is non-empty.

## IX. If it's not in the report, it didn't happen

- `docs/report.tex` plus `TODO.md` are the canonical record.
- Every shipped result appears in `report.tex` with evidence; negative results documented like positives.
- `SUCCESSSES.md` / `FAILURES.md` are append-only; corrections are dated entries, never silent rewrites.

## X. Verification scripts before claims

- Any numeric claim in code, docs, or UI must have a verification script under `scripts/verify/` checked against brute force on small cases.
- Every new formula ships with a `check_*` script; gate enforces presence by glob match (see `scripts/verify/registry.txt`).

## Domain extras — Compute

- One experiment per GPU (no oversubscription); log-file hygiene (no logs >100MB in repo); OOM mitigation documented (batch size backoff).

## Domain extras — Math

- Derivations before code; precise notation; counterexample-first reasoning (every theorem attempt preceded by search for counterexample).

## Receipts

Every failure emits a why-receipt: what ran, command/route, exit/status, last
output, expected result, evidenced cause, unblocker. Stored append-only in
`FAILURES.md` and in SurrealDB `receipt` table once Phase 2 lands.

## Locks (incorporated, binding)

- Store lock (SYSTEM_WORKFLOW_LOCK.md): SurrealDB 8000 mind graph, Qdrant 6333 RAG, HyperDX 8080 telemetry; never collapse; no Postgres in v1; no cloud relay.
- Port lock (ROADMAP.md): 8000 SurrealDB only; 4810 backend; 6331 stale.
- Typed workflow (SYSTEM_WORKFLOW_LOCK.md): mission → packet → lane → judge → receipt → atom; N3 load order; no synthetic runs ship.
- Holomorphic math lock (docs/HOLOMORPHIC_FUNCTION_SPACE.md): see normative lock; runtime-cohomology claims forbidden (Commandments III and X).
- Evidence labels: VERIFIED / REPORTED / INFERRED / PROPOSED on every claim.
- Landed means live: verified against running process, routes, and UI, not a file on disk.

## Enforcement entry points

- `npm run gate` → `scripts/verify/gate.sh`
- `.git/hooks/pre-commit` → `scripts/verify/gate.sh`
- `.github/workflows/verify.yml` → CI gate
- Zed task / hook (optional) → calls same script
