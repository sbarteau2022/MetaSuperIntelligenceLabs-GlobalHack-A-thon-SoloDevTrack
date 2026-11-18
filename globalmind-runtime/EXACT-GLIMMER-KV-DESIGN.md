# Exact Glimmer K/V continuation gate

## Preregistered numerical diagnostic, 2026-11-04

The independent MLX oracle found that the current Rust forward first differs
at the quantized embedding output. The real BOS row became byte-identical
offline when the quantized row was rounded to bfloat16 before its f32
out-scale multiplication and rounded again afterward. A gated Rust ablation
may apply exactly those two roundings to embedding rows only. The control is
the unchanged Rust forward on the same token tape; the oracle is the frozen
MLX checkpoint. Measure raw embedding equality, final-logit max/mean
absolute error, and argmax. This ablation does not select a production
precision profile or certify the full model. A worsening final-logit error
benches the change; an improvement merely advances stage-local diagnosis.
The original exact-K/V restart result must be rerun if the profile is ever
made default.

Observed: the gated profile matched the real MLX BOS embedding exactly but
slightly worsened final-logit max and mean absolute error. It stays benched
and disabled by default. The next precision experiment must isolate the
layer-0 pre-attention RMS norm and mixed-bit projection output contracts
before altering any attention or Metal kernel.

This is a component gate for the frozen Glimmer checkpoint, not a claim of
model-neutral serving or coherent GlobalMind use. Use the native Glimmer
executor's `AttentionBackend` seam: inputs are normalized and have the trained
position transform applied. Keep every eligible K/V vector in token order,
apply the sliding-window lower bound before softmax, and perform no geometric
top-k selection. A full-attention layer has a zero lower bound.

For each forward position, all layers must see the same position and exactly
one K/V append per layer. Reject gaps, duplicate positions, changed shape,
non-finite values, and missing layer/head state. Snapshot state is an owned,
typed Rust value, copied only for the continuation comparison. The base model
weights remain immutable. Run a real frozen-weight prefix, snapshot the cache,
then compare the next token's logits with an uninterrupted continuation using
separate cache instances. Exact equality is the gate; no answer-quality claim
follows from it.

The component test `tests/glimmer_native_full_pass.rs` now writes complete
per-layer/head pages to disposable SurrealKV, restarts the server, uses a
live-attested MCP child to recall the prefix witness, verifies `mind.kv_exact`,
hydrates all pages by exact identity, and compares frozen-weight continuation
logits bit for bit. Negative controls reject changed model/page digests and a
missing head; a perturbed K/V value changes the output. The live port 8000 is
never used. See `../results/glimmer-forward/mcp-surreal-kv-full-pass-2026-11-04.md`.

The recalled prefix witness is not distilled knowledge. Full promotion still
requires an admitted, teacher-neutral atom from a real distillation run,
source-grounded answer/use ablation, and a governed mission receipt binding
the model output to that atom and its exact tensor pages. The current
`glimmer_coherent_mind` candidate contains no-op persistence and entropy-only
judging, so it is not a qualification gate. The native executor used by this
test remains untracked reference work; its promotion into the model-neutral
runtime is a separate source-ownership gate.

## Native server promotion gate

The existing `/v1/forward` route uses Glimmer's geometric-memory path and a
JSON body. It cannot serve as the exact tensor-K/V model boundary. Add a
separate loopback-only, versioned binary endpoint whose request carries bounded
little-endian token IDs and maximum output count. Its one invocation must run
`forward_with_backend` with a fresh `ExactKvCache`, return actual generated
token IDs, the digest of the final f32 logits, and a digest covering every
layer/head K/V page in deterministic order. Reject truncated, oversized, or
unknown-version frames before entering the model. Do not silently fall back
to geometric memory. The existing port 8011 process remains untouched until
an isolated alternate-port smoke proves the new route.

The server's current seat identity hashes config and tokenizer only. A bound
receipt also needs frozen shard hashes and the index hash, computed by
streaming at startup and pinned for the process lifetime. The MCP tool receipt
and exact K/V storage receipt must be joined to this same forward receipt by
the harness; a separate generate call is not acceptable evidence. A component
endpoint alone does not qualify coherent shared-atom use or the automaton.

## Recall content gate

`mind.recall` currently yields atom IDs, addresses, and digests without the
content needed for model use. For each ranked member, the MCP adapter may
hydrate only its inline `content_text` from the same workspace and exact atom
ID, verify SHA-256 against the ranked content digest, enforce a bounded text
size, and include the verified text in the MCP control response. A URI-only
atom remains metadata-only until a separate source fetch is attested. The
model-facing adapter must convert verified content to Glimmer token IDs before
the binary forward call; MCP JSON itself never crosses that model boundary.

The serving adapter owns model-specific tokenization. A separate binary
tokenize route accepts bounded UTF-8 semantic text and a BOS flag, returns
bounded token IDs plus the pinned tokenizer digest, and binds request and
response with SHA-256. The harness may parse MCP JSON on its control side,
but sends only the verified atom text bytes to this route and only token IDs
to exact forward. No Glimmer tokenizer is linked into the backend.
The write-side `CrystallizedAtom` validator must enforce the same digest for
inline text, so invalid content cannot enter by a non-CLI writer and wait to
fail only at recall.

Artifact digest contract: `weights_sha256` hashes a canonical roster made
from the index file digest and four ordered shard file digests;
`tokenizer_sha256` and `config_sha256` hash their raw files;
`composite_sha256` hashes a version tag followed by those three raw digests.
All files are streamed in bounded chunks at server startup. The exact binary
response carries all four raw 32-byte values, while `/v1/info` retains its
older seat identity for compatibility. A missing or changed file refuses
startup rather than silently swapping the model under a pinned session.
