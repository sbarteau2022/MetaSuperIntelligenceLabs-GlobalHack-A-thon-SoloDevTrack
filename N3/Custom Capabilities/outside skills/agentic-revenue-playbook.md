
# COMPLIANCE HEADER — audited copy, 2026-10-30 (original body preserved verbatim below)

- Date: 2026-10-30
- Status: PASS-WITH-NOTES (audited copy; source carried no Date/Status lines)
- Source: `/Users/stewartbarteau/Desktop/skills/agentic-revenue-playbook.md` (134 lines; original untouched, READ-ONLY this pass)
- Audit contract: `skills/audit-contract-2026-10-30.md`, Coder B partition (contract VERIFIED read 2026-10-30)
- Verdict: PASS-WITH-NOTES — 8-point gate met with recorded warnings (Corrections C1–C3; NOT-CHECKED footer). Agent output is REPORTED until critic re-checks.
- Legend (AGENTS.md §7): VERIFIED = measured/read from the source itself; REPORTED = a source says so, unchecked; INFERRED = reasoned from evidence; PROPOSED = a plan, not a fact.
- Gate §6 secrets: VERIFIED clean 2026-10-30 (0 pattern hits). Gate §7 bulk actions: none present. Gate §8 web scope: N/A (INFERRED — strategy memo, no HTML/CSS/client-JS; no `latest` pins).

---
# Agentic Revenue Playbook
*Field research, compiled live from vendor + investor sources. Grounded, not vibes.*

> **Method note:** `web_search` had no backend configured, so I drove a real browser and fetched pages directly. Search engines (Google, DuckDuckGo, Bing) all bot-walled the VM's IP, so the strongest evidence came from **direct fetches** of primary sources: LangChain's customer + pricing pages, a16z's AI thesis page, and OpenAI's business page. Every number below is cited to where I read it.

---

## 1. The headline the market is sending

The category has **stopped selling "chatbots" and started selling outcomes.** The whole value stack has reorganized around a loop:

```
build agent  →  observe it  →  evaluate it  →  improve it  →  deploy it
 (LangGraph)   (tracing)      (evals)        (engine)      (infra)
```

LangChain's product page literally sells this as five named products: **Agent Improvement Engine, Observability, Evaluation, Deployment, Sandboxes.** The money is no longer in the model call — it's in the **reliability machinery around the agent.**

That's the single most important strategic fact: **the defensible revenue is in the wrapper, not the brain.**

---

## 2. The pricing shift (this is the actionable part)

LangSmith's pricing page shows the industry's unit of account migrating:

| Plan | Price | Unit |
|------|-------|------|
| Developer | $0/seat + pay-as-you-go | 5k traces/mo included |
| Plus | **$39/seat/mo** + pay-as-you-go | 10k traces/mo included |
| Enterprise | custom, self-hosted/hybrid | annual invoice |

And the killer line: **`1 LSU = $1.00`** — the *entire* platform (compute, memory, sandboxes, evals, gateway) is metered in **LangChain Standard Units.**

**What this means:** the market is moving from **per-seat** (SaaS) to **per-unit-of-work** (agents do *work*, so charge for the work). Compute is `0.0675 LSU/vCPU-hr`, memory `0.009 LSU/GiB-hr`, a tuned evaluation `0.015 LSU/run`.

> **Takeaway for us:** if we build an agent product, we should price **per outcome or per unit of work**, not per login. "You pay per report generated / per ticket resolved / per hour saved" — that's where the margin and the pitch both live.

Scale proof: LangSmith reports **100M+ traces/mo, 7K+ teams, 5 of 10 Fortune 10.** The infrastructure layer is already a real, priced market.

---

## 3. The value proof (who is paying, and for what)

LangChain's customer page is a list of *quantified* outcomes. These are the revenue archetypes, sorted by what's being sold:

