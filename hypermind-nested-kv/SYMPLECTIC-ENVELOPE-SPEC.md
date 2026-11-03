# Symplectic MLIR Envelope — Spec (Goal 7b6eecba, Step 1: SPEC)

**Date.** 2026-10-28. **Status.** SPEC — build follows this doc. **Scope.** `hypermind-nested-kv` only.
No vault file modified.

## 0. Problem (measured)

- A39 run: treatment fails 80/80 at `mlir_op#26`, `superposition fail-closed after 3 iterations`
  [VERIFIED — `A39-RunCertification-2026-10-27.md` §1–2].
- Root cause class: typed SSA proves the call is well-formed
  (`arg_types/ret_type` mask); it cannot prove the *trajectory exists*.
  The arc carries `compiled_mlir_op: MlirOpId(u32)` — a label, not dynamics.
- Vault grep: zero hits for `symplectic|Hamiltonian|T*Q|cotangent` in any vault
  `src/` file [VERIFIED this pass]. The geometry has momentum (phases, rapidity,
  Wigner transport); the IR has only types.

## 1. Design: symplectic envelope, not a new dialect

Every SSA value already has a position half: the atom coordinate
`(coord: [f64;4] Lorentz, phase: [f64;2] T²)`. The envelope adds the
**momentum half** derived from quantities the map already stores:

- `p_phase` = phase velocity proxy: `score` gradient is unavailable pre-query,
  so momentum = `(phase_drive, rapidity)` carried per atom, seeded at ingest
  from the TPPE triple `(θ, r, amp)`: `p = (dθ, dr)` with `dr` from the
  spiral-out law slope and `dθ` from the fast/slow drive split.
- Per-arc traversal check (before dispatch, fail-explained not fail-silent):
  1. **Form check:** `ω(q1,p1) == ω(q2,p2)` within tolerance — the move is a
     canonical transformation, not a teleport. Concretely: phase-distance
     `d_T²` and Lorentz `cosh(d)` must satisfy the triangle budget
     `cosh(d_total) ≤ cosh(d_1)·cosh(d_2) + sinh(d_1)·sinh(d_2)` along the
     arc chain (exact hyperbolic triangle inequality — no new math).
  2. **Energy check:** arc cost `C = retrieve + attend` (the existing
     `map.attend` variational cost) must not exceed the F-term budget carried
     by the traversal receipt. Over-budget → `fail-explained {reason, cost,
     budget}` instead of `fail-closed at mlir_op#N`.
  3. **Frame check:** Wigner transport across the hop must keep
     `|det(W) − 1| ≤ ε` (rotation, not shear) — stored per arc, verified at
     traversal.

## 2. Data model (additive — no existing field changes)

```rust
/// Momentum half of an SSA value. Position half is Atom{coord, phase}.
struct SymplecticState {
    /// Phase-velocity proxy from the TPPE triple at ingest.
    p_phase: [f64; 2],   // (dθ_f, dθ_s)
    /// Radial momentum from the spiral-out slope dr/dw.
    p_radial: f64,
    /// Rapidity η carried from the encoder (energy scale of the move).
    rapidity: f64,
}
/// Per-arc traversal contract, stored beside the arc, checked before dispatch.
struct TraversalContract {
    from_atom: u64,
    to_atom: u64,
    mlir_op: u32,            // existing MlirOpId — unchanged semantics
    form_residual: f64,      // |ω1 − ω2|, must be ≤ tol
    energy_cost: f64,        // C = retrieve + attend at carve time
    energy_budget: f64,       // F-term budget; cost ≤ budget to dispatch
    frame_det_residual: f64, // ||det W| − 1|, must be ≤ ε
}
```

- `SymplecticState` defaults to zero-momentum (`p = 0, η = 0`) for all 1,631
  live atoms → existing snapshots stay valid; only new ingests populate it.
- `TraversalContract` is a NEW tool (`map.traverse`), not a change to
  `map.query`/`map.attend`. Old paths keep working; the envelope is opt-in
  per call.

## 3. What `mlir_op#26` would have reported

Under the envelope, the A39 failure becomes:
`fail-explained {op: 26, reason: "no connecting orbit at budget",
cost: C, budget: B, form_residual: r}` — a provable statement about the
trajectory (no arc chain satisfies form+energy+frame), instead of a null
lookup after 3 blind iterations. Same fail-closed behavior, with a receipt
that names the missing piece (which budget to raise, which arc to carve).

## 4. Build order (Steps 2–3 of the goal)

1. `SymplecticState` on ingest (derive from existing `(θ, r, amp)` — no new
   inputs) + `map.traverse` tool implementing the three checks.
2. Unit tests: form triangle inequality holds on carved chain; over-budget
   traversal returns `fail-explained` with cost/budget; zero-momentum atoms
   traverse iff form check passes at `p = 0`.
3. Replace: live map restart with the new binary; old atoms keep working
   (zero-momentum default); re-run the 12-probe audit + a traversal probe.
4. Receipt: `map.traverse` emits the same `receipt_sha256` chain as every
   other tool.

## 5. Non-goals (explicit)

- No new dialect, no change to `MlirOpId` numbering or `MlirTypeSig`.
- No vault code touched. No judge gate (separate goal).
- No claim that this fixes semantic retrieval (token-hash limit stands —
  separate fix).
