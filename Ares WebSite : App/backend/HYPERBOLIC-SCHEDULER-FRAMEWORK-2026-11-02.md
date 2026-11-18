# Hyperbolic Scheduler Framework: workflows, forward-pass math, FEP binding

Date: 2026-11-02. Status: PROPOSED (framework design, not a build directive).
Skills invoked: `autonomous-math-olympiad` (L0 reasoning only) + `research-runbook`
(evidence handling). Level used: **L0** — derivations and workflow design before
code. L1 (formula verification scripts) and L2 (staged experiments) are deferred
with their probes named in §7. No L3/L4.

## 0. Research frame (research-runbook §1–§2, pre-registered)

**Question.** What workflows execute a scheduler whose KV is hyperbolic and
nested in the hyperbolic mind — where the capability tree is likewise nested
in mind-space as a hierarchical contextual relational graph — and what does
that do to the math of a forward pass, under an active-inference / free-energy
bound?

**Decision this unblocks.** Whether to promote hyperbolic-KV-in-forward-path
from PROPOSED bench/retrieval geometry to a kernel-register candidate: the
necessity ablation (vs the fixed-cap-512 incumbent, same machine/inputs, one
variable) cannot be written until the workflows and the changed math are
stated. This doc is that statement. It authorizes no code.

**Acceptance criteria (dated 2026-11-02, before synthesis).**

- C1: workflows listed per problem solved, each naming the brute-force step it eliminates.
- C2: forward-pass math stated as identities/deltas vs dense baseline, with the oracle bound (VIII) computed on paper.
- C3: FEP binding stated as the single bound each workflow descends, with the three-way tradeoff (surprise / dissipation / quality) priced.
- C4: at least one hand-constructed counterexample per load-bearing claim (math extra).
- C5: every claim labeled; no code written (L0 discipline).

## 1. What exists on disk (VERIFIED reads this session — the foundation, not the proposal)

- **Mind substrate.** `hypermind-nested-kv/src/main.rs`: atoms on H3 x T2
  (Lorentz coords, `hyperbolic_dist` L761, shell projection L765 for
  salience-invariance), VP-tree exact-prune kNN on ALL query paths
  (`vp_tree_matches_brute_force`, parity-tested), rank structure
  (rank 0 = chunk space, rank 1 = document partition-and-aggregate parent,
  RoPE-family `rotate_rank` in `mapmath.rs`).
- **Runtime attention already built.** `map.attend` (L3531–3660): VP shortlist
  (128, exact-prune) → score → **data-dependent depth gate** over
  k in {1,2,3,4,8,16,32}, `cost = ln(k)+k − qmass·8 + k·C_ROW_J` — the
  Recall-vs-Recompute Law §3 with the dissipation term inside the argmin
  (Gap B closed in code: the receipt exposes `fep_terms` so the session
  bound prices the SAME decision). Measured REPORTED (cert doc):
  50/50 exact-text top-1 at N=500, p50 ~2.6 ms, k=1, debug build.
- **KV with lineage.** `map.kv_store` / `map.kv_fetch` (B-tree, producer ids,
  hit counts, snapshot round-trip proven). Gap A OPEN: no dependency edges
  yet — invalidation design exists (`ATTENTION-ECONOMICS-FEP.md` §4, SoA
  relational DAG) but is not built.
- **FEP core.** `research-grade-harness/src/fep.rs`:
  `F = complexity − accuracy + β·dissipation`, Landauer anchor
  `2.87e-21 J/bit`, `SessionBound::descend`, `requires_iteration`
  (mean+2σ spike → plan/research, never push-through), `precision_update`.
- **Law + meters.** `ATTENTION-ECONOMICS-FEP.md`: I-RECALL invariant
  (PROPOSED — no derivation runs while a valid cached conclusion exists;
  violations receipted `wasteful`), six-row meter table, honest §7
  (no code changed by that doc; F-agreement meter unmeasured).
