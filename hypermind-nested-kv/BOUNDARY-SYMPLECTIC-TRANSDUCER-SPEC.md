# Boundary Symplectic Transducer — Frozen Contract `bst-v1`

**Date.** 2026-11-03. **Status.** FROZEN — implementation in
[`src/boundary_symplectic_transducer.rs`](src/boundary_symplectic_transducer.rs:1).
**Scope.** `hypermind-nested-kv` only. No vault file modified.

This document freezes the contract required by work-queue item 1 of
[`GLOBALMIND-MULTIMODEL-TRANSDUCER-HANDOFF-2026-11-02.md`](GLOBALMIND-MULTIMODEL-TRANSDUCER-HANDOFF-2026-11-02.md:50):
ingress/egress schemas, semantic canonicalizer, MLIR dialect,
geometry/projector, Q32.31 numeric contract, and adapter — each versioned
independently. A version change requires a new address revision and a replay
receipt, never an in-place rewrite.

## 1. What this is and is not

The transducer converts semantic input into a typed symplectic MLIR module
and converts validated model output back. Native typed/binary
representations on this path: hand-rolled little-endian canonical bytes,
no Python, no JSON.

It is NOT a proof that natural language has a unique machine
representation, and it is NOT a claim that a hyperbolic coordinate plus a
phase is automatically a symplectic structure. The default geometry label
is **phase-augmented**. The label `SymplecticCanonical` is attached only
after [`check_canonical_transport()`](src/boundary_symplectic_transducer.rs:1)
proves preservation of the frozen two-form within tolerance — and the
residual travels with the claim so any consumer can re-check it.

## 2. Frozen versions (`bst-v1`)

| Component | Version constant | Meaning |
| --- | --- | --- |
| Transducer | `TRANSDUCER_VERSION = "bst-v1"` | Whole-contract identity |
| Ingress schema | `INGRESS_SCHEMA_VERSION = 1` | Accepted source shapes |
| Egress schema | `EGRESS_SCHEMA_VERSION = 1` | Decoded result shapes |
| Canonicalizer | `CANONICALIZER_VERSION = 1` | Semantic key derivation |
| Dialect | `DIALECT_VERSION = 1` | `gm` ops, types, effects |
| Geometry | `GEOMETRY_VERSION = 1` | H3xT2 chart + cotangent derivation |
| Numeric | `NUMERIC_CONTRACT_VERSION = 1` | Q32.31 fixed-point rules |
| Adapter | `ADAPTER_CONTRACT_VERSION = 1` | `ModelAdapter` trait shape |
| Address revision | `ADDRESS_REVISION = 1` | Bump on ANY change above |

All seven versions plus the address revision are sealed into every module
digest. [`validate_module()`](src/boundary_symplectic_transducer.rs:1)
rejects any module whose versions differ from frozen — a version change
produces a NEW module and a [`replay()`](src/boundary_symplectic_transducer.rs:1)
receipt, never an edit.

## 3. Dialect (`gm` v1): ops, types, effects

Single-block, straight-line module. Ops carry dense SSA ids `0..n` in
order; operand `k` of op `i` must satisfy `k < i` (dominance by
construction, checked).

| Op | Result type | Max operands | Effects |
| --- | --- | --- | --- |
| `gm.text_emit` | `Utf8` | 1 (thread) | `EMIT` permitted |
| `gm.code_emit` | `Utf8` | 1 (thread) | `EMIT` permitted |
| `gm.observe` | `Bytes` | 1 (thread) | `OBSERVE` required |
| `gm.tool_call` | `Handle` | 1 (thread) | `CALL` required |

Every op additionally carries: source span `[start, end)`, position
`(H3, T2)`, separately-typed cotangent component, and a typed payload
(`Text` / `Code{language}` / `Observe{mime}` / `Tool{tool, args}` where
args are typed `Int | RealBits | Bool | Text | Bytes` — never JSON).

Ingress builds one op per source line for text/code (spans partition
`[0, len)` exactly), one op per observation, one op per tool event.

## 4. Geometry and the two-form

- **Position** `(H3, T2)`: point on the hyperboloid `<x,x>_L = -1`,
  `x0 > 0`, in `R^{3,1}`; two torus phases in `[0, TAU)`. Derived
  deterministically from span bytes (`BST-ADDR-v1` hash stream).
