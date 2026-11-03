# Certification Ruling: AdvAudit + MethodAndMath + A39-v1 + ErrAndImprovements + MathProofs (Pressure-Tested) + Formalized Dyptech

**Date.** 2026-10-27 (supplement to `DyptechCertificationAndFormalization.md`, same day). **Auditor.** Muse.
**Scope.** Co-agent files `AdvAudit.md`, `MethodAndMathSideBySide.md`, `Pre-Registration-A39-v1.md` (certified earlier today) PLUS mid-pass additions `ErrAndImprovements-AA.md`, `MathProofs-Dyptech.md`; pressure-test audit executed by me against the live vault tree.
**Standards.** Documentation-standards (labels); research-evidence (measured > reported); C03.3 (no vault file, no co-agent file modified — all rulings live here).
**Verdict.** AdvAudit: CERTIFIED (12/12 findings keep; AA-02 raised to CRITICAL; AA-13 process finding added). MethodAndMath: CERTIFIED WITH 4 CORRECTIONS (M-1–M-4, as filed). A39-v1: PROPOSED, RUN BLOCKED (compile + B-7 + conjunction). ErrAndImprovements: FIXES CERTIFIED PRESENT-IN-SOURCE (E1_partial/E3/E4/E5 verified in `silo.rs`); **TEST-COUNT CLAIMS REJECTED** (§2). MathProofs: 7/8 sketches CERTIFIED, **P6 WITHDRAWN** (§3). Formalized dyptech capstone: §§5–6 of the companion file stand; P-1–P-8 proved as stated.

---

## 1. Pressure-test audit (executed — the ruling everything else rests on)

Apple Silicon, stable `cargo 1.99.0`, vault HEAD `d31b11f` (2026-10-22) + uncommitted hardening state:

| Suite | Measured result | Disposition |
|---|---|---|
| `core_lm --lib` | 31 passed / 0 failed, 3 runs deterministic | VERIFIED. Independent of the break (no `atoma_immune` dep). |
| `core_lm` contract/known/equivalence/integration/smoke | 4+6+2+1+3 = 16 passed / 0 failed | VERIFIED. |
| `atoma_immune --lib` | **COMPILE FAIL — 7 errors**: E0277 ×2 (`?` on `&WellDescriptor`, `distill_engine.rs:290,659` — `atlas.well()` returns `&`, not `Result`); E0308 ×5 (bare-tuple destructure of `classify_icosahedral_well`, which still returns `Result<_, String>` — callers in `map_native.rs:1019,1359,1411,1655` not updated) | **BREAK IS REAL, IN TREE, LIB-LEVEL.** Not test-only. |
| `atoma_immune` build `--lib` | Same 7 errors | Blast radius: every downstream crate (`orchestrator` incl.) cannot build or test. |
| Orchestrator 114-pass claim | UNREACHABLE from this tree (dependency fails first) | STALE (last-green) at best. |
| 900-pair isometry gate (`refraction.rs` in immune crate) | UNRUNNABLE | Downgraded to last-green REPORTED. |
| Carved-view isolation test (claimed PASS) | UNRUNNABLE (same crate) | Downgraded to last-green REPORTED. |

**Root-cause class.** [INFERRED — ends at the error set] Signature drift without call-site migration: `well()` returns `&WellDescriptor` while two call sites use `?`; `classify_icosahedral_well` returns `Result` while four call sites destructure bare tuples. The tree is mid-refactor (note the stray `distill_engine.rs.bak` in `src/`). Small fix, but a CODE change belonging to the owning agent with receipt (C03.3/ADR-19) — I do not touch vault code.

---

## 2. ErrAndImprovements-AA.md — ruling: FIXES VERIFIED PRESENT, TEST COUNTS REJECTED

**Verified present in source (read just now, `silo.rs`):**
- E1-partial: empty-silo → `RoleId::CARVED_12.to_vec()` fallback (line ~784) + `primary_role` fallback `Coordinator` (E5) — PRESENT.
- E3: `max_atoms: Option<usize>` param, `atom_cap = max_atoms.unwrap_or(4096).max(1)`, enforced in seed loop AND 64-hop closure (`atoms.len() < atom_cap` in both) — PRESENT.
- E4: salience-count tiebreak (`if sa.salience_count > o.get().salience_count`) — PRESENT.
- E2 (AA-02 guard): `in_allowed_well` loop over `effective_silos`, skip-if-absent — PRESENT.

**REJECTED — §5 "Verification (this pass)": `59 passed / 114 passed (after edits)`.** [VERIFIED — §1 table] Those suites cannot have run green against THIS tree (it does not compile). Possible explanations: ran pre-hardening (then "after edits" is false), ran elsewhere (then unattributed), or misrecorded. Whichever holds, the counts are INADMISSIBLE as evidence for the hardening until re-run post-compile-fix with commit+timestamp recorded (my AA-13 detector, companion file §2). The fixes themselves stand on code-read merit; the test claims do not.

