# Scheduler Deep-Dive: Erlang-Frame Audit + arXiv Pattern Read + 3-Wave Prediction

Date: 2026-11-02. Status: PROPOSED (assessment, not a build directive).
Session scope: the Rust server + research harness backend of the agentic research lab.
No Erlang code is used anywhere. Erlang is the analytical frame only.

## 0. Research frame (research-runbook §1–§2, pre-registered before sources)

**Question.** What scheduling discipline should the Rust inference-engine
runtime use — framed in Erlang-scheduler terms — given what Rapid-MLX,
oMLX, SGLang, and Swift-MLX each solve and why, plus where the current
arxiv scheduler literature is moving?

**Decision this unblocks.** The scheduler architecture for the backend
`harness::runtime::Engine` + RUST SERVER `serving/scheduler.rs` line:
admission, ordering, preemption granularity, prefix locality, failure
containment. No scheduler code is written on the basis of this doc alone;
a kernel-register necessity ablation is still required before any novel
path enters a forward path (per backend README §RG-AH rulings).

**Acceptance criteria (dated 2026-11-02, before source rows below).**

- C1: each audited scheduler's admission + ordering + preemption named with a source.
- C2: each audited scheduler's cache/memory story named with a source.
- C3: ≥5 arxiv preprints from the Oct–Nov 2026 window recorded at abstract level.
- C4: implementation patterns stated as INFERRED with the evidence boundary named.
- C5: exactly 3 predicted waves, each PROPOSED, each with a confirm/falsify trigger.

## 1. The Erlang frame (the lens, not the code)

Erlang's scheduler, reduced to the five properties this audit maps onto
each inference scheduler [REPORTED — training knowledge, not re-verified
on disk this session]:

1. **Quantum by reductions, not time.** A process runs ~2000 reductions,
   then is preempted. Fairness unit = work done, not wall clock.
   Inference analogue: the fair quantum is **tokens / batch-token budget
   per iteration**, not milliseconds.
2. **Per-core run queues + migration.** Each scheduler has a local queue;
   work migrates only under imbalance (with limits). Inference analogue:
   per-lane queues with **prefix-affinity placement** (migrate a request
   to where its prefix state already lives) instead of blind FIFO.
3. **Preemption only at yield points.** A process yields at defined
   points; long native work blocks its scheduler. Inference analogue:
   chunked prefill and per-step decode are the yield points; an
   unbounded prefill with no chunk boundary is a scheduler block.
4. **Isolation ("let it crash").** A process dies alone; supervisors
   decide restart. Inference analogue: a poisoned/overflowing sequence
   fails **alone** (reject/fail that sequence), never the batch, and a
   contract fault latches admission closed rather than corrupting state.
5. **Backpressure at the edge.** Message queues are bounded by monitoring
   and overload control, not by hoping producers behave. Inference
   analogue: admission caps + 503-with-Retry-After **before** allocation,
   not OOM after it.

## 2. Scheduler audits (one per server)

### 2.1 Rapid-MLX 1.0.0 (local install, this Mac)

**Discipline.** Continuous batching with FCFS default + opt-in
cache-aware ordering [VERIFIED — `rapid-mlx serve --help` output read
directly 2026-11-02; `rapid_mlx/scheduler.py` `SchedulerConfig` fields
read directly, L397–767].

- Waiting queue FCFS; `--scheduling-policy {fcfs,shortest_validated_tail}`
  with `--scheduling-max-deferrals 8` — the opt-in policy grants an open
  prompt slot to the request with the least cache-validated work and
  forces FIFO after 8 deferrals (starvation bound built in).
- Admission before allocation: `--max-num-seqs` (default 256),
  `--max-concurrent-requests` (default 256, 503 + Retry-After over cap),
  `--prefill-batch-size 8`, `--completion-batch-size 32`,
  `--prefill-step-size 2048` with adaptive prefill guard, plus
  D-METAL-CAP admission (projected `active + tokens × kv_bytes_per_token`
  vs cap — rejects a 32k prefill **before** it allocates past the cap).
- Decode protection: `--decode-stall-target-ms 500` caps concurrent
  prefill chunk size while any row decodes; `--shared-prefix-wait-tokens
  1024` lets a request wait for an in-flight shared prefix instead of
  recomputing.
