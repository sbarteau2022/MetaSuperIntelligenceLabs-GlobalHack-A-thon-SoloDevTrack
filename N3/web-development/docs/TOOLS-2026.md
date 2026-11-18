# TOOLS-2026.md — 2026 checkers + 3 custom builds

Date 2026-10-30. PROPOSED. PARTITION F. Acceptance: `TOOL-CONTRACT.md`. Truth GIVEN (§2), installs PROPOSED-only. No MISSING tool feeds GATE.

## 1. Catalog — 25 targets × 7 fields

| # | name | target | install-or-build | command | gate | machine | evidence |
|---|---|---|---|---|---|---|---|
| 1 | tsc | Zustand | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 2 | vite+tsc | Vite build | `npm i -D vite typescript` | `vite build` | RECORD→CLAIMS_REGISTER.md | INSTALLED (tsc) / MISSING (vite) | tsc 7.0.2 npm-global 2026-10-30; vite not on PATH |
| 3 | tsc | CodeMirror | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 4 | tsc | Monaco | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 5 | audit.py | Tailwind | N/A (content gate) | D-rule `no-hardcoded-color` | GATE→hooks/audit.py | VERIFIED 2026-10-30 (no v5) | npm tailwindcss latest 4.3.3 — no v5 exists; v3/v4 coverage stands |
| 6 | tsc | React Three Fiber | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 7 | tsc | WebGL/three.js | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 8 | tsc | React Flow | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 9 | tsc | Dockview | `npm i -D typescript` | `tsc --noEmit` | RECORD→CLAIMS_REGISTER.md | INSTALLED | §2 + tsc 7.0.2 npm-global 2026-10-30 |
| 10 | clippy | Tauri 2.0 | cargo + `cargo install tauri-cli` | `cargo clippy -- -D warnings` | PROVE→release-checklist | INSTALLED / NEEDS-VERIFY (cli) | §2: rustc 1.99.0; cli unpinned |
| 11 | wasm-pack | WASM | `cargo install wasm-pack` | `wasm-pack build` | RECORD→CLAIMS_REGISTER.md | INSTALLED / VERIFIED 0.15.0 2026-10-30 | wasm-pack 0.15.0 Homebrew pre-existing, measured 2026-10-30 |
| 12 | tsc | TypeScript 7.1 | `npm i -D typescript` | `tsc --noEmit` strict | RECORD→CLAIMS_REGISTER.md | INSTALLED 7.0.2 stable / 7.1 BETA-ONLY 2026-10-30 | Correction 2026-10-30 (supersedes '7.1 unreleased'): npm dist-tags latest 7.0.2, beta 7.1.0-beta, next 7.1.0-dev.20261030.1; tsc 7.0.2 npm-global measured 2026-10-30; pin 7.1 only when latest reports 7.1.x |
| 13 | cargo | Rust | preinstalled 1.99.0 | `cargo clippy --all-targets -- -D warnings` | PROVE→release-checklist | INSTALLED | §2: rustc 1.99.0 Homebrew |
| 14 | cargo | Tokio | preinstalled | `cargo test` | PROVE→release-checklist | INSTALLED | §2: same 1.99.0 toolchain |
| 15 | xcrun metal | Metal/MSL | preinstalled (Xcode) | `xcrun metal -std=metal3.0` | PROVE→release-checklist | INSTALLED | §2: Xcode toolchain; Linux-VM-unavailable, see §2c |
| 16 | psql | Postgres | preinstalled client 17.11 | `psql` smoke + EXPLAIN ANALYZE | PROVE→release-checklist | INSTALLED / NEEDS-VERIFY (PG18) | §2: psql 17.11; PG18 unpinned |
| 17 | sqlite3 | SQLite | preinstalled 3.54.0 | `sqlite3 DB "PRAGMA integrity_check;"` | PROVE→release-checklist | INSTALLED | §2: sqlite3 3.54.0 |
| 18 | ruff | Python | preinstalled + `pip install mypy` | `ruff check` + `ruff format --check` | PROVE→release-checklist | INSTALLED / VERIFIED mypy 2.4.1 2026-10-30 | §2: py3.14.7; mypy 2.4.1 via pipx measured 2026-10-30 |
| 19 | clang | C | preinstalled 21.0.0 | `clang -std=c17 -Wall` | PROVE→release-checklist | INSTALLED | §2: Apple clang 21.0.0 (+D-warn c-malloc-check) |
| 20 | clang++ | C++ | preinstalled | `clang++ -std=c++20 -Wall` | PROVE→release-checklist | INSTALLED | §2: same clang 21 (+D-warn cpp-raw-new) |
| 21 | html-validate | HTML5 | `npm i -D html-validate` | `html-validate` | RECORD→CLAIMS_REGISTER.md | VERIFIED 11.17.0 2026-10-30 | html-validate 11.17.0 npm-global measured 2026-10-30; D-rules cover no-div-button |
| 22 | javac | Java | `brew install --cask temurin` | `javac -Xlint:all` | RECORD→CLAIMS_REGISTER.md | MISSING | §2: shim only, no runtime |
| 23 | node --test | JavaScript | node v26.9.0 + `npm i -D eslint` | `node --test` | PROVE→release-checklist | INSTALLED / VERIFIED eslint 10.14.0, biome 2.6.4 2026-10-30 | §2: node v26.9.0; eslint v10.14.0 npm-global, biome 2.6.4 brew, measured 2026-10-30 |
| 24 | go vet | Go | preinstalled 1.27.3 | `go vet ./...` | PROVE→release-checklist | INSTALLED | §2: go1.27.3 darwin/arm64 |
| 25 | shellcheck | Shell | `brew install shellcheck` | `shellcheck -S error` | RECORD→CLAIMS_REGISTER.md | INSTALLED / VERIFIED 0.11.0 2026-10-30 | shellcheck 0.11.0 + shfmt 3.14.1 brew measured 2026-10-30; D-warn sh-unquoted-var |
| 26 | pnpm | Node.js pkg mgr | preinstalled 12.3.4 (Homebrew; corepack shim absent — `corepack` not on PATH, no install needed) | `pnpm --version` + `pnpm install --frozen-lockfile` | RECORD→CLAIMS_REGISTER.md | INSTALLED / VERIFIED 12.3.4 2026-10-30 | pnpm 12.3.4 via /opt/homebrew/bin/pnpm measured 2026-10-30; method: preinstalled, not corepack |
| 27 | node --version | Node.js runtime | preinstalled v26.9.0 | `node --version` | PROVE→release-checklist | INSTALLED | node v26.9.0 measured 2026-10-30 (confirms row 23) |
| 28 | ruby -wc | Ruby | preinstalled 4.0.7 + `gem install rubocop rspec` (both MISSING) | `ruby -wc` + `bundle exec rspec` | PROVE→release-checklist | INSTALLED (ruby) / MISSING (rubocop, rspec) | ruby 4.0.7 +PRISM [arm64-darwin27], gem 4.0.20 measured 2026-10-30; rubocop/rspec not on PATH |
| 29 | perl -c | Perl | preinstalled 5.34.1 (cpan 1.64; perlcritic/perltidy MISSING) | `perl -wc` + `prove -lr t/` | PROVE→release-checklist | INSTALLED (perl) / MISSING (perlcritic, perltidy) | perl v5.34.1 darwin-thread-multi-2level, $]=5.034001, cpan 1.64 measured 2026-10-30; write v5.10-compatible syntax |

