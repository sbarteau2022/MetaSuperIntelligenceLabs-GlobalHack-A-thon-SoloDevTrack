# Workspace rule — read first, applies to every agent, every session

CANONICAL WORKSPACE: this folder, and only this folder.
It is the git repo. If you are not working inside it, you are in the
wrong place.

ONE PLACE TO SUBMIT WORK: this repo folder. Submission means the file
shows up in `git status` inside this repo. A file anywhere else is
not submitted, no matter what was claimed about it.

NEVER write, create, or stage files in:

- any other folder under ~/Desktop/GitHub (this repo lives at ~/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack; do not write into sibling or parent folders),
- ~/Desktop, ~/Desktop/AtomaApp, ~/Desktop/Special Ops,
- ~/workspace, /tmp, or any other agent scratch area,
  except as a temporary draft you explicitly hand back for placement.

If a path, a tool, or a previous session points you outside this repo,
STOP and ask Stewart. Do not silently place the file outside and do
not invent a new home for it.

Build outputs are never staged: target/, node_modules/, dist/, *.log.

# Agent Code Review & Mathematical Invariance Protocol

1. **Compiler Gate & Exact Surgical Edits Protocol**:
   - **No Regex/Fuzzy Pattern Replacements**: Agents must never use fuzzy regex string replacements that can duplicate markers or misalign closing braces. Edits must target exact, unique contiguous code blocks.
   - **Compiler Verification & Immediate Rollback Gate**: Every edit to Rust (`.rs`) or TypeScript (`.ts`/`.tsx`) must immediately run through `cargo check --tests` or `npx tsc --noEmit` / `tsgo --noEmit` (TypeScript 7.1 beta boolean check). The gate strictly returns a boolean pass/fail (`True`/`False`). If the compiler rejects the edit, the file must be rolled back to its pristine state immediately before surfacing errors.
   - **Use Native Compiler-Guarded Tooling**: Use the `compiler_guarded_edit` tool to ensure edits compile clean before landing on disk.

2. **Mathematical Invariance & Zero-Mental-Math Protocol**:
   - **Zero Mental Math Law**: The model is strictly prohibited from computing mathematical, geometric, or dimensional values in prose or token space. All derivatives, limits, matrix inversions, hyperbolic transformations ($Poincaré$ projections, curvature tensors), bit-shifts, and boundary inequalities must be dispatched to native execution tools (`math-cas` / `sympy_eval` or `z3-solver`). Any numeric or analytical claim lacking an execution receipt is considered a hard hallucination.
   - Any numerical algorithm or optimization routine must document analytical limits, bounds, $\epsilon$-tolerances, and IEEE-754 overflow/underflow hazards verified by machine CAS.

3. **Verification Gates Over Assertions**:
   - A code review pass is never "passed" on visual inspection alone. Review passes require command receipts (`cargo test`, `cargo clippy --all-targets -- -D warnings`, or compiler verification).
   - If an issue is flagged, provide the minimal reproduction and the invariant violation reason.

4. **Strict Substrate & Boundary Invariants (Zero-Tolerance Laws)**:
   - **NO JSON, NO PYTHON WITHIN THE MODEL'S I/O BOUNDARY (NO EXCEPTION)**: Binary, tokenless, and strongly typed native protocol boundaries only. Neither JSON nor Python may cross the model's runtime I/O boundary. All serialization must use typed native representations (e.g. bincode / native transducers / strongly typed structs).
   - **NO SYNTHETIC TESTS OR WALK-AROUNDS**: No data or tests are permitted to 'pass' gates with synthetic data, simulated runs, mocks, or walk-arounds. Verification must run on real hardware substrates against real, non-synthetic datasets. A synthetic pass is a hard FAIL.

5. **Atoma Global Engineering Law Alignment**:
   - Adhere strictly to Law 1 (Quality over speed), Law 2 (Gates before ship), and Law 8 (Hardware runs before anything ships; no synthetic claims).

Before reporting anything done, run `git status` in this repo and
show it. No git status proof, no "done."
