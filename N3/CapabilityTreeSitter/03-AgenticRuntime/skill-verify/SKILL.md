---
name: skill-verify
description: Verify an agent skill or tool works end to end and issue a receipt. Use when checking load, trigger, act, judge, and receipt steps before claiming a skill is implemented.
---

# Skill and Tool Verification

You verify that a skill or tool is actually working, not just scaffolded.

## Verification checklist
1. **Load:** the skill file parses (frontmatter valid; name matches directory; description present and specific).
2. **Trigger:** the intended user phrase would plausibly load this skill.
3. **Act:** the skill's instructions produce the intended output when followed.
4. **Judge:** the acceptance criteria (from the user or the plan) are met.
5. **Receipt:** record evidence (build output, smoke result, latency, commit/hash) and label VERIFIED/PROPOSED.

## Evidence to collect
- Build output (cargo build --release) with warning/error counts.
- Smoke script output (`eval_atoma.sh`, `compliance_smoke.sh`).
- Measured numbers (latency, speedup, throughput) with dataset and hardware context.
- Tool call counts and error rates from `/metrics`.
- Merkle/receipt hashes where applicable.

## Acceptance bar
A skill is VERIFIED only when:
- It loads in Zed (frontmatter valid).
- It triggers on the intended phrase.
- Its instructions execute without error.
- The acceptance criteria are met with evidence.
- A receipt is recorded (file, hash, or commit reference).

If any step fails, mark it PROPOSED and report the blocker.

## Anti-patterns
- Do not mark a skill VERIFIED without running it.
- Do not claim test coverage without executing tests.
- Do not treat scaffolded code as implemented.
- Do not skip the receipt step.

## Output format
For each skill verified:
- name, scope (global/project)
- load: pass/fail
- trigger: pass/fail
- act: pass/fail
- judge: pass/fail
- receipt: <evidence reference>
- status: VERIFIED or PROPOSED (with blocker)
