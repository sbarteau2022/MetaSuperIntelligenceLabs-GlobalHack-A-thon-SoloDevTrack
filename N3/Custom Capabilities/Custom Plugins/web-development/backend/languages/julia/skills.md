# julia: type-stable, allocation-aware, package-structured
**Done means these pass:** `julia --project -e 'using Pkg; Pkg.test()'`, JuliaFormatter (`format(".")` leaves no diff), and for libraries `Aqua.test_all` and JET (`@report_opt`/`report_package`).
## Rules
1. Keep functions type-stable: a function's return type should depend on argument types, not values. Check hot code with `@code_warntype` or `@inferred`.
2. No non-`const` globals in performance code; pass data as arguments or use `const`. Put work in functions, not at top level.
3. Preallocate and use in-place functions (`mul!`, `push!`, `.=`); use broadcasting (`f.(x)`, `@.`) and views (`@view`) to avoid copies. Check allocations with `@time` or `@allocated`.
4. Dispatch over `if isa` chains; annotate arguments only as loosely as dispatch needs (`AbstractVector{<:Real}`), not concrete types. Struct fields should be concretely typed (or parametric).
5. Arrays are 1-based and column-major: iterate the first index fastest; use `eachindex`, `axes`; use `OffsetArrays` only deliberately.
6. Errors: throw specific exception types with messages (`ArgumentError`, `DomainError`); do not use exceptions for ordinary control flow; avoid `try` in inner loops.
7. Packages: `Project.toml` with compat bounds, a committed `Manifest.toml` for apps, `src/`, `test/`; use `@testset` and `@test` (with `isapprox` for floats); `using` specific names (`using LinearAlgebra: mul!`).
8. Concurrency: `Threads.@threads` for CPU parallelism with no shared mutable writes (use per-thread buffers or locks); `Distributed` for processes.
9. Mutation convention: functions that mutate end in `!` and take the mutated argument first.
## Pitfalls
`Vector{Any}` from untyped literals; global-scope loop slowness; closures capturing and boxing variables; integer overflow wraps silently (`Int64`); `1:n` ranges with `n = 0`; comparing floats with `==`.
## Versions
Pinned: Julia 1.12 (1.12.6 (Apr 2026); 1.13 was at release candidate 4 on 2026-09-03; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Julia 1.9.
Rule: first run `julia --version` and read the project's pin (Project.toml [compat] julia, Manifest.toml). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: the Julia manual (Performance Tips, Style Guide), the Blue Style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/julia.md).

Canonical source: `plugins/web-development/backend/languages/julia/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/julia.md` (reference; do not author).
