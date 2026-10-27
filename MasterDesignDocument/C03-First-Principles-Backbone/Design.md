# C03 — First-Principles Engineering Backbone

**Status.** VERIFIED as design intent (principles). **Parent:** Master Design Document index.

## Purpose

Non-negotiable engineering principles that gate every implementation decision.

## Subcategories (broad topics)

- C03.1 Verifiable execution (holes, Merkle receipts, judge verdicts, approval gates)
- C03.2 Evidence over claims (no metric without workload/hardware/baseline; no badge without receipt)
- C03.3 Append-only truth (events/workflows/ledgers reject update/delete; corrections appended)
- C03.4 Least privilege (tools are data; risky execution behind seatbelt + approval; secrets by slot)
- C03.5 Determinism (resolve v2 deterministic hashes; receipts hash-derived)
- C03.6 Cascade memory (Org Policy → User → Project Rules → Session Auto; tight context budget)
- C03.7 Security/compliance before build (threat model, role matrix, audit log, erasure plan first)

## Formal Foundation: Active Inference as Variational Free Energy Minimization

The first-principles backbone is not a checklist—it is a **thermodynamic contract**. Every principle derives from minimizing variational free energy $F$ while aligning resource consumption $R$ to quality output $Q$:

$$
F[q] = \mathbb{E}_{q(s|o)}[-\log p(o,s)] + \mathcal{H}[q(s|o)]
$$

$$
\mathcal{L} = F + \lambda \cdot \frac{R}{Q}
$$

where $q(s|o)$ is the agent's approximate posterior over hidden states given observations, $p(o,s)$ is the generative model, and $\mathcal{H}$ is entropy. The seven principles are the **KKT conditions** of this optimization:

| Principle             | Free Energy Term                                      | Gate Form                                           |
| --------------------- | ----------------------------------------------------- | --------------------------------------------------- |
| Verifiable execution  | Expected energy $\mathbb{E}[-\log p]$                 | Receipt $\leftrightarrow$ energy certificate        |
| Evidence over claims  | Surprisal $-\log p(o)$                                | No metric without $-\log p(o \mid \text{baseline})$ |
| Append-only truth     | Entropy $\mathcal{H}$ monotonicity                    | Corrections $\to$ entropy increase                  |
| Least privilege       | Precision-weighted prediction error                   | Seatbelt = precision cap                            |
| Determinism           | Deterministic $q$ update                              | Hash = trajectory fingerprint                       |
| Cascade memory        | Hierarchical $q$ (policy $\to$ project $\to$ session) | Budget = precision allocation                       |
| Security before build | Prior $p$ over threat model                           | Audit = prior validation                            |

## Resource-Quality Alignment Theorem

**Theorem.** Under the active inference transformer (ACFT), the expected free energy per token $\mathbb{E}[F]_{token}$ is minimized when the hyperbolic retrieval depth $d$ satisfies:

$$
d^* = \arg\min_d \left( \underbrace{\mathcal{C}_{retrieve}(d)}_{O(\log d)} + \underbrace{\mathcal{C}_{reason}(d)}_{O(d)} - \underbrace{\mathcal{I}(answer; context_d)}_{\text{mutual info}} \right)
$$

This is exactly what **TwisterAttention** computes: it builds the attention slice only up to the depth where marginal mutual information gain falls below the marginal compute cost. No full forward pass.

## Measurable Gates (Promoted from Principles)

| Principle             | Gate ID | Metric                      | Threshold                   |
| --------------------- | ------- | --------------------------- | --------------------------- |
| Verifiable execution  | C15.1   | Receipt coverage            | 100% mutating actions       |
| Evidence over claims  | C09.3   | Claim evidence ratio        | 1.0 (every claim)           |
| Append-only truth     | C15.1   | Mutation rejection rate     | 100% update/delete rejected |
| Least privilege       | C16.8   | Secret scan false negatives | 0                           |
| Determinism           | C09.5   | Replay hash equality        | 100%                        |
| Cascade memory        | C24.2   | Context budget adherence    | $\leq$ 200 lines default    |
| Security before build | C15.6   | Threat model sign-off       | Before Day 1 feature code   |

## Open questions / gaps

- Promote each principle to a measurable gate in C08/C15.

---

## Contributor Deepening — Muse (2026-10-26)

### C03.8 Principle conflict-resolution order (depth, not novelty)

The backbone lists seven non-negotiable principles, but principles collide in practice. Frozen precedence for conflicts, highest first [PROPOSED — precedence is my proposal; no team ratification yet]:

1. **Security/compliance before build** — overrules velocity claims from any other principle.
2. **Least privilege** — overrules verifiable-execution convenience (a receipt never justifies a broader grant).
3. **Append-only truth** — overrules determinism repair (a broken hash is corrected by appended entry, never by rewriting history).
4. **Evidence over claims** — overrules cascade-memory brevity (a claim without evidence is dropped, not summarized).
5. **Verifiable execution** — overrules determinism performance shortcuts.
6. **Determinism** — overrules cascade-memory cache reuse (a stale cache hit that breaks replay is a bug, not an optimization).
7. **Cascade memory** — yields to all of the above.

**Worked example:** a reviewer asks to widen a lane's tool allowlist to make handoff receipts cheaper to produce. Precedence says least privilege (#2) beats verifiable-execution convenience (#5): deny the widening, keep the receipt expensive.

### C03.9 Backbone compliance checklist (per-category, per-gate)

Every category deepening must answer, in its `Design.md`, one line per principle: satisfied / violated-with-waiver / not-applicable-with-reason. Waivers require an appended C36 correction entry citing the conflicting principle and the precedence rule applied. A category with an unanswered line fails Gate 1 (ADR-21). [PROPOSED — checklist format is my proposal; enforcement via Gate 1 follows from ADR-21.]

What was NOT checked here: whether C08/C15 already contain per-principle gate IDs (C15/Design.md not yet read in this pass — verify at C15).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 3. First-principles engineering backbone- Active Inference Aware as first class dynamic systems architceture - Minimizing Variational Free Energy while Optimizing for an aligment of Resource Consumption-to-QualityOutput.

1. **Verifiable execution:** every mutating action runs in a hole, produces a receipt (Merkle
   fingerprint + judge verdict), and is approval-gated where required.
2. **Evidence over claims:** no metric without workload/hardware/baseline; no badge without a
   receipt.
3. **Append-only truth:** events/workflows/ledgers reject updates and deletes; corrections are
   appended, never rewritten.
4. **Least privilege:** tools are data; risky execution stays behind seatbelt + approval; secrets
   by slot, never literal.
5. **Determinism:** resolve v2 validates tool refs deterministically; receipts are hash-derived.
6. **Cascade memory:** Org Policy → User → Project Rules → Session Auto; tight context budget.
7. **Security/compliance before build:** threat model, role matrix, audit log, right-to-erasure
   plan precede feature code.

---
