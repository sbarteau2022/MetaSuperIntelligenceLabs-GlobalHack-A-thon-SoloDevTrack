---
name: ares-sandboxing
description: "Seatbelt and sandbox management for Ares: profile generation, fallback, confinement, unusable-probe, hole isolation. Use when debugging sandbox failures or hardening execution."
---

# Ares Sandboxing

## Scope
- Terminal: `backend/src/terminal.rs` (seatbelt profile, scrubbed env, 30s timeout, 256KiB cap).
- Holes: `backend/src/holes.rs` (run_in_hole, merge, approval).
- Probe: `seatbelt_usable()` cached per process.

## Rules
- Seatbelt preferred; fallback labeled honestly.
- Network denied; writes confined to cwd/tmp.
- Confinement: source under ~/Desktop or /tmp.
