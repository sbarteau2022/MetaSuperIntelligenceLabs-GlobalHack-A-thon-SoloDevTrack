# ToolingC03 — Tooling Certification for SKL-03 (Agentic Runtime Orchestration)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-03` (`SkillsC1.md`) — Agentic Runtime Orchestration (Skills/Tools/Lanes/Brigade/Holes).
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS

**Necessity re-verified [VERIFIED — `C28/C29/C30/C25/C34` deepened specs read directly]:** the five runtime categories form a dependency chain (manifests → registries → lanes → holes → personas); no other skill operates it. Gates G3/G4/G6/G7 cannot execute without a bootable runtime. A11–A21 are runtime measurements. Necessity holds. (A16 LSP-stress belongs SKL-03 operation, SKL-06 spec — shared custody noted below.)

**Gate mapping correction (Scheme B → dual citation, per C15.9):**

| Day gate (SkillsC1) | Artifact gates (C15.x) operated |
| --- | --- |
| G3 Unit + Integration | C15.1 + C15.2 |
| G4 Integration + Contract | C15.2 + C15.4 |
| G6 Property + Chaos | C15.8 (property subset) |
| G7 Chaos/Recovery + Performance | C15.8 (chaos subset) |

**Ablations:** A11–A15, A17–A21 (A16 shared with SKL-06; A19 subagent caps overlap ADR-12/C30.4 — same ablation, two consumers, one run).

**Conditions:**

1. Hook count: operate on the AUDITED 23 (C28.7 P-CORR-1 pending). Any tool below emitting a 24th/25th moment fails its own Gate 3.
2. Veto-immune audit hooks (P-CORR-10) REQUIRED before RB-RUNTIME-02 (veto storm) is executable — a veto-storm runbook without immune audit hooks cannot distinguish storm from silence.
3. Work-stealing operates under P-CORR-11 authority-transfer rule; the scheduler tool (TOOL-03.04) encodes it — stealing outside the rule is a C16 elevation event.
4. `protobuf` for session types: schemas are GENERATED from the C34 session-type grammar, never hand-written (C19.8 drift class). Hand-written `.proto` for handoffs rejected at Gate 3.
5. `wrangler` in SKL-03's list is the deploy/bindings path (ADR-08 bootstrap validation); secret-slot operations via wrangler belong SKL-07. Boundary: SKL-03 reads bindings, SKL-07 writes slots.

---

## Tool Specs

### TOOL-03.01 `cargo-xtask-manifest` — Compile-in manifest builder (skills + tools)

1. **Tool Name:** `cargo xtask skills-manifest` / `tools-manifest` (build-time manifest compiler)
2. **Tool Structure:** typed-JSON (source scan dirs in; `manifest_hash`, `topo_order`, `binding_report` out; hash recorded in build receipt C15.7).
3. **Grammar Constraint Mask:** manifest JSON Schema (C28.3/C29) validated at generation AND at boot — generation without boot-validation is half a tool (ADR-08).
4. **Tool Parser Required:** YES — `manifest-diff-parser` (drift between source tree and manifest → typed `added/removed/staleness` for CI fail messages).
5. **Definition/Purpose:** Sole producer of `skills.manifest.json` + `tools.manifest.json`; enforces DAG acyclicity (Kahn) and effect monotonicity (CORR-05) at compile time.
6. **Draft Code:** (Rust xtask sketch)

```rust
fn build_manifest(src: &Path) -> Result<ManifestReceipt> {
    let dag = scan_and_sort(src)?;          // Kahn; cycle => fail closed
    check_effect_monotonicity(&dag)?;       // caller effect <= tool effect
    let m = Manifest::emit(&dag)?;
    Ok(ManifestReceipt { hash: m.hash(), order: m.topo_order() })
}
```

### TOOL-03.02 `hook-dispatch` — 23-moment hook runtime

1. **Tool Name:** `hook-dispatch` (compile-time moment table + runtime dispatcher)
2. **Tool Structure:** typed-JSON (moment id + context-hash in; `HookResult|Veto` + audit immunity flag out).
3. **Grammar Constraint Mask:** moment table is `const` (ADR-10) — runtime registration calls rejected at compile; unknown moment string = 400 (C27.7 scheme).
4. **Tool Parser Required:** NO (context carried by hash only — ADR-06; dispatcher never parses payload).
5. **Definition/Purpose:** Executes C28.2 hook algebra with P-CORR-10 veto-immune audit hooks; emits HookVeto receipts; feeds C28.8 veto-rate monitor.
6. **Draft Code:** (Rust — dispatch with immunity)