- **Capability DAG.** `research-grade-harness/src/capability.rs`: seven §19
  families, hardcoded family edges, Kahn validation, digest-pinned nodes,
  manifest parsed by hand scanner (no serde_json). Static registry today —
  NOT yet embedded as mind coordinates (that embedding is §2/W2 below).
- **Honesty boundary (holds for all of §2–§4).** No hyperbolic tensor math
  runs in any live token forward path. RUST SERVER forward = regular
  fixed-cap-512 KV (backend README ruling, unchanged). Hyperbolic KV is
  Layer-A math + retrieval geometry. Everything below is INFERRED/PROPOSED
  from the VERIFIED substrate above unless labeled otherwise.

## 2. Workflows (t1) — each stated as: problem solved → brute force eliminated → mechanism

**W1. Ingest-and-index (write path).**
Problem: every future recall re-pays embed + scan. Mechanism: text →
`coord_from_text_idf` (shell-projected, rank-rotated) → atom + VP-tree
insert (rebuild iff stale, L262–267) + basin fold (VP-range refold,
O(N log N)-shaped, L1106+). Eliminates: re-embedding stored atoms (per-atom
caches backfilled once at open) and all future full scans over this item.
[VERIFIED mechanism; INFERRED framing]

**W2. Capability dispatch by geometry (the tree-sitter half).**
Problem: tool selection today is linear scan over a tool list or an extra
LLM routing round-trip per call. Mechanism (PROPOSED — the one genuinely
new build): embed each capability node **by the problem it solves** (its
docs/intake text → same `coord_from_text_idf` pipeline, family as a
coarse prior, e.g. family centroid as probe bias) into the same H3 x T2
space; a task probe does VP-range retrieval over capability leaves, then
the §19 family DAG admits/denies (can't invoke 06-DataKnowledge without
03+05 — the hardcoded edges become geometric + logical two-stage gate).
Eliminates: brute-force tool-list scan (O(N) → O(log N) average) AND the
router-LLM call. The hierarchy is why it works: families → skills →
leaves is a tree, and trees embed in H^n with arbitrarily low distortion
(§3, identity H1). [PROPOSED mechanism; VERIFIED premises]

**W3. Recall-vs-recompute per reasoning step (the hot loop).**
Problem: paying full-forward burn for already-concluded information.
Mechanism (largely VERIFIED — `map.attend` IS this): probe → VP shortlist
→ score mass → argmin_k F_recall(k) over {1,2,3,4,8,16,32}; cached
conclusion with valid producers → depth-0 fetch; else derive at depth k*.
Eliminates: fixed-width attention (width is a decision variable) and
recompute-on-every-step. I-RECALL (PROPOSED) makes violations auditable
(`wasteful` receipts → worst-first optimizer). [VERIFIED core; PROPOSED teeth]

**W4. Runtime attention build.**
Problem: static graphs materialize work the bound never justified.
Mechanism: materialize exactly k* rows per step; receipt carries
(complexity, inaccuracy, dissipation); session bound descends the same
number. Holds many steps of execution "at once" because holding = referencing
subtree roots (O(1) handles), not materializing rows. Eliminates:
build-maximal-then-mask. [INFERRED — boundary: row materialization at
token-tensor level is the unbuilt half; conclusion-level is VERIFIED]

**W5. Exact invalidation (closes Gap A).**
Problem: TTL/eviction guessing forces blind recompute of whole prefixes.
Mechanism (PROPOSED, designed in FEP doc §4): KV nodes carry
`producers[]`; producer re-ingest walks `edges` downstream, marks stale;
next fetch re-derives once. Eliminates: TTL tuning, full-prefix
recompute on any change, and dense-KV approximate eviction (H2O-style)
— excision is O(1) surgical because atoms are discrete, not entangled.
[Honest status: designed, not built; meter row 4 demands precision 1.0]

