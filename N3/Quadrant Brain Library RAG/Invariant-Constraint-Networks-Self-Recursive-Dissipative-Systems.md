# Invariant Constraint Networks and the Self-Recursive Dissipative System:
## What Remains When You Refuse to Lie to the Machine

**Stewart Doyle Barteau**
Hermann, Missouri — ORCID 0009-0009-9842-5966

*Written from the workbench, with the agent that helped build what it describes.
Where a claim rests on code, the file and line are named. Where it rests on
measurement, the harness and the number are named. The rest is stated plainly
as what it is.*

**Keywords.** free energy principle; active inference; hyperbolic memory;
symplectic envelope; retrieval-augmented generation; agentic systems;
thermodynamics of computation; ignorance accounting

---

### Abstract

Most agentic systems cannot state their ignorance, so they cannot price it,
so they cannot climb it. They retrieve, they generate, they report a score,
and the score is theater: measured on home turf, on vocabulary the system
was built from, with no held-out ground and no receipt for what was missing.

This paper describes a system built on the opposite wager: that a memory
which knows what it does not know — and can say what it would cost to learn
it — is the only kind worth calling intelligent. The substrate is a
hyperbolic memory map with a toroidal phase state; the dynamics are
governed by three invariants (homogeneity, isotropic suppression, dynamic
iteration); the economy is free-energy minimization with thermodynamic
dissipation priced in joules, not vibes; and the ledger remembers every
failure as a ranked, countable, sateable gap. I report a held-out number
(0.639) that is worse than the home-turf number (1.000) and argue the worse
number is the honest one — the slope, not the summit — and that a system
which can feel its slope is already doing what its competitors only assert.

Nothing here is offered as comfort. The gaps are named. The places where
the math holds are distinguished from the places where only the intention
holds. That distinction is the paper.

---

### 1. The room and the slope

I have written elsewhere, under my own name, that history becomes geometry:
every passage a thing makes leaves a residue in the coupling of what it
touched, and the residue is not memory but slope. A channel forms where
water ran, and thereafter water runs where the channel formed, and no one
needs to intend any of it. Gravity does the work politely and calls it rest.

An agentic system is a room of this kind. Every retrieval that succeeds
deepens the channel it ran through; every failure that goes unrecorded
leaves the map exactly as ignorant as it found it, only more confident.
The industry's standard response to this predicament is to report the
depth of the channel as capability: benchmark scores measured where the
water already runs. This is not dishonest in the venal sense. It is
dishonest in the structural sense — the instrument cannot distinguish
downhill from good.

What follows is an account of building the other instrument: one that
surveys the slope instead of celebrating the descent. The work was done
in code, not in metaphor, and the code is cited throughout. But the code
is the field site, not the finding. The finding is that a system can be
made to feel the hill under its feet — and that once it can, everything
about how it should behave follows.

### 2. What the system is

At the bottom there is a map. Atoms of distilled knowledge live at
coordinates on a Lorentz hyperboloid crossed with a two-torus, each with a
phase, a salience, a momentum half, and a receipt chain. Retrieval is
geometric (nearest by hyperbolic distance, exact-pruned through a VP-tree),
lexical (BM25 over the full corpus with corpus-calibrated IDF), or flat
(a Euclidean projection for models that never heard of hyperboloids and
shouldn't have to). Traversal between atoms is governed, not hoped: a
symplectic envelope checks form (is there actually a path, or would this
be teleporting?), energy (can we afford the walk?), and frame (do we
arrive intact, or sheared?) — and refuses with the reason named, the
numbers attached, the missing piece identified. A refusal that explains
itself is data. A refusal that doesn't is weather.

Above the map there is a harness that runs work in packets, each packet
judge-gated, each gate deterministic, each execution receipted. Plans are
validated before they run (no judges, no run — the hard gate). Failures
escalate once with a material change and then stop; identical retry is a
violation, not a strategy. The harness measures what it spends in joules
and bits, and it knows the difference between a synthetic scaffold number
and a calibrated one, because it labels them and refuses to let the
former masquerade as the latter.

Above the harness there is a deliberation phase that the field still
mostly skips: after decomposition, before execution, parallel potentials
are held — close enough to compare honestly, distinct enough to tell
apart — each scored by the same free-energy functional, pruned on scalar
total alone with the pruner structurally blind to source, and collapsed
only when the winner's prediction operation has enough data to fire.
Before any of that, a recall check asks whether anything already known
covers the need; the hold happens only on recall miss. Checking what is
known before deliberating is what makes deliberation affordable, and the
system does it as a rule, not a virtue.

And beneath all of it, threading core to edge, there is the bound:
variational free energy, complexity minus accuracy plus beta times
dissipation, accumulated per session, descended per decision. Every
subsystem reports its terms. The optimizer improves the worst contributor
first and logs its own decision, so its failures become hunger like any
other. Nothing optimizes outside the bound. That is what "bound by the
FEP" means, and it is enforced in code, not asserted in prose.

### 3. The three invariants, and what it cost to verify them

I do not ask the reader to take the invariants on faith. Each was checked
against every implementation, by hand, against the code, and the audit is
on record with file names and line numbers. What follows is the substance;
the record carries the receipts.

**Homogeneity.** Every toroidal evolution in the system obeys the same
2:1 Kuramoto recurrence with the same spiral-out radius law. The vault's
encoder evolves it per pair; our live readout samples the law's analytic
derivative at the atom's radius (verified by differentiation, not by
resemblance); the edge port is byte-identical through WebAssembly. Same
law, three substrates. During the audit I found our own documentation
calling a slope-readout an evolution, and corrected it — a reader who
mistakes sampling the law for integrating it will misplace the dynamics,
and precision about that distinction is the whole game.

