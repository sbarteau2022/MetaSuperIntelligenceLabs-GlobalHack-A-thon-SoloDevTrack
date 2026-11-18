# scala: immutable, total and compiler-checked
**Done means these pass:** `sbt scalafmtCheckAll "scalafixAll --check" test` (or the `scala-cli`/`mill` equivalents), with `-Wunused`, `-Wvalue-discard`, `-Xfatal-warnings` (Scala 2: `-Xlint`) enabled.
## Rules
1. Use Scala 3 where possible. Prefer `val`, immutable collections, `case class`, `enum`, and `opaque type` for domain values; model alternatives with sealed traits or enums and match exhaustively (no catch-all `case _` on types you own).
2. Never use `null`; use `Option`. Do not call `.get` on `Option`, `Try` or `Either`; use `fold`, `map`, `getOrElse`, `match`. Avoid `return` and `throw` for expected failures: use `Either[E, A]` or `Try`.
3. Prefer expressions and `for`-comprehensions over mutable state and loops. `var` and mutable collections only when local and justified.
4. `given`/`using` for context (Scala 3) and extension methods; keep implicits/givens explicit and discoverable; no implicit conversions without a reason.
5. Concurrency: `Future` with an explicit `ExecutionContext`, or effect systems (cats-effect, ZIO) consistently; never block inside a `Future` on the default pool; `Await.result` only at the program edge.
6. Pattern-match on case classes and enums rather than `isInstanceOf`/`asInstanceOf`. Use `scala.util.boundary` or folds instead of exceptions for early exit.
7. Spark and big-data code: avoid `collect()` on big data, avoid UDFs when built-in functions exist, keep closures serialisable, never capture a `SparkContext`.
8. Tests with ScalaTest, MUnit or Weaver; property tests with ScalaCheck; cover empty collections and `None`.
## Pitfalls
`Option` boxing `null` via `Some(null)`; `List` append in loops (quadratic); shadowed names in `for`; mutable state captured by closures in `Future`s; type erasure in pattern matches.
## Versions
Pinned: Scala 3.8 (3.8.4 (Jun 2026); the 3.3 LTS line is at 3.3.8; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Scala 3.3 LTS.
Rule: first run `scala-cli version` and read the project's pin (build.sbt scalaVersion, project/build.properties). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Scala 3 Book, Scala style guide, "Effective Scala".

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/scala.md).

Canonical source: `plugins/web-development/backend/languages/scala/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/scala.md` (reference; do not author).