- **Momentum**: separately-typed cotangent component, finite, projected to
  `T_h H3` (`<h, lin>_L = 0` within `1e-9`).
- **Frozen two-form**: `OMEGA = [[0, I5], [-I5, 0]]` on the 10-D phase
  chart (5 position + 5 momentum). Pinned nondegenerate by unit test
  (`omega_is_nondegenerate`: skew + square `= -I`, hence `det = ±1`).
- **Canonical-transport claim**: for a Jacobian `J`, residual
  `||JᵀΩJ − Ω||_F` must be `<= tol` with `tol <= 1e-6`, else the claim is
  rejected and the module stays **phase-augmented**. The identity passes
  (`residual ~ 0`); a `2x` position shear fails as required.

## 5. Q32.31 numeric contract (frozen copy)

`value = raw / 2^31`; decisions compare raw values exactly. Float contact
only at the checked boundary: `from_f64` rejects NaN/inf/out-of-range.
Independent copy inside the transducer module so the contract survives
even if the harness copy changes.

## 6. Validation (fail-explained, never silent)

[`validate_module()`](src/boundary_symplectic_transducer.rs:1) checks, in
order: frozen versions + address revision → domain/scope non-empty and
bounded → budgets (bytes, op count) → provenance non-empty → dense SSA ids
→ per-op type/arity/effect signature → dominance (`k < i`) → span bounds
→ name/payload match + strict UTF-8 of text/code spans + finite typed args
→ finite coordinates, H3 residual `<= 1e-9`, T2 range, cotangent tangency
→ **coverage: spans partition `[0, len)` exactly** (gap or overlap is an
error) → symplectic-claim residual check → digest recomputation.
First failure returns its reason; nothing is dropped.

## 7. Model adapter seam

[`ModelAdapter`](src/boundary_symplectic_transducer.rs:1) is the ONLY place
model-bound bytes exist. It binds adapter digest, weights/config digests,
shape and execution contract, plus a written disclosure. Two
implementations ship:

- `PassthroughTestAdapter` — test-only loopback; disclosed as such. Must
  never be used to claim tokenless end-to-end execution on a frozen model.
- `OpaqueFrozenAdapter` — encodes NOTHING; refuses with a disclosure naming
  the gap (e.g. "needs native embedding IDs"). A frozen model that cannot
  consume typed IR directly gets this refusal, not a silent substitution.

Model-native bytes and digests are NEVER inputs to the shared semantic key.

## 8. Semantic key (shared) vs model bytes (never shared)

`semantic_key = H("GMSEM-v1" || canonicalizer_version || domain || scope
|| payload)` where payload = source bytes + formats + op names/types/
spans/payloads. Excluded: geometry addresses, momenta, digests, adapter
and model bytes. Pinned by unit test (`semantic_key_excludes_model_bytes`).

## 9. Egress

[`egress_decode()`](src/boundary_symplectic_transducer.rs:1) indexes results
by SSA id and checks: duplicates → failure; unknown ids / missing ids →
failure with named lists; effect-bits mismatch → failure; payload
type-mismatch → `failed_inverse`; non-UTF-8 text payload → `failed_inverse`;
tool args must round-trip name-for-name, value-for-value. Loss = source
bytes of unmapped ops. Any failure returns
[`EgressFailure`](src/boundary_symplectic_transducer.rs:1) with
per-category receipts — never a plausible string.

## 10. Round-trip evidence

- Unit tests in-module: nondegenerate form, identity-canonical,
  shear-rejected, phase-augmented default, Q32.31, UTF-8 rejection,
  multilingual round-trip, key exclusion, adapter disclosure, unknown-id
  and bad-UTF-8 refusal.
- Integration: [`tests/boundary_symplectic_roundtrip.rs`](tests/boundary_symplectic_roundtrip.rs:1)
  covers multilingual text, code, tool calls, multimodal bytes, and
  malformed inputs (invalid UTF-8, empty provenance, oversized arg,
  unknown/missing/dup IDs, bad UTF-8 payload, effect mismatch, geometry
  tamper, digest tamper, version tamper, opaque-adapter refusal, replay).
  Zero silent loss: every failure names its bytes/ids.

## 11. Change rule

Freeze is mechanical: change anything in §§2–9 → bump the corresponding
version constant AND `ADDRESS_REVISION`, add a new `replay()` receipt, and
record it here. No in-place rewrites of sealed modules.
