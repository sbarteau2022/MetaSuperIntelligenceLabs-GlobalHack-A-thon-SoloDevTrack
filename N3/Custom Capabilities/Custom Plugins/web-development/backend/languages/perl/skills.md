# perl: strict, warned and tested
**Done means these pass:** `perl -wc file.pl`, `perlcritic --stern .`, `perltidy` leaves no diff, and `prove -lr t/` passes.
## Rules
1. Every file starts with `use strict; use warnings;`. Check the target perl first (`perl -e 'print $]'`; macOS ships 5.34): write `use v5.36;` (which also enables signatures and `say`) only if the target is 5.36 or newer; otherwise use `use v5.10;` for `say` and avoid signatures, `isa` and `try`. A version your interpreter lacks is a compile failure, not a style choice. Declare variables with `my` in the smallest scope.
2. Never interpolate user input into shell commands or regexes: use list-form `system`/`exec`/`open` (`open my $fh, '<', $path`), `quotemeta` or `\Q...\E`, and taint mode (`-T`) for untrusted input. Use three-argument `open` with a lexical filehandle and always check the result (`or die "...: $!"`).
3. Use `Carp` (`croak`/`confess`) in modules, `die` with objects or clear messages, and `Try::Tiny` or `feature 'try'` for exception handling; do not rely on `$@` without protecting against `eval` failure idioms.
4. Data: use `//` (defined-or) rather than `||` where 0 and "" are valid; `exists` vs `defined`; `scalar(@array)`; avoid parallel arrays, use hashes of hashes or small classes. Be explicit about context (`wantarray`).
5. Modules, not scripts: `package`, `Exporter` with `@EXPORT_OK`, a `.pm` per package, `1;` at the end (or `use v5.38`); object orientation with `Moo`/`Moose`/`Object::Pad` or core `class` (5.38+), not hand-blessed hashes unless tiny.
6. Regexes: `/x` for readability, named captures, anchors (`\A`, `\z`), non-greedy where needed; beware catastrophic backtracking on untrusted input; `use utf8` and `binmode` for Unicode text; decode input, encode output.
7. Do not use `local $_` carelessly; prefer lexical loop variables (`for my $x (@list)`); no barewords; no indirect object syntax (`new Foo`); avoid `$&`.
8. Tests with `Test::More`/`Test2`, one `.t` file per concern, `prove -lr`; use `Test::Exception` for failures; cover empty lists and undef.
9. Never name your own sub after a Perl builtin (`shift`, `push`, `print`, `join`, `reverse`...): call it `shift_char` and the like, because `shift(...)` then resolves to the builtin and fails with "Ambiguous call" under `-w`.
## Pitfalls
`my $x = ... if cond` (undefined behaviour); list vs scalar assignment (`my ($x) = @a`); autovivification creating keys; `==` vs `eq`; numeric strings; `each` iterator state; modifying `$_` aliases.
## Versions
Pinned: Perl 5.34.1 (measured 2026-10-30: `This is perl 5, version 34, subversion 1 (v5.34.1)` darwin-thread-multi-2level, `$]` = 5.034001; cpan 1.64; perlcritic MISSING; perltidy MISSING; write `use v5.10;` style, no 5.36+ syntax — supersedes 5.42 knowledge-pin 2026-10-07). This skill assumes at least Perl 5.10.
Rule: first run `perl -e 'print $]'` and read the project's pin (cpanfile, Makefile.PL MIN_PERL_VERSION). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Perl Best Practices, perldoc perlstyle, Modern Perl.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/perl.md).

Canonical source: `plugins/web-development/backend/languages/perl/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/perl.md` (reference; do not author).
