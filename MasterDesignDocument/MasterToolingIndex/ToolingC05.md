# ToolingC05 — Tooling Certification for SKL-05 (Verification & Ablation Framework)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-05` (`SkillsC1.md`) — Verification & Ablation Framework Engineering.
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS (meta-skill — stricter bar)

**Necessity re-verified [VERIFIED — `C09/C13/C15/C23` read directly]:** SKL-05 operates the gates (C15.9 dual scheme), runs ablations A1–A37 (C06.15/A36 + C06.16/A37 included — the SkillsC1 "A1–A35 (ALL)" line is stale by two ablations), and owns the Monte Carlo go/no-go (ADR-20). Removing it collapses all evidence. Necessity holds absolutely — but as the meta-skill it is held to self-application (C33.7 principle): the framework must satisfy its own rules.

**Gate mapping correction (operates ALL Day gates G1–G8 → ALL artifact gates C15.1–C15.8):** no delta in coverage; citation form changes to dual per C15.9.

**Ablations:** A1–A37 (SkillsC1 text says A1–A35 — CORRECTED to include A36 depth-gated speculative recall and A37 judge-drift monitor, preregs filed in `PRE-REGISTRATION-A11-A35.md`).

**Conditions:**

1. A36/A37 prereg TEXT is filed (done in this pass) but HASHES are empty — per C11.3/ADR-22 no A36/A37 run may execute until hashes are filed. The framework cannot exempt its own newest ablations.
2. The framework's own tools below are THEMSELVES subject to Gate 2 (static) + Gate 6 (property): `verify_metric_evidence.py` with a bug is a miscalibrated judge (C06.16's exact threat). TOOL-05.05 (judge-drift monitor operator) watches the framework's own judges — self-monitoring is mandatory, not recursive paradox: the monitor's OWN band is set by manual Day-1 review, bootstrapping the chain.
3. Monte Carlo MUST run at Day 1–2 exit with recorded seed (C32.7 item 5) — a framework that defers its own schedule validation to Day 10 cannot gate the sprint it schedules.
4. `R` in the tool list: any R code in the evidence path must be pinned (`renv.lock` analogue) or replaced with the Python path — unpinned R breaks ADR-23 reproducibility for Bayes-factor computations.

---

## Tool Specs

### TOOL-05.01 `prereg-file` — Preregistration hash filer (C11.3 enforcer)

1. **Tool Name:** `prereg-file` (hash filing + content-checklist audit per C09.9)
2. **Tool Structure:** typed-JSON (ablation id + prereg text + 8-item checklist verdicts in; `filed_hash` + `checklist_pass: bool` out).
3. **Grammar Constraint Mask:** C09.9 checklist is MANDATORY fields (hypothesis-falsifiable, workload, hardware, baseline, judge-band, effect-size, null-handling, claims-draft) — missing field = filing REJECTED (hash-without-content fails Gate 1 audit per C09.9).
4. **Tool Parser Required:** NO (checklist is structural validation, not language parsing).
5. **Definition/Purpose:** The C11.3 integrity gate made executable: no filed hash + passing checklist, no run. Null/negative results filed as first-class entries (same tool, `outcome: null` path).
6. **Draft Code:** (Python — filing gate)

```python
def file_prereg(ablation: str, text: str) -> str:
    verdicts = audit_checklist(text)  # 8 items, C09.9
    assert all(verdicts.values()), f"checklist fail: {[k for k,v in verdicts.items() if not v]}"
    h = sha256(text); record(ablation, h, verdicts)
    return h  # run without this hash => disqualified (C11.3)
```

### TOOL-05.02 `ablation-run` — Baseline/treatment executor + judge binder

1. **Tool Name:** `ablation-run` (workload runner binding baseline + treatment + judge in one receipted execution)
2. **Tool Structure:** typed-JSON (prereg_hash + workload id + hardware ids + seeds in; `baseline_metrics`, `treatment_metrics`, `judge_verdict`, `effect_size`, `ci_95` out — the C09 claims-register evidence shape).
3. **Grammar Constraint Mask:** prereg_hash must RESOLVE to a filed entry (TOOL-05.01 output) — unfiled hash = run REFUSED pre-execution, not flagged post-hoc.
4. **Tool Parser Required:** YES — `workload-parser` (corpus manifest → typed prompt list with seed binding; unmanifested prompt in workload = 400-class rejection).
5. **Definition/Purpose:** Executes A1–A37 runs including group-sequential interim analyses (Lan-DeMets α-spending, ρ=2 per C09) with futility stopping at conditional power < 0.2.
6. **Draft Code:** (Bash — gated run)

