# clojure: data first, immutable, no surprises
**Done means these pass:** `clj-kondo --lint src test` (no warnings), `cljfmt check`, and the tests (`clojure -X:test`, `clojure -M:test` or `lein test`).
## Rules
1. Prefer plain data (maps, vectors, sets, keywords) over objects; keep functions pure and data transformation pipelines using `->`, `->>`, `some->`, and transducers where they pay off. Use namespaced keywords for domain data.
2. Immutability: `def`/`defn` at top level only for constants and functions, never as mutable variables; use `atom` with `swap!` (pure update fn), `ref`/`dosync` for coordinated state, `agent` rarely. No side effects inside `swap!` functions or `dosync` transactions.
3. Errors: `ex-info` with a data map and a cause; catch specific exceptions; validate at boundaries with `clojure.spec` or Malli and coerce at the edges; return data (`{:error ...}`) for expected failures when composing.
4. Laziness: realise lazy sequences before leaving a scope (`doall`/`vec`/`into`), never rely on a lazy side-effect; avoid holding the head of a big seq; use `reduce`/`transduce` over huge data.
5. Reflection and boxing: set `*warn-on-reflection*` and `*unchecked-math* :warn-on-boxed`; add type hints only in measured hot paths.
6. Style: kebab-case names, `?` suffix for predicates, `!` for side effects, `->` prefix for conversions; docstrings on public vars; small namespaces with `:require [... :as alias]` (no `:refer :all`); private helpers with `defn-`.
7. Concurrency: `future`/`pmap` for simple parallelism (shut down with `shutdown-agents` in scripts), `core.async` for pipelines with bounded buffers; never block inside `go` blocks (use `<!`/`>!`, not `<!!`).
8. Tests: `clojure.test` or Kaocha with `are`/`testing`; generative tests with `test.check`; cover `nil` and empty collections.
9. Dependencies in `deps.edn` with exact versions (`antq` to update); REPL-driven but commit the tests.
## Pitfalls
`nil` punning hiding errors; `=` vs `identical?`; integer overflow (`+'` for auto-promotion); `for` is lazy and returns a seq; shadowing core names (`name`, `count`); `defonce` state surviving reloads unexpectedly.
## Versions
Pinned: Clojure 1.12 (1.12.4 on the official downloads page (Dec 2025); a newer patch is likely; 1.13 is alpha; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Clojure 1.11.
Rule: first run `clojure --version` and read the project's pin (deps.edn org.clojure/clojure). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Clojure style guide, "Clojure Applied", clj-kondo documentation.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/clojure.md).

Canonical source: `plugins/web-development/backend/languages/clojure/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/clojure.md` (reference; do not author).
