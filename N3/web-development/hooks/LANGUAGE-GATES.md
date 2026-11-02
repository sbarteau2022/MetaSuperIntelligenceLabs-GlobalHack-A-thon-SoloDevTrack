# LANGUAGE-GATES.md — checker per target, and which gate runs it

Date: 2026-10-30. Status: PROPOSED.
D = content gate (`audit.py` deny/warn rules). S = structure gate (`discipline.py` D1–D8). E = external toolchain the harness must run and paste (audit/discipline cannot execute compilers).

| # | Target | Checker command(s) | Gate | This machine 2026-10-30 |
|---|---|---|---|---|
| 1 | Zustand | `tsc --noEmit` + eslint zustand rules (no direct CLI) | E | tsc MISSING |
| 2 | Vite | `vite build` + `tsc --noEmit` | E | tsc MISSING |
| 3 | CodeMirror | `tsc --noEmit` | E | tsc MISSING |
| 4 | Monaco | `tsc --noEmit` | E | tsc MISSING |
| 5 | Tailwind | v3: config content-scan; v4: `@theme` in CSS (D-rules `no-hardcoded-color`) | D | n/a (content gate) |
| 6 | React Three Fiber | `tsc --noEmit` + R3F eslint plugin | E | tsc MISSING |
| 7 | WebGL/three.js | `tsc --noEmit`; shader strings via `glsl.md` review | E | tsc MISSING |
| 8 | React Flow | `tsc --noEmit` | E | tsc MISSING |
| 9 | Dockview | `tsc --noEmit` | E | tsc MISSING |
| 10 | Tauri 2.0 | `cargo clippy -- -D warnings` + `cargo tauri build` | E | cargo INSTALLED (1.99.0); tauri-cli NEEDS-VERIFY |
| 11 | WASM | `wasm-pack build` + `cargo clippy --target wasm32-unknown-unknown` | E | cargo INSTALLED; wasm-pack NEEDS-VERIFY |
| 12 | TypeScript 7/7.1 | `tsc --noEmit` (strict + noUncheckedIndexedAccess) | E | tsc MISSING |
| 13 | Rust | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | E | INSTALLED 1.99.0 |
| 14 | Tokio | `cargo test` + `cargo clippy` (no-Mutex-across-await is review, not regex) | E | INSTALLED |
| 15 | Metal/MSL | `xcrun metal -std=metal3.0` + metallib | E | swiftc 6.4.2 INSTALLED (Xcode toolchain) |
| 16 | Postgres | `psql` smoke + `EXPLAIN (ANALYZE, BUFFERS)` review + `pg_prove` | E | psql 17.11 client INSTALLED |
| 17 | SQLite | `sqlite3` + `PRAGMA integrity_check` | E | INSTALLED 3.54 |
| 18 | Python | `ruff check` + `ruff format --check` (+ `mypy --strict` for typed code) | E | ruff INSTALLED; mypy MISSING |
| 19 | C | `clang -std=c17 -Wall` + `clang-tidy` | E/D | clang 21 INSTALLED; `c-malloc-check` = D-warn |
| 20 | C++ | `clang++ -std=c++20 -Wall` + `clang-tidy` | E/D | clang 21 INSTALLED; `cpp-raw-new` = D-warn |
| 21 | HTML5 | html-validate + `audit.py` a11y D-rules (`no-div-button` etc.) | E/D | html-validate NEEDS-VERIFY |
| 22 | Java | `javac -Xlint:all` + checkstyle/spotbugs | E | java runtime MISSING |
| 23 | JavaScript | `node --test` / vitest + eslint/biome | E | node v26.9.0 INSTALLED; eslint/biome MISSING |
| 24 | Go | `go vet ./...` + `gofmt -l` | E | INSTALLED 1.27.3 |
| 25 | Shell | `shellcheck -S error` + `sh -n` | E/D | shellcheck/shfmt MISSING; `sh-unquoted-var` = D-warn |

Deny-rules live in `rules-table.json` + `rules-languages.json` (union loaded by `_engine.py`, D8-verified). A MISSING external checker is a RECORD item (waiver or install ticket), never a silent pass.
