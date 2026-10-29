# Distillation v1 vs v2+Depth — Formalization, Pressure Test, and Proofs

**Date.** 2026-10-28. **Status.** PROPOSED formalization + VERIFIED code facts + executed math (script `/tmp/distill_depth_proof.py`, rerunnable).
**Scope.** What the Distillation protocol (v1) IS, what v2+depth ADDS, and proof that depth buys a continuous single pass with no fine-tuning.
**Sources pinned.** Local: `DistillationProtocol/{PROTOCOL,AUDIT,CONVERSION_MATH}.md` @ `2035c39`.
Vault (read-only, outer-repo paths per global access §8): `monster-hybrid-server/immune_system/implementation/src/{distill_engine,silo,refraction}.rs`,
`docs/BREATHING_DEPTH_DESIGN.md`, `papers/{paper1-weightless-inference,remembering-instead-of-recomputing}.md`,
`design/{live_brain.atoma_map,glimmer_52l_map_native.atoma_map}`.
**Standards.** Documentation-standards (§7 personal AGENTS.md): every claim labeled; corrections appended, never rewritten.

---

## 0. Terminology lock (read first — there is no file literally named "v2")

- **v1** = the audited two-pillar engine: `distill_static_weight_modes` (uniform-stride scan, unconditional count)
  + `distill_reasoning_chain` (entropy-sieved trajectory arcs), as specified in `PROTOCOL.md` §§2–3 and pressure-tested in `AUDIT.md` (C-AUD-0/2/5). [VERIFIED — symbols at `distill_engine.rs:227,343`]
- **v2+depth** = the depth-aware upgrade stack IN THE SAME ENGINE [INFERRED from code — ends at the symbol list in §2;
  it is a capability level, not a separate file]: `distill_quantized_layer_zero_copy` (`:530`, analytic spectral sieve + greedy top-k + Fisher bit-width priority),
  `distill_live_step_novel_only` (`:1997`, chained continuous pass, true novelty bools),
  `distill_safetensors_{shard_bytes,file_novel_only}` (`:974,:1134`, full-model novelty-metered pass),
  with layer-depth-band routing already inside `classify_icosahedral_well` (`:121`, depth bands `:157–200`).
