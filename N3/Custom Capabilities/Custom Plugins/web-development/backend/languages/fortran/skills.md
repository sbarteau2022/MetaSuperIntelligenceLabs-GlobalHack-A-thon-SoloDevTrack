# fortran: explicit, modern and bounds-checked
**Done means:** it compiles with `gfortran -std=f2018 -Wall -Wextra -pedantic -fcheck=all -fbacktrace -ffpe-trap=invalid,zero,overflow`, tests pass (`fpm test`), and `fprettify --diff` is clean.
## Rules
1. Every program unit starts with `implicit none`. Use modules (`use mod, only: name`) and explicit interfaces; no `common` blocks, no `equivalence`, no computed `goto`.
2. Declare intent for every dummy argument (`intent(in)`, `out`, `inout`), and mark pure functions `pure` or `elemental` when they have no side effects. Use `allocatable` arrays, not pointers, by default; they free themselves.
3. Kinds: use `use, intrinsic :: iso_fortran_env, only: real64, int32` and write literals with the kind (`1.0_real64`). Never rely on default `real` precision.
4. Arrays are 1-based and column-major: loop with the first index innermost. Use whole-array operations, `sum`, `matmul`, `where`, `do concurrent` for independent iterations; pass assumed-shape arrays (`x(:)`), not explicit sizes.
5. Check `iostat=` and `iomsg=` on every `open`, `read`, `write`; use `newunit=`; close units; use `error stop` with a message for fatal errors.
6. Floating-point compare with a tolerance (`abs(a-b) <= tol*max(abs(a),abs(b))`); guard divisions; use `epsilon`, `huge`, `tiny` rather than magic numbers.
7. Strings are fixed-length and blank-padded: use `trim`, `len_trim`, and `character(len=:), allocatable` for variable text.
8. Build with `fpm` (or CMake); keep sources in `src/`, tests in `test/`; compile debug builds with checks and release builds with `-O2`.
9. Tests exercise boundaries (zero-size arrays, 1-element arrays, NaN/Inf handling).
## Pitfalls
Implicit typing (`i-n` integers) when `implicit none` is missing; integer division; out-of-bounds reads that compile silently without `-fcheck=all`; aliasing between dummy arguments; uninitialised locals.
## Versions
Pinned: Fortran 2023, gfortran 16 (GCC 16.1.0 (May 2026); flags below use -std=f2018 as the floor; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Fortran 2018.
Rule: first run `gfortran --version` and read the project's pin (fpm.toml, CMakeLists.txt). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Modern Fortran (Curcic), the Fortran 2018 standard, the Fortran Standard Library style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/fortran.md).

Canonical source: `plugins/web-development/backend/languages/fortran/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/fortran.md` (reference; do not author).