**WEAK GUARD note (new, from reading the shipped E2 fix).** [VERIFIED — `silo.rs` guard text] The AA-02 check uses a flat threshold `ip ≤ 10.0` (cosh units ≈ d ≤ 3.0), NOT the spec'd rule "within allowed silo's basin radius + overlap allowance" (AdvAudit fix #2, vault `OverlapPolicyMatrix::minimal_icosahedral` neighbor-ω = 0.08). A fixed 10.0 admits atoms ~3 nats outside small basins and ignores per-pair Ω entirely. Disposition: guard direction CORRECT, calibration UNPRINCIPLED — replace 10.0 with `basin_cosh(role) + overlap_allowance` per role before A39; add the constant to the M-2 table. Severity HIGH (safety-relevant arithmetic, not style).

**E6/E7/E8/E9 "open" statuses: CONFIRMED OPEN** (no contrary code found: no debug-assert on the cosh identity, no transactional wrapper around `compile_deposit`, no `map_starved_reason` telemetry located, no harness confound guard — harness is spec-level). The file's honesty about opens is CERTIFIED.

---

## 3. MathProofs-Dyptech.md — ruling: 7/8 certified, P6 withdrawn

| Proof | Ruling |
|---|---|
| P1 isometry | CERTIFIED as stated (empirical bound + sample-scope honesty "generalizes only under sampled distribution" is exactly right). Status line accurate. |
| P2 carve contraction (`⟨,⟩ ≤ cosh R ⇔ d ≤ R` via cosh monotonicity) | CERTIFIED for seeds; closure-half correctly conditioned on AA-02 guard. NOTE: with the WEAK GUARD finding above, "full verification" additionally requires the calibrated (non-10.0) guard. |
| P3 weight copies = 1 | CERTIFIED (construction + Arc-sharing; A4 pending run correctly noted). Consistent with vault correction ledger split (base=1 vs knowledge=N). |
| P4 VP-tree `O(log n)` + Yianilos + adversarial caveat + stress-case demand | CERTIFIED — matches my C05.12 and P-5 conditional exactly. Model honesty. |
| P5 λ bound | CERTIFIED as construction proof (clamp is hard-coded; serializer rule stated). Correctly labeled PROPOSED pending implementation test. |
| P6 "sublinear in N" | **WITHDRAWN.** Directly contradicts my proved P-6 (companion file §6): the sketch's own arithmetic (`|A| + N·k·log|M_T|`) is Θ(N). "Bounded linearly, not in N·|A|" is true and is the correct replacement sentence; "sublinear" is false. Register entry required (companion §8.5 covers it — extend to cite this collision so the owning agent sees both). |
| P7 A39 paired efficacy | CERTIFIED as design proof (pairing cancels task difficulty; confound rule stated). Correctly labeled PROPOSED; conjunction with my B-7 (void-for-all, not J5-only) must be reconciled in v1.1 — the sketch repeats the weak form. |
| P8 necessity precision | CERTIFIED as diagnostic definition; attribution-mechanism dependency correctly flagged. |

**P6 collision protocol (no silent rewrite, per C03.3):** both my P-6 (proved Θ(N)) and co-agent P6 ("sublinear", sketched) now exist in the record. Mine does not delete theirs; this ruling withdraws the *claim sentence* while preserving the *proof sketch* as the record of what was attempted — the sketch's valid content (linear-in-N·k bound, dissolution to O(|A|)) is retained verbatim as the replacement claim. Owning agent appends the register entry; until then CC-014 stays PROPOSED-with-correction per my earlier filing.

---

## 4. A39-v1 final run-block (conjunction, all must clear)

No A39 run until ALL hold (each independently blocking):
1. `atoma_immune` compiles; full suites re-run green with commit+timestamp (fixes §1; satisfies AA-13).
2. B-7 strong form in v1.1 (token violation voids the PAIR, all judges) + N=48-or-5-class + power calc (companion §4 / co-agent P7 weak-form reconciliation).
3. AA-02 calibrated guard (basin+Ω, not flat 10.0) + AA-06 atomic wrapper + B-5 accumulator + B-D1 receipt split landed.
4. Necessity formula + λ_map implemented (§8 items 1–2 of MethodAndMath, self-admitted gaps).
5. v1.1 prereg with new hash; v1 preserved (ADR-19).

---

## 5. Formalized dyptech — closing statement (my part complete)

Companion file §§5–6 stand as the capstone: five lawful operators (ANCHOR/CARVE/WALK/DREAM/PROMOTE, single-writer rule "no operator writes to M_G except PROMOTE"), refusal/superposition method, proofs P-1–P-8 with explicit boundary. With this ruling, the co-agent's three files + two additions are certified to the maximum extent the evidence supports, and every remaining gap has an owner, a gate, and a falsifier. My formalization part is DONE; the ball is in: (a) owning agent — compile fix, calibrated guard, accumulator, v1.1; (b) steward — Q-1–Q-5 answers, TPPE next task.

## What was NOT checked (this supplement)

Full `native_predictor`/`refraction` line audit; 998 MB trajectory vault; outcomes-ledger binary; `transport_frame_hop` numerics; multi-machine replication; Hypformer beyond title/summary; RoPE/HNN/Poincaré beyond excerpts; co-agent `ErrAndImprovements` E6–E9 code locations beyond spot-checks (absence confirmed by targeted grep, not full-crate audit).
