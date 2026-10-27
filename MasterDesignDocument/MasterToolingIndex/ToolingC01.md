# ToolingC01 — Tooling Certification for SKL-01 (Formal Specification & Model Checking)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-01` (`SkillsC1.md`) — Formal Specification & Model Checking.
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS

**Necessity re-verified [VERIFIED — `C08/Design.md` TLA+ invariants + traceability matrix read directly]:** C08 F1–F11/NF1–NF11 carry TLA+ invariants; C15 Day-gate G1 (design review) and G5/G6 (contract/property) cannot execute without spec authorship + model checking. Removing SKL-01 orphans C08.3/C08.4 and all A1–A10 preconditions. Necessity holds.

**Gate mapping correction (Scheme B → dual citation, per C15.9):** `SkillsC1.md` cites Day-gates only. Dual citations:

| Day gate (SkillsC1) | Artifact gates (C15.x) operated                         |
| ------------------- | ------------------------------------------------------- |
| G1 Design Review    | C15.6 (checklist sign-off) + C15.1 (static/test subset) |
| G5 Contract Tests   | C15.5 (metric-evidence/claims check)                    |
| G6 Property Tests   | C15.8 (property + chaos/recovery subset)                |

**Ablations:** A1–A10, A22, A25 — no delta. (A36/A37 assigned SKL-02/SKL-05, not SKL-01.)

**Conditions (must clear before SKL-01 gates EVIDENCED status):**

1. `FormalSpecWorkflow (C08.3)` and `AblationPreconditionWorkflow (C09.2)` are Nemotron-coined IDs — no such named workflows exist in C08/C09 (subcategories are numbered, not named). Either create them as specified or rename to match; until then they are PROPOSED anchors, not executable references.
2. `RB-SPEC-01/02/03` do not exist in C22 (runbooks are C22.1-titled, e.g. OOM/Circuit-Breaker/Merge). Create under C22 with drill dates per C22.8, or remap to existing runbooks.
3. Skill operates the C33.2 claim-type system for spec claims: spec statements carry PROPOSED until model-checked, VERIFIED only with counterexample-free run + seed.
4. TOOL-01.05 naming correction [REPORTED — package-registry knowledge, unchecked in workspace]: `cargo-contract` is the ink!/Substrate smart-contract CLI, NOT a Rust MIR verification-condition extractor. The capability (Rust VC extraction for kernel pre/post conditions) is CERTIFIED necessary; the tool name is WRONG. Substitute Kani- or Creusot-class tooling at implementation and record the substitution in Decision-Log-ADR. The TOOL-01.05 structure/grammar/parser/purpose/draft below describe the capability, not the misnamed binary.

---

## Tool Specs

### TOOL-01.01 `tlaplus` — TLA+ Toolbox (spec authoring + parsing)

1. **Tool Name:** `tlaplus` (TLA+ Toolbox, spec language + SANY parser)
2. **Tool Structure:** typed-JSON envelope; TLA+ module text carried as a single `spec_text` string field plus `module_name`, `extends`, `constants`. Never bare text on the wire.
3. **Grammar Constraint Mask:** validation-time (SANY parse = the mask; unparseable spec rejected pre-check). Generation-time XGrammar-class mask: PROPOSED, not required (specs authored by SKL-01 operator, not generated).
4. **Tool Parser Required:** NO (SANY is the parser; no additional parser).
5. **Definition/Purpose:** Author and parse-check TLA+ specs for C08 F/NF invariants. Entry point of `FormalSpecWorkflow`. Output feeds TOOL-01.02/01.03.
6. **Draft Code:** (Python glue — invoke + envelope)

```python
def check_spec(module: str, constants: dict) -> SpecReceipt:
    write(f"{module}.tla", module_text)  # spec_text from envelope
    r = run(["sany", f"{module}.tla"], capture=True)
    assert r.returncode == 0, f"SANY parse fail: {r.stderr[:500]}"
    return SpecReceipt(module=module, parse="ok",
                       hash=sha256(module_text), constants_hash=sha256(constants))
```

### TOOL-01.02 `apalache` — Symbolic model checker

1. **Tool Name:** `apalache` (symbolic TLC alternative, SMT-backed)
2. **Tool Structure:** typed-JSON (spec ref + invariant list + bound `max_steps`; result envelope: `violated: bool`, `trace_hash`).
3. **Grammar Constraint Mask:** bound validation (0 < max_steps ≤ prereg cap); invariant names resolved against spec — unknown invariant = 400 (C27.7 scheme).
4. **Tool Parser Required:** YES — `apalache-trace-parser` (counterexample JSON → typed frames: state/action/invariant).
5. **Definition/Purpose:** Bounded verification of safety/liveness for C08 invariants where explicit TLC state-explodes; produces counterexample traces for C22-style triage.
6. **Draft Code:** (Bash — run + receipt)