```bash
prereg-file --verify "$HASH" || exit 1        # C11.3: no hash, no run
ablation-run --hash "$HASH" --workload A1_corpus --seed "$SEED" \
  --interim 0.25,0.5,0.75 --futility 0.2 \
  | judge-bind --out claims_evidence.json
```

### TOOL-05.03 `gate-exec` — Day-gate/artifact-gate executor (dual citation)

1. **Tool Name:** `gate-exec` (`cargo xtask gateN` family + C15.x artifact verifiers, dual-cited per C15.9)
2. **Tool Structure:** typed-JSON (Day gate G + artifact gate C15.x + category scope in; `pass|fail|skipped(reason,owner,remediation_date)` out — SKIPPED is a first-class outcome per C15.10, never silent).
3. **Grammar Constraint Mask:** skip protocol enforced (C15.10): downstream gates of a SKIPPED gate inherit SKIPPED — the tool REFUSES a PASS verdict while any upstream is SKIPPED (no leapfrogging by construction).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** Operates the G1–G8 schedule against C15.1–C15.8 evidence; produces the Day-10 go/no-go input (any REQUIRED SKIPPED = no-go).
6. **Draft Code:** (Rust — no-leapfrog executor)

```rust
fn exec_gate(g: DayGate, scope: Scope) -> GateOutcome {
    for up in upstream(g) { if status(up) == Skipped { return inherit_skip(up); } }
    run_artifacts(g.artifacts(scope))  // C15.x verifiers
}
```

### TOOL-05.04 `monte-carlo-sched` — PERT schedule simulator (seeded)

1. **Tool Name:** `monte-carlo-sched` (10,000-run Beta-PERT simulation over the C04.10 DAG — max-joins, not sums, per C23.7)
2. **Tool Structure:** typed-JSON (task PERT params + DAG edges + seed in; `P10/P50/P90`, `P(finish≤10d)`, `critical_path` out).
3. **Grammar Constraint Mask:** seed MANDATORY (unseeded run rejected — C03.5/C23.7); DAG-structured (sum-total retained as documented upper bound only).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** ADR-20 go/no-go rule input (P90 ≤ 10 days AND P ≤ 10d ≥ 80%); runs at Day 1–2 exit (C32.7), not Day 10.
6. **Draft Code:** (Python — DAG-aware simulation)

```python
def simulate(tasks: dict, dag: DAG, seed: int, n: int = 10000) -> Schedule:
    rng = random.Random(seed)  # recorded seed, reproducible (C03.5)
    totals = [makespan({t: pert(rng, *p) for t, p in tasks.items()}, dag) for _ in range(n)]
    # makespan uses max() at joins per C23.7 -- sum() retained as upper_bound only
    return Schedule(p10=pct(totals,10), p50=pct(totals,50), p90=pct(totals,90))
```

### TOOL-05.05 `judge-drift-monitor` — C06.16/A37 operator (framework self-watch)

1. **Tool Name:** `judge-drift-monitor` (sequential Lan-DeMets monitor over judge verdict pass-rates)
2. **Tool Structure:** typed-JSON (judge id + prereg band `[l,u]` + verdict stream in; `JudgeMiscalibrated` + quarantine list out on breach).
3. **Grammar Constraint Mask:** band REQUIRED from prereg (unbanded judge cannot be monitored — monitoring starts at preregistration, C06.16 design).
4. **Tool Parser Required:** YES — `verdict-stream-parser` (verdict log → typed `(pass: bool, confidence)` pairs; malformed verdict records quarantined as parser incidents, not dropped).
5. **Definition/Purpose:** Closes the evidence loop: ablations earn claims, this tool un-earns them when judges rot; ALSO watches the framework's own judges (TOOL-05.02 verdicts, TOOL-05.03 gate outcomes) — self-application per condition 2.
6. **Draft Code:** (Python — sequential boundary)

```python
def watch(judge: str, band: tuple[float,float], stream) -> None:
    S, t = 0.0, 0
    for v in verdict_stream_parser(stream):
        t += 1; S += v.pass_ - midpoint(band)
        if abs(S) > lan_demets_boundary(t):  # breach
            quarantine_claims_citing(judge); emit_correction(judge, S, t); block_gates(judge)
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-05.01–05.05 | G1–G8 → C15.1–C15.8 (ALL) | A1–A37 (ALL, incl. A36/A37) | C09.9, C11.3, C15.9/15.10, C23.7, C06.16 |