## 2. Custom builds (problem → tool → gate)

(a) Pins drift from reality → **pin-audit** → RECORD→CLAIMS_REGISTER.md. Parses pin manifests, runs `--version` per binary, emits pinned/drifted/missing rows. Inputs: repo root + manifests. Outputs: per-tool table (declared, observed, verdict). Exits: 0 all pinned; 1 drift/missing; 2 manifest unreadable. Serves DECLARE — toolchain claims become checkable, not chat memory.

(b) Unfired gates are decorative → **gate-tracer** → RECORD→Failures.md. Maps each SKILL.md to declared invocations + last exit status; flags zero-invocation gates NEVER-EXERCISED. Inputs: SKILL.md declarations + logs. Outputs: per-gate counts, last exit, never-exercised list. Exits: 0 all exercised; 1 any never-exercised; 2 logs missing. Serves GATE — a gate nobody runs blocks nobody.

(c) Absence risks silent pass → **skip-reporter** → RECORD→CLAIMS_REGISTER.md. Emits one skip-receipt per unavailable capability: capability, reason, machine, disposition (dated waiver or install ticket). Inputs: LANGUAGE-GATES.md + truth table. Outputs: JSONL receipts. Exits: 0 written; 1 unwritable; 2 truth-table stale. Serves RECORD + WAIVE-OR-OBEY — absence is data, never a pass.

## Report

`docs/TOOLS-2026.md`: 25 rows, 7/7 contract fields each; no MISSING tool feeds GATE. NEEDS-VERIFY marks (2, 2026-10-30): #10 tauri-cli latest (install out of scope, deferred); #16 PG18 server vs 17.11 client (no docker per scope). Flipped 2026-10-30: #5 tailwind v5 VERIFIED-absent (npm latest 4.3.3); #11 wasm-pack VERIFIED 0.15.0; #12 TS 7.1 BETA-ONLY (latest 7.0.2 stable, beta 7.1.0-beta, next 7.1.0-dev.20261030.1 per npm dist-tags 2026-10-30; Correction 2026-10-30 supersedes VERIFIED-absent); #21 html-validate VERIFIED 11.17.0.
