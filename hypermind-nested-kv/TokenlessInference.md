# TokenlessInference.md — Design + Regurgitation Sketch

**Status.** DESIGN (this doc) + AUDIT (vault code read, all refs VERIFIED this pass).
**Scope.** The `BoundaryLexicalTransducer` as it exists in
`monster-hybrid-server/immune_system/implementation/src/continuous_active_mind.rs`
(:92–200 ingress/egress, :605+ span folding, :1028+ decode paths),
`AtomicIrAlphabet` in `first_principles.rs:631-780`,
`MlirAtomCatalog` in `mlir_native.rs:174-260`,
`NativeReader::address` in `native_reader.rs:85-130`.
**No vault file modified. No code written — sketch only (§5).**

---

## 0. What the transducer IS (one paragraph, no mystique)

A reversible byte↔op table at the outer skin. **Ingress:** UTF-8 bytes →
greedy longest-match against `vocab_encode` (seeded 0–255 + learned pieces,
`register_token_piece`) → raw carrier IDs → `atomic_ir.encode` (folds known
multi-token spans into macro-opcodes ≥ `base_vocab_size`) → `Vec<MlirOpId>`.
**Egress:** ops → strip BOS/EOS → `atomic_ir.expand` → per ID: vocab piece
bytes, else `<mnemonic>` for catalog ops, else raw byte if < 256, else
**nothing** → `String::from_utf8_lossy`. That is the whole machine.
Everything else in this doc is about what it destroys.

---

## 1. Where it fails (studied, ranked, with code refs)

### F-1. Silent drops — information destroyed without receipt [CRITICAL]

Two sites, same sin:

1. `transduce_egress_ops` (:441–466): the per-ID chain is
   `vocab_decode → catalog mnemonic → raw byte if <256 → ELSE NOTHING`.
   An ID ≥ 256 in neither table vanishes. No error, no count, no receipt.
2. `atomic_ir.expand` (first_principles.rs): unregistered atom IDs ≥
   `base_vocab_size` are skipped by the same no-else pattern.
   (`expand_checked` exists and errors properly — but egress uses the
   silent `expand`, not the checked one.)

**Why critical:** this violates the system's own Qualification #9 (name the
loss). Every other subsystem receipts its loss; the skin — the one place
all I/O passes — swallows bytes silently. A dropped byte in a multilingual
stream is a dropped character in someone's name.

### F-2. The mnemonic/coherence contradiction [LOGIC BUG]

`transduce_egress_ops` renders catalog ops as `<mnemonic>` strings
(`<atoma.xxx>`, `<arith.xxx>`). Then `is_coherent_egress_text` (:468+)
**rejects any text containing `<atoma.`** as incoherent. The transducer
emits what the coherence gate calls garbage. One of them is wrong:

- If mnemonics are legitimate output (they are — they're the only way an
  op survives egress as anything but a raw byte), the gate must accept
  the mnemonic namespace.
- If mnemonics are diagnostics that must never reach a user, egress must
  never emit them on the user path (separate diagnostic egress).

Today it does both: emits on the only path, rejects on the only gate.

### F-3. `from_utf8_lossy` — silent corruption with a smile [LOSS, UNRECEIPTED]

Egress ends in `String::from_utf8_lossy` (:465). Invalid sequences become
U+FFFD and the function returns `Ok`-shaped success. The strict twin
`transduce_mlir_to_utf8` does it right (`from_utf8` → `Err`), but the
egress path — the one that actually runs — uses the lossy twin. Note the
coherence gate then rejects U+FFFD-containing text, so the pipeline is:
corrupt silently → reject opaquely. The loss is real twice over and
measured zero times.

### F-4. Egress→ingress is NOT the identity [REGURGITATION-BREAKER]

`expand(encode(x)) == x` holds (greedy deterministic fold, exact inverse
table — Invariant I2 stands). But `encode(expand(y)) == y` FAILS for
hand-crafted op streams: expanded tokens `[101,102,103,104]` re-encode to
the `struct.tool_call.open` macro-op, not back to raw carriers. Consequence:
**an emitted op stream fed back through ingress does not reproduce itself.**
Regurgitation — emit, re-ingest, compare — drifts on the first cycle for
any stream containing foldable spans. The user's word is precise:
regurgitation is currently lossy by construction, and the loss is in the
fold, not the bytes.

### F-5. The coherence gate blocks MultiLingual [SPEC BUG vs STATED GOAL]

