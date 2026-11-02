# SUBAGENT-LIFECYCLES.md — Spawn → Work → Verify → Merge

Date: 2026-10-30. Status: PROPOSED.

Authority: `SUBAGENT-CONTRACT.md` §§1–6 (delegation law); `SUBAGENT-PROTOCOLS.md` (roles + routing). PARTITION-first throughout: one mind per vertical whole-slice (rule 60a); siloed swarms REQUIRED.

## 1. Lifecycle (no phase skipped, no phase self-certified)

- SPAWN: 5-field delegation (§2) + CONTRACT-GATE green (§3) + write-sets pairwise disjoint on paper — else re-partition before fan-out, never after.
- WORK: the Coder builds its whole slice front-to-back inside its write-set only; everything shared is read-only (§4).
- VERIFY: three layers in order — (a) per-edit self-check; (b) typecheck + build + audit, run by Tool-Runner with output pasted; (c) live Critic review. Critic patches MUST re-enter at layer (b): the owning Coder applies the fix, re-runs (b), and only then faces (c) again.
- MERGE: only after LOOP-EXIT passes (§5) AND COMPLETION-LOCK releases (§6); append + archive, never last-write-wins overwrite.

## 2. Delegation-message template (contract §6, verbatim) + worked example

Template — every delegation uses exactly these 5 fields:

```text
TO: <role from the ROLES table>
1. BRIEF: <one sentence: what + scope> | RISK: <1/2/3> | BLAST-RADIUS: <exact paths> | CONTRACT-FIRST: <paths read>
2. WRITE-SET: <exclusive paths this agent owns; disjoint from all concurrent agents>
3. READ-SCOPE: <what to read first, in full, before acting>
4. RETURN-SHAPE: <exact structure of the reply: headings, tables, labels required>
5. DONE-CRITERION: <checker command or observable condition that settles completion>
```

Worked example — all four calibration fields present, write-set disjoint by construction:

```text
TO: Coder-Forms
1. BRIEF: build the forms slice front-to-back in N3/web-development/areas/forms/ | RISK: 3 | BLAST-RADIUS: N3/web-development/areas/forms/ | CONTRACT-FIRST: N3/web-development/docs/SUBAGENT-CONTRACT.md, N3/web-development/areas/forms/CONTRACT.md
2. WRITE-SET: N3/web-development/areas/forms/ (exclusive; no concurrent agent holds it)
3. READ-SCOPE: SUBAGENT-CONTRACT.md sections 1-6 + areas/forms/CONTRACT.md, in full, before any write
4. RETURN-SHAPE: headings Done / Files / Checks + per-claim label table (PROPOSED until re-checked) + pasted checker output
5. DONE-CRITERION: discipline.py exit 0 AND audit exit 0 on the slice, pasted, plus Critic pass
```

## 3. CONTRACT-GATE (four items: name, artifacts, enforcer, veto)

- (a) Gate name: `spawn-gate`. (b) Artifacts on disk before fan-out: slice `CONTRACT.md` (reviewed, path named) + frozen delegation messages + write-set map proving pairwise disjointness. (c) Enforcer: transcript review — every delegation carries BRIEF / RISK / BLAST-RADIUS / CONTRACT-FIRST and the 5-field template verbatim. (d) Veto: fan-out STOPS; the gap is archived as a Failures entry; re-authorisation needs a new reviewed delegation.
- Freeze rule: contracts on disk, reviewed before spawn, FROZEN during fan-out; any change = full re-review + re-delegation, never a mid-flight edit.

## 4. OWNERSHIP

- Exclusive subtrees: one Coder per subtree; two coders MUST NOT share a file; overlap detected at delegation time = re-partition before fan-out.
- Shared files read-only: anything outside the write-set (shared tokens, root configs, sibling slices) is read-only; a needed shared change returns to Architect as a contract change (re-review), never a direct edit.

## 5. VERIFY layers + LOOP-EXIT (three items: predicate, evaluator, veto archive)

- Layer (a) per-edit: Coder self-checks each edit against the slice contract.
- Layer (b) typecheck + build + audit: Tool-Runner runs, pastes output; critic findings re-enter HERE (owning Coder applies fix, re-runs (b)).
- Layer (c) live critic: Critic re-runs checkers, re-reads source, judges pass/fail; no self-approval — a Coder never judges its own slice.
- LOOP-EXIT: (a) exit predicate, ALL true and pasted not summarised — gates green, zero runtime errors on the live path, Critic pass recorded — else re-enter at the failing layer; (b) evaluator: Critic (judgement) / Tool-Runner (checker layers), never the judged Coder; (c) veto archive: append + archive, never last-write-wins overwrite.

## 6. COMPLETION-LOCK (stop-gate veto)

- MERGE stays locked until LOOP-EXIT passes; pending issues are fixed or each recorded with a dated reason before completion is claimed.
- The stop-gate veto is final within a session; only an independent Critic pass releases the lock — Coders never self-approve.
