# matlab: vectorised, preallocated and checked
**Done means these pass:** `checkcode` (or `mlint`) shows no warnings on changed files, and the tests pass: `matlab -batch "results = runtests; assertSuccess(results)"` (GNU Octave: `octave --eval "test myfun"`).
## Rules
1. Vectorise instead of looping over elements; use implicit expansion and built-ins (`sum`, `cumsum`, `find`, `logical` masks). Preallocate arrays (`zeros`, `nan`, `cell`) before filling in a loop.
2. One function per file, named like the file; local functions at the end. Use functions, not scripts, for reusable code; scripts pollute the base workspace.
3. Validate arguments with an `arguments` block (class, size, validators) instead of ad-hoc `nargin` checks. Throw errors with identifiers: `error("pkg:badInput", "message %d", x)`.
4. Indexing is 1-based and column-major. Use `end`, `numel`, `size(x, dim)` (not `length` on matrices). Handle empty arrays explicitly (`isempty`).
5. Floating point: compare with a tolerance (`abs(a-b) < tol`, `ismembertol`), never `==`. Beware integer saturation (`uint8(250)+10` is 255).
6. Avoid `eval`, `evalin`, `assignin`, `global`. Use structs, tables and `containers.Map` for structured data; use `string` arrays rather than char arrays for text.
7. Close what you open (`fclose`, `onCleanup`). Use `fullfile` for paths. Do not change directories in library code.
8. Tests: function-based or class-based unit tests with `verifyEqual` (with `AbsTol`/`RelTol`), covering empty and NaN inputs. Profile with `profile on`/`tic`/`toc` before optimising.
## Pitfalls
Growing arrays in loops; `==` on floats; `length` of a matrix returning the largest dimension; implicit scalar expansion hiding a shape bug; shadowing built-ins (`sum = 0`); char vs string arrays.
## Versions
Pinned: the latest MathWorks release (or GNU Octave) (release not verified; check `version` in MATLAB or `octave --version`; checked 2026-10-07, source: not verified online). This skill assumes at least R2019b (arguments blocks).
Rule: first run `octave --version` and read the project's pin (the project README, toolbox requirements). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: MATLAB coding guidelines (MathWorks), the MATLAB documentation on performance and testing.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/matlab.md).

Canonical source: `plugins/web-development/backend/languages/matlab/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/matlab.md` (reference; do not author).
