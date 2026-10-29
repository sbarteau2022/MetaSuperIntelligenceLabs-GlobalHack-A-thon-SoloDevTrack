---
name: ares-verifiable-execution
description: "Verifiable code execution and context compression for Ares: holes, seatbelt/fallback, merge gates, checkpoint folding, receipt fingerprints. Use when executing code safely or compressing context without loss."
---

# Ares Verifiable Code Execution & Context Compression

## Scope
- Holes: `backend/src/holes.rs` (isolated execution, run_in_hole, merge approval).
- Seatbelt: `backend/src/terminal.rs` (profile, scrubbed env, caps).
- Checkpoint: `Handoff.md` fold at 60% context; pin system + original request.
- Receipts: SHA-256/FNV fingerprints on transitions.

## Rules
- Code runs in holes or seatbelt, never project root, until merge.
- Context compression keeps system prompt + original request pinned.
- Never claim execution without evidence (exit codes, output).
