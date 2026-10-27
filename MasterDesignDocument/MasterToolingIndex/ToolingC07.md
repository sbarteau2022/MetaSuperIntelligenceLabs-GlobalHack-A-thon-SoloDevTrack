# ToolingC07 — Tooling Certification for SKL-07 (Security, Sandboxing & Compliance)

**Status.** PROPOSED certification record. **Parent:** Master Design Document index (scaffold appendix).
**Certifies:** `SKL-07` (`SkillsC1.md`) — Security, Sandboxing & Compliance Engineering.
**Naming:** `ToolingC0X` = tooling certification record for `SKL-0X` (not C-series category C0X).

---

## Certification Verdict: CERTIFIED WITH CONDITIONS (ship-blocker skill — strictest bar)

**Necessity re-verified [VERIFIED — `C16/C25/C22` + Decision-Log-ADR ADR-05/06 read directly]:** C03.7/C16.7 make security sign-off a Day-1 precondition for ALL feature code — SKL-07 gates the sprint itself. Seatbelt/holes (C25), runbooks (C22), kill switch (C10.4/ADR-05), privacy (ADR-06), STRIDE/RBAC/secrets/erasure (C16) have no alternate operator. A10/A23 are sandbox measurements. Necessity holds; failure of this skill is sprint failure, not scope reduction.

**Gate mapping correction (Scheme B → dual citation, per C15.9):**

| Day gate (SkillsC1) | Artifact gates (C15.x) operated |
| --- | --- |
| G2 Static Analysis + Unit | C15.1 (scan subset: secrets/paths) |
| G6 Property + Chaos | C15.8 (seatbelt/hole property subset) |
| G7 Chaos/Recovery + Performance | C15.8 (kill-switch drill, breaker drills) |
| G8 Performance + Build Receipt | C15.7 (signed receipt) |

**Ablations:** A10, A22 (tamper-evidence consumer — SKL-07 READS A22 ledger evidence, SKL-04 RUNS the crash injections; custody split recorded), A23 (privacy enforcement — SKL-07 operates the IFC pipeline, SKL-04 operates the fuzz; joint custody).

**Conditions:**

