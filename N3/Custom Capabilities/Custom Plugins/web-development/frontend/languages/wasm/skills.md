# wasm: small binaries, explicit boundaries
**Done means these pass:** `wasm-pack build --target web`, `cargo clippy --all-targets -- -D warnings`, `wasm-opt -Oz` size check, `npx vite preview` (or trunk serve) smoke test in a real browser.
## Rules
1. Build with `wasm-pack build --target web` (bundler target only when the bundler consumes it); check in neither `pkg/` nor `target/` — CI rebuilds them.
2. Cross the JS/Rust boundary with `wasm-bindgen` types (`JsValue`, `js_sys`, `web_sys` with explicit feature flags); serialise with `serde-wasm-bindgen`, never ad-hoc string poking.
3. Keep the boundary narrow: pass plain data (numbers, strings, typed arrays), not graphs of live references; each exported fn does one job and returns a `Result<JsValue, JsValue>`.
4. Optimise size deliberately: `lto = true`, `opt-level = "z"`, `codegen-units = 1` in the wasm profile, then `wasm-opt -Oz` (binaryen) and record the byte count in CI.
5. Async across the boundary uses `wasm-bindgen-futures` (`future_to_promise`, `JsFuture`); never block the browser thread waiting on Rust.
6. For Rust frontends (Leptos/Dioxus/Yew) serve with `trunk serve` and build with `trunk build --release`; keep `Trunk.toml` public-url correct for subpath deploys.
7. Handle panics explicitly: `console_error_panic_hook` in dev, and a user-visible error surface in prod; silent `unreachable` in wasm is a blank page.
8. Test the contract, not just the logic: `wasm-pack test --headless --chrome` (or firefox) for boundary tests, native `cargo test` for pure logic.
9. Gate web-sys features tightly: enable only the APIs used; each extra feature grows the glue and the audit surface.
## Pitfalls
`println!` goes nowhere without a logger (`console_log`/`tracing-wasm`); `Instant::now` panics on some targets so use `performance.now` via web-sys; 4GB memory ceiling and no threads unless SharedArrayBuffer + COOP/COEP headers cooperate; Debug builds ship megabytes of nothing — always measure release + wasm-opt; trunk's default dist/ clashes with Vite's — keep one pipeline per app.
## Versions
Pinned: wasm-pack VERIFIED 0.15.0 2026-10-30, wasm-bindgen 0.2-era, binaryen wasm-opt absent (not probed), trunk 0.2-era deferred-install (NEEDS-VERIFY 2026-10-30 — trunk/wasm-opt versions not probed; rustc 1.99.0 installed on this machine). This skill assumes at least wasm-bindgen 0.2 with Rust 1.75.
Rule: first run `rustc --version`, `wasm-pack --version`, and `wasm-opt --version`, and read the project's pins (Cargo.toml, Trunk.toml). Write only for those versions; if the project pins older ones, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the wasm-pack/wasm-bindgen/trunk release notes and update this section in the same change.
Sources: wasm-pack book, wasm-bindgen guide, binaryen docs, trunk docs.
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/frontend/languages/wasm/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/wasm.md` (reference; do not author).
