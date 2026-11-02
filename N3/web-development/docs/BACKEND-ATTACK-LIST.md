# BACKEND-ATTACK-LIST.md — backend master attack queue

Date 2026-10-30. Status PROPOSED. PARTITION E (SWARM E). Ground truth: `N3/languages/plugin.json` entry ids (55 total) + `TOOLS-2026.md` machine-truth table + adapter-contract (ENGINEERING-GOVERNANCE.md §adapter-contract). No invented versions; NEEDS-VERIFY where unsure.

## Ranked attack queue (OpenDesign-backend need first, then blast radius)

| # | Target | Status | Why this rank (one line) | Acceptance bar: skill shape + checker + gate |
|---|---|---|---|---|
| 1 | postgres | DONE | Primary OpenDesign datastore; EXPLAIN-gated queries are the backend hot path | skill `postgres` exists; `psql` smoke + EXPLAIN ANALYZE; PROVE→release-checklist |
| 2 | rust | DONE | Backend service language; clippy-deny is the correctness floor | skill `rust` exists; `cargo clippy --all-targets -- -D warnings` + `cargo test`; PROVE→release-checklist |
| 3 | tokio | DONE | Async runtime under every Rust service; un-awaited tasks are silent outages | entry `tokio` exists (runtime slice of rust); `cargo test` incl. `#[tokio::test]`; PROVE→release-checklist |
| 4 | python | DONE | Scripts, migrations, ML-adjacent tooling; ruff+mypy gate is cheap | skill `python` exists; `ruff check` + `ruff format --check` (+`mypy` NEEDS-VERIFY install); PROVE→release-checklist |
| 5 | sqlite | DONE | Local-first / embedded store; integrity_check is one line | skill `sqlite` exists; `sqlite3 DB "PRAGMA integrity_check;"`; PROVE→release-checklist |
| 6 | sql | DONE | Shared query discipline across pg+sqlite; lint once, apply twice | skill `sql` exists; `sqlfluff` (NEEDS-VERIFY install) + `psql`/`sqlite3` smoke; PROVE→release-checklist |
| 7 | shell | DONE | Deploy/migration scripts; shellcheck catches the outage class | skill `shell` exists; `shellcheck -S error`; RECORD→CLAIMS_REGISTER.md |
| 8 | go | DONE | Secondary service language; vet is zero-cost | skill `go` exists; `go vet ./...`; PROVE→release-checklist |
| 9 | docker/containers | MISSING | No entry id; unpinned images are the #1 deploy-drift vector | NEW skill `docker` (pin-digest + no-latest rules); checker NEEDS-VERIFY (no docker per TOOLS-2026 scope); gate PROVE→release-checklist once toolchain lands |
| 10 | caching/redis | MISSING | No entry id; cache-invalidation bugs outlive deploys | NEW skill `redis` (TTL + eviction + stampede rules); checker NEEDS-VERIFY (no redis-cli on machine per TOOLS-2026); gate RECORD→CLAIMS_REGISTER.md |
| 11 | message-queues | MISSING | No entry id; at-least-once semantics need codified retry/idempotency | NEW skill `message-queues` (ack + DLQ + idempotency rules); checker NEEDS-VERIFY (broker TBD); gate RECORD→CLAIMS_REGISTER.md |
| 12 | c | DONE | Native extensions / FFI surface; -Wall is non-negotiable | skill `c` exists; `clang -std=c17 -Wall` (+D-warn c-malloc-check); PROVE→release-checklist |
| 13 | c++ | DONE | Same FFI rationale, narrower blast radius than C here | skill `cpp` exists; `clang++ -std=c++20 -Wall` (+D-warn cpp-raw-new); PROVE→release-checklist |
| 14 | wasm | DONE | Edge-compute slice of backend; pack-build is the gate | skill `wasm` exists; `wasm-pack build` (VERIFIED 0.15.0); RECORD→CLAIMS_REGISTER.md |
| 15 | tauri | NEEDS-EXPAND | Desktop shell around backend; cli unpinned per TOOLS-2026 #10 | entry `tauri` exists, skill thin; `cargo clippy -- -D warnings` (cli NEEDS-VERIFY); PROVE→release-checklist |
| 16 | java | NEEDS-EXPAND | Entry exists but runtime MISSING (shim only, TOOLS-2026 #22); low OpenDesign need | entry `java` exists, toolchain absent; `javac -Xlint:all` once `temurin` installed (PROPOSED-only); gate RECORD→CLAIMS_REGISTER.md |
| 17 | ruby | NEEDS-EXPAND | Entry exists, no TOOLS-2026 row; legacy-integration only | entry `ruby` exists; checker NEEDS-VERIFY (`rubocop`/`rspec` unmeasured); gate RECORD→CLAIMS_REGISTER.md |
| 18 | php | NEEDS-EXPAND | Entry exists, no TOOLS-2026 row; legacy-integration only | entry `php` exists; checker NEEDS-VERIFY (`phpstan`/`phpunit` unmeasured); gate RECORD→CLAIMS_REGISTER.md |

## Counts

DONE 11 (postgres, rust, tokio, python, sqlite, sql, shell, go, c, c++, wasm) · NEEDS-EXPAND 4 (tauri, java, ruby, php) · MISSING 3 (docker/containers, caching/redis, message-queues).

## NEXT-UP top-3

1. **docker/containers** — biggest ungoverned blast radius (deploy drift); write skill + pin-digest rule first.
2. **caching/redis** — TTL/eviction rules prevent the longest-lived bug class; checker install is small.
3. **message-queues** — retry/idempotency contract unblocks async OpenDesign workers; broker choice is the open decision.
