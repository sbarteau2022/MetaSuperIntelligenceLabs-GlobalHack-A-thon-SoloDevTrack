# lean: proofs that actually close, no sorry
**Done means:** `lake build` succeeds with no errors and no `sorry` warnings, `lake test` passes if tests exist, and `#print axioms theorem_name` shows only the standard axioms (`propext`, `Classical.choice`, `Quot.sound`) or none.
## Rules
1. Never leave `sorry` or add `axiom`s to make something compile. A theorem is done only when `lake build` reports no `sorry`.
2. State first, prove second. Write the exact statement with explicit types and hypotheses; check it with `example` or `#check` before proving. Name lemmas descriptively and keep them small.
3. Use structured proofs: `intro`, `cases`/`rcases`, `induction ... with`, `calc`, `have`, `obtain`. Prefer `simp`, `omega`, `decide`, `norm_num`, `linarith`, `ring` where they apply, but know what they do; avoid `simp` without `only` in library proofs when it is brittle, and replace terminal `simp?` results with the suggested `simp only [...]`.
4. Definitions that compute should compile as total functions: use structural recursion or provide `termination_by`/`decreasing_by`; use `partial` only for programs, never for definitions you prove things about.
5. Use `Option`, `Except` or dependent types to make partial operations total; use `Fin n`, `Vector`, or subtypes (`{x // p x}`) for invariants.
6. Organise with `namespace`, `section`, `variable`, `open ... in`; keep `import` minimal (Mathlib modules by name); follow Mathlib naming and style when using it (`snake_case` for theorems, `UpperCamelCase` for types).
7. Tests for executable code: `#eval` smoke checks and `lake test`; for specs, prove properties as theorems rather than testing them.
8. Pin the toolchain in `lean-toolchain` and dependencies in `lakefile.lean` and `lake-manifest.json`.
## Pitfalls
`decide` on large propositions timing out; universe issues; `simp` looping or rewriting differently after a Mathlib update; confusing `Nat` subtraction (truncated) with integer subtraction; coercion arrows hiding the real type.
## Versions
Pinned: Lean 4.34 (4.34.0 (2026-09-14); 4.35 was at release candidate 1; checked 2026-10-07, source: web, Sep 2026). This skill assumes at least Lean 4.0.
Rule: first run `lean --version` and read the project's pin (lean-toolchain, lakefile, lake-manifest.json). Write only for that version; if the project pins an older one, follow the project, and if it pins nothing, add a pin. When the installed toolchain is newer than the pin above, or the check date is over 12 months old, re-verify this skill against the release notes and update this section in the same change.
Sources: Theorem Proving in Lean 4, Functional Programming in Lean, the Mathlib style guide.

Date: 2026-10-30. Status: PROPOSED (scaffolded 2026-10-30; content carried from N3/languages/lean.md).

Canonical source: `plugins/web-development/backend/languages/lean/skills.md` (this file). Legacy flat copy: `~/Desktop/skills/plugins/languages/skills/lean.md` (reference; do not author).
