---
name: swarm-brigade
description: Run the full Meta-to-swarm pipeline in one master skill — decompose the goal, preregister the plan, carve fibers and orchestrate them with budgets and receipts, then gate quality and certify. Use when spawning any subagent swarm, executing any multi-step plan with agents, or certifying swarm work.
---

# SwarmBrigade — Meta-to-Swarm Master Skill

One master skill encompassing four phases: **Decompose → Preregister → Orchestrate (carve + tactical run) → Gate + Certify**.
Meta (you) owns all four. Fibers own exactly one assigned task each.

Bound by the engineering governance core: DECLARE, RETRIEVE, GATE, PARTITION, PROVE, RECORD, WAIVE-OR-OBEY.
(FORBID's ban-list is domain paint — the repo JSON/Python/tokenizer bans apply at the model I/O boundary per project rule, not from this skill.)

## Operating law (L-01, repo log of record)

- Meta owns: task-decomp, swarm assignment (one task per fiber, disjoint write scopes, budgets), quality gating (receipts per step, kill-and-restart on wild spend or coherence loss), certification at the end (pressure test, numbers, verdict).
- Fibers own: exactly one assigned task, read-only unless their task is the edit task, budgeted, receipted. A fiber with no mappable tool stops and names the missing capability (hunger gap) — never approximates.
- Clip rule: any fiber exceeding scope, writing outside assignment, or drifting incoherent is killed and restarted with a narrower task. No second chances inside the same fiber.
- Certification is meta-only: fibers report, meta verifies (recounts, reruns, greps). A fiber's self-reported PASS is REPORTED until meta re-checks it.

## Phase 1 — Decompose (from task-decomp)

1. Write the goal in one sentence. If you can't, ask the user (don't guess).
2. Decompose to leaves. Each leaf is one verb + one object. Stop when a leaf maps to exactly one tool call (or exactly one fiber task).
3. State each leaf's information need: required sub-problems, then the minimal set covering them (coverage default 0.8; depth from the verb: retrieve = 1, chain = hops, synthesize = 2+).
4. Necessity test: name the ONE fact whose absence voids the leaf. No load-bearing fact = leaf too big — split it.
5. Depth-budget check: never pay depth-3 cost for a depth-1 leaf; never starve a depth-3 leaf with depth-1 budget.
6. Emit the task tree: goal, sub-tasks, leaves with necessity fact + assigned executor. Unassigned leaves fail at plan time, never mid-run.

## Phase 2 — Preregister (from preregistration practice)

Before any fiber runs, file the plan:

1. Falsifiable hypothesis per leaf (what would void it, with a named threshold — e.g. R ≥ 1.20 or it demotes to OPTIONAL).
2. Workload + hardware + baseline (exact corpus, exact machine, exact comparison commit or path).
3. Judge + judge band (conjunctive judges, named band) and effect-size plan.
4. Null-handling (what a miss means — recorded as missed, never silent).
5. Claims-register draft: every claim pre-labeled VERIFIED / REPORTED / INFERRED / PROPOSED with its evidence pointer. Labels are corrected by entry, never rewritten (ADR-19).
6. No hash, no run. Any edit to frozen text forks a new version + new hashes. Bars from swarm-phase0 apply where relevant: prefix sharing ≥ 70% where spawns share roles; correctness ≥ 2/3 runs; swarm not worse than single-agent paired. Failure handling: at most two fix-and-repeat cycles with bars unchanged; a missed bar is reported as missed.

## Phase 3 — Orchestrate (from carving-the-swarm + tactical-orchestration)

### 3a. Contracts first (from SUBAGENT_SWARM_LOGIC)

Shared contracts exist on disk and are reviewed before any parallel fiber starts. Coders never self-approve. For code work: types/store/contracts land first; parallel builders get non-overlapping file ownership; the QA critic runs last against the live artifact.

### 3b. Carve

1. Classify the task (code / memory / workflow / research / security / general). Non-general classes restrict to their required domains.
2. Carve a task-scoped fiber view: score basins by relevance × salience × geo-gate × necessity, keep necessity ≥ 0.50 plus pinned, truncate atom-budget pinned-first. Weights are never duplicated.
3. Verify the carve receipt: weight-copies == 1, no raw atom text in digests, global count unchanged, views heterogeneous across tasks.

### 3c. Tactical run

1. Map each leaf to exactly ONE executor (a map tool, a carved fiber, or a traversal arc) with energy + token + depth budgets. Disjoint write scopes for parallel fibers.
2. Order by dependency. A leaf runs when its inputs' receipts exist.
3. Receipt everything (`receipt_sha256`). No receipt = the step didn't happen.
4. Handle fail-explained: over-budget → raise or split; form violated → carve the missing arc first, then re-traverse; frame shear → smaller hops. Never retry identically.
5. Sundowning watch past ~40 steps: re-check every traversal at dispatch; a mid-plan fail-explained means re-ground, never push through.
6. Close out: consolidate (reinforce used, decay untouched floor 0.05, sate covered gaps), dissolve fibers (fingerprint XOR must match — mismatch = quarantine, never force-merge), snapshot.

### 3d. Budgets (defaults — override per leaf, record the override)

Traverse energy 16.0 (default) / 100.0 (generous) / 0.0 (probe-only, never production). Frame eps 1e-2 dispatch / 1e-3 audit. Form tol 1e-9 (leave it). Attend depth data-dependent; explicit k only to bound cost. Fiber word + tool-call budgets set per assignment; exceeding either is wild spend → kill and restart narrower.

## Phase 4 — Gate + Certify (meta-only)

1. Re-verify, don't trust: recount fiber numbers, re-run their key commands, grep their absence claims. Fiber PASS = REPORTED until meta checks it.
2. Pressure test: full suite green (repo gates: crate tests + observability suite + harness tests), `git status` clean of fiber writes outside assignment.
3. Coherence check: no fiber wrote outside its scope, no manifest/registry drift unrecorded, no silent drops (loss counter-dead rule: every loss lands in a receipt or a ledger row).
4. Certify with numbers or don't certify: pass/fail counts, measured latencies with hardware + workload + comparison, open leftovers listed with labels. A recorded failure beats an asserted success.

## Failure doctrine (whole skill)

- Leaf with no mappable tool → stop, name the hunger gap. Never approximate.
- Fiber wild spend / scope breach / incoherence → kill, restart narrower.
- Dissolve void → quarantine, re-read, re-dissolve. Never force-merge.
- Missing receipt mid-plan → halt the branch, re-run the leaf.
- Bar missed → reported as missed. Two fix cycles max, bars unchanged.
