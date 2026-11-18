# NativeGlimmerExecutor hardware smoke — 2026-11-04

Test: `Ares WebSite : App/backend/tests/native_glimmer_live.rs` (ignored). Real
`globalmind-mcp` child over a disposable in-memory Surreal (random port, never
8000) and an isolated Metal `serve_glimmer` on 18111 (frozen Muse Glimmer).
Weights pin from `native-exact-binary-smoke-2026-11-04.md`.

One bound call: mind.recall -> digest-verified atom text "The capital of France
is" -> GTK2 tokenize -> GFX2 exact forward -> one BoundOutputReceipt (validate() ok).

generated_token_ids=[13796, 26, 1573, 373, 290, 2167, 26899, 1212]
logits=149ea41c029a3061780e760c3ffc1cbdc2078f68a20123b8a98c1e89ff7a6ead
kv=55a03fddcf8bd197a4c5e94f769a2d04e40475e145658aacc48ae327845ba9c9
artifact_composite=dbe78c78f2072bd53add370dcce8d39bd9779c477af79cab8797747683941a7e
tool receipts: mind.recall 397c4a58…5032, native.tokenize c45a93fa…39eb,
native.forward-exact 2b94716b…0d53. (chain_hash includes dt_ns, so it varies per run.)

Negative controls (both refused): wrong weights pin; an in-range atom whose
text does not match its stored digest. The second proves the call fails, not
which layer rejected it. Schema makes content_text read-only, so tampering
after write is impossible by design.

Limits: the atom is a hand-made witness, not distilled knowledge (blocker 1);
generated tokens are not detokenized or shown to be consumed by a later forward
(item 5); K/V is a digest, not persisted pages (item 3); one checkpoint, one run.
