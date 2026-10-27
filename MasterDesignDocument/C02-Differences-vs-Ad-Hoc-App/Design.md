# C02 — Differences vs. the Current Ad-Hoc App

**Status.** PROPOSED. **Parent:** Master Design Document index.

## Purpose

Enumerate every structural difference between the experimental local prototype and the enterprise
design so no ad-hoc behavior is silently carried forward.

## Subcategories (broad topics)

- C02.1 Difference 1 — KV cache: O(N²) → Nested Hyperbolic Poincaré Dynamic Phase-State Space (C05)
- C02.2 Difference 2 — (GAP) Single binary → dependency-injected composition root (C14.2)
- C02.3 Difference 3 — (GAP) One-shot chat → real tool loop in lanes (C14.3)
- C02.4 Difference 4 — (GAP) Non-streaming → streaming default with measured TTFT (C14.4/C18)
- C02.5 Difference 5 — (GAP) 3 fixed slots → N data-driven lanes (C14.7/C32-ish)
- C02.6 Difference 6 — (GAP) Hand-edited hooks.json → compiled-in manifest (C14.8)
- C02.7 Difference 7 — (GAP) Absolute paths → workspace-root resolution (C14.10)
- C02.8 Difference registry and parity checklist (prototype feature → enterprise disposition)

## Difference Registry with Automated Parity Checking

| Diff ID | Prototype Behavior          | Enterprise Behavior                               | Verification                              | Status   |
| ------- | --------------------------- | ------------------------------------------------- | ----------------------------------------- | -------- |
| D1      | O(N²) flat KV cache         | Nested hyperbolic Poincaré (Lorentz H³ canonical) | A1 ablation                               | FROZEN   |
| D2      | Single binary `main.rs`     | DI composition root + module manifest             | `cargo test --locked` + module graph diff | DESIGNED |
| D3      | One-shot `/agent/run`       | Tool loop: plan→tool→observe→continue             | E2E mock-lane test                        | DESIGNED |
| D4      | Non-streaming, no TTFT      | Streaming default + measured TTFT                 | `scripts/main_ui_e2e.py` TTFT check       | DESIGNED |
| D5      | 3 fixed `AgentConfig` slots | N data-driven lanes in `configurations.json`      | Migration test + schema                   | FROZEN   |
| D6      | Hand-edited `hooks.json`    | Compile-time hook manifest (23 moments)           | Build fails on drift                      | DESIGNED |
| D7      | Absolute paths `/Users/...` | Workspace-root + `ATOMA_WORKSPACE_ROOT`           | Portability smoke (NF4)                   | FROZEN   |
| D8      | Implicit tool permissions   | Explicit risk levels + hole/approval gating       | Property tests (C25)                      | DESIGNED |
| D9      | No memory injection         | Auto-inject canonical memory at tab/session       | A9 ablation                               | DESIGNED |
| D10     | No structured summaries     | Mandatory 5W summary deposit (C07.2)              | A9 + summary_missing receipt              | DESIGNED |

## Automated Parity Checker (Design)

```rust
// parity_check.rs — runs in CI, fails on drift
fn check_parity() -> Result<(), ParityError> {
    let prototype = scan_prototype_artifacts();
    let enterprise = load_enterprise_spec();

    for (feature, proto_impl) in prototype {
        let ent_spec = enterprise.get(&feature).ok_or(ParityError::MissingSpec(feature))?;
        if !spec_covers(proto_impl, ent_spec) {
            return Err(ParityError::Uncovered(feature, proto_impl));
        }
    }
    Ok(())
}
```

**Invariant:** Every prototype feature must have a corresponding enterprise disposition (REPLACE, WRAP, DEPRECATE, DEFER). No silent carry-forward.

---

## Contributor Deepening — Muse (2026-10-26)

### C02.9 Parity audit protocol (depth, not novelty)

The registry above is PROPOSED [PROPOSED — checker code sketched, never executed]. It becomes verifiable only with an audit protocol:

1. **Freeze the prototype surface first.** Before Day 1 feature code, `scan_prototype_artifacts()` output is committed as `PROTOTYPE_SURFACE.json` (file list + entry-point hashes of the ad-hoc app). Any prototype file not in the registry table gets disposition TBD and blocks Gate 1 (ADR-21). [PROPOSED — no surface file exists yet.]
2. **Disposition semantics (frozen definitions):**
   - `REPLACE` — prototype behavior removed; enterprise spec is the only implementation. Old code must be deleted, not left reachable.
   - `WRAP` — prototype behavior kept behind an enterprise interface (adapter + receipt). Adapter carries the approval/hole policy of the enterprise side.
   - `DEPRECATE` — prototype behavior ships but emits a `deprecated` receipt on every use and has a dated removal entry in C36.
   - `DEFER` — prototype behavior explicitly out of hackathon scope; re-entry only via appended C36 correction.
