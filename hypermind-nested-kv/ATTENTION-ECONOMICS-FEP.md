# Attention Economics × Free Energy: Battling Agentic Sundowning
## The Recall-vs-Recompute Law

**Status.** FORMALIZATION (this doc) + LIVE MECHANISMS (cited, all VERIFIED this pass).
**Parents.** `AttentionEconomicsAndActiveInference-Spec.md` (reward `R = Q/(H + c·N)`,
depth-gated allotment, I-HEAT), `ORCHESTRATION-PROTOCOL.md` §4 (sundowning),
research harness `fep.rs` (session bound), `recursive.rs` (worst-first optimizer),
`map.attend` depth gate + `map.kv_store/fetch` (live tools).
**No code changed by this doc. One new invariant proposed (§6).**

---

## 0. The claim in one breath

An agent should never pay full-forward-pass burn for every token when most of
what it needs was already computed. The KV cache holds not tokens but
*conclusions with provenance* — and once a conclusion exists with its
producers receipted, the only attention worth building is the amount
necessary to *recall* it. Everything else is waste heat. This doc makes
that sentence a law with a meter, a data structure, and a failure detector.

---

## 1. What exists today (all VERIFIED live)

| Piece | Where | What it proves |
|---|---|---|
| Depth-gated attention | `map.attend` (`main.rs` ~L2299): VP shortlist (128, exact-prune) → variational gate `cost = ln(k) + k − mass×8` → minimal k | Attention is already built minimally, not maximally. Receipted per call. |
| KV with lineage | `map.kv_store/fetch`: key-ordered B-tree, producer ids, hit counts, snapshot round-trip proven (1 entry store→wipe→recover) | Conclusions persist with provenance; hits accumulate (measured 1,2,3). |
| FEP session bound | `fep.rs`: `F = complexity + inaccuracy + β·dissipation`, `descend()` per step, `requires_iteration` on >mean+2σ spikes | Every decision already prices into one bound. |
| Sundowning detector | `ORCHESTRATION-PROTOCOL.md` §4 + `map.traverse` fail-explained mid-plan | Drift has a tripwire; re-ground, don't push through. |
| Reward law | Vault spec §1: `R = Q/(H + c·N)`, metered heat, falsifiable vs F-only optimizer | Quality-per-joule is the maximand, not tokens-per-second. |

## 2. What's missing (two gaps, honestly named)

**Gap A — KV has no dependency edges.** A step-5 conclusion cannot invalidate
step-3's cache entry when step-3's producers change. Staleness is impossible
today only because nothing ever updates. That is luck, not a design.

**Gap B — Two minimizers, one system.** The depth gate minimizes its own
variational cost; the session bound minimizes F. They agree by construction
today. Nothing *forces* agreement — the depth gate doesn't read the session
bound, and the session bound doesn't price the depth decision.

---

## 3. The Recall-vs-Recompute Law (the formalization)

Per reasoning step, let:

