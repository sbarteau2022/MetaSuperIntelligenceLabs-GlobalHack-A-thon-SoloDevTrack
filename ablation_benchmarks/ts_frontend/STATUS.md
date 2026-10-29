# ts_frontend — Scaffold Status (2026-10-29)

**Status: PROPOSED. `src/` is intentionally empty — no TS written, measured,
or committed. This marker is the disposition, not a placeholder for code.**

## Why empty (VERIFIED 2026-10-29)

Toolchain probe this Mac: `tsc` ABSENT, `bun` ABSENT, `deno` ABSENT
(only `node` v26.9.0 present). Per `research-grade-harness/TS-CLEARANCE.md`
(REPORTED from the clearance bar, re-verified absent today): no TS 7.1-beta
toolchain on this machine, nothing TS built, measured, or committed.

## Scope rule (standing, not preference)

Per `TS-CLEARANCE.md`: the decision math (scores, gates, priorities,
traversal contracts, fingerprints) is native Rust, `#![forbid(unsafe_code)]`
in the map crate. **TS does not touch the decision core. Ever.** The ONE
role TS may earn is MCP JSON-RPC boundary throughput — and only by winning
the shootout defined in `TS-CLEARANCE.md` (cold start, sustained
round-trips/sec at 1k/10k/100k calls vs the measured Rust bar, byte-identical
receipts under fuzz, no decision creep: one deciding `if` = removal).

## What this marker is NOT

Not a result, not a claim, not a promise of TS code. If the shootout shows
Rust wins the boundary too, TS ships NOWHERE and that outcome is recorded in
`TS-CLEARANCE.md`. Do not stage `node_modules/`, `dist/`, or any TS build
output — build outputs are never staged (repo `AGENTS.md`).