- Speculative lanes: `--speculative-config` methods `dflash`, `ddtree`,
  `mtp` (native MTP / q8 sidecar / continuous self-MTP, default off),
  `suffix` (drafter-free suffix-tree, `num_speculative_tokens 8`).

**Cache/memory.** Radix-trie prefix index default
(`--prefix-cache-index {radix,hash}`, radix = O(prefix_len) lookup +
dedup accounting); memory-aware eviction at ~20% RAM; `--hybrid-cache-entries`
for non-trimmable recurrent-state prefixes; TurboQuant k8v4 KV
compression (~4.6x, opt-in); response cache for exact greedy repeats;
opt-in paged cache (`--use-paged-cache`, block 64) gated to plain
full-attention KV layouts.

**What it solves and why.** Single-Mac, long-agent-session serving:
bursty tool-calling traffic with heavy shared prefixes (system prompts,
ReAct loops, JSON schemas) on unified memory with no VRAM wall to hide
behind. Every knob prices one tradeoff explicitly — prefill batch size
8 note admits staggered ragged rows push batched attention onto a slower
path; KV int4/int8 dequant costs O(context) per decode step (measured
−27%/−36% at 16k ctx). It solves **protect-the-desktop + amortize-repeat
-work**, and says the prices out loud [VERIFIED — help text].

### 2.2 oMLX 0.7.0 (local install, this Mac)

**Discipline.** FCFS waiting queue + capped admission + continuous
batching through mlx-lm `BatchGenerator` [REPORTED — delegated source
read 2026-11-02: `omlx/scheduler.py` waiting deque FCFS,
`max_num_seqs 256`, prefill external then `insert()` with
`prefill_batch_size=1`, `step()` admits → chunked prefill → single
`next_generated()` for all running rows; queue cap `max(4×seqs,32)` →
503; CLI flags `--max-concurrent-requests` (default 8) VERIFIED from
`omlx serve --help` read directly].

**Cache/memory.** Two-tier prefix reuse: paged **SSD** cache
(256-token blocks, LRU index, ~100 GB default) + in-RAM **hot** tier
(disabled by default, promotes on hit, write-back evictions); delay
prefix lookup to admission so same-prefix requests rendezvous with
in-flight writes.

**Memory guard.** `ProcessMemoryEnforcer`: soft = evict idle LRU models
+ pause admission; hard = abort a non-pinned victim's in-flight work;
pinned actives die only under emergency pressure; decode loop defers
`mx.clear_cache()` to avoid IOKit `completeMemory` kernel panics
[REPORTED — delegated source read].

**What it solves and why.** Multi-model single-user agent serving on one
Mac: one process hosts many LRU-evicted models, multiplexes bursty
long-context agentic requests, and biases every decision toward **the
machine survives** (reserves, pauses, evictions) over raw throughput. It
solves **desktop coexistence**, not multi-tenant fair-share — no tenants,
quotas, or priorities on the data path [INFERRED — boundary: no
priority/tenant constructs found in the delegated read; absence is weak
evidence].

### 2.3 SGLang ("SgLang")

**Discipline** [REPORTED — training knowledge, not re-verified on disk;
no SGLang checkout exists on this Mac]. RadixAttention prefix cache
(token-trie KV store, ref-counted against live decodes) + **cache-aware
scheduling** (score queued requests by matched-prefix length — locality
first, not strict FIFO) + continuous batching with **chunked prefill**
+ prefill/decode disaggregation options (KV transfer, Mooncake-style) +
draft-model speculative decoding.

**What it solves and why.** Multi-tenant serving where the same system
prompts, few-shot blocks, tool schemas, and chat histories repeat across
requests: without sharing, every request repays full prefill compute and
memory bandwidth, TTFT inflates, throughput collapses. It solves
**cross-request repeat-work at fleet scale**, trading strict fairness for
hit-rate [INFERRED — boundary: standard published characterization;
corroborated by vault holding `preprints/2023-12__Unknown__2312.07104__
RadixAttention-SGLang-2312-07104.pdf`, existence VERIFIED, content not
opened].

### 2.4 Swift-MLX

