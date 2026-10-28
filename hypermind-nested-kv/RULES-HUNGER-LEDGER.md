# Rules — Hunger Ledger (C31 Hunger Extension, goal 134e5fbb)

These rules apply to the Hunger Ledger and inherit C03 (first-principles
backbone), C31 Rules (append-only, telemetry privacy, claim labeling), and
the N³ substrate rule (deterministic, model-independent; tools are substrate,
the decision to call them is harness).

## H-1. Ignorance is first-class data

Every retrieval miss, traversal refusal, judge rejection, and low-coverage
query mints a hunger receipt. Hunger is not an error log — it is the
training signal for what to distill next. Loss = uncovered sub-problems
stops being a metaphor; it is a queryable table.

## H-2. No model in the tap path

Hunger receipts are minted by deterministic code at the failure site
(fail-explained arm, judge gate, coverage counter), never by a model call.
A hunger receipt contains counts, ids, terms, and scores — never model
judgment. Aggregation is arithmetic (counts, rates, ranks), never inference.

## H-3. Telemetry privacy holds (C31 Rule 9)

Hunger receipts carry: query-term hashes (not raw text), atom ids, scores,
depths, timestamps, counts. Never: full query text, atom text, user identity.
The ledger answers "what is missing" without recording "who asked."

**Enforcement.** `map.hunger` returns term hashes + ids + ranks. Raw text
reconstruction happens only inside `map.lexical`/`map.query` at read time,
gated by the caller's existing role — the ledger never stores it.

## H-4. Append-only, coalescing counts (C31 Rule 1)

Repeat hunger for the same gap does not mint a new row — it increments
`hits` and updates `last_seen`. Corrections are entries: a satisfied gap
(flipped to covered by a later distill) is marked `sated`, never deleted.
The ledger remembers what it was hungry for even after eating.

## H-5. Hunger ranks distill priority

`map.hunger` returns gaps ordered by `priority = hits × depth × (1 + recency)`.
The distill scheduler (human or automated) works top-down. Baseten GPUs
encode whatever the ledger says is hungriest — no guessing.

## H-6. Consolidation is receipted sleep

`map.consolidate` (the sleep cycle): reinforce traversed atoms, decay
untouched ones, merge hunger duplicates, mark sated gaps. Emits a sleep
receipt (reinforced/decayed/sated counts + fingerprint). Wake = reweight +
fresh snapshot. Never silent, never destructive (decay floors at 0.05 —
nothing starves to zero).

## H-7. Taps connect with zero permissions (tap structure, §3)

Taps are read-only subscribers on the tool-result stream. They observe
`(tool_name, status, ids, scores)` — the receipt envelope, never the payload.
Because they read receipts (already public inside the server) they require
no capability grant, no role, no approval. A tap cannot invoke tools, edit
arguments, or emit instructions (same narrow-blocking rule as N³ hooks:
deny/append-context only — and hunger taps don't even deny; they count).

## H-8. Aggregation prior, not posterior (Tap'n'Trace, §2)

The T&O log aggregates hunger counts BEFORE any model sees them. The model's
first sight of hunger is the ranked `map.hunger` output — pre-aggregated,
de-duplicated, prioritized. No model ever scans raw failure streams; the tap
structure guarantees the aggregation happened upstream.

## H-9. Falsifiability (the ledger proves itself)

1. `map.hunger` top-1 predicts the next held-out failure (testable).
2. One consolidate cycle lifts held-out `p_useful` (measured: 0.639 → ?).
3. Hunger receipts accumulate across restarts (persisted in snapshot).
Fail any → the ledger is decoration, and this rule file says so.

## H-10. Claim labels ride every receipt

Hunger counts are VERIFIED (deterministic counters). Priority ranks are
INFERRED (formula + weights stated). Sated marks are VERIFIED (distill
receipt proves coverage). Anything else is PROPOSED until receipted.
