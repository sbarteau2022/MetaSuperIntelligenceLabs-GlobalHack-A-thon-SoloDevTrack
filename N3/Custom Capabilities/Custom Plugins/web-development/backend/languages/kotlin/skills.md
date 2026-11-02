# kotlin: null-safe, immutable, structured concurrency
**Done means these pass:** `./gradlew ktlintCheck detekt test`.
## Rules
1. `val` over `var`; immutable collections in APIs; `data class` for values; `sealed` classes or interfaces with an exhaustive `when` (no `else` on types you own).
2. Null-safety: avoid `!!`. Use `?.`, `?:`, `requireNotNull(x) { "why" }`. Annotate Java interop so platform types do not leak.
3. Model expected failures as sealed results (or `Result`); throw for bugs. Never swallow exceptions, and rethrow `CancellationException` in coroutine code.
4. Coroutines: structured concurrency only (`coroutineScope`, `supervisorScope`); never `GlobalScope`. Inject dispatchers; use `withContext(Dispatchers.IO)` for blocking calls; keep cancellation cooperative; use `Flow` for streams (`stateIn`, `collectLatest`).
5. Scope functions (`let`, `apply`, `also`, `run`, `with`) only when they read better than the plain version; do not nest them.
6. Use named and default arguments instead of overload piles. `const val` for constants. `lateinit` sparingly; prefer constructor injection.
7. Closeables use `use { }`. Prefer `Sequence` for long collection chains; keep `List` for small ones.
8. Interop annotations (`@JvmStatic`, `@JvmOverloads`) only when Java callers need them.
9. Security and money follow the Java rules: parameterised SQL, `BigDecimal`, `java.time`, no untrusted deserialisation.
10. Tests: JUnit 5 or Kotest, `runTest` for coroutines, MockK or fakes, one behaviour per test.
## Pitfalls
`equals` on arrays compares identity; `lazy` default is synchronised; swallowing `CancellationException` breaks cancellation; `==` vs `===`; mutable state captured by lambdas.
## Versions
Pinned: Kotlin 2.3 (2.3.20 (Mar 2026); 2.4 is in beta; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Kotlin 2.0.
Rule: first run `kotlinc -version` and read the project's pin (build.gradle(.kts), gradle/libs.versions.toml). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Kotlin coding conventions, Kotlin coroutines guide, Android Kotlin style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/kotlin.md).

Canonical source: `plugins/web-development/backend/languages/kotlin/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/kotlin.md` (reference; do not author).
