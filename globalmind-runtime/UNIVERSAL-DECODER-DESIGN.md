# Universal decoder contract (candidate, 2026-11-03)

## Boundary

The harness, automaton, MCP, GlobalMind recall, and receipt format bind to a
model-neutral `ForwardRequest`/`ForwardReceipt` binary ABI. The native decoder
loads a validated graph plus immutable weights and executes registered
operators. No model name, version, layer count, head count, or vocabulary size
is compiled into that ABI. On-disk checkpoint metadata may be JSON;
model-to-harness I/O is typed binary only. Mind atoms remain teacher-neutral.
Transformer K/V remains model-bound and is fetched only with the complete
checkpoint, tensor, position, address, and payload identity.
Exact continuation also requires the complete trained attention state for the
applicable causal/window range. A bounded geometric top-k of prior K/V atoms
changes softmax whenever an omitted atom has nonzero attention mass; storing
those atoms losslessly does not repair that selection loss. The native decoder
must export and restore exact per-layer/head K/V tensor pages before this
contract can claim parity. Geometric top-k belongs to a separately judged
retrieval ablation, with window eligibility applied before ranking.

## Admission

1. Hash the checkpoint artifacts and tokenizer. Read bounded metadata and
   tensor headers. Reject duplicate tensor names, missing tensors, unsupported
   dtypes, overlapping offsets, and unsafe paths.
2. Compile the declared architecture into a typed operator graph. The graph
   records operator version, input/output shapes, tensor bindings, quantization
   layout, position rule, cache rule, and numerical precision. Reject unknown
   operators and shape mismatches before allocating weights or serving.
   Graph admission must also reject empty graphs, duplicate node IDs, zero or
   unbounded dimensions, and empty/duplicate tensor bindings. Canonical graph
   attestation uses domain-separated, length-delimited fields and explicit
   vector counts; concatenated dimensions without shape boundaries are
   ambiguous and are not an acceptable execution identity.
3. Attest an `execution_contract_digest` over the canonical graph, operator
   versions, numerical profile, and tokenizer contract. The model identity is
   the artifact digest; a new version with compatible operators can use the
   same engine without code changes. A new operator requires a native operator
   implementation and independent parity evidence.
4. Bind each mission to one immutable model identity. A model switch starts a
   new session and invalidates model-bound K/V; neutral atoms remain reusable.

## Engine operator families

Native embedding lookup; affine mixed-bit linear; normalization; gated MLP;
Q/K normalization; position transform (including trained RoPE and NoPE);
causal sliding/full attention; residual; final projection and logit cap.
The graph expresses ordering and dimensions. The engine owns memory, dispatch,
cache invalidation, and receipts. Glimmer is a graph profile using these
operators, not a special branch in the harness. TPPE addressing is a separate
GlobalMind transform; replacing trained Q/K position behavior is an ablation.

## Falsifiers and promotion gates

- Changing only checkpoint revision with the same architecture must require
  no source edit, but must miss the old K/V identity.
- A different supported architecture must run through the same binary ABI.
- Unknown operator, changed tensor layout, incompatible tokenizer, or forged
  receipt must fail before forward execution.
- Compare full-model logits and continuation against an independent frozen
  reference on real held-out inputs. Then compare uninterrupted continuation
  with exact K/V restore from disposable SurrealKV. Finally run a real
  harness→automaton→MCP→mind→decoder round trip and benchmark latency,
  quality, peak memory, and energy proxy with preregistered controls.

This design does not certify a universal executor. The current Glimmer
preflight only rehashes artifacts; no native Glimmer graph compiler or full
forward receipt exists yet.
