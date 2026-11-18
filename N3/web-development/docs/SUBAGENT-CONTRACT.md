# SUBAGENT-CONTRACT.md — Frozen Interface for Subagent Partitions

Date: 2026-10-30. Status: PROPOSED.

## Purpose

This file is the ACCEPTANCE TEST for PARTITION E (subagent protocols). E writes the protocol; a PARTITION E deliverable passes if and only if it satisfies every MUST below. Anything E invents beyond these shapes is advisory, not law.

## 1. Mandatory brief fields (MUST)

Every delegation message opens with a one-line calibration carrying all four fields. A delegation missing any field FAILS the contract gate (§5).

- `BRIEF:` — one sentence: what is wanted, in which scope.
- `RISK:` — 1 (read-only recon), 2 (markdown/contracts/prose only), 3 (code or config writes).
- `BLAST-RADIUS:` — exact writable scope (directory subtree or file list). Nothing outside it may be written.
- `CONTRACT-FIRST:` — the contract or register entries read before delegating, named by path.

## 2. Disjoint-write-set rule (MUST)

Every delegated task names an exclusive WRITE-SET (directory subtree or explicit file list). Two concurrent subagents MUST NOT hold overlapping write-sets; two coders MUST NOT share a file. Overlap detected at delegation time = re-partition before fan-out, never after. Vertical slices (feature/surface/contract), never horizontal layers — PARTITION law.

## 3. No-duplicate-delegation rule (MUST)

The same question or task MUST NOT be delegated to two agents for "confirmation." Redundant exploration requires a dated, named justification recorded BEFORE fan-out (e.g. competing designs), and each redundant agent gets a disjoint write-set. Same answer twice is waste, not verification.

## 4. REPORTED-until-rechecked rule (MUST)

Subagent output enters registers and decisions as REPORTED, never VERIFIED. Promotion to VERIFIED requires an INDEPENDENT re-check: re-run the checkers, re-count the files, re-read the source — never re-read the agent's own summary of its work. (Authority: RULES.md rule 74; AGENTS.md §7.)

## 5. Correction-entry rule (MUST)

A claim that turns out wrong gets a dated correction entry naming what it replaces. Never silently rewrite history, never delete the wrong claim, never edit a register row in place. Withdraw in the same breath as replacement. (Authority: AGENTS.md §7.3.)

## 6. Delegation-message template (MUST — exactly these 5 fields)

```
TO: <role from the ROLES table>
1. BRIEF: <one sentence: what + scope> | RISK: <1/2/3> | BLAST-RADIUS: <exact paths> | CONTRACT-FIRST: <paths read>
2. WRITE-SET: <exclusive paths this agent owns; disjoint from all concurrent agents>
3. READ-SCOPE: <what to read first, in full, before acting>
4. RETURN-SHAPE: <exact structure of the reply: headings, tables, labels required>
5. DONE-CRITERION: <checker command or observable condition that settles completion>
```

## 7. EXACT shapes PARTITION E must produce (MUST)

E's protocol document is complete only if it contains all four, in these shapes:

**ROLES table** — one row per role, exactly these columns:

| role | responsibility | write-set |
| ---- | -------------- | --------- |
| `<name>` | `<one sentence: what this role owns front-to-back>` | `<exclusive subtree or files; disjoint by construction>` |

**ROUTING rule** — each rule is one line: `WHEN <observable condition> THEN <role> VIA <trigger form>`. No prose routing; conditions must be checkable in the transcript.

**CONTRACT-GATE definition** — each gate states four items: (a) gate name; (b) artifacts required on disk before fan-out (contracts reviewed, paths named); (c) the checker or transcript review that enforces it; (d) veto behavior (what stops, what archives, who re-authorises).

**LOOP-EXIT condition** — each loop states three items: (a) exit predicate (checker output or criterion, pasted not summarised); (b) who evaluates it (role, never the coder being judged); (c) archive behavior on veto (append + archive, never last-write-wins overwrite).

## 8. Acceptance checklist (E passes iff all hold)

- [ ] Every delegation in E's doc carries BRIEF/RISK/BLAST-RADIUS/CONTRACT-FIRST.
- [ ] Write-sets across concurrent roles are pairwise disjoint on paper.
- [ ] No task is assigned to two roles without a dated justification.
- [ ] Agent output defaults to REPORTED with a named re-check path to VERIFIED.
- [ ] Corrections are entries, not edits (dated, replacing, withdrawing).
- [ ] Delegation messages use the 5-field template verbatim.
- [ ] ROLES table has all three columns; ROUTING is WHEN/THEN/VIA; CONTRACT-GATE has all four items; LOOP-EXIT has all three items.

## NOT checked

Whether E's roles are optimally named; live multi-agent execution against this contract (no swarm run in this partition).
