# Live model seat attestation for benchmark setup

Date: 2026-11-03. The backend's model list must describe loaded serving state, not a hardcoded Qwen name. The Rust native server exposes `GET /v1/info` as one size-prefixed AMST metadata frame containing its 64-hex model identity and profile. The backend already validates this frame format for generation.

`/api/models` will probe the configured native MTP base URL, parse exactly one metadata frame, reject a missing/invalid identity or wrong frame kind, and report the attested identity and profile only when the probe succeeds. A down native server is `up=false` with no invented model. oMLX and gateway seats remain reported from their own live endpoints; they are not treated as interchangeable native forward implementations. A successful info probe proves only the serving seat identity, not model output, tokenizer parity, K/V reuse, or Glimmer support.

Gate: parser tests for valid metadata, wrong kind, truncated frame and identity mismatch; route output must not hardcode `Qwen3.6-MoE-Combined`. A model swap creates a fresh mission executor and invalidates incompatible prefix/K/V pages. The first benchmark is a frozen native baseline; TPPE Q/K replacement is separately ablated because the Glimmer checkpoint was trained with RoPE/NoPE.

## 2026-11-04 recovery gate: no fabricated model output

The recovered mission executor calls only the GlobalMind MCP child. It cannot
produce model token IDs, logits, or a frozen-weight identity. Its previous
bound-receipt implementation derived fake token IDs from a tool receipt,
hashed MCP text as fake logits, and used placeholder model digests. This
violates the bound-output contract. The corrected executor must retain
MCP-only calls as tool receipts and reject bound model calls. The mission
route must identify that executor accurately and fail closed until a
single native invocation binds actual generated token IDs, logits digest,
model artifact identity, tool results, and exact K/V pages. A 30–50-step
run cannot certify full integration until that source-backed seat exists.

The dormant native MTP executor has the same risk: its text stream supplies
generated text and seat identity, but no logits or artifact digests. Its
ordinary `call` may report that text with a backend binding digest; its
`call_bound` must refuse the bound-output contract until the native wire
supplies actual token IDs, logits digest, exact input, and artifact identity.
The Glimmer HTTP seat must also carry no placeholder artifact identity while
its bound call is unimplemented; future artifact fields come only from an
attested server response or locally verified checkpoint digests.
For the existing HTTP component probe, validate the returned forward receipt
against the documented wire formula over submitted input IDs, returned output
IDs, and logits digest. Reject any mismatch. This checks transport binding;
it does not independently verify logits or qualify the JSON HTTP protocol as
the Rust-native model I/O boundary.
The live packet runner must account for every declared tool call. A call
outside the implemented map/mind namespace is an explicit unsupported-tool
failure, never a silent skip. Otherwise a long mission could appear complete
without executing part of its declared plan.
The same rule applies to `ToolCall.declared=false`: reject before wire contact
and mark the packet failed. The declaration bit must not be a dead field.

Bound model receipts must include the native exact K/V digest from the same
forward invocation, and the chain hash must cover it. The harness verifies
input-byte and artifact-composite digests independently before accepting a
receipt. An MCP tool receipt can attest persisted K/V page reads, but cannot
substitute for the native forward's K/V digest.
