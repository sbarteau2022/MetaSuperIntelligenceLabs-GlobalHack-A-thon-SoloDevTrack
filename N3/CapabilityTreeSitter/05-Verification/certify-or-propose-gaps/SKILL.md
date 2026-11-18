---
name: certify-or-propose-gaps
description: "Decide certification for a pressure-tested implementation - certify only what is proven, and propose explicit gaps for the rest."
---

# CertifyOrProposeGaps

After a pressure audit, issue a certification verdict. Be rigorous and say what the artifact waits on.

1. **Certify only proven claims.** A claim is certified only with: a passing gate, a receipt, and a reproducible command. Everything else is PROPOSED/OPEN.
2. **Next steps.** List concrete actions in dependency order (code, tests, docs, ablations). Each step must be small enough to verify in one pass.
3. **Blocked-on list.** For each OPEN gate, state exactly what it waits on: missing measurement, missing code, missing hardware/lane, or a human decision. No hand-waving.
4. **No premature certification.** Complexity, throughput, safety, or security claims without measured evidence are never certified — they are labeled and gated.
5. **Output.** Produce: certified list (with evidence), blocked list (with owners), and the single next command to run.

Fail-closed rule: an uncertified gate is a blocker, not a footnote.