`is_coherent_egress_text` gates on `prompt_text.is_ascii()`: any ASCII
prompt with non-allowlisted-non-ASCII output is `false`. The allowlist is
~25 punctuation/math symbols. **All CJK, Arabic, Devanagari, Thai, Korean —
incoherent by definition.** The byte path itself is language-agnostic
(UTF-8 passthrough works); the gate is the blocker. Additional over-breadth:
`len > 36 && no space` kills URLs, code identifiers, hashes, German
compounds; the triple-word rule kills rhetoric and poetry; single-char
non-digit rejection kills CJK single-character replies. The gate is a
bag of English-prose heuristics wearing a universal uniform.

### F-6. Atom→op projection discards the geometry [ARCHITECTURAL]

An arc stores `compiled_mlir_op: MlirOpId(u32)` — one opcode number.
The atom knew `(lorentz, phase, payload, salience, Wigner frame)`; the op
says `arith.add`. Coordinates, energy budget, traversal contract travel
**beside** the op (silo fields, symplectic envelope), never **in** it.
Egress from the op alone (`get_op` → mnemonic) recovers the *name* of the
operation and none of its geometric context. Coherent atom synthesis —
regurgitating an atom as the op-stream that *means the same thing* —
is therefore unmeasurable today: there is no score comparing source atom
to emitted ops except string matching on decoded text (which F-1..F-5
already corrupt).

### F-7. No structured egress [MISSING MILE]

JSON and Qwen3-XML — the user's named last mile — have no path. Egress
returns flat text; there is no grammar-constrained op selection, no
well-formedness gate, no schema receipt. (Not a bug — absent scope.)

---

## 2. The loss chain, staged (what each conversion preserves and destroys)

```
UTF-8 bytes
 │ ingress: greedy longest-match + atomic fold        LOSS: fold non-injectivity (F-4)
 ▼
MlirOpIds (carriers + macro-ops)
 │ atom synthesis: arcs pin (from, to, op, frame)     LOSS: geometry discarded, op keeps number only (F-6)
 ▼
Stored atoms + arcs (lorentz, phase, payload, frames)
 │ cursor walk: Wigner transport per hop               LOSS: none (exact transport) — the healthy stage
 ▼
Emitted op stream
 │ egress: expand + per-ID bytes/mnemonic/drop        LOSS: silent drops (F-1), lossy UTF-8 (F-3), mnemonic substitution
 ▼
Decoded text
 │ coherence gate                                     LOSS: false rejections, esp. non-English (F-5); self-contradiction (F-2)
 ▼
User / JSON / XML  (F-7: no structured path exists)
```

**Measured today: nothing on this chain.** Every loss above is VERIFIED by
code reading, QUANTIFIED by nothing. §4 fixes that: every stage gets a
meter before any stage gets a rewrite.

---

## 3. Design principles (binding on the sketch)

1. **No silent anything.** Every stage emits a loss receipt: bytes in,
   bytes out, IDs dropped (named), U+FFFD count, fold collisions.
   Qualification #9 applies to the skin first, not last.
2. **Checked expansion everywhere.** `expand_checked` replaces `expand`
   on all egress paths. An unknown ID is an error with the ID in it,
   never a vanishing.
3. **Round-trip is geometric, not byte-identical.** Paraphrase is valid
   output. The regurgitation metric is: re-ingested egress lands within
   `cosh ≤ ε` of the source atom AND preserves the symplectic contract
   (form/energy/frame within tolerance). Byte-identity is the wrong
   meter and would punish correct behavior.
4. **Coherence is per-modality, versioned.** One gate per output kind
   (prose-EN, code, multilingual-script-aware, structured) with the kind
   declared in the receipt. No universal heuristic.
5. **Structured egress is grammar-constrained selection**, not
   post-hoc validation. The op stream is chosen from the grammar's
   language; well-formedness is a construction property with a receipt,
   not a prayer after the fact.
6. **Geometry travels in-IR or not at all.** The atom→op projection
   must carry (or reference by receipt) the coordinate, phase, and
   contract — sidecar fields are a migration stage, not the design.

---

## 4. Loss meters (measure first — procedures, not wishes)

| Stage | Meter | Procedure |
|---|---|---|
| Ingress fold | `fold_collision_rate` | Random byte streams → encode → expand → re-encode; fraction where 2nd encode ≠ 1st. Expect >0 (F-4); the number sizes the fix. |
| Silent drops | `drop_count` | Instrument egress with a counting sink (no behavior change): IDs hitting the no-else branch, per 1k emissions. Expect >0 on manifold-op-heavy trajectories. |
| UTF-8 corruption | `fffd_rate` | Count U+FFFD in egress output per 1k emissions, split by script (Latin vs CJK vs Arabic...). Correlate with F-1 drops (a dropped continuation byte *causes* downstream FFFD). |
| Mnemonic leakage | `mnemonic_rate` | Fraction of egress outputs containing `<...>`; fraction of those then failing coherence (F-2 contradiction rate — expect ~100% for `atoma.*`). |
| Coherence false-reject | per-script reject rate | Coherent non-English corpus through the gate; measure reject rate by script. Expect ~100% for CJK/Arabic (F-5). |
| Atom→op→atom drift | `cosh_drift` + contract residuals | Regurgitation harness (§5): source atoms → ops → bytes → ops → atoms; shell-distance + form/energy/frame residuals. THE headline number. |
| Structured validity | `wellformed_rate` | Grammar-constrained egress vs post-hoc validation on JSON + Qwen3-XML schemas; invalid-output rate per 1k. Baseline today: N/A (no path). |

