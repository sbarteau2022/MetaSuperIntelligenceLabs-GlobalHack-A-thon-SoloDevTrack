# ocaml: types do the proof, exceptions stay local
**Done means these pass:** `dune build @check @fmt` (no diff from `ocamlformat`), `dune test`, with warnings as errors in the dev profile.
## Rules
1. Use variants and records, and match exhaustively; leave warnings 8 (non-exhaustive) and 9, 26/27 (unused) as errors. Avoid wildcard `_` arms on variants you own.
2. Prefer `option` and `result` for expected failures (`Result.bind`, `let*` with `Result`/`Option` binders). Exceptions are for truly exceptional cases and stay inside a module; never leak `Not_found`/`Failure` from a public API (use `find_opt`, `List.assoc_opt`, `int_of_string_opt`).
3. Public modules have an `.mli` with abstract types and documentation comments (`(** ... *)`); expose the minimum.
4. Immutability by default; `ref` and mutable fields only when local and needed. Prefer `List`/`Seq`/`Map`/`Set` from the standard library (or `Base`/`Containers` consistently); avoid `List.nth`, `List.hd` on possibly empty lists, and `@` in loops (quadratic).
5. Tail recursion for long lists (`List.rev_append`, accumulators, `List.fold_left`); `Stdlib.compare` is polymorphic and slow: define typed equality and comparison functions.
6. Resources: `Fun.protect ~finally`, `In_channel.with_open_text`; close what you open. Strings are immutable bytes: handle Unicode explicitly (`Uutf`).
7. Concurrency (OCaml 5): `Domain` for parallelism with no shared mutable data without `Atomic`/`Mutex`; use `Eio` (or `Lwt` consistently) for I/O; never mix blocking calls into the scheduler.
8. Tests: `alcotest`/`ppx_expect` and `qcheck` properties; cover empty inputs; keep expect-test output stable.
9. Layout: a `dune-project`, `dune` stanzas per directory, a `.ocamlformat` file checked in, opam file generated from `dune-project`.
## Pitfalls
Structural equality (`=`) on cyclic or functional values raising; value restriction; label order mistakes; shadowing hiding a bug; `Printf` format-type errors; polymorphic comparison on floats with `nan`.
## Versions
Pinned: OCaml 5.4 (5.4.1; the 4.14 LTS line is at 4.14.4; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least OCaml 5.0.
Rule: first run `ocaml -version` and read the project's pin (the .opam file, dune-project (lang dune N)). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Real World OCaml, the OCaml manual, the OCaml community style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/ocaml.md).

Canonical source: `plugins/web-development/backend/languages/ocaml/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/ocaml.md` (reference; do not author).
