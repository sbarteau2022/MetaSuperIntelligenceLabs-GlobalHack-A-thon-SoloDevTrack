# swift: value types, no force-unwraps, structured concurrency
**Done means these pass:** `swift build` (or `xcodebuild`) with strict concurrency checking, `swift test` / XCTest or Swift Testing, `swiftlint`, `swift-format lint`.
## Rules
1. Follow the API Design Guidelines: clarity at the call site, argument labels that read as grammar, no needless words.
2. Prefer `struct` and `enum` (value semantics); `final class` when you need identity; `let` over `var`; `private` by default.
3. No `!`, `try!` or `as!` outside tests and genuinely unreachable states. Use `guard let` with early exit, `if let`, and `??`.
4. Errors: `throws` with typed error enums, and specific `catch` clauses. No empty `catch`.
5. Concurrency: `async`/`await`, `actor` for shared mutable state, `@MainActor` for UI, `Sendable` types across isolation, structured concurrency (`async let`, `TaskGroup`). Check cancellation (`try Task.checkCancellation()`); never block the main thread.
6. Retain cycles: use `[weak self]` in escaping closures that can outlive `self` (timers, observers, long tasks); `unowned` only when lifetimes are provably nested.
7. SwiftUI: keep `body` cheap and free of side effects; own state with `@State` (private) or `@StateObject` / `@Observable`; give `ForEach` stable ids; small views.
8. `Codable` with explicit `CodingKeys`; switch exhaustively (`@unknown default` for framework enums); use `URLSession` async APIs.
9. Secrets go in the Keychain, not `UserDefaults` or source; honour App Transport Security; validate all external input.
10. Tests use XCTest or Swift Testing (`#expect`); inject clocks and networking; test edge tables.
## Pitfalls
Retain cycles through closures, delegates and `Timer`; `Dictionary` and `Set` order is unspecified; integer overflow traps; `lazy` is not thread-safe; capturing `self` strongly in `Task`.
## Versions
Pinned: Swift 6.4 (6.4 installed (Xcode 27.0); checked 2026-10-07, source: this machine). This skill assumes at least Swift 5.9.
Rule: first run `swiftc --version` and read the project's pin (.swift-version, Package.swift swift-tools-version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Swift API Design Guidelines, The Swift Programming Language, Apple's concurrency documentation.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/swift.md).

Canonical source: `plugins/web-development/frontend/languages/swift/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/swift.md` (reference; do not author).
