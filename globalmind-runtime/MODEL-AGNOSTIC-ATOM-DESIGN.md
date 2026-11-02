# Model-agnostic atom admission — design gate

Status: proposed correction, not an implementation or certification. Applies to the fresh GlobalMind runtime in this repository. Keep the existing SurrealDB `hypermind/core` quarantined.

## Failure in the current slice

`extract::atom_key` hashes trace kind, dtype, shape, layer, head, and payload digest. Removing teacher ID from that preimage permits only byte-identical tensor coalescence. Different architectures, tensor shapes, heads, and learned representations can encode the same knowledge yet receive unrelated keys. `address_for` derives H3 and T2 coordinates from hash bytes, so geometric proximity does not encode semantic proximity. `replay::scoped_id` includes the run, creating separate `atom` rows for identical candidates from separate runs. `replay::build_atoms` writes `replayed atom <digest>` as the content, which cannot answer a task. The current judge checks structural evidence and mean-square energy, not truth, entailment, or cross-model equivalence. These records are trace witnesses and integration fixtures, not certified shared knowledge.

## Two populations and immutable identity

1. Keep raw activations, weight slices, Jacobian candidates, TPPE traces, and tensor K/V as model-bound witnesses. Their exact model, tensor shape, source step, and source digest remain mandatory. Never call their raw hash a semantic atom ID.
2. A shared rank-0 atom must contain a typed, actionable payload: proposition, procedure, constraint, tool capability, or observation; canonical subject/predicate/object or precondition/action/effect fields; scope, validity conditions, evidence references, and an abstain condition. A bare coordinate, label, embedding, or opaque hash fails admission. Facts from the RAG library are cited evidence, not the GlobalMind atom itself.
3. Derive `semantic_key_v2 = H(domain || canonicalizer_version || canonical_payload_bytes || scope)` using a deterministic native binary encoding. The key has no teacher, run, layer, head, shape, tokenizer, or tensor bytes. Exact canonical matches may share one atom. Semantic equivalence across different bytes requires an independently judged `equivalent_to` link; do not silently collapse them. Contradictory claims remain separate with explicit `contradicts` evidence.
4. Make the shared atom ID stable across runs. Each successful teacher/run creates an append-only origin and evidence link to that atom. Packet IDs and tensor witnesses remain run-scoped. In one transaction, insert the atom if absent, or attach a new origin if present; enforce a unique `(workspace, semantic_key_v2)` index and fail closed on a concurrent mismatch. No overwrite of old content or evidence.
5. Version address placement separately from identity. A frozen, explicitly versioned semantic projector maps the canonical payload to H3 and T2, with a measured retrieval neighborhood. Hash bytes must not be used as geometry. Keep old address revisions immutable and record projector digest, boundary receipt, and calibration corpus. The nested K/V may link to an address but is never a shared semantic payload.

## Admission sequence

Frozen teacher artifact receipt → real task/trajectory witness → native typed candidate payload → canonicalization → evidence and contradiction checks → blinded independent judge → replay with cited outcome → atomic shared-atom/origin/edge write. A frozen weight slice alone may inform candidate generation but cannot certify a fact or procedure. Rank-1 transitions must name admitted shared atoms and typed conditions; rank-2 loops require replayed closure; rank-8 charts require an actual bounded Jacobian/effect measurement, not merely eight weight values.

## Falsifiers and release gates

- Same knowledge, different teachers and tensor shapes: independent payloads must retrieve the same relevant atom or a justified `equivalent_to` neighbor, with neither teacher ID nor source tensor exposed in ranking.
- Different knowledge with superficially similar language must remain distinguishable; contradictory answers must surface conflict and provenance.
- Blind held-out tasks must show correct citation, execution or answer utility, abstention on insufficient evidence, and improvement over the no-map baseline. Count false merges, false splits, recall, task accuracy, latency, and resource cost.
- Restart and competing-writer tests must preserve one canonical atom with multiple immutable origins. A provenance mutation must not alter the semantic key or retrieval rank. An exact-model K/V mismatch must miss, regardless of semantic atom match.
- Until these gates pass, label the current four-atom Layer-3 run `trace-fixture` and prohibit claims of model-agnostic knowledge, semantic geometry, or swappable-model memory.

The first implementation slice should introduce the native typed canonical payload and a separate candidate table, then test two real teachers on the same held-out task before changing the live atom store or reingesting the model roster.
