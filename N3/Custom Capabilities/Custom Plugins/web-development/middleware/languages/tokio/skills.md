# tokio: async with backpressure and an exit plan
**Done means these pass:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, plus `cargo test --release` for timing-sensitive paths.
## Rules
1. Pick the runtime deliberately: `#[tokio::main]` defaults to multi-thread; use `flavor = "current_thread"` only for single-threaded embeds/tests; set `worker_threads` from measured load, not habit.
2. Bound all ingress with `Semaphore` (or a bounded `mpsc` channel); never `spawn` unboundedly off network input — acquire the permit before doing work, release by dropping.
3. Blocking or CPU-heavy work goes in `spawn_blocking` (or `block_in_place` inside async only when justified); never `std::thread::sleep` or rayon-style spin inside an async task.
4. Every await gets a deadline: `tokio::time::timeout` on network/IO calls with a typed timeout error; document the duration choice next to the call.
5. Write cancel-safe code: `select!` losers get dropped, so guard side effects (use `CancellationToken` or fuse cleanup into `Drop`); only documented cancel-safe futures directly in `select!` arms.
6. Never hold a `Mutex` (or `RwLock` write guard) across `.await`; scope the guard, clone what you need, or switch to channels/`watch` for shared state.
7. Prefer `tokio::sync` primitives (`mpsc`, `oneshot`, `watch`, `Notify`) over `std::sync` + `spawn_blocking` bridges; size channels explicitly and handle `SendError`/lagging receivers.
8. Shut down cleanly: join all `JoinHandle`s, close listeners before workers, drain with a grace timeout; abort handles only after the grace expires, and log what was dropped.
9. Test with `#[tokio::test]` + `start_paused = true` for timer logic; inject clocks/durations instead of sleeping real seconds in tests.
10. Observe: name tasks (tracing spans), expose semaphore permits and queue depths as metrics; an unbounded queue with no gauge is a latent outage.
## Pitfalls
`MutexGuard` held across await silently serialises the runtime; `select!` without `biased;` starves under load; `interval` drifts unless `MissedTickBehavior` is set; `spawn`ed tasks that panic are silent unless the `JoinHandle` is awaited; `oneshot` senders dropped without sending hang receivers forever; mixing `std::time::Instant` math with paused test clocks flakes the suite.
## Versions
Pinned: Tokio 1.x on Rust 1.99, edition 2024 (rustc 1.99.0 installed on this machine 2026-10-30). This skill assumes at least Tokio 1.0 with Rust 1.75.
Rule: first run `rustc --version` and read the project's pin (Cargo.toml tokio version, rust-toolchain.toml). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the Tokio release notes and update this section in the same change.
Sources: Tokio docs (runtime, sync, time, select!, testing).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/tokio/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/tokio.md` (reference; do not author).
