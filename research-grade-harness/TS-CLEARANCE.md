# TypeScript 7.1-Beta — Clearance Bar (NOT SHIPPED — toolchain absent)

**Status.** PROPOSED. No TS 7.1-beta toolchain on this machine (checked:
no `tsc`, no `bun`, no `deno`; only node 26). Nothing TS was built,
measured, or committed in this pass. This doc is the bar TS must clear
when the toolchain lands — not a claim it clears it.

## Why TS is not the decision language here

The decision math (scores, gates, priorities, traversal contracts,
fingerprints) is native Rust, 34/34 tested, `#![forbid(unsafe_code)]`
in the map crate. Porting it to TS would re-prove what is already proven,
in a slower language, for no gain. **TS does not touch the decision core.
Ever.** That is a standing rule, not a preference.

## The ONE role TS may earn: MCP boundary throughput

The MCP protocol is JSON-RPC. Every tool call crosses a JSON boundary.
Today the Rust server pays serde parse+serialize per call. A TS shim at
the boundary MIGHT move high-fan-out tool calls faster IF all of these
hold (measured, not asserted):

1. **Cold start:** TS worker responds to first request faster than the
   Rust binary's process spawn on the target machine (measure both,
   same hardware, `hyperfine`-style, n=50).
2. **JSON-boundary throughput:** sustained tool-call round-trips/sec
   (parse args → route → serialize envelope) beats the Rust server's
   measured rate at 1k/10k/100k calls. The Rust number is the bar —
   recorded below on first bench run.
3. **No correctness drift:** byte-identical receipts for the same inputs
   (fuzz both, diff outputs — any divergence voids the TS shim).
4. **No decision creep:** the shim routes and formats. It computes
   NOTHING (no scores, no gates, no priorities). One `if` that decides
   anything beyond routing = removal.

## How to run the shootout (when the toolchain lands)

```sh
# 1. Install the beta toolchain
npm install -g typescript@7.1-beta   # or per release notes at the time
# 2. Bench the Rust boundary (the bar)
cd research-grade-harness && cargo bench --bench boundary  # TO BE WRITTEN with the TS harness
# 3. Write the TS shim against the SAME bench harness
# 4. Winner ships the boundary. Loser is deleted, not kept "just in case".
```

## Re-verification (2026-10-29, VERIFIED on this Mac)

- Toolchain probe: `tsc` ABSENT, `bun` ABSENT, `deno` ABSENT;
  `node` v26.9.0, `cmake` 4.4.3, `cargo` 1.99.0 present.
  No TS toolchain has landed; nothing TS built, measured, or committed.
- `ablation_benchmarks/ts_frontend/src/` is empty; a `STATUS.md`
  disposition marker now records that emptiness as PROPOSED, not as a
  result. The boundary-race shootout above remains the only path for TS
  to ship, and the decision core stays Rust regardless of outcome.
- Disposition: OUT OF SCOPE for this pass. The shootout is pursued only
  if a TS 7.1 toolchain lands AND the boundary race is explicitly
  requested — not by default, per the standing rule.

## What was NOT done (explicit)

- No TS written, no TS measured, no TS committed.
- No `tsconfig`, no `package.json`, no node_modules in this repo for TS.
- The "faster than just about anything" claim is UNPROVEN — it is a
  hypothesis with a falsification procedure above, not a result.
- If the shootout shows Rust wins the boundary too, TS ships NOWHERE
  and this doc records that outcome. The fastest system wins; the
  language is not the point.
