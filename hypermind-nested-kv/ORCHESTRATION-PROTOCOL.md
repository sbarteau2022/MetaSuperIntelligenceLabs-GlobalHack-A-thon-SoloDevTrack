# The Protocol: From User Query to Ruthlessly Effective Agent (Step 3, goal bd01f172)

**In plain language first.** A user asks for something. The system must
decide: what does the agent actually need to know, how deep must it go,
who does what, and how does it get better. This doc is that protocol —
formal where it must be, plain everywhere else.

---

## 1. The pipeline: Meta → tactics → orchestration

```
USER QUERY
  │
  ▼
META (task decomposition)     "What is being asked, really?"
  │  output: task tree (goal → sub-tasks → leaf actions)
  ▼
NECESSITY FILTER (§2)         "What MUST the agent know? At what depth?"
  │  output: minimal information set per leaf (see min-info rule)
  ▼
TACTICAL MAPPING              "Which capability executes each leaf?"
  │  output: leaf → tool/fiber/arc assignment with budgets
  ▼
ORCHESTRATION                 "Who runs, in what order, with what receipts?"
  │  output: execution ledger (every step receipted, fail-explained)
  ▼
SUPERIOR QUALITY (expectation) — measured, not asserted (§2, §5)
```

**Meta** decomposes: one goal becomes sub-tasks becomes leaf actions. Each
leaf states its *information need* explicitly (see §2) — a leaf that can't
name what it needs is not ready to execute.

**Tactical mapping** assigns every leaf to exactly one executor (a map tool,
a carved fiber, a traversal arc) with a budget (energy, tokens, depth).
Unassigned leaves fail-closed at plan time, never at 3am mid-run.

**Orchestration** runs the leaves in dependency order, receipting every
step. A leaf that fails returns `fail-explained` (cost, budget, missing
piece) — the orchestrator re-plans around the named gap instead of retrying
blindly. This is the live `map.traverse` verdict logic, lifted to the plan
level.

---

## 2. The min-info rule: necessary, at depth — not merely sufficient

**The question.** "What is the minimum information that must be given to
have the expectation of Superior Quality?"

**The answer, formal.** For a leaf action `L` with required sub-problems
`U(L)`, the minimal information set `I*(L)` is:

```
I*(L) = smallest I such that cov(I) ≥ θ AND depth(I) ≥ δ(L)
```

- `cov(I)` = fraction of `U(L)` covered by `I` (same coverage math as
  `CONVERSION_MATH.md`: retrieval replaces recomputation on covered ground).
- `θ` = coverage threshold, preregistered per task class (default 0.8:
  four of five sub-problems covered; the fifth is the agent's judgment call,
  not a lookup).
- `δ(L)` = depth required: the number of *reasoning steps* (arcs traversed),
  not documents read. A leaf needing a 3-hop chain (`A→B→C→answer`) is not
  satisfied by three disconnected facts — it needs the *connected path*.
  This is why the envelope checks form (is there a path?) before dispatch:
  disconnected sufficiency is the failure mode `mlir_op#26` demonstrated.

**Necessary vs sufficient, plainly.** Sufficient = "with this, a good agent
*could* succeed." Necessary = "without this, even a perfect agent *must*
fail." The protocol ships necessary: for each leaf, name the one fact
whose absence voids the leaf. If no single fact is load-bearing, the leaf
is too big — decompose further.

**Depth, plainly.** Shallow = the answer is IN the retrieved text (lookup).
Deep = the answer is at the END of a chain (reasoning). The protocol sets
depth per leaf at plan time (`depth_k` in `map.attend` is the live version:
data-dependent, not fixed). Ruthless effectiveness = never paying depth-3
cost for a depth-1 leaf, never starving a depth-3 leaf with depth-1 budget.

---

## 3. Where gradient descent happens (it does — just not in weights)

**No weight updates. Ever.** `dW = 0` by construction (frozen reads only).
The gradient flows in *four other places*:

1. **Coverage gradient.** `∂(task success)/∂(atoms carved)`: each distill
   pass adds atoms where retrieval failed. The A39 loop IS gradient descent
   on the map: loss = uncovered sub-problems, step = carve the missing arc,
   learning rate = salience reinforcement (+0.05 per coalescence).
2. **Budget gradient.** `∂(cost)/∂(depth_k)`: `map.attend`'s variational
   gate descends cost-vs-quality per query. The minimum is found, not fixed.
3. **Epsilon gradient.** The traverse sweep (16 configs, 40 pairs) descended
   the frame-eps landscape to the operating point (1e-2 default, 1e-3 audit).
   Preregistered = converged.
4. **IDF gradient.** `map.reweight` recomputes term weights from live
   content — the map's vocabulary model descends toward its own usage.
   Encoder training (GPU step) is the fifth, still open.

**Plainly.** The system learns by growing the map where it's ignorant,
spending budget where it pays, and tightening gates where they're loose.
Weights never move; everything around them does.

---

## 4. Agentic sundowning (named, bounded, handled)

**The phenomenon.** Agent performance degrades over long runs: context fills,
early instructions fade, later steps drift. Like sundowning — worse as the
day (context window) wears on.

**Where it bites here.** Three places, each with a live countermeasure:

1. **Context fill → compaction loss.** Long sessions compact; detail dies.
   Countermeasure: `map.compact` checkpoints (objective + recent state +
   active ids + tool ledger) BEFORE compaction, and `map.checkpoint`
   restores. The capsule travels with the agent, not through the carve.
2. **Salience drift → old atoms dominate.** Frequently-reinforced atoms
   outrank fresh-but-right ones. Countermeasure: recency is a first-class
   field (basin `recency`), and the fusion weights are re-calibrated by
   `map.reweight` — stale dominance decays as vocabulary shifts.
3. **Plan decay → leaves executed stale.** A 50-step plan's step 40 runs on
   step-1 assumptions. Countermeasure: traversal contracts re-check
   form+energy+frame AT DISPATCH, not at plan time. A path that was valid
   at planning and sheared by step 40 fails explained — and replans.

**The rule.** No agent runs longer than its receipts stay valid. When the
form check fails mid-plan, that IS the sundowning detector firing — stop,
re-ground (fresh query + fresh traverse), continue. Never push through a
failed check; the check is the symptom telling you the context rotted.

---

## 5. Superior Quality as expectation (how we know, not what we hope)

Superior Quality is not a vibe. It is:

```
E[Quality] = cov(I*) × p_useful × (1 - drift)
```

- `cov(I*)`: did we ship necessary info at depth? (plan-time, §2)
- `p_useful`: of what we retrieved, what fraction helped? (measured:
  0.639 held-out, 1.000 home-turf)
- `(1 - drift)`: did the long run hold together? (traverse pass rate:
  36/40 at default eps; sundowning detector = any mid-plan fail-explained)

All three are measured on the live map today. When all three are green,
Superior Quality is the *expectation* — the math says so, not the marketing.

---

## 6. What remains (next goals, in order)

1. GPU encoder (cleared, `GPU-ENCODER-CLEARANCE.md`) — lifts geo paraphrase.
2. Held-out task families beyond 12 (breadth on the 0.639).
3. Salience-normalized cross-shell ranking weights (raw radius as tie-break
   is heuristic; calibrate it).
4. Multi-plan orchestration ledger (this doc's pipeline as a live tool,
   not just a spec).