| Company | Domain | Claimed outcome | Revenue archetype |
|---------|--------|-----------------|-------------------|
| **Klarna** | Customer support | 80% faster resolution, 85M users | **Support agent** (deflect + resolve) |
| **C.H. Robinson** | Logistics ops | 600+ hrs/day saved on orders | **Ops automation** |
| **MUFG Bank** | Sales | 10x sales efficiency | **Sales/research agent** |
| **Stripe (Kai)** | Internal tooling | 16x user growth in 4 wks | **Internal copilot** |
| **City of Hope** | Healthcare | 1000+ clinician-hrs saved | **Clinical doc/knowledge** |
| **Morningstar** | Financial research | 30% of analyst time saved | **Research assistant** |
| **Madrigal Pharma** | Drug dev | 12 wks → 2 wks (6x) | **Time-to-production** |
| **Factory** | Dev tooling | 550,000+ dev-hrs saved | **Engineering agent** |
| **Trellix** | Cybersecurity | log parsing days → minutes | **Security ops** |
| **AppFolio** | Real estate | ~80% better text-to-data | **Structured extraction** |
| **WordSmith** | LLM ops | 10x cost reduction | **Cost/eval optimization** |
| **New Computer** | Memory | 50% higher recall | **Retrieval quality** |
| **Clay** | GTM | 40% better ROI | **Monitoring/eval** |
| **monday.com** | Dev loop | 9x faster feedback loops | **Eval infra** |
| **Rippling** | Product | AI-native in 6 months | **Platform migration** |

**Pattern:** the buyers are **enterprises with expensive, repetitive, document- or data-heavy workflows** (finance, logistics, healthcare, legal-adjacent, security). The agent isn't a novelty — it's a **line-item cost reducer** with a number attached.

---

## 4. What a16z is betting on (the investor lens)

From a16z's AI page, the theses that map to revenue:

- **"The Next $25 Trillion"** (David George & Jack Altman) — autonomy is the next enterprise wedge.
- **"The Agent Era: Building Software Beyond Chat"** (Aaron Levie, Box) — agents that *act*, not just answer.
- **"Why Companies Are Becoming a Series of Loops"** (Anish Acharya) — the company itself becomes an agent loop.
- **"AI Agents and the Fight for Customer Data"** — **the data moat is the real moat.** Whoever owns the retrieval layer over a company's data owns the agent.
- **"The $1 Trillion AI Buildout"** — infra spend is the current revenue; application revenue is the next wave.
- **"Who Grades the AI Models?"** — **evaluation is becoming its own product category.**

Two of these are directly relevant to *our* stack: **the data/retrieval moat** and **evaluation as a product.** We already have a hybrid retriever + a writer flow + a memory layer. We are, structurally, already inside the "customer data + retrieval" thesis.

---

## 5. OpenAI's framing (the platform lens)

OpenAI's business page now sells **by function, not by model**: Finance, Data analytics, Sales, Marketing, Operations, Engineering, Design, Security. "ChatGPT Work" = *background task automation + docs/decks/spreadsheets + pairs with your tools.*

**Takeaway:** the platform layer is commoditizing the *model* and pushing value up to the **workflow/function layer.** That's the layer a solo builder can actually own, because it's **domain-specific** and **data-specific.**

---

## 6. Synthesis — where *we* fit (mapped to our actual stack)

Our assets: **hybrid retrieval (pgvector) + LangGraph + persistent memory + a writer that cites its sources + a hard-won honesty posture** (I refuse to fabricate; I show the empty KB, the broken import, the bot-walled search).

Ranking the plays by **fit-to-stack × market pull × time-to-first-dollar:**

### 🥇 Play 1 — "Grounded Document Intelligence" (do this first)
*The smallest delta from what we already have, and the clearest buyer.*
- **Product:** throw a pile of messy docs (contracts, 10-Ks, tickets, call logs, leases) at it → back **structured, cited** output. No hallucinated fields; every claim carries a source.
- **Why us:** we *already* have retrieve→write with citations. The moat is **grounding** — and "shows its work" is a genuine differentiator in a market drowning in confabulation.
- **Price:** per-document or per-report (per-unit-of-work, per §2).
- **First customer:** pick *one* painful doc type (e.g. "lease abstractor" or "support-ticket clusterer") and charge for it before polishing.