**W6. Agentic execution over the mind (mission layer).**
Problem: schedulers schedule *requests*; agentic work is *DAGs with tools*.
Mechanism (PROPOSED composition of VERIFIED parts): the existing
`Engine` packet loop stays the mission layer (sequential, judged,
receipted); packet admission consults W2 (capability geometry) + cached
prefix state (session artifact, TOPAS/TAO-style: longest-remaining-path
vs cache-credit); tool-return readiness gates admission (never admit a
packet whose tool is not ready). Eliminates: FIFO-over-DAGs,
admit-then-block-on-tool. [PROPOSED]

**W7. Consolidation and sundowning (the learning loop).**
Problem: systems recompute forever; rot is silent. Mechanism (VERIFIED
detector + PROPOSED meter): recall fraction `hits/(hits+derives)` must
rise across consolidations (learning to stop computing); bound drift
(`requires_iteration` spike) fires re-grounding — which §3 of the FEP
doc identifies as the law switching arms from recall to recompute.
Eliminates: push-through-on-rot. [VERIFIED detector; PROPOSED trend meter]

## 3. What hyperbolic KV does to a forward pass (t2) — the math deltas

Baseline identities (dense; REPORTED standard): prefill O(T²·d), decode
step O(T·d) with full KV scan, residency O(L·H·T·d), positions are integer
slots, eviction is approximate because representations are entangled.

**D1. Addressing: slots → neighborhoods.** A KV "position" becomes a
manifold neighborhood (Poincaré ball / Lorentz sheet). Lookup is k-NN
under the metric, not array index. Identity H1 (the load-bearing one):
**trees embed quasi-isometrically in H^n with O(1) distortion while
Euclidean needs unbounded dimension** [REPORTED — classical (Gromov;
Nickel-Kiela Poincaré embeddings — both in the vault); never VERIFIED
numerically here]. Execution traces (plan → steps → tool calls → conclusions)
ARE trees, so the trace store gets exponentially more headroom per
dimension than a flat cache: volume grows ~e^r. That is the precise
content of "hold many steps at once without bloat." [INFERRED]

**D2. Prefill: materialize → insert.** Instead of computing all L·H·T·d
entries, prefill embeds tokens as coordinates and inserts (VP amortized
O(log N); refold O(N log N)-shaped [VERIFIED in `main.rs` L1106+]).
Shared prefixes are never recomputed: they are shared ancestor paths
(the geometric form of radix-tries — vault 2606.01143 schedule-level
prefix reuse is the same idea in Euclidean clothing). Cost moves from
O(T²·d) to O(T log N + k·d) for prefix-hit-dominant traffic. [INFERRED]

