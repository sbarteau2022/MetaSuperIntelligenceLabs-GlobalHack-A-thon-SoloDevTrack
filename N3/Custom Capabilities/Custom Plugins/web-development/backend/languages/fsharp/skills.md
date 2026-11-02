# fsharp: immutable, total, typed
**Done means these pass:** `dotnet fantomas --check .` (no diff), `dotnet build -warnaserror` with `<Nullable>enable</Nullable>` (F# 9) and `--warnon:3390`, `dotnet test`.
## Rules
1. Immutability by default (`let`, records, discriminated unions); `mutable` only locally. Model alternatives with discriminated unions and match exhaustively (warning FS0025 is an error); do not use wildcard arms for unions you own.
2. No `null` in F# code: use `option` (`Option.map`, `bind`, `defaultValue`) and `ValueOption` in hot paths. Do not use `.Value` on options; pattern-match.
3. Expected failures are `Result<'ok,'err>` (railway-oriented with `Result.bind`/a `result { }` builder); exceptions are for bugs and I/O failures. Never swallow with an empty `with _ ->`.
4. Prefer pipelines (`|>`), small functions, partial application, and `List`/`Seq`/`Array` module functions; avoid `for` loops with mutation; use `Seq` for lazy, `Array` for performance; watch quadratic `@` on lists.
5. Use units of measure for quantities; single-case unions for ids; `[<Struct>]` for small value types; `[<RequireQualifiedAccess>]` on unions that clash.
6. Async: `task { }` for .NET interop (hot, cancellable with `CancellationToken`), `async { }` for F#-native cold computations; never `.Result`/`.Wait()`; dispose with `use`/`use!`.
7. Interop: handle .NET nulls at the boundary (`Option.ofObj`, `isNull`), annotate types for overloaded methods, `ConfigureAwait` where relevant.
8. Tests: Expecto/xUnit with FsCheck properties; cover empty collections and `None`.
9. Keep file order meaningful in the `.fsproj` (compile order matters); one module per file with a clear name; interface files (`.fsi`) for public surface where useful.
## Pitfalls
Indentation/offside mistakes changing meaning; value restriction; incomplete matches hidden by `_`; structural equality on records with functions; accidental mutation captured in closures; `List.length` in loops.
## Versions
Pinned: F# 10 on .NET 10 (ships with the .NET 10 SDK; checked 2026-10-07, source: from knowledge, with .NET 10 confirmed by the web). This skill assumes at least F# 8.
Rule: first run `dotnet --version` and read the project's pin (global.json, TargetFramework). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the F# style guide and component design guidelines, Domain Modeling Made Functional.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/fsharp.md).

Canonical source: `plugins/web-development/backend/languages/fsharp/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/fsharp.md` (reference; do not author).
