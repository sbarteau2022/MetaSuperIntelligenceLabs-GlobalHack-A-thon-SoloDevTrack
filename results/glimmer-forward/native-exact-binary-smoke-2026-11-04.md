# Native Glimmer exact-forward binary component smoke — 2026-11-04

Built release `serve_glimmer` and `smoke_exact_wire`. Confirmed port 18111
free, started a separate server against the frozen local Muse Glimmer
checkpoint, then ran the Rust binary client. The client sent the same bounded
GFX2 token-ID request twice and verified both GFO2 response receipts and
equality of all fields. The server was stopped afterward; existing 8011
Glimmer and 8000 SurrealDB listeners were untouched.
The final client version also streamed and hashed the checkpoint independently
and matched the server's four artifact digests. A response with a valid outer
receipt but an inconsistent composite digest is rejected by the parser.

Observed output:

```text
token_ids=[13796]
logits=84d6f5c41c75916ce8ee2bc10f940d84e3784a4dfa911563dadeae486fd72f43
kv=21e7ccb622f946b412cd9982af41fbc094febe52e5c6f6e668ef18f2a6c30d2c
weights=4ab802106d6dbb7650a70a781c7c06a105b77e294c799e108712d8970b3480d5
tokenizer=c9dbee66967b58f31a7c27f723c3760da3526ccd0427578e8905b0abb0031c4d
config=3ce829f4654d2f5f5f43acca424fe17d72e74015d375d385a86b7fff24d4a031
composite=dbe78c78f2072bd53add370dcce8d39bd9779c477af79cab8797747683941a7e
receipt=1f904fa800a8f268cc79be4f41d1e55ac38b1a9623ba923a004fed38e99ced6c
```

The output token matches the earlier isolated exact-K/V MCP component test.
The server streamed the frozen index and shard files before and after load;
it refuses startup on a changed artifact. This proves a real frozen-weight
native forward through ExactKvCache with deterministic tensor-page digest.
It does not prove HyperMind recall, persisted nested K/V, model-neutral atom
use, or a governed mission. The 30–50-step integrated loop remains open.