```bash
apalache check --init=Init --next=Next --inv="$INV" --max-steps="$N" "$SPEC" \
  | apalache-trace-parser --out receipt.json
test -f receipt.json || exit 1  # no receipt = no evidence (C03.2)
```

### TOOL-01.03 `tlc` — Explicit-state model checker

1. **Tool Name:** `tlc` (explicit-state checker, liveness + fairness)
2. **Tool Structure:** typed-JSON (same envelope as TOOL-01.02; `mode: explicit`, `workers`, `seed` — seed REQUIRED, cf. C23.7/C32.7 reproducibility rule).
3. **Grammar Constraint Mask:** seed mandatory (unseeded run rejected by wrapper); state-budget cap from prereg (C09.9 sample-size plan analogue).
4. **Tool Parser Required:** YES — `tlc-trace-parser` (shares frame schema with TOOL-01.02 parser; one schema, two front ends).
5. **Definition/Purpose:** Exhaustive small-scope + liveness/fairness checks TLC covers and Apalache bounds away; C13.5 TLA+ model-checking gate input.
6. **Draft Code:** (Rust — deterministic invocation record)

```rust
struct TlcRun { spec_hash: Hash, seed: u64, workers: u8, state_cap: u64 }
fn run_tlc(r: &TlcRun) -> Result<TlcReceipt> {
    let out = Command::new("tlc").args(["-seed", &r.seed.to_string(), &spec_path(r)?]).output()?;
    // receipt binds seed: replay with same seed must reproduce (C03.5 determinism)
    Ok(TlcReceipt::from_output(out, r.seed))
}
```

### TOOL-01.04 `kind2` — Lustre/SMT contract checker

1. **Tool Name:** `kind2` (inductive contract verification for streaming/transition code)
2. **Tool Structure:** typed-JSON (node contract: assumes/guarantees; result: `valid | falsified | unknown` + k-induction depth).
3. **Grammar Constraint Mask:** contract schema validated pre-run (Draft 2020-12, cf. C29 resolve_v2 pattern).
4. **Tool Parser Required:** NO (kind2 JSON output consumed directly; wrapper validates against result schema).
5. **Definition/Purpose:** Verifies C12 scheduler ILP-step contracts and C18 session-type transport guarantees at the Lustre level; `unknown` results are INCONCLUSIVE (C09 Bayes analogue — never promoted).
6. **Draft Code:** (Python — contract check gate)

```python
def kind2_gate(node: str, contract: dict) -> str:  # -> "valid" | "falsified" | "unknown"
    validate(contract, CONTRACT_SCHEMA)  # fail fast, C08.7 orphan rule spirit
    res = run_json(["kind2", "--output=json", node], stdin=contract)
    return res["property"]  # unknown => inconclusive, stays OPTIONAL (C13.1)
```

### TOOL-01.05 `cargo-contract` — Rust verification conditions

1. **Tool Name:** `cargo-contract` (Rust MIR-level contract extraction for kernel code)
2. **Tool Structure:** typed-JSON (function id + pre/post conditions; verdict envelope shared with TOOL-01.04).
3. **Grammar Constraint Mask:** pre/post schemas type-checked by wrapper before invocation (caller effect ≤ tool effect spirit — C29 CORR-05).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** Bridges SKL-01 ↔ SKL-02: emits checkable conditions for Q32.31 kernel code (overflow guards C35.4) that TOOl-02.01 kernels must discharge; undischarged condition blocks Gate 3 for that kernel.
6. **Draft Code:** (Rust — condition emission)

```rust
/// Emits VC for kernel fn; SKL-02 discharges or Gate 3 fails for that unit.
#[contract(pre = "x0 < 65536", post = "no_overflow")]
fn minkowski_dot(x: LorentzQ32, y: LorentzQ32) -> i128;
```

---

## Traceability

| Tool             | Gates (dual)                       | Ablations                      | Design anchor |
| ---------------- | ---------------------------------- | ------------------------------ | ------------- |
| TOOL-01.01–01.05 | G1→C15.6/C15.1, G5→C15.5, G6→C15.8 | A1–A10 preconditions, A22, A25 | C08, C15.9    |