### 🥈 Play 2 — "The Honest Agent" as positioning
*Not a product, a wedge.*
- The entire conversation we just had is the pitch: I **refused to fake web research**, showed the **empty KB**, named the **broken import**, and only then went and *did* the real research. In a market of confabulating demos, **"here's what I found, here's what's missing, here's what I can't do"** is a trust moat.
- **Use it** on Play 1: every output carries its confidence + gaps. That's the brand.

### 🥉 Play 3 — Retrieval/Memory-as-a-Service (the data moat)
- a16z's "fight for customer data" thesis. Sell the **retrieval + memory layer** that makes *any* agent reliable over a company's own data. Higher ceiling, more lift, but it's the most defensible long-term.

### Play 4 — Eval-as-a-Product (the sleeper)
- "Who grades the AI models?" is becoming a category. We could sell **eval harnesses + regression monitoring** for agent deployments. Adjacent to what we'd need anyway; a natural second product.

---

## 7. Recommended next 72 hours
1. **Commit to Play 1** (grounded doc intelligence). It's the fastest path to a *real* first dollar.
2. **Build the missing `home/` package** (retriever + writer + clean ingest) so the flow is a product, not a sketch. *(This also fixes the broken import we found.)*
3. **Pick one doc type + one buyer.** Build the demo on *their* actual documents.
4. **Price per outcome**, not per seat.
5. **Ingest the `uploads/` research stack** into the KB so the agent can cite the architecture literature (test-time learning, latent reasoning, linear attention) as domain authority.

---

## Sources
- LangChain — LangGraph product page (retrieved live)
- LangChain — Customer Stories (28 stories; 15 quantified outcomes extracted)
- LangChain — LangSmith Pricing (LSU unit, $39/seat, 100M traces/mo)
- a16z — AI focus page (thesis list)
- OpenAI — Business page (function-based positioning, ChatGPT Work)

*Search engines (Google/DuckDuckGo/Bing) bot-walled the VM; all evidence above is from direct primary-source fetches, not search snippets.*

---

# AUDIT APPENDIX — 2026-10-30 (appended; history above preserved verbatim)

## Claim-label map (gate §2)
- §2 LangSmith pricing ($39/seat, LSU=$1.00, compute/memory/eval rates, trace quotas): REPORTED — pricing-page read via author's direct fetch; pages change — never cite as current without re-fetch.
- §2 scale proof ("100M+ traces/mo, 7K+ teams, 5 of 10 Fortune 10"): REPORTED — vendor claim, unverified.
- §3 customer-outcome table (Klarna 80%, C.H. Robinson 600+ hrs/day, MUFG 10x, etc.): REPORTED — vendor-published case claims; no hardware/settings/workload/baseline (gate §3) — never cite as measured results.
- §4 a16z thesis list; §5 OpenAI function positioning; Method note: REPORTED — author's vendor-page reads, not independently re-checked.
- §6 plays 1–4, §7 72-hour plan: PROPOSED — recommendations, not facts.
- Gate §5 sources: file lists its Sources; re-verification NOT done (see NOT CHECKED).

## Corrections
- [2026-10-30] C1: Added header/appendix for gates §1/§2/§4/§5. No body text altered.
- [2026-10-30] C2 (gate §3): vendor pricing/outcome figures are REPORTED only; none ship with hardware + settings + workload + baseline. No performance verdicts asserted.
- [2026-10-30] C3 (gate §5): linked LangChain/a16z/OpenAI pages NOT re-fetched this pass.

## NOT CHECKED
- NOT CHECKED: current LangSmith pricing/LSU rates/quotas; any customer-outcome claim; a16z/OpenAI page contents.
- NOT CHECKED: critic re-check (this output is REPORTED per contract).