**D3. Decode: extend-row → recall-gated derive.** Each step computes
F_recall(k) vs F_recompute and materializes only k* rows — or zero rows
on a valid cached conclusion. Oracle (VIII): per-step floor drops from
O(T·d) to O(log N + k*·d). [INFERRED — the oracle this whole framework
is measured against; L1's first verification script]

**D4. DecodeT stall structure changes.** Chunked prefill exists to bound
the decode stall (Rapid: 500 ms target). Under D2/D3 the stall source
shrinks: prefills that hit shared subtrees cost O(log N), and decodes
that hit conclusions cost O(1) fetch. The scheduler's yield points move
from token-chunk boundaries to **recall-verify boundaries** (Erlang §1.3
of the scheduler audit, unchanged in spirit). [INFERRED]

**D5. Eviction → excision + edge walk.** Dense eviction must guess
importance (approximate, quality-risking). Discrete atoms permit exact
O(1) excision with Merkle transition proof (already VERIFIED pattern:
surgical erasure with 0-residual + root change, backend README) plus
exact downstream invalidation (W5). The price: SOMEONE must maintain the
producer edges (Gap A build) — without them staleness is luck, as the FEP
doc honestly states. [INFERRED with VERIFIED precedent]

**D6. Rank structure = built-in hierarchy.** Rank 0 (chunk) / rank 1
(document-aggregate) with `rotate_rank` phase suppression of cross-rank
inner products [VERIFIED `mapmath.rs`] is the two-level prototype of
nested KV: deeper ranks = deeper tree levels, each a rotated subspace.
Prefill-share across ranks (vault: PrefillShare 2602.12029, ACCT
2610.42116 cross-model transfer) has a native geometric home here.
[INFERRED extension of VERIFIED mechanism]

## 4. FEP binding (t3) — one bound, three prices, every workflow a descent

The law (VERIFIED `fep.rs` + FEP doc): **`F = complexity − accuracy +
β·dissipation`**, every decision descends it, session total is the
accumulator. The user's three constraints map 1:1 onto the three terms
[INFERRED mapping, stated boundaries]:

- **Minimize prediction error** = minimize inaccuracy (−score mass,
  −log-likelihood). Precision-weighting (`precision_update`:
  confidently-wrong moves beliefs far) and the `requires_iteration`
  spike gate are the mechanisms. Quality of execution output is not a
  fourth term — it IS accuracy, judged (judge-passed tasks), never
  self-reported. [VERIFIED mechanisms]
- **Minimize thermodynamic cost** = minimize dissipation (Landauer anchor
  2.87e-21 J/bit; per-row C_ROW_J inside the depth-gate argmin, so the
  gate prices its own cost instead of reporting it after the fact —
  Gap B closed in `map.attend`). [VERIFIED]
- **β is the exchange rate** between the two (DEFAULT_BETA 1.0; audit
  modes raise it). There is no tension-free optimum; there is only the
  argmin at a stated β. Any proposal that "minimizes both" without naming
  β is moving the goalposts (Commandment II). [INFERRED — discipline rule]

I-RECALL is the audit teeth on all three: a derivation that runs while a
valid cached conclusion exists is receipted `wasteful` — right answer,
overpriced bound — and three such marks feed the worst-first recursive
optimizer automatically. [PROPOSED invariant, VERIFIED optimizer hook]

## 5. Counterexamples (math extra — the fastest way to kill a conjecture)

- **CE1 (kills "brute force is eliminated, unconditionally").**
  Adversarial query equidistant to all atoms: VP pruning proves nothing,
  search degrades to full scan O(N). Correctness holds (exact-prune never
  lies — asserted by overlap==k), speed doesn't. Claim restricted:
  elimination is *average-case over tree-structured load*, worst case is
  scan-parity, never worse. [INFERRED]
- **CE2 (kills "hyperbolic fits everything").** Expander graphs and dense
  cross-linked DAGs have Gromov δ large — distortion lower bounds bite.
  The framework's win is confined to tree-like structure (traces, plans,
  prefix hierarchies, capability families). Heavy-mesh dependency graphs
  should stay Euclidean/adjacency-indexed. One variable (VI): the
  distortion probe is per-workload, not global. [INFERRED]
- **CE3 (kills "recall always wins").** Stale-producer + no-edge case:
  recall returns a confident wrong conclusion cheaper than recompute —
  F drops while truth doesn't (Goodhart on the bound). This is exactly
  why Gap A (dependency edges) BLOCKS any claim that recall is safe,
  and why invalidation precision meter demands 1.0 exact. Until W5 is
  built, recall is gated to receipted-producer-stable conclusions only.
  [INFERRED — falsifier for premature recall claims]
- **CE4 (kills "β = 1 always").** Under hard SLO (agentic tool window),
  β→0 frugality misses the deadline; under energy audit, β→∞ accuracy
  blows the joule budget. β must be set per session class and receipted,
  not defaulted silently. [INFERRED]

## 6. Provenance rows