---

## 5. Sketch: the version that regurgitates (build order)

### Stage A — Stop the bleeding (no new math, all receipts)

1. Swap `expand` → `expand_checked` on the egress path. Unknown IDs error
   with the ID named.
2. Replace `from_utf8_lossy` with strict `from_utf8` + U+FFFD counting
   sink: corruption becomes a counted, receipted event, then an error.
3. Add the drop counter (no behavior change yet): every no-else branch
   increments `dropped_ids{n}`. Run the existing test suite; the number
   that comes out IS the F-1 measurement (§4, row 2).
4. Version the coherence gate per modality; ship `prose-en-v1` (current
   heuristics, labeled English-only) + `multilingual-v1` (script-aware:
   accept any well-formed UTF-8 in the prompt's script family; reject
   control chars + U+FFFD only). The ASCII gate is renamed, not removed —
   honesty over deletion.

### Stage B — Regurgitation harness (the meter that matters)

```
source atoms → synthesize op stream (cursor walk) → egress bytes
    → re-ingress → re-carve candidate atoms
    → ASSERT: shell_dist(candidate, source) ≤ ε AND contract residuals ≤ tol
```

- ε starts at the dedup threshold (`cosh ≤ 1e-7` family) and loosens only
  with measured justification. The harness reports `cosh_drift` per atom,
  per dialect, per script.
- Paraphrase passes by construction (geometric metric); byte-identity is
  asserted NOWHERE.
- This harness is the acceptance gate for every later stage: nothing
  ships that moves `cosh_drift` the wrong way.

### Stage C — Structured egress (JSON + Qwen3-XML)

- Grammar-constrained op selection: the emitter may only choose ops whose
  decoded bytes keep the output inside the target grammar (JSON values,
  XML well-formedness). Implement as a prefix-validity check per emission
  step (the cursor already walks step-by-step — the grammar check rides
  the same hop).
- Well-formedness becomes a construction property; the receipt carries
  `grammar + schema_hash + valid: true`. Post-hoc validation stays as
  the audit (belt and suspenders), never the mechanism.
- Qwen3-XML first (the harness's native dialect), JSON second (grant
  interop). Both gated by `wellformed_rate` (§4, row 7).

### Stage D — Geometry in-IR (the symplectic close)

- Extend the op descriptor (additive: new optional fields, old receipts
  valid) with the geometric half: source coordinate reference
  (atom id + snapshot digest, not raw floats — content-addressed),
  phase, and the traversal contract that produced it.
- Egress of such an op emits the mnemonic PLUS the geometric reference
  in a machine-readable envelope (not angle brackets — a real channel).
- The `coherent atom synthesis` claim becomes: emitted envelope
  re-ingests to within ε AND the referenced contract verifies. That is
  F-6 closed: the op finally *means* the atom, receiptedly.

### Stage E — Tokenizer retirement (last, not first)

- `atomadb::tokenize` (and every hot-path splitter) retires only after
  the regurgitation harness is green on all scripts AND structured
  egress is receipted. Order matters: the byte path must prove itself
  before the word path is removed.
- The retirement receipt: full test suite green with the splitter
  compiled out (`#[cfg]` gate, not deletion — archaeology matters).

---

## 6. What was NOT checked (explicit)

- `token_carrier_address` numerics (pure function of the integer — read
  the docstring, not the arithmetic; needs its own audit before it
  carries load).
- The `GeodesicCursor` transport math (named healthy in §2 by construction
  from `transport_frame_hop` — construction is not verification).
- `SPEC_atomic_ir.md` Invariants I1–I3 (cited, not opened).
- LLVM lowering (F-7 notes its absence; no vault LLVM backend was found —
  if one exists outside `immune_system/implementation/src`, this doc's
  chain gains a stage).
- Any measurement (all §4 numbers are procedures; the harness in §5-B
  produces them).

---

## 7. The one-line version

The transducer converts correctly at the byte level and loses information
at every level above it — silently, unreceipted, and (for non-English and
structured output) by design flaw rather than by necessity. Fix the
receipts first, meter the losses, build the regurgitation harness, and
the tokenizer retires itself for lack of work. That is the whole plan,
and it is in dependency order.
