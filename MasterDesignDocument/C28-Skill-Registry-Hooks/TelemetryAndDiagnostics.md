# C28 — Telemetry & Diagnostics

**Category:** C28-Skill-Registry-Hooks  
**Purpose:** Observability contract for skill registry and hooks.

---

## Event Ledger Entries (C31.1)

| Event | Fields | Privacy |
|---|---|---|
| `SkillLoaded` | `skill_id`, `version`, `load_latency_ms`, `lazy` | ids/latency only |
| `SkillHookFired` | `skill_id`, `moment`, `duration_ms`, `result: ok/veto` | ids/latency/result only |
| `SkillVeto` | `skill_id`, `moment`, `reason_code` | ids/reason only |
| `SkillRevoked` | `skill_id`, `revocation_clock`, `reason` | ids only |
| `ManifestValidated` | `manifest_hash`, `skill_count`, `validation_latency_ms` | ids/latency only |
| `BrigadeSkillProjected` | `lane_id`, `skill_id`, `fiber_radius`, `projection_latency_ms` | ids/latency only |

**Never logged**: hook context payloads, skill Wasm memory, capability arguments, veto reason text.

---

## Prometheus Metrics (C31.3)

| Metric | Type | Labels | Description |
|---|---|---|---|
| `skill_load_total` | Counter | `skill_id`, `result` | Load attempts |
| `skill_hook_duration_ms` | Histogram | `skill_id`, `moment` | Hook latency |
| `skill_hook_veto_total` | Counter | `skill_id`, `moment`, `reason_code` | Veto count |
| `skill_revocation_total` | Counter | `skill_id`, `reason` | Revocations |
| `manifest_validation_duration_ms` | Histogram | — | Boot validation time |
| `brigade_projection_duration_ms` | Histogram | `lane_id` | Fiber projection time |

---

## Taps (C31.3)

| Tap | Trigger | Emits |
|---|---|---|
| `skill.load` | PostSkillLoad | `skill_id`, `latency` |
| `skill.hook` | Any hook moment | `skill_id`, `moment`, `latency`, `ok` |
| `skill.veto` | Veto fired | `skill_id`, `moment`, `reason_code` |
| `skill.revoke` | Revocation | `skill_id`, `clock` |

---

## Receipts & Fingerprints (C31.3)

| Action | Receipt Type | Fingerprint Input |
|---|---|---|
| Skill load | `SkillLoadReceipt` | `manifest_hash + skill_id + wasm_hash + clock` |
| Hook veto | `HookVetoReceipt` | `skill_id + moment + context_hash + clock` |
| Revocation | `RevocationReceipt` | `skill_id + clock + reason_hash + prior_state_hash` |
| Manifest validation | `ManifestReceipt` | `manifest_hash + bindings_hash + clock` |

---

## Neural Map Roll-Up (C31.4)

End-of-day aggregation:
- Skill activation frequency per lane
- Veto rate by moment
- Average hook latency by skill
- Brigade projection success rate

**No raw context** in roll-up; only counts, rates, latencies.

---

## Diagnostics Commands

```bash
# Skill registry health
curl localhost:8088/api/skills/health

# Manifest validation
cargo xtask skills-manifest --check

# Hook trace for skill
curl localhost:8088/api/skills/{id}/hooks/trace?limit=100

# Brigade projection audit
curl localhost:8088/api/lanes/{id}/brigade/audit
```

---

## Privacy Compliance (C16.10, ADR-06)

- **Counts/ids/latency only** — enforced by telemetry process schema validation.
- No hook arguments, skill memory, or veto reason text in any metric, tap, receipt, or log.
- Violations caught by `scripts/compliance_smoke.sh` (C15).
