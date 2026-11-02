# Workspace rule — read first, applies to every agent, every session

CANONICAL WORKSPACE: `/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack` (the git repo).
CORRECTION 2026-10-31: an earlier revision of this file said "this folder only." That claim is WITHDRAWN as
harmful if this file is read outside the repo — e.g. the Zed workspace root at
`/Users/stewartbarteau/.Trash/MetaSuperIntelligenceLabs-Global-HackAthon-Design` is a trashed, empty shell, NOT the repo.
If you are not working inside the canonical repo path above (verified: `git -C <that path> status`), you are in the wrong place.
Native Zed file tools cannot reach the repo (workspace-scoped); use the terminal tool with absolute paths.

LEGACY NOTE: `master design.md` (52 KB, 2026-10-27) is SUPERSEDED by the C01–C35 split under `MasterDesignDocument/`
but KEPT ON DISK per ADR-19 (history preserved, do not delete). Canonical design source is the C01–C35 folders.

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

# Standing port key — locked 2026-11-02, do not reassign without Stewart

8000  SurrealDB ONLY — the persistent mind. Surreal clients only
      (GLOBALMIND_SURREAL_URL, HYPERMIND_SURREAL_URL). NO model server,
      NO other client, NO new default may ever point at 8000.
8001  Rapid-MLX (Qwen3.6 MoE rapid profile). Killed 2026-10-31, port
      assignment stands; do not reuse it for another server.
8002  oMLX — Qwen3.6-MoE-Combined, Qwen3.8, Nemotron. Live 2026-11-02.
8003  Next model server (e.g. QwQ if restarted — QwQ was on 8001 and
      was killed 2026-11-02; it must NOT restart on 8001).
30000 SGLang — Muse Glimmer.
6331  Qdrant — 127.0.0.1:6331, Stewart's address. Not answering at
      last check 2026-11-02 08:17 CST; treat as reserved, verify
      before use — do not start another service on 6331.

8080  HyperDX — localhost:8080, telemetry. Verified live
      2026-11-02 08:17 CST (pid 1918).

Rules: before starting any server, check `lsof -nP -iTCP:<port> -sTCP:LISTEN`.
If the port is taken by the wrong owner, STOP and ask Stewart — never
kill the owner to free a port, and never kill SurrealDB (8000) for any
reason. A port change must be made in every live default at once
(harness bins, fleet scripts, launch agents, client profiles), not in
one file alone, and this key must be updated in the same change.