3. **CI binding.** `check_parity()` runs in Gate 1 (`cargo xtask gate1`) and fails the build on `MissingSpec` or `Uncovered`. The parity result (pass/fail + uncovered list) is recorded in the build receipt (C15 Gate 8 artifact), not just console output.
4. **Per-diff evidence pointers [INFERRED — derived from the registry status column; ends at C02/Design.md]:**

| Diff                  | Why the current status is justified                        | What would promote it to FROZEN                             |
| --------------------- | ---------------------------------------------------------- | ----------------------------------------------------------- |
| D1                    | Specified by user; canonical geometry frozen (C05, ADR-01) | A1 pass on two hardware targets                             |
| D5                    | `configurations.json` schema exists in C30.1/C34           | migration test + schema validation green (C30 verification) |
| D7                    | ADR-04 frozen; `ATOMA_WORKSPACE_ROOT` contract in C17      | portability smoke on non-dev machine (NF4)                  |
| D2/D3/D4/D6/D8/D9/D10 | Design text only, no executable check wired                | the verification listed in-row executed and receipted       |

### Traceability (D → category → gate → ablation)

| Diff | Owning category | Gate     | Ablation                               |
| ---- | --------------- | -------- | -------------------------------------- |
| D1   | C05 / C12       | Gate 8   | A1                                     |
| D2   | C14.2           | Gate 2   | — (static analysis)                    |
| D3   | C14.3 / C30     | Gate 3   | A5                                     |
| D4   | C14.4 / C18     | Gate 3   | TTFT check in `scripts/main_ui_e2e.py` |
| D5   | C30.1 / C34     | Gate 3/4 | A5, A17, A28                           |
| D6   | C28 (ADR-08/10) | Gate 2   | A7                                     |
| D7   | C17 (ADR-04)    | Gate 3   | — (portability smoke)                  |
| D8   | C29 / C25       | Gate 5   | A7, A10                                |
| D9   | C07.1           | Gate 3   | A9                                     |
| D10  | C07.2           | Gate 3   | A9 + `summary_missing` receipt         |

What was NOT checked here: whether `scan_prototype_artifacts()` has a defined input path (the ad-hoc app repo location is not pinned in this workspace).

## Open questions / gaps

- Audit the current app against this registry before Day 1 feature code (§3.7).

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 2. Differences vs. the current ad-hoc app

### Difference 1 — KV cache: O(N²) → Nested Hyperbolic Poincaré Dynamic Phase-State Space

1. **Problem in the current transformer path.** Every generated token re-runs a full forward pass
   over the entire context, giving O(N²) cost and O(N) memory growth. That is the OOM ceiling
   that keeps solo developers from shipping enterprise-grade systems.
2. **Target.** Replace the flat attention KV cache with a _Nested Hyperbolic Poincaré Dynamic
   Phase-State Space_. Memory is stored as coordinates in a hyperbolic memory map; retrieval is
   geometric, so the system builds only what it needs to recall and complete the task, not a
   full forward pass over everything irrespective of reasoning depth.
3. **Where it nests.** _Inside the Atlas_ — the Multi-Model Phase State Space. The Atlas is the
   spatiotemporal/phase-state container that stores memory as coordinates in a Hyperbolic Memory
   Map. The cache is a plug-in: it attaches to the Atlas rather than replacing the whole stack.
4. **Plug-in mode.** The Multi-Model Mind can run either:
   - (a) the standard transformer (frontier-class knowledge for the agentic team at scale), or
   - (b) the plug-in hyperbolic phase-state cache (nested Poincaré) for memory/computation.
     Either mode avoids the soul-crushing OOM that blocks enterprise-grade solo shipping.
5. **Cognition upgrade — Active Inferencing Transformer (ACFT).** Standard transformers hold
   frozen weights in the normal sense. ACFT does not. It uses _TwisterAttention_, which builds
   only what is needed to retrieve from the map and to complete the task: no full forward pass
   for everything without respect to the depth of the reasoning + recall. Weights/recall are
   active-inference-driven, not statically frozen.

6. **Telemetry and diagnostics are first-class.** Tooling and diagnostics are built-in engineering
   principles, not bolted-on dashboards. Token cost, latency, throughput, cache hit/miss, recall
   depth, and OOM headroom are measured and emitted from the serving core.
7. **Security and compliance are first-class.** They are engineered _before_ the first feature is
   built, not installed as tools or run as outside processes. Role access, Merkle receipts,
   audit logs, secret-by-slot, and regulatory mapping are part of the core contract.
8. Active Memory State Mangement and Forward Iterative Focus
9.

---
