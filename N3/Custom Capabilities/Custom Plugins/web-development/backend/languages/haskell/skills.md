# haskell: total functions, explicit effects
**Done means these pass:** `cabal build --ghc-options="-Wall -Wcompat -Werror"`, `cabal test`, `hlint .`, and `ormolu --mode check` (or `fourmolu`) shows no diff.
## Rules
1. Keep functions total. Avoid `head`, `tail`, `fromJust`, `!!`, `read`, and incomplete patterns; return `Maybe`/`Either`, use `NonEmpty`, `uncons`, and `readMaybe`. Compile with `-Wincomplete-patterns` (in `-Wall`).
2. Give every top-level binding a type signature. Use `newtype` for domain values and records with strict fields (`!Int`) for data you accumulate; use `Data.Map.Strict`/`foldl'` (not lazy `foldl`) to avoid thunk build-up.
3. Make illegal states unrepresentable: sum types and smart constructors; do not export data constructors that break invariants. Use `Either e a` for errors with a real error type; reserve exceptions (`throwIO`, `bracket`, `finally`) for IO and resource safety.
4. Keep the core pure and push `IO` to the edges. Use `mapM_`/`traverse_`/`for_`; avoid `unsafePerformIO` and `trace` in committed code.
5. Strings: use `Text` (or `ByteString`) for real text, not `String`; encode and decode explicitly.
6. Laziness: know what is forced; `seq`, bang patterns and strict folds where you accumulate; avoid space leaks with lazy state monads and lazy accumulators; use `Data.IORef.atomicModifyIORef'`.
7. Type classes: derive `Eq`, `Show`, `Ord` only when meaningful; write lawful instances; do not use orphan instances.
8. Concurrency: STM for shared state, `async` for structured concurrency, `MVar` locks held briefly; use `bracket`/`mask` for cleanup.
9. Tests: `tasty` or `hspec` with `QuickCheck` properties (round trips, idempotence) plus examples for edge cases.
## Pitfalls
Partial record selectors; `length` on a tuple; lazy IO (`getContents`/`readFile`) holding files open; `Double` equality; `Integer` vs `Int` overflow; accidental quadratic `++` and `nub`.
## Versions
Pinned: GHC 9.14 (9.14.1 (Jan 2026); the 9.12 line is at 9.12.4; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least GHC 9.6.
Rule: first run `ghc --version` and read the project's pin (cabal.project, tested-with in the .cabal file, stack.yaml resolver). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the GHC user guide (warnings), "Haskell in Production"-style guides, hlint hints.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/haskell.md).

Canonical source: `plugins/web-development/backend/languages/haskell/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/haskell.md` (reference; do not author).