```rust
fn fire(m: HookMoment, ctx: Hash) -> HookOutcome {
    if IMMUNE.contains(&m) { return audit_hook(m, ctx); } // veto cannot kill
    for hook in topo_hooks(m) { if let Veto(r) = hook.run(ctx)? { return veto_emit(m, r); } }
    HookOutcome::Continue
}
```

### TOOL-03.03 `seatbelt-profile-gen` — Sandbox profile generator

1. **Tool Name:** `seatbelt-profile-gen` (generates HOLE_SEATBELT* profiles from shared constants)
2. **Tool Structure:** typed-JSON (effect level + temp-dir constant + egress list in; profile text + `profile_hash` out; hash bound in hole receipt per C25.8).
3. **Grammar Constraint Mask:** SBPL syntax validated by dry-load (`sandbox-exec -n` parse check); unparseable profile blocks hole creation (fail closed).
4. **Tool Parser Required:** YES — `seatbelt-violation-parser` (deny-log lines → typed `syscall/path/effect` for RB-SEC-01 triage).
5. **Definition/Purpose:** Implements C25.8: profile text generated from the SAME temp-dir constant the code uses; separate `HOSEATBELT_NET` variant only with explicit approval + egress receipt.
6. **Draft Code:** (Python — generation, not hand-writing)

```python
def gen_profile(effect: str, tmp: str, egress: list[str]) -> str:  # tmp = shared constant
    assert re.fullmatch(r"/tmp/hole_[a-f0-9]+", tmp_root_pattern())  # single source of truth
    return render("seatbelt.tmpl", effect=effect, tmp=tmp, egress=egress, net=bool(egress))
```

### TOOL-03.04 `autorunner-admit` — Ready-packet admission + stealing arbiter

1. **Tool Name:** `autorunner-admit` (admission control + work-stealing under C30.8/C30.9 rules)
2. **Tool Structure:** typed-JSON (packet + lane state + backpressure reading in; `admit|defer|steal(origin_caps)` + receipt out).
3. **Grammar Constraint Mask:** backpressure gate is HARD (C30.9 precedence 1) — admit under memory pressure rejected regardless of per-lane room; steals carry origin authority token (P-CORR-11).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** Encodes the two-throttle precedence (backpressure necessary, max_concurrent fairness) and authority-transfer stealing; emits CrossLaneMessage-class receipts for steals (C30.8 item 4) so I(i,j) counts them.
6. **Draft Code:** (Rust — admission predicate)

```rust
fn admit(p: &Packet, lanes: &Lanes, bp: &Backpressure) -> Admission {
    if !bp.allows() { return Defer(MemoryPressure); }          // hard gate first
    if isolated(p.lane) { return Escalate(AlternativeLane); }   // C27.8
    if lanes.at(p.lane) >= max_concurrent(p.lane) { return Defer(Fairness); }
    Admit
}
```

### TOOL-03.05 `brigade-project` — Fiber projection / dissolution operator

1. **Tool Name:** `brigade-project` (parallel-transport projection + dissolve-back for Brigade specialists)
2. **Tool Structure:** typed-JSON (lane submanifold anchor + radius + task hash in; `fiber_view_hash` out; dissolve returns `merge_receipt` with `weight_copies` attestation).
3. **Grammar Constraint Mask:** radius ≤ lane `brigade_radius` (config cap); projection requests beyond cap rejected (no silent widening — C06.14 interference budget).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** Operates C06.12/C30.6 fiber mechanics; EVERY dissolve attests `weight_copies = 1` (A21/A4 judge input); linearity enforced by consuming the projection token (C28.5 provenance-token pattern reused).
6. **Draft Code:** (Rust — linear projection token)

```rust
struct FiberToken { lane: LaneId, view_hash: Hash, used: bool }
fn dissolve(t: FiberToken) -> Result<MergeReceipt> {
    assert!(!t.used, "double-dissolve = weight fork");  // linearity (C28.5 class)
    Ok(MergeReceipt { weight_copies: count_live_weights()?, .. })
}
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-03.01–03.05 | G3→C15.1/C15.2, G4→C15.2/C15.4, G6/G7→C15.8 | A11–A15, A17–A21 (A16 shared) | C25.8, C27.8, C28.7/28.8, C30.8/30.9 |