| # | Source | Version / date | License | Claim carried |
|---|---|---|---|---|
| P1 | `hypermind-nested-kv/src/main.rs` + `mapmath.rs` + `ATTENTION-ECONOMICS-FEP.md` | on disc, read 2026-11-02 (attend L3531–3660, VP L108–267, dist L761–780, gate L1499–1510) | repo (unchanged) | §1 substrate, W1/W3/W4/W7 mechanisms |
| P2 | `research-grade-harness/src/fep.rs`, `capability.rs`, `map_plugin.rs` | on disc, read 2026-11-02 | repo | §1 FEP core + capability DAG (W2/W6 premises) |
| P3 | `backend/SCHEDULER-AUDIT-2026-11-02.md` (this repo, prior session step) | 2026-11-02 | repo | Erlang frame §1.3/§1.5 referenced; Wave-1 (TAO/TOPAS) as W6 prior art |
| P4 | Vault: Nickel–Kiela Poincaré embeddings, Gulcehre hyperbolic attention, 2606.01143 schedule-prefix-reuse, 2602.12029 PrefillShare, 2610.42116 ACCT, 2312.07104 RadixAttention | existence VERIFIED 2026-11-02; contents NOT opened | n/a (local notes) | H1 tree-embedding premise; prefix-reuse/KV-transfer corroboration |
| P5 | Backend README §RG-AH rulings + N3 CapabilityTreeSitter manifest/dirs | read 2026-11-02 | repo | Honesty boundary (§1): hyperbolic KV = retrieval geometry, not forward path; capability dirs exist, mind-embedding PROPOSED |

## 7. Exit disposition + staged probes (VII/VIII/X, deferred with reason)

- C1 MET (§2, seven workflows, brute-force step named each). C2 MET (§3,
  six deltas + oracle O(T·d)→O(log N + k*·d) on paper). C3 MET (§4,
  term-by-term mapping + β rule). C4 MET (§5, four counterexamples).
  C5 MET: labels used — VERIFIED (read on disc), REPORTED (source-said /
  classical), INFERRED (reasoned, boundary named), PROPOSED (unbuilt).
  Zero code written — L0 discipline holds (Commandment IV: plan's next
  steps are L1 scripts, not this session's work).
- Deferred L1 (formula verification, Commandment X): `scripts/` probes —
  (a) VP-range vs brute-force parity on capability-coordinate fixtures
  n≤10; (b) depth-gate argmin recomputed independently from receipted
  `fep_terms`; (c) distortion probe: tree vs mesh fixture sets, Gromov-δ
  proxy vs recall Hit@k. Each one variable (VI), each with frozen inputs
  + oracle pre-registered (VIII) before running.
- Deferred L2: staged n=1/10/100/full recall-fraction + joule ledgers on
  the mind MCP path only (never the token forward path until the kernel
  register ablation promotes it — anti-pattern: bench kernel ≠ forward pass).

**Verdict**: L0 framework complete — workflows, math deltas, and FEP
binding stated with counterexamples and staged probes named. No confident
solution is claimed for the token-level forward pass (CE1–CE4 + Gap A
block it); the conclusion-level path (`map.attend` + I-RECALL + W5) is
the falsifiable next build.
**Level used**: L0; L1/L2 deferred per §7 (Commandment I — recorded, not silent).
**Methods**: read-then-derive; oracle-first; counterexample-first; label-everything.
**Answer**: §2 (workflows) + §3 (forward-pass deltas) + §4 (FEP binding).
**Report**: this file (`backend/HYPERBOLIC-SCHEDULER-FRAMEWORK-2026-11-02.md`).
**Verification**: no scripts this level by design; L1 probe list in §7 is the
pre-commitment (X) for the next session.
**Self-verification notes**: holes closed — W2 mechanism specified (problem-
embedded capability coords + DAG gate), oracle stated, β-goalpost rule set.
Holes remaining — [GAP: Gap A edges unbuilt — CE3 blocks safe-recall claims],
[GAP: token-level row materialization unbuilt — W4 half], [GAP: β schedule per
session class unreceipted]. Commandments touched: I (deferred list §7), II
(β rule + frozen-oracle rule), III (P1–P5 tiers), VIII (oracle §3/D3), X
(L1 pre-commitment).