1. Prompt-injection IFC pointer: SkillsC1 cites `C07.8` — the pipeline is at **C07.5** (recorded C11.7/C16.9/C32.7/C14.13-class correction; P-CORR-16 covers). SKL-07 operates C07.5, not C07.8 (which is injection-budget arbitration — related but distinct).
2. `wrangler` secret slots: SKL-07 OWNS slot writes; SKL-03 reads bindings (boundary recorded in ToolingC03 condition 5 — mirrored here).
3. Kill-switch drill authority: SKL-07 OWNS drill execution + grading (C10.8); a drill graded by the team that built the switch is self-grading — the P5 audit drill (drill #2) must be WITNESSED by the QA lead (SKL-05) with independent receipt.
4. Force-close APIs (`circuit-breaker force-close`, hole `--force` approve per C22.3/C22.8) are Admin-role + receipted + time-boxed (auto re-open review in 24h): permanent force-close without review is a control bypass with a receipt printer.
5. `codesign` entitlements: any entitlement beyond sandbox-inherit (network-client for MCP? JIT?) must be listed in the C16 threat model as an accepted risk with expiry — unlisted entitlements fail Gate 2.

---

## Tool Specs

### TOOL-07.01 `secret-scan` — Literal-secret + baked-path grep gate

1. **Tool Name:** `secret-scan` (CI grep/scan per `compliance_smoke.sh` + NF3)
2. **Tool Structure:** typed-JSON (repo root + pattern-set version in; `violations[{file,line,pattern_class}]` out — line numbers only in telemetry, never matched text per ADR-06).
3. **Grammar Constraint Mask:** pattern set VERSIONED (new secret formats added via tracked update, not ad-hoc regex — an unscanned format is an unguarded vector, C16 STRIDE).
4. **Tool Parser Required:** YES — `match-classifier` (maps raw hits to `key|token|path|fp_risk`; content hashes recorded, content never logged).
5. **Definition/Purpose:** Operates the Gate-2 secrets/paths portion + C16.3 slot discipline; runs pre-build (fail fast — secrets in, build refused).
6. **Draft Code:** (Bash — scan gate, content-blind output)

```bash
secret-scan --root . --patterns v3 --out scan.json  # out carries file:line:class:HASH only
jq -e '.violations == []' scan.json || exit 1       # any hit => Gate 2 FAIL, secret text never printed
```

### TOOL-07.02 `seatbelt-exec` — Sandbox confinement operator

1. **Tool Name:** `seatbelt-exec` (macOS Seatbelt executor; profiles from TOOL-03.03 generator)
2. **Tool Structure:** typed-JSON (profile_hash + command_argv_hash + resource limits in; `exit_code` + `deny_log_hash` + receipt out — argv by hash in telemetry, full argv only in the hole receipt + human approval view per C29.7).
3. **Grammar Constraint Mask:** profile MUST come from `seatbelt-profile-gen` output (hand-written profile text rejected — C25.8); `Destructive` argv requires human-approval receipt bound BEFORE exec (post-hoc approval = violation).
4. **Tool Parser Required:** YES — `seatbelt-violation-parser` (shared with TOOL-03.03; deny lines → typed syscall/path records for RB-SEC-01).
5. **Definition/Purpose:** Confines C25 hole execution + C30 subagent loops; the OS layer of the two-layer confinement (effect system = intent layer, C25.8 item 2).
6. **Draft Code:** (Bash — confined exec)

```bash
seatbelt-exec --profile-hash "$PH" --limits mem=512,cpu=30s,files=100m \
  --argv-hash "$AH" -- "$@" || deny-parser --log seatbelt.log --receipt deny.json
```

### TOOL-07.03 `ifc-sanitize` — C07.5 prompt-injection sanitization pipeline operator

1. **Tool Name:** `ifc-sanitize` (tag → classify → quarantine/redact → audit; operates the C07.5 pipeline — NOT C07.8)
2. **Tool Structure:** typed-JSON (memory block + provenance tag in; `sanitized_block` + `risk_score` + `quarantine|redact|pass` + audit receipt out).
3. **Grammar Constraint Mask:** risk threshold `tau_inject` from config (C20 blast class BEHAVIORAL — threshold change triggers C09.8 re-run of A10); blocks below threshold pass UNMODIFIED (over-sanitization destroys recall — the A9/A10 tension is tuned here, not wished away).
4. **Tool Parser Required:** YES — `injection-classifier` (ML risk scorer; version + training-corpus hash bound in every receipt — unversioned classifier output is inadmissible).
5. **Definition/Purpose:** Implements C07.5 non-interference (`Low(injected)` must not flow to `High(tool_calls)`); A10 red-team target; A10 re-run at P5 mandatory (C16.10 — Day-1 sanitizer faces Day-10 attack surface).
6. **Draft Code:** (Python — pipeline with audit)

```python
def sanitize(block: MemoryBlock) -> SanResult:
    tag(block); score = classifier(block.text, version=CLASSIFIER_VER)
    if score > TAU: return quarantine(block, score)   # separate collection, Operator+ only
    if score > TAU_WARN: return redact(block, spans(block), trust="LOW")
    return passed(block, score)  # receipt in all three paths -- no silent path
```

### TOOL-07.04 `kill-switch-drill` — C10.8 drill executor + grader

1. **Tool Name:** `kill-switch-drill` (live flip under load + 4-assert grading)
2. **Tool Structure:** typed-JSON (phase id + load profile in; `switch_ms`, `dropped_receipts`, `atlas_readable: bool`, `toggled_event: bool` + `drill_pass` out).
3. **Grammar Constraint Mask:** drill under LOAD (idle drill rejected as non-evidence — the switch that matters is the switch under pressure); P5 drill requires QA-lead witness receipt (condition 3).
4. **Tool Parser Required:** NO.
5. **Definition/Purpose:** Operates C10.8 (drill per phase gate P1–P5); failed drill blocks phase exit exactly like a failed ablation; rollback.sh executed (not reviewed) before P5 on scratch root.
6. **Draft Code:** (Bash — drill with teeth)

```bash
kill-switch-drill --phase P4 --load A1_corpus --assert "switch_ms<100" \
  --assert dropped_receipts=0 --assert atlas_readable --assert toggled_event \
  --witness qa-lead  # P5 drill only; unwitnessed P5 drill => drill FAIL
```

### TOOL-07.05 `compliance-smoke` — C15 Gate-3 smoke orchestrator (0 FAIL)

1. **Tool Name:** `compliance-smoke` (`scripts/compliance_smoke.sh`: secrets, paths, privacy, claims, five-file, crosslink, ADR traceability)
2. **Tool Structure:** typed-JSON (workspace root in; per-check `pass|fail` + aggregate `FAIL count` out; 0 required).
3. **Grammar Constraint Mask:** check SET is closed (7 checks); a skipped check is recorded `SKIPPED(reason,owner,date)` per C15.10 — never silently absent from the aggregate.
4. **Tool Parser Required:** NO (orchestrates TOOl-07.01 + claim-lint + crosslink + ADR tools; parses only their typed verdicts, not raw output).
5. **Definition/Purpose:** Day-10 go/no-go input for the compliance axis; runs at every phase exit as regression tripwire (cheap, fast, broad).
6. **Draft Code:** (Bash — orchestrator)

```bash
compliance-smoke --root MasterDesignDocument --record-skips \
  | jq -e '.fail == 0 and ([.checks[] | select(.status=="SKIPPED" and .required)] | length == 0)'
# required-check SKIPPED => same as FAIL at go/no-go (C15.10 item 3)
```

---

## Traceability

| Tool | Gates (dual) | Ablations | Design anchor |
| --- | --- | --- | --- |
| TOOL-07.01–07.05 | G2→C15.1, G6/G7→C15.8, G8→C15.7 | A10, A23 (joint), A22 (reads) | C07.5, C10.8, C16.9/16.10, C25.8, C29.7 |
