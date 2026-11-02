# tauri: the webview is untrusted, the core is the gatekeeper
**Done means these pass:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `tsc --noEmit` (frontend).
## Rules
1. Capabilities first: declare permissions per window in capability files (least privilege); never enable broad `core:*` or shell-open without an allowlist.
2. All privileged work happens in Rust commands; the frontend invokes, never implements. Validate every command argument server-side (frontend validation is UX, not security).
3. Gate `shell.open` / filesystem / notification access behind explicit user action + scoped allowlists; log invocations.
4. Pin `@tauri-apps/*` npm packages to the same major as the Rust `tauri` crate; mismatched majors break IPC at runtime.
5. Updater: pin the public key, serve over HTTPS, test the full update cycle on a scratch channel before release.
6. Bundle identifiers, versions, and entitlements live in `tauri.conf.json` under review; no ad-hoc signing flags in CI.
7. Frontend: treat `invoke()` like a network call — timeouts, error envelope, loading and offline states.
8. Test commands headless (`cargo test` on command logic) before any webview test; webview tests assert IPC contracts, not Rust internals.
## Pitfalls
Over-broad capabilities turn any XSS into native code execution; unvalidated command args are injection holes; version drift between npm plugin and Rust crate produces silent IPC failures; updater without key pinning is a supply-chain hole; storing tokens in localStorage of the webview persists them in cleartext.
## Versions
Pinned: Tauri 2.0-era (capabilities, plugin packages; NEEDS-VERIFY 2026-10-30: confirm latest 2.x on this machine via cargo registry before pinning exact numbers). This skill assumes at least Tauri 2.0.
Rule: first read the project's pins (Cargo.toml tauri version, package.json @tauri-apps/* versions). Write only for those versions; if the project pins older ones, follow the project, and if it pins nothing, add pins. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Tauri 2.0 docs (capabilities, commands, updater, bundler).
Date: 2026-10-30
Status: active

Canonical source: `plugins/web-development/middleware/languages/tauri/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/tauri.md` (reference; do not author).
