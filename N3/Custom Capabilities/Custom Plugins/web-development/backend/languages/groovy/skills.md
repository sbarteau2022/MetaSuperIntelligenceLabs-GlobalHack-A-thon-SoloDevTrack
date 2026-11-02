# groovy: typed where it counts, safe in pipelines
**Done means these pass:** `./gradlew codenarcMain codenarcTest test` (or `npm-groovy-lint`), with `@CompileStatic` on non-DSL code and the Spock/JUnit tests green.
## Rules
1. Use `@CompileStatic` (or `@TypeChecked`) for library and application code; reserve dynamic Groovy for DSLs and build scripts. Declare types on method signatures and fields; use `def` only for locals where the type is obvious.
2. Equality: `==` calls `equals` (and `compareTo` for Comparables); use `.is()` for identity. Use the safe-navigation `?.` and the Elvis `?:` for null handling; avoid truthiness surprises (`0`, `""`, empty collections are falsey).
3. Strings: single quotes for plain strings, double quotes only when interpolating (`"${x}"`); `GString` is not a `String` as a map key or in `equals` (call `.toString()`); triple-single-quoted strings for multiline literals.
4. Collections: use literals (`[1, 2]`, `[a: 1]`), `collect`, `findAll`, `inject`; remember maps use `[:]` for empty and `.get(key, default)`; avoid mutating a collection while iterating.
5. Closures: know `delegate`, `owner`, and resolve strategy; keep closures small; do not capture mutable outer state.
6. Jenkins pipelines (`Jenkinsfile`): declarative syntax by default; never put secrets in the file (use `credentials()`/`withCredentials`); wrap shell steps with `set -euo pipefail`; avoid heavy Groovy in the CPS-transformed pipeline (non-serialisable objects break on resume, use `@NonCPS` methods for them); pin agent images and shared library versions; use `timeout`, `retry` and `post { always { ... } }` for cleanup; never interpolate untrusted values (branch names, PR titles) into `sh` strings (pass via environment variables).
7. Gradle build scripts: prefer the Kotlin DSL for new builds; configuration avoidance (`tasks.register`), version catalogs, no logic at configuration time.
8. Tests: Spock `given/when/then` with data tables (`where:`) for edge cases; no `sleep`.
## Pitfalls
`GString` hash-key mismatches; `==` on mixed numeric types; `any`/`every` on empty collections; missing `return` (last expression is returned); script-binding variables leaking as globals.
## Versions
Pinned: Groovy 5.0 (from knowledge; Jenkins and Gradle embed their own Groovy, which can be older; checked 2026-10-07, source: not verified online). This skill assumes at least Groovy 3.0.
Rule: first run `groovy --version` and read the project's pin (build.gradle, the Jenkins version). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Groovy language documentation and style guide, CodeNarc rules, Jenkins pipeline best practices.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/groovy.md).

Canonical source: `plugins/web-development/backend/languages/groovy/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/groovy.md` (reference; do not author).
