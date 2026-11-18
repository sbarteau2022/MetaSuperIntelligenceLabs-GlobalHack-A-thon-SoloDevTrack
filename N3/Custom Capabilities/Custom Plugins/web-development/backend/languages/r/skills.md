# r: vectorised, reproducible and tidy
**Done means these pass:** `lintr::lint_package()`, `styler::style_pkg()` (no diff), `devtools::test()` (testthat), and for packages `R CMD check --as-cran`.
## Rules
1. Vectorise; avoid growing objects in a loop (preallocate or use `vapply`). Use `vapply`/`purrr::map_*` (typed) rather than `sapply`, whose return type can change.
2. Use `seq_len(n)` and `seq_along(x)`, never `1:length(x)` (which counts down when empty). Use `isTRUE()`/`isFALSE()` for conditions that may be `NA`.
3. Read data explicitly: set types (`readr::read_csv(col_types = ...)`), `stringsAsFactors` is FALSE by default in R 4+; check for `NA`s deliberately (`anyNA`, `na.rm = TRUE` with a reason).
4. No `setwd()` or absolute paths in code (use `here::here()` or project-relative paths). No `attach()`. No `library()` calls inside functions; use `pkg::fun()`.
5. Reproducibility: `set.seed()` before randomness, `renv` lockfile, `sessionInfo()` in reports; do not rely on global state or the workspace image.
6. Functions: explicit arguments with defaults, `stopifnot()` or `rlang::abort()` for validation, informative messages (`cli`), return values explicitly with `return()` only for early exits; no `<<-` unless truly needed.
7. Data frames: prefer tibbles; avoid partial matching (`$` on lists); `dplyr` joins state the `by`; check join cardinality; avoid `ifelse` on factors; factor levels set explicitly.
8. Numerics: `==` on doubles is wrong (`all.equal`/`isTRUE(all.equal())`, `dplyr::near`); integer overflow gives `NA` with a warning.
9. Tests with `testthat` (`expect_equal`, `expect_snapshot`), including empty and `NA` inputs. Document with roxygen2.
## Pitfalls
`sapply` returning a list or matrix unexpectedly; recycling silently; `T`/`F` redefinition (use `TRUE`/`FALSE`); `factor` to integer conversion; `NULL` vs `NA`; copy-on-modify surprises in loops.
## Versions
Pinned: R 4.6 (4.6.1 (2026-06-24); checked 2026-10-07, source: web, Sep 2026). This skill assumes at least R 4.1.
Rule: first run `Rscript --version` and read the project's pin (DESCRIPTION Depends, renv.lock). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Advanced R, R Packages, the tidyverse style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/r.md).

Canonical source: `plugins/web-development/backend/languages/r/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/r.md` (reference; do not author).