**Status: NOT FOUND as a server** [REPORTED — delegated disk check
2026-11-02: no Swift/MLX server in `~/Desktop/GitHub/`, `/opt/homebrew/bin`,
pip, or the `MLX Servers/` vault dir; Swift 6.4.2 toolchain present but
no MLX-Swift checkout]. The repo's own intake note already treats
`mlx-swift` (Apple's `ml-explore/mlx-swift`) as bindings/examples for a
future Swift boundary, and cites SGLang/RadixAttention for
radix-prefix + ref-count concepts. Plausible reading: the user means
Apple's `mlx-swift` package or a hypothetical Swift port — neither is a
production OpenAI-compatible server on this Mac today. No audit possible
beyond: **a Swift lane would inherit whichever discipline it ports; the
discipline decision is language-independent** [INFERRED].

### 2.5 RUST SERVER `serving/scheduler.rs` (+ MTP/device paths)

**Discipline** [REPORTED — delegated source read with file:line
citations, 2026-11-02; `scheduler.rs` L1–80 VERIFIED directly].

- Bounded admission at submit (prompt length, max-new, context sum,
  vocab; `mpsc` queue cap) → active set capped (`max_sequences`) →
  **per-iteration token budget** (`max_batch_tokens`): prefill takes
  `min(chunk, budget)`, decode emits exactly 1 token per row,
  `budget -= count`. Fair iteration with continuation-safe FIFO.
- **Budget admitted before compute** (`Codec::reserve_frame` /
  `sequence_bytes` fail → fail that sequence), workspace reservation
  fails the whole pending set; backend contract validated after every
  step (cardinality, IDs, KV shape, tokens); **contract/compute fault →
  poison latch + fail batch**, stream-full invariant, orderly shutdown.
- MTP lane: same fair `Engine` as a `ModelBackend`; draft width 1–8,
  pending draft queue drains one validated step at a time; overflow =
  **reject/fail, never evict**.
- Device lane: opt-in packed `decode_batch` for equal-position rows
  with scalar fallback + A/B counters.

**Cache/memory.** Live forward path = **regular fixed-cap 512-token
KV** (context 512, chunk 8; MTP 511/512 split); a configurable path
(512–262144, 256-aligned) exists but is not the live default
[REPORTED — delegated read; consistent with backend README ruling].

**What it solves and why.** Bounded, starvation-free continuous serving
on a fixed-memory Apple-native path: admit only what fits, interleave
chunked prefills with single-token decodes, pack only
geometry-compatible rows, speculate via MTP, fail-or-latch loudly
instead of evicting live KV. It solves **deterministic survival under
bursty variable prompts** — the Erlang properties §1.3–§1.5 are already
its native shape [INFERRED].

### 2.6 Backend `harness::runtime::Engine` (this repo, today)

**Status.** `Engine` (`backend/src/harness/runtime.rs`, read directly
2026-11-02, L129–598) is a **sequential DAG-packet executor, not a
concurrent inference scheduler**: `run_sequential` and
`run_sequential_live` both loop `ready_packets → run one → judge →
pass/retry/escalate/skip-dependents`. `max_parallel: 4` exists as a
field but no execution path reads it [VERIFIED — both functions read in
full]. Fail-closed honesty floor holds (no executor → every packet
ESCALATEs, never synthetic PASS). This is the correct **mission layer**;
the token-scheduling layer it needs underneath (admission, batching,
prefix locality, backpressure) does not exist in this crate yet and is
exactly what this audit scopes [INFERRED].

## 3. Provenance rows

| # | Source (URL / path) | Version / date | License | Claim carried |
|---|---|---|---|---|
| P1 | `/opt/homebrew/bin/rapid-mlx` → `.../rapid_mlx/scheduler.py`, `serve --help` | rapid-mlx 1.0.0, read 2026-11-02 | NOT CHECKED (license file not opened) | §2.1 discipline + cache knobs |
| P2 | `/opt/homebrew/bin/omlx` → `omlx/scheduler.py`, `cache/paged_ssd_cache.py`, `process_memory_enforcer.py`, `serve --help` | oMLX 0.7.0, delegated read 2026-11-02; CLI help direct | NOT CHECKED | §2.2 discipline + tiers + guard |
| P3 | SGLang design (training knowledge) + vault `preprints/2023-12__Unknown__2312.07104__RadixAttention-SGLang-2312-07104.pdf` | paper 2312.07104; file existence VERIFIED, content not opened | Apache-2.0 [REPORTED, not verified] | §2.3 |
| P4 | Disk search (delegated): `~/Desktop/GitHub/`, `/opt/homebrew/bin`, pip, vault `MLX Servers/` | 2026-11-02 | n/a | §2.4 NOT-FOUND |
| P5 | `RUST SERVER/src/serving/{scheduler, native_mtp_scheduler, native_device_scheduler, batch_transaction}.rs` + `docs/{SERVING_ARCHITECTURE,GRID_THROUGHPUT,SPEC15_*}.md`; twin `Rust-TS-Tokio-Inference-Engine` diff-clean | delegated read + diff 2026-11-02; `scheduler.rs` L1–80 direct | LICENSE file present, contents not opened | §2.5 |
| P6 | `Ares WebSite : App/backend/src/harness/runtime.rs` L129–598 | read directly 2026-11-02 | repo license (unchanged) | §2.6 |
| P7 | arxiv listing `cs.DC` recent + abstracts: 2610.41508 (HiState), 2610.39216 (PDScope), 2610.37810 (ConfluxServe), 2610.38355 (FlatCake), 2610.41160 (ARK), 2610.40287 (spec-energy), 2610.41359 (LEAP), 2610.37307 (EdgePace), 2610.37230 (TAO), 2610.33501 (DualPace) | v1, 24–30 Oct 2026; abstracts fetched 2026-11-02 | per-paper license links present, individual texts not opened | §4–§5 |
| P8 | Vault abstract records: TOPAS 2608.25523, ASPIRE 2609.17943, Elastic-KV 2608.23658, Power-Provisioning 2609.24639 | vault files read 2026-11-02 (abstract-level records, not full papers) | n/a (local notes) | §4 |

## 4. Implementation patterns (INFERRED — boundary: audits §2 + abstracts §3, no new measurements this session)

1. **Admission-before-allocation is universal.** FCFS + caps (oMLX),
   D-METAL-CAP projection (Rapid), token/byte budgets + poison latch
   (RUST SERVER), power-provisioned PD ratios (Yan 2609.24639). Every
   mature design rejects before allocating. The pattern's source: Metal
   OOMs and 179 GB runaway growth whole-machine kill the server, so the
   gate moved to admission everywhere.
2. **Prefill/decode split, five ways.** Chunked co-batching (SGLang,
   Rapid `decode-stall-target-ms`), phase-separated pools + KV transfer
   (disaggregation literature; ARK 2610.41160 fixes its network
   collisions; Yan models its power Pareto), phase-specific expert
   policies (PDScope), per-operator parallelism (ConfluxServe XP),
   decode-only host engine (DualPace). Source: prefill is compute-bound,
   decode is memory-bound — one policy cannot serve both.
3. **Prefix locality beats FIFO, with a starvation bound.** Radix hit
   scoring (SGLang), `shortest_validated_tail` + max-8 deferrals
   (Rapid), shared-prefix wait (Rapid), request sorting = prefix sharing
   (vault 2609.13692), workflow-aware DAG scoring + aging (TOPAS). Every
   locality win ships with its anti-starvation device. Source: agentic
   traffic repeats prefixes; blind FIFO recomputes them.
4. **KV hierarchy flattens toward session scope.** SSD-paged + hot RAM
   (oMLX, Rapid disk tiers), checkpoint-on-SSD + KV-in-fast-mem for
   hybrids (HiState), flat session-contiguous placement, never migrate
   (FlatCake), cross-model sparse transfer (ACCT 2610.42116). Source: hot
   DRAM as a staging tier breaks under bimodal reuse (one-shot blocks +
   multi-GB agent sessions).
5. **Speculation goes conditional and async.** Continuous/allowed-off MTP
   (Rapid), async per-request draft/verify choice by acceptance estimate
   (ASPIRE), energy ledger where low acceptance **raises** energy/token
   at large batch (Ma 2610.40287 — speculation's saving shrinks as
   batching already amortizes the weight read). Source: verification
   dominates cost on dense models; blind speculation is a tax.
6. **Length/work-awareness without retraining cost.** LEAP reuses
   prefill hidden states + RLS online readout on CPU (30,569× fewer
   params, +64.7% rank correlation under shift); DWISE refines from
   first-round hidden states. Source: predictors rot under domain shift;
   retraining is priced out, so prediction moved to free signals +
   online adaptation.

## 5. The next 3 waves (PROPOSED — predictions, not findings)

**Wave 1 — Workflow-aware, tool-aware admission (nearest).**
Schedulers stop scheduling *requests* and start scheduling *task DAGs*:
TAO's tool-readiness + cache-credit + token-footprint value function and
TOPAS's longest-remaining-path scoring are the early shape; LEAP-style
length signals feed the same decider. Confirm trigger: an SGLang/vLLM
mainline PR merges tool-return-readiness into admission scoring.
Falsify: 6 months with no mainline adoption beyond papers.

**Wave 2 — Phase-split execution as default topology (middle).**
Prefill/decode disaggregation graduates from fleet exotic to standard
shape, with the collision/power/parallelism problems solved in the open:
ARK-style flow reservation, Yan-style power Pareto provisioning,
Conflux-style per-operator parallelism, and — for single-machine Apple
targets — DualPace-style host-as-decode-engine. Confirm trigger:
first-class PD-split support in an MLX/ecosystem server or a published
unified-memory PD-split design. Falsify: disaggregation stays
H100-cluster-only through 2027.

**Wave 3 — Flat, session-scoped, cross-model KV as the memory substrate
(farthest).** FlatCake's never-migrate session-contiguous placement +
HiState's SSD-checkpoint/fast-KV split for hybrids + ACCT-style
cross-model transfer converge: the KV cache becomes an addressable,
transferable session artifact, not a per-engine ephemeral. Scheduling
then means *routing to where the KV already is* (CacheRoute affinity,
ARK flows), and length/prefix predictors (LEAP) become the routing
index. Confirm trigger: a serving paper or system demonstrating
cross-engine KV reuse at ≥50% conversion win on agentic traces.
Falsify: KV stays engine-private with transfer costs dominating.

## 6. What this means for the Rust runtime (PROPOSED directives)

1. Keep the Erlang §1 shape already native to `scheduler.rs`: token
   budget = reductions quantum; chunk boundaries = yield points; fail
   sequence / latch admission = let-it-crash supervision; caps before
   allocation = edge backpressure. Do not regress any of these.
2. First scheduler upgrade with measurable payoff: **opt-in
   prefix-aware ordering with a deferral bound** (Rapid's
   `shortest_validated_tail` + max-deferrals is the proven template) —
   gated by a necessity ablation vs FIFO on repeated-prefix agent load.
3. Second: **conditional speculation** — MTP/draft paths governed by a
   live acceptance-rate cost model (ASPIRE/Ma-energy logic), default
   off, never unconditional.
4. Do not build: engine-private elaborate KV tiers before Wave-3
   evidence lands on this hardware; the fixed-512 live path + reject
   (never evict) stays until an ablation dethrones it.
5. `harness::runtime::Engine` stays the mission layer (sequential,
   judged, receipted). If a concurrent token scheduler is ever added
   under it, `max_parallel` must be wired to a real semaphore + broadcast
   shutdown — today it is an unread field, and an unread concurrency
   knob is a latent lie [VERIFIED §2.6].

## 7. Exit-criteria disposition

- C1 MET (P1, P2, P3, P5, P6). C2 MET (same). Swift-MLX dispositioned as
  NOT-FOUND with search paths named (P4) — unmet through no-server-existing,
  recorded, not hidden.
- C3 MET: 10 Oct–Nov-2026 abstracts (P7) + 4 vault records (P8).
- C4 MET: §4, each pattern INFERRED with boundary stated.
- C5 MET: §5, three PROPOSED waves with confirm/falsify triggers.
- Zero unlabeled claims is asserted for this doc; labels used: VERIFIED
  (direct read this session), REPORTED (delegated read / training /
  CLI-adjacent), INFERRED (reasoned, boundary named), PROPOSED (plan,
  not fact).
