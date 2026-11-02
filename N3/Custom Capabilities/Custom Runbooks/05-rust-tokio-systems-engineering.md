---
name: rust-tokio-systems-engineering
description: Write deterministic, bounded-concurrency Rust and Tokio backend services with append-only SQLite WAL ledgers and optimistic concurrency.
depends_on: [hole-isolation-and-verification]
load_mode: lazy
---

# Rust & Tokio Systems Engineering Doctrine

All backend orchestration, runbook state, and serving primitives execute in Rust with zero FFI hops inside core contracts.

## Invariants

1. **Bounded Concurrency & Backpressure**:
   - Gate incoming HTTP/TCP streams behind bounded semaphores (`tokio::sync::Semaphore`) and run blocking I/O on `spawn_blocking` pools.
   - Enforce explicit timeouts and byte caps (`4 MiB` provider responses, `256 KiB` terminal streams) on every external boundary.
2. **Append-Only SQLite WAL Event Spine (`ares_events.sqlite3`)**:
   - Use `PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;` with `BEFORE UPDATE` and `BEFORE DELETE` triggers that `RAISE(ABORT)`.
   - Persist state via atomic temp-file write + `sync_all()` + `rename()`, and guard user file saves with SHA-256 content revisions (`HTTP 409 Conflict` on stale revision).
3. **Zero Credential Leakage**:
   - Resolve secrets at call time from `0600` `secrets.env` slots (`secret:SLOT`). Never serialize keys into JSON snapshots, telemetry events, or child environment variables.