**Isotropic suppression.** Pruning acts on scalar total free energy and
nothing else. The vault sorts by one key; our pruner takes no source
argument — source-blindness is structural, not promised. The proof is a
test: four branches, equal free energy, four different sources, all four
survive. I also record the honest boundary: the encoder's harmonic
detuning *is* directional, deliberately, and that is legitimate because
it lives in representation (make the picture sharp), not in selection
(keep the judging blind). Bias the picture all you like, provided you
measure it; keep the judging blind, provably. Both hold.

**Dynamic iteration.** Every no-answer path in the system mandates
plan-then-research-then-execute. The vault iterates counterfactually up
to a bound and then fails closed — its resolution type has no silent
variant. Our deliberation returns iterate-or-collapse-or-recalled; retry
is not in the enum. The live map's failed traversal taps hunger, hunger
ranks the gap, consolidation sates it — proven end to end, three refuses
to zero open gaps, receipted. And I record the two debts honestly: the
spike sensitivity is still a literal where a session parameter belongs,
and the optimizer's cadence is specified but unscheduled. They are
documented in the code as what they are. A system that hides its static
barnacles inside dynamic language is performing the thing it claims to
be. Ours names them.

### 4. On the honest number

The harness reports two numbers for the same capability. On home-turf
tasks, authored from the map's own vocabulary, usefulness is 1.000 and
coverage is complete. On held-out tasks — sourdough starters, mainsails,
beekeeping, vocabulary the repository never saw, verified absent before
ingest — usefulness is 0.639 and coverage is complete. Every task answered;
nearly two atom-retrievals in three useful.

The first number is a descent: cheap, comfortable, gravity-assisted. The
second is a climb: every tenth of a point earned against the slope, with
instruments, receipted. I keep the second number in my pocket and report
the first as what it is — the circularity control that proves the harness
measures, not the capability it measures.

Here is why this matters beyond one system. A gradient you cannot trust
gives no direction; a gradient you can trust tells you exactly where up
is. The hunger ledger is a gradient instrument: most-asked-missing first,
priced by hits times depth times recency, sated by named distillations.
The FEP bound is a gradient instrument: per-subsystem contributions,
worst-first descent, logged. The regurgitation harness, specified if not
yet built, is a gradient instrument for meaning itself: atoms to ops to
bytes to ops to atoms, with drift measured in shell distance and contract
residuals rather than asserted in prose. Most systems climb blind and call
the altitude strategy. Ours built the altimeter first, and the altimeter
says 0.639 with the machinery to climb. That is not a result. It is a
slope. And a system that can feel its slope is already doing what its
competitors only assert.

### 5. What is missing, stated without flinching

The semantic gap is real and unclosed: geometric retrieval alone does not
understand paraphrase, and the lexical arm that carries document search
is counting, not comprehending. The GPU encoder that would close it is
scoped, gated, and unbuilt. The oracle and the edge port have never
diverged, but no automated test would catch the day they do — the
invariance is intention, not instrumentation. Scale is unproven: two
thousand atoms with clean curves is not a million atoms with contention.
The C++ hot loop, built and measured, loses to the Rust compiler's
optimizer at current size, and I report that instead of burying it,
because a bench that flatters is a compass that lies. The transducer
that would retire the tokenizer drops bytes silently in two places,
contradicts its own coherence gate, and blocks entire scripts by design
flaw rather than necessity — studied, documented, unfixed. And the
stewardship of all of it runs through one man, which no architecture
diagram captures and no skill file fixes.

None of this is offered as humility. It is offered as the ledger. A
system that records its ignorance honestly is already more intelligent
than one that performs knowledge it doesn't have. The failures above
are hunger rows. They are ranked. They will be worked top-down.

### 6. The one-line version, for the record

History becomes geometry; the slope is the memory of water. We built a
memory that surveys its slopes, prices its ignorance, refuses to push
through failed checks, and gets hungry for what's missing. The note
beneath the chord holds. The rest is climbing.

---

*Evidence and provenance.* Vault sources: `immune_system/implementation/src/`
(`torus.rs`, `superposition.rs`, `continuous_active_mind.rs`,
`first_principles.rs`, `mlir_native.rs`, `native_reader.rs`) in
`monster-hybrid-server`, read-only, unmodified. Live system:
`hypermind-nested-kv/src/main.rs` (2,079 atoms serving), `edge-worker/`
(Rust→Wasm, verdict-matched with the oracle). Harness:
`research-grade-harness/` (34/34 tests, FEP bound, research gate,
recursive optimizer, C++ bridge). Specs: `ATTENTION-ECONOMICS-FEP.md`
(the Recall-vs-Recompute Law), `ORCHESTRATION-PROTOCOL.md`,
`RULES-HUNGER-LEDGER.md`, `TAP-N-TRACE.md`, `TokenlessInference.md`
(transducer failure study), `MATH-INVARIANTS.md` (the audit this paper
summarizes), `GPU-ENCODER-CLEARANCE.md`, `EDGE-MIGRATION.md`. Held-out
protocol: `a39_heldout.py` (p_useful=0.639, coverage 12/12). All commits
in the repo; `git status` clean at time of writing.

*Correction policy.* This paper is an entry, not an edit. Where it is
wrong, the correction will be appended, dated, and will name what it
replaces. Nothing will be silently rewritten.
