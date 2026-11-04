# Replay atoms now carry their source language — 2026-11-04

Problem (handoff blocker 1): `build_atoms` wrote the placeholder `replayed atom <slice>` as
atom text while `content_digest` was a tensor-derived digest, so recall refused the atom and
no model could ever read meaning from it. Trace envelopes are activation tensors and hold no text.

Fix: `globalmind-runtime/src/replay/source.rs` (`SourceText`, GMSR v1) binds the token tape's
decoded text plus a byte offset per token position, with a trailing SHA-256. With a source,
each rank-0 atom's `content_text` is the exact source prefix ending at its earliest token
position, `content_digest` is the SHA-256 of those bytes, and the replay receipt additionally
binds the source digest and the atom-content bundle. The walk artifact stores the source as an
optional section 7, so crystallize/audit re-verify from it. Without a source an atom is a
metadata-only `gmtrace:<digest>` reference with no inline text; placeholder text is gone.
New stages: `globalmind-distill source <manifest> <tape> <out.gmsrc>` (build with
`--features source-stage`, uses the rostered tokenizer) and `replay-source <trace> <verdicts>
<source> <walk>`.

Evidence:
- Real tokenizer (frozen Muse Glimmer): `tests/source_text_real_tokenizer.rs` — English, a
  multi-sentence text and CJK/accented text round-trip; every span is a monotone exact prefix.
- Isolated SurrealKV restart: `tests/distillation_surreal_live.rs` — atoms hydrate after restart
  via `fetch_atom_content_exact` as "The" and "The capital", digest-verified.
- Lib tests: 110 passed, 3 ignored (outside the sandbox); new replay tests cover exact spans,
  metadata-only fallback, source-bound receipt, fail-closed on a short source, artifact restart.

Limits (not claimed): the live Qwen trace -> source -> replay run was not executed (needs the
Qwen weights/trace on hardware), so no real distilled run has been regenerated yet; cross-model
atom coalescence is still not proven (atom_key remains tensor-based; this repairs the text
carrier, not model-neutral identity); atoms are prefix text up to a position, not sentence-level
summaries; no source-grounded answer judging or held-out cross-model task (items 5/6).
