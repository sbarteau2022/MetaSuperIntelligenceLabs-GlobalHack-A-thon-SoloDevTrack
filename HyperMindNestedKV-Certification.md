# HyperMind-Nested-KV — Build & Pressure-Test Certification

**Date.** 2026-10-27. **Auditor.** Zed agent (Stewart). **Target.** `hypermind-nested-kv` (MCP server, bin `hypermind-mcp` v0.1.0). **Status.** CERTIFIED WITH CORRECTIONS.

## Build

- `cargo build`: PASS (warnings only, from vendored `atomadb`; no errors). VERIFIED.
- `cargo test`: 0 tests present (no unit/integration tests in crate). REPORTED — tests needed.

## Corrections applied (code, justified)

1. `coord_from_text`: replaced 3-bucket `tf_embed(dim=3)` (high collision) with per-token random-projection unit vectors on S² plus bigram channel. Reduces near-duplicate collisions.
2. `score`: negated Lorentz inner product (originally larger inner = larger distance; ranking was inverted). Phase term retained as penalty. Effect: exact-match text now ranks #1.

Both changes are minimal and confined to `src/main.rs`.

## Pressure test (`pressure_test.py`)

| Step | Result |
|---|---|
| initialize | ok, protocol 2024-11-05 |
| ingest 200 atoms @ salience 0.5 | 2160 atoms/s, all receipts present |
| query (exact text) | top-1 = the ingested atom (score -1.0), ms retrieval |
| snapshot | status ok, sha256 recorded, atoms=200 |
| recover | status ok, atoms restored=200, epoch preserved |
| checkpoint compact | ok, receipt sha present |
| export passport | sha256 + atom_count=200 |
| bulk ingest 500 | 651 atoms/s |
| query under load | 0.4 ms |

Receipt presence: every tool result carried `receipt.receipt_sha256`. Snapshot hash round-trip verified. All assertions PASS (after fixes).

## Known gaps

- No unit/integration tests in crate; pressure harness is external Python and ad hoc.
- Retrieval is brute-force over atoms (not VP-tree); nested-retrieval speed gate is trivially satisfied only at this scale; gate must be re-run with a flat baseline index under preregistration (A39/A40 style) before any claim.
- Embedding quality depends on salience radius; cross-salience exact text does not always rank #1 (see MasterN³ doc §5.1). Claim restricted to same-salience or radius-normalized queries.
- Vendored `atomadb` has compile warnings and (per prior audits) some builds fail in `atoma_immune`; not touched here.

## Verdict

Build PASS, pressure test PASS after two corrections. Retrieval correctness now holds for exact-text queries at matched salience. Gate G-NN1 (same top-1 as flat) remains PROPOSED until a preregistered ablation runs.