- `F_recall(k)` = bound-cost of building attention of depth k over cached
  conclusions: `complexity = ln(k) + k` (the live gate's form, now in nats),
  `accuracy = −score_mass(k)` (the live `qmass` term, negated as a cost),
  `dissipation = k · c_step` (k rows touched × per-row joule constant).
- `F_recompute` = bound-cost of deriving from scratch: full forward cost
  in the same units (measured once per task class, receipted, not estimated).

**Law: build attention of depth k* = argmin_k F_recall(k) iff
min_k F_recall(k) < F_recompute; otherwise recall the cached conclusion
at depth 0 (a KV fetch, near-zero bound).**

Consequences that fall out without extra machinery:

1. **The depth gate IS the law's argmin.** The live `cost = ln(k) + k −
   mass×8` is `F_recall(k)` with `β·dissipation` folded into the linear
   term and accuracy scaled by 8. No rewrite — relabel with units.
2. **The KV hit rate is the prior.** Each hit lowers the effective
   `F_recall(0)` (cached conclusion, verified producers) relative to
   recompute. The more a conclusion is reused, the stronger recall wins —
   the system *learns to stop computing* what it already knows. That is
   the self-recursive optimization, priced.
3. **Sundowning is bound drift.** Context rot raises inaccuracy (stale
   premises predict poorly); the spike trigger (`requires_iteration`)
   fires; the mandate is plan/research, never push-through. Same detector,
   now with the law behind it: drift raises `F_recall` until recompute
   (fresh grounding) wins — **re-grounding IS the law switching arms.**
4. **I-HEAT is the envelope.** Sustained dissipation rate ≤ thermal
   envelope bounds the sum, per invocation. The law optimizes within it;
   the audit role (`audit` kernel, vault spec §6.5) enforces it.

## 4. Data structure: SoA relational DAG (closes Gap A)

KV entries become nodes in a dependency DAG, stored structure-of-arrays
for the scan path:

```
nodes:      id[] | key[] | value_hash[] | producers[] | hits[] | epoch[]
edges:      dependent[] | depends_on[]          // invalidates downstream
scores:     bm25[] | geo[] | fused[]            // recomputed on reweight
```

- **Compute once:** first derivation stores the node + its producer edges.
- **Reuse:** fetch hits increment `hits[]`, cost ~O(1) + B-tree lookup.
- **Invalidate exactly:** when a producer's atom is re-ingested (new
  salience, new text), walk `edges` downstream and mark stale —
  no TTL guessing, no blind eviction. Stale nodes re-derive on next
  fetch (pay `F_recompute` once, then cheap again).
- **SoA, not AoS:** the hot scan (BM25 over keys, hits ranking) walks
  contiguous arrays; the cold path (value bytes) stays pointer-chased.
  This is the "structure of arrays" from your spec: cache lines, not
  cache wishes.
- **Receipts:** every store/fetch/invalidate carries the existing
  receipt chain (`receipt_sha256` per op). The DAG is auditable or it
  doesn't ship.

## 5. Meter table (what proves the law holds)

| Meter | Procedure | Pass bar |
|---|---|---|
| Recall fraction | `KV hits / (hits + derives)` per session | Rises across consolidations (learning to stop computing) |
| Depth minimality | `k_selected` vs oracle-best-k on fixed probe set | ≥ 95% agreement (gate isn't under/over-building) |
| F-agreement | depth-gate argmin vs session-bound argmin on same decisions | No systematic divergence (Gap B closed) |
| Invalidation precision | stale-marked nodes that were actually stale / all marked | 1.0 exact (edges, not TTL — must be perfect) |
| Sundowning catch rate | injected context rot → detector fires before quality drops | Fires first, every time (fail-explained, not silent) |
| Dissipation per quality | metered J / judge-passed task, before vs after | Decreases (the reward law, measured) |

## 6. The one new invariant (PROPOSED)

**I-RECALL (recall-before-recompute):** no derivation executes while an
uninvalidated cached conclusion with matching producers exists for the
same information need. Violation → the derivation's receipt is marked
`wasteful` (not failed — the answer may be right, but it was overpriced).
Three `wasteful` marks on one subsystem in a session trigger the
recursive optimizer (`rank_subsystems` already worst-firsts on bound —
waste flows into the score automatically).

This is the teeth: the law isn't advice, it's audited. The optimizer
doesn't need a new objective — waste raises bound, bound drives rank.

---

## 7. What was NOT done (explicit)

- No code changed. Gap A (DAG edges) and the I-RECALL audit flag are
  designed, not built. The depth gate keeps minimizing its local cost;
  the session bound keeps its global one; §3 asserts their identity
  but no probe has *measured* F-agreement yet (meter table row 3 is
  the experiment).
- No C++ / SIMD / GPU claim. The SoA layout is cache-friendly by
  construction; "faster than anything" is a bench, not a paragraph —
  `benches/smoke.rs` is where that number goes or the claim doesn't exist.
- TS 7.1-beta: untouched. This law is language-agnostic; whichever
  substrate runs it must pass the same meters.

---

## 8. The one-line version

Build only the attention the bound justifies; remember everything you
concluded and what it depended on; invalidate exactly; let the spike
detector, not pride, decide when to re-ground. Waste is measured in
joules, and the ledger never forgets.