- **Depth** = salience-stratified budget allocation across layer bands (spend where the teacher's reasoning lives),
  NOT "more layers scanned." This definition is what Theorem 3 proves.

---

## 1. What distillation ACTUALLY does (formalized method — VERIFIED mechanics)

Every entry point applies only these updates (no other write path exists in the distill surface):

```
M' = M ∪ { mint(project(content)) }   (insert novel atom/arc)
M' = M[salience += 1]                 (reinforce on coalescence, silo.rs:1270-1280)
W' = W                                (teacher weights: shared ref, never written)
```

Concretely, per teacher element:

1. **Tap.** Read frozen weights (`&FrozenBaseWeights`, `distill_engine.rs:229` — no `&mut` exists anywhere in the file [VERIFIED by grep this pass]) or a trajectory step `(hidden_state, next_op_probs, layer_index)`.
2. **Project.** `TangentProjector::project` → tangent + `retained_energy_ratio` + `teacher_entropy_bits`.
3. **Route.** `classify_icosahedral_well(atlas, hidden_state, layer_index, total_layers, entropy)` → role + radial offset + certainty `1/(1+H)`. Signature takes NO teacher-identity argument [VERIFIED `:121–126`]. Depth enters ONLY as `depth_ratio = layer/total` band bonus (+0.35 to the in-band roles, `:157–200`) — position-in-architecture, not teacher identity.
4. **Refract + carve.** `OpticalLens::refract` → `keep_and_carve_novel_only` → coalesce at `cosh(d) ≤ 1.0000001` within the 27-bucket + same invariant class (`silo.rs:1206–1290`), else insert; arcs via `record_novel_reasoning_arc` / `record_trajectory_arc` (`silo.rs:1325,1357`) carrying Wigner frames + compiled MLIR ops; closed role loops evaluated by Gauss–Bonnet holonomy → salience reinforcement on `DialecticalSynthesisVerified`.
5. **Efficacy mechanism (PROPOSED magnitude, VERIFIED mechanism).** Capability lift is the information term, not a parameter change:
   `ΔCap = E[JudgePass(T,π_M) − JudgePass(T,π_0)] + η·EffGain` (`TrajectoryToCapabilityFormalization` §4),
   driven by `Cap(T) = Cap0(T) + α·cov(T)·R̄` (`CONVERSION_MATH.md` §2) — coverage of required sub-problems by carved wells/arcs times mean retained energy. Retrieval replaces recomputation on covered ground; the base weights are unchanged.

---

## 2. v1 vs v2+depth — operator table (all code refs VERIFIED this pass)

| # | v1 behavior | v2+depth behavior | Code anchor |
|---|---|---|---|
| 1 | Pillar 1 scans with uniform stride `chunk = (mode_idx·stride) % total` (`:246`), layer swept as `(mode_idx·L)/max_modes` (`:251`) — budget share follows SCAN ORDER, not salience | Analytic sieve scores ALL rows first (`E_est(r) = out_scale[r]²·Σ_g G·((q_mean·s+b)²+q_var·s²)`, `:522,550–567`), sorts descending, dequantizes ONLY top-k (`:574–591`); Fisher bit-width priority boosts sensitive layers' certainty → deeper refract (`:527–528,:644`) | `distill_engine.rs:227–341` vs `:530–695` |
| 2 | `minted += 1` unconditional (`:312`); `keep_and_carve_with_origin` discards the novelty bool (`.0`, `silo.rs:1315`) → counted ≡ scanned ALWAYS | Novelty bool threaded end-to-end: `LiveNoveltyDistillationReceipt{atom_was_novel, arc_was_novel}` (`:2124`); `FullModelDistillationReceipt` splits `novel_atoms_minted` vs `deduplicated_atoms_coalesced` (`:2107`) | `distill_engine.rs:300–312,1997–2110` |
| 3 | Pillar 2 sieves entropy (`:368`, `H > 2.75 bits` rejected); Pillar 1 has NO entropy sieve | Same sieve on the live path (`:2011`); unchanged threshold, wider application | `:95–104,:368,:2011` |
| 4 | One-shot batch calls; no cross-step chaining | `distill_live_step_novel_only(tap, …, previous_atom_and_role, …)` threads `(prev_atom, prev_role)` → chains single pass preserving cross-layer order; repeats coalesce → pass is idempotent | `:1997–2085` |
| 5 | Judge gate: ABSENT (0 `judge` refs in `distill_engine.rs` + `refraction.rs` [VERIFIED by grep]) | Judge gate: STILL ABSENT — v2 does not land finding 4. `Cap(T)`'s `R̄`/`p_useful` remain unmeasurable; A39 pending | grep `judge` = 0 (both files) |
| 6 | Deterministic by index arithmetic (0 RNG hits [VERIFIED]; BTreeMap carve order) | Same determinism preserved (stable-tied sorts, BTreeMaps at `:879,:1190,:1914`) | grep `rand\|rng\|random` = 0 |

---

## 3. Pressure-test results (executed 2026-10-28, all VERIFIED by direct grep/count)

- **No-gradient invariant.** 0 hits for `backward|.grad|optimizer|adam|sgd|fine-tun` in `distill_engine.rs`; 0 `&mut`-weights signatures; weights enter as `teacher_weights: &FrozenBaseWeights` (`:229`). Consequence: `dW = 0` identically across every entry point — fine-tuning is impossible BY CONSTRUCTION, not by policy.
- **Determinism.** 0 RNG hits; carve/aggregation paths use `BTreeMap`; top-k sorts stable-tied (AUDIT C-AUD-5, stands).
- **Novelty truth.** `keep_and_carve_novel_only` returns `(AtomId, bool)` with coalescence at `cosh ≤ 1.0000001` (`silo.rs:1226`); `with_origin` discards the bool (`silo.rs:1315`); Pillar-1 `minted += 1` unconditional (`:312`). v1 inflation is MECHANICAL, not hypothetical (Theorem 2).
- **MGS gap (carried).** 3 `mgs|orthogonal` mentions in engine, none on Pillar-1 path (AUDIT finding 3 CONFIRMED, unfixed).
- **Judge gap (carried).** 0 `judge` refs — PROTOCOL §2-step-6 unenforced in code; `CONVERSION_MATH.md` validity condition already rewritten to the interim structural gate (`knowledge_atoms_minted == modes_scanned` AND role-distribution sums, AUDIT §7).
- **Build.** C-AUD-0 stands: lib + orchestrator green (66 + 114 passed at audit time; not re-run this pass — stated, not re-claimed).

---

## 4. Proofs (executed: `python3 /tmp/distill_depth_proof.py` — all outputs VERIFIED from that run)

**Theorem 1 — greedy top-k dominates stride sampling (expected retained energy per budget).**
Spectrum Zipf `E_i ∝ 1/i^s`, budget k of N. Stride ≡ uniform sample → `E[mean] = μ`. Greedy takes the k largest order statistics, each ≥ μ unless the spectrum is degenerate-constant. Measured: N=1000/k=64/s=1.0 → stride 0.0075 vs greedy 0.0741 (**9.90×**); N=4096/k=128 → **19.55×**; s=0.7 → 5.84×; s=1.2/N=256 → 6.29×. The AUDIT's independent probe (0.0204 vs 0.0741 = 3.63×) is the same inequality on a milder skew. Strict gain ⟺ non-constant spectrum — the normal case for weight-mode spectra. ∎

**Theorem 2 — v1 inflates its count; v2's novelty bool removes inflation.**
`counted_1 = modes_scanned` always; `true_novel_1 = Σ novel_bool ≤ counted_1`. Inflation `I = 1/(1−dup_rate)`: 1.11× at 10% dups, 1.43× at 30%, 2.00× at 50%, 3.33× at 70%. v2 receipts expose the bool per atom/arc → `I = 1.00` observable. The volume floor (8,644 atoms / 104,312 arcs in `live_brain.atoma_map` [REPORTED — PROTOCOL §7]) is therefore only meaningful under v2 accounting; v1 counts may include swamp. ∎

**Theorem 3 — depth: salience-stratified allocation dominates uniform sweep (THE DEPTH PROOF).**
L=52, 3 bands, budget B=120, mid-heavy teacher (bands 0.2/0.5/0.3): uniform-sweep retained 2.22 vs salience-proportional 2.60 (**1.17×**). Exchange argument: move one budget unit from layer i to layer j with `E_j > E_i`; retained changes by `(E_j − E_i) > 0`; repeat to the water-filling fixed point. v1's uniform sweep is the START of this process; v2's analytic sieve + Fisher priority is a step toward the fixed point. Strict gain ⟺ band energies differ — the normal case (mid/late layers carry reasoning; early layers carry sensory/symbolic per the depth-band prior `:160–200`). Depth = budget spent where reasoning lives. ∎

**Theorem 4 — continuous single pass requires no fine-tuning (no-gradient invariant).**
Every entry point's update is `M ∪ {mint(project(content))}` or `salience += 1`, with W read through `&` and never written (§3 greps). Hence `dW = 0` ⟹ no gradient exists ⟹ no fine-tuning occurs BY CONSTRUCTION. One pass suffices: each teacher element is read once (O(h) readers, no O(h²) updates); re-reading is idempotent because the novelty gate returns `(id, False)` on repeats (Theorem 2). Capability lift, when measured, is attributable to `I(answer; M_T | q)` — an external latent the policy conditions on — never to parameter change. The no-finetune claim is PROVED regardless of A39; the capability MAGNITUDE claim is NOT (needs the judge gate + A39). ∎

**Theorem 5 — teacher-invariance survives v2's depth signals.**
v2's depth inputs (layer index, `depth_ratio` band bonus, bit-width/Fisher priority, quant stats) are functions of CONTENT + POSITION-IN-ARCHITECTURE. `classify_icosahedral_well` takes no teacher-identity parameter [VERIFIED signature `:121–126`]; fingerprints live only in the lineage table, the `origin_model_fingerprint` audit field, and receipts. Twin-fingerprint test (same content under fp_A vs fp_B → byte-identical coordinates, single-atom coalescence) is PRESCRIBED (`PROTOCOL.md` §8) but not yet receipted — proof conditional on that run. ∎

---

## 5. Verdict (pressure-tested, labeled)

- **v1 DOES** [VERIFIED]: frozen-read extraction → project → depth-band route → refract → carve → coalesce → arcs (Wigner + MLIR ops) → holonomy-gated salience. Evidence: audited symbols + green suite at audit time + paper1's 1,144-atom two-teacher map (Glimmer 576 + Qwen-35B 568 atoms, 12/12 silos, 158 µs query [REPORTED — paper1 appendix 1.b/1.c/1.e]).
- **v1 DOES NOT do** [VERIFIED]: novelty accounting (inflates per Theorem 2), salience-ordered selection (loses per Theorem 1), depth-stratified budget (loses per Theorem 3), judge gating (0 refs), MGS on Pillar 1.
- **v2+depth ADDS — PROVED**: strictly more retained energy per budget (Thm 1 + Thm 3), metered novelty making the volume floor meaningful (Thm 2), chained continuous pass preserving cross-layer order (§2-row-4), all with `dW = 0` so the pass is fine-tune-free by construction (Thm 4), and teacher-invariant coordinates preserved (Thm 5, test pending).
- **Efficacy, honestly bounded**: the MECHANISM (retrieval replaces recomputation on covered ground) is proved from code + math; the MAGNITUDE (`ΔCap > 0`, "30B-class on covered ground") remains PROPOSED — blocked on finding 4 (judge gate) + A39. Paper1's 16× figure is a kernel speedup with exact-match output, NOT layer-skipping [REPORTED with paper2-§9's own correction: mixing those instruments was wrong].
- **"Depth allows a continuous pass with no fine-tuning" — PROVED** in the precise sense: depth-stratified salience makes ONE read per teacher element sufficient (nothing is left behind that a second pass would find — repeats coalesce), and the pass contains zero gradient-carrying operations, so chaining it continuously (live-step threading) cannot become fine-tuning by accumulation.

---

## 6. What was NOT checked (explicit — do not claim beyond this)

1. A39/M-CAP runs (magnitude unmeasured); judge gate unlanded; MGS-on-Pillar-1 unverified in code beyond grep.
2. Twin-fingerprint invariance test prescribed but unrunned — Theorem 5 conditional.
3. Spectral-skew numbers are Zipf models + the AUDIT's probe, not a fresh teacher-checkpoint measurement.
4. Suite green-ness not re-run this pass (carried from C-AUD-0, dated 2026-10-27).
5. `live_brain` floor counts REPORTED from PROTOCOL §7, not re-censused from the map file this pass.
6. Correction log: none yet — no prior claim in THIS file has been superseded. (AUDIT.md's C-AUD-0/2/5 corrections to the earlier audit stand and are incorporated above.)
