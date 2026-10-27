# C28 — Skill Registry & Hooks

**Status.** PROPOSED (deepened from scaffold). **Parent:** Master Design Document index.

## Purpose

Skill tree as a DAG with topological sort; hook moments as compile-time manifest; provenance as linear types. Deterministic gating of skill lifecycle from registration through execution to revocation.

## Subcategories (broad topics)

- C28.1 Skill registry + tree (depends_on, topo sort, lazy/eager)
- C28.2 Hook event map (23 moments), blocking veto, receipts
- C28.3 Compile-in manifest vs hand-edited JSON (C14.8)
- C28.4 Auto-population at boot (C14.6)
- C28.5 Skill activation scoped to brigade neighborhood (C06.12)
- C28.6 Provenance and revocation

---

## Canonical Decisions (frozen for design)

| Decision                                                                                                                | Rationale                                                             |
| ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| **Skill tree = DAG** with `depends_on` edges; cycles rejected at manifest compile                                       | Deterministic load order; enables parallel init of independent skills |
| **23 hook moments** fixed at compile time; no runtime hook registration                                                 | Prevents hook-order races; manifest is auditable                      |
| **Compile-in manifest** (`skills.manifest.json`) is source of truth; hand-edited JSON forbidden in ship builds          | Supply-chain integrity; manifest hash in build receipt                |
| **Boot scan** (C14.6) validates manifest against `wrangler.jsonc` bindings; mismatch = build failure                    | Fail-fast on missing bindings/secrets                                 |
| **Brigade-scoped activation** (C06.12): skill fiber projected along geodesic γ_i; dissolution = parallel transport back | Single weight set invariant; no skill leakage between lanes           |
| **Provenance as linear type**: each skill instance carries `ProvenanceToken` consumed on revoke                         | No double-revoke; revocation is auditable                             |

---

## Mathematical Specification

### 1. Skill Registry as Typed DAG

Let **S** be the set of all skills. A skill `s ∈ S` is a tuple:

```
s = (id: SkillId, version: SemVer, depends_on: Set<SkillId>,
     hooks: Set<HookMoment>, entry: WasmModule,
     caps: CapabilitySet, provenance: ProvenanceToken)
```

**Dependency DAG**: `G = (S, E)` where `E = {(s, t) | t ∈ s.depends_on}`.

**Invariant**: `G` is acyclic. **Verification**: Kahn's algorithm at manifest compile.

**Topological load order**: `σ: S → ℕ` such that `∀(s,t)∈E: σ(s) < σ(t)`.

**Lazy vs eager**: `s.lazy = true` iff `s` has no `Init` hook and no eager dependents.

### 2. Hook Moment Algebra

**HookMoment** ∈ {
`PreSkillLoad`, `PostSkillLoad`,
`PreToolCall`, `PostToolCall`,
`PreSubagentSpawn`, `PostSubagentSpawn`,
`PreSubagentReturn`, `PostSubagentReturn`,
`PreHoleEnter`, `PostHoleEnter`,
`PreHoleExit`, `PostHoleExit`,
`PreMerge`, `PostMerge`,
`PreCompaction`, `PostCompaction`,
`PreCheckpoint`, `PostCheckpoint`,
`PreApproval`, `PostApproval`,
`PreRevocation`, `PostRevocation`,
`SessionStart`, `SessionEnd`,
`ErrorBoundary`
}

**Hook signature** (linear type):

```
hook : (SkillId, HookMoment, HookContext) ⊸ (HookResult ⊕ Veto)
```

- **Blocking veto**: `Veto(reason)` aborts the enclosing action; receipt emitted.
- **Non-blocking**: `HookResult(modified_context)` continues; receipt emitted.
- **Receipt**: `MerklePath(hook_id, moment, skill_id, input_hash, output_hash, timestamp)`.

**Execution semantics**: For moment `m`, execute hooks in topo order of skills declaring `m`. A single `Veto` short-circuits remaining hooks for `m` and aborts.

### 3. Compile-In Manifest

**Manifest schema** (`skills.manifest.json`):

```json
{
  "manifest_version": 1,
  "skills": [
    {
      "id": "skill.arena.retrieval",
      "version": "1.2.0",
      "depends_on": ["skill.core.memory"],
      "hooks": ["PreToolCall", "PostToolCall", "PreCompaction"],
      "entry": "wasm/skill_arena_retrieval.wasm",
      "caps": ["memory.read", "index.query"],
      "provenance": "sha256:...",
      "lazy": true
    }
  ],
  "topo_order": ["skill.core.memory", "skill.arena.retrieval", ...],
  "manifest_hash": "sha256:..."
}
```

**Generation**: `cargo xtask skills-manifest` runs at build (C14.8). Output hash recorded in build receipt.

**Validation at boot** (C14.6):

1. Load `skills.manifest.json`
2. Verify `manifest_hash` matches embedded hash
3. For each skill: verify Wasm module hash, check `depends_on` resolved, validate `caps` against `wrangler.jsonc` bindings
4. Emit `SkillRegistryReady` event to ledger (C31.1)

### 4. Brigade-Scoped Skill Activation (C06.12)

Let `M = H³ × T²` be the Atlas manifold. A **brigade fiber** for lane `i` is a geodesic neighborhood:

```
F_i = { x ∈ M | d_M(x, U_i) < ε_i }
```

where `U_i ⊂ M` is the lane's submanifold, `ε_i` is the brigade radius.

**Skill projection**: When lane `i` activates skill `s`, the skill's Wasm module receives a **projected view** of `M` restricted to `F_i`. Formally:

```
s.view_i = π_{F_i}(s.global_view)
```

where `π_{F_i}` is orthogonal projection onto the tangent space at `U_i` followed by exponential map.

**Dissolution**: On task completion, `s.view_i` is parallel-transported back along `γ_i` (the geodesic from origin to `U_i`) and merged into the global weight set. The **invariant**:

```
weight_copies = 1  ∀t
```

**Interference bound**: If `d_M(U_i, U_j) < τ_interfere`, lane `i` and `j` cannot both activate skills with overlapping `caps` without circuit breaker (C06.5, C22.3).

### 5. Provenance and Revocation as Linear Types

**ProvenanceToken** is a linear resource:

```
ProvenanceToken = (skill_id, manifest_hash, issuance_clock, capabilities_hash)
```

**Operations** (all linear — token consumed):

- `activate(token) → (instance, token')` — token' has incremented clock
- `revoke(token) → RevocationReceipt` — token consumed; skill instance terminated
- `transfer(token, target_lane) → token'` — cross-lane handoff with receipt

**RevocationReceipt** = `MerklePath(skill_id, revocation_clock, reason, prior_state_hash)`.

**No double-revoke**: Type system enforces `token` consumed exactly once.

---

## Verification Gates (C15)

| Gate                 | Test                                          | Threshold              |
| -------------------- | --------------------------------------------- | ---------------------- |
| DAG acyclicity       | Kahn's algorithm on manifest                  | 0 cycles               |
| Manifest hash        | Embedded vs computed SHA-256                  | Match                  |
| Boot validation      | All skills resolve against `wrangler.jsonc`   | 0 missing bindings     |
| Hook veto            | Single veto aborts action, receipt emitted    | 100%                   |
| Brigade isolation    | Skills in lane i cannot access caps of lane j | 0 cross-lane cap leaks |
| Provenance linearity | Token consumed exactly once per revoke        | 100%                   |
| Manifest determinism | Same source → same manifest hash              | 100%                   |

---

## Preregistration Entries (C09)

### A11 — Skill Registry Integrity

- Hypothesis: Manifest hash deterministic; boot validation catches all binding mismatches.
- Baseline: Ad-hoc skill loading (no manifest).
- Judge: Build reproducibility + boot failure injection test.
- Status: TBD.

### A12 — Hook Veto Correctness

- Hypothesis: Single veto at any moment aborts action and emits receipt.
- Judge: 23 moments × veto/non-veto matrix; receipt presence.
- Status: TBD.

### A13 — Brigade Skill Isolation

- Hypothesis: Lane i skill cannot invoke caps of lane j without circuit breaker.
- Judge: Cross-lane cap access attempts logged and denied.
- Status: TBD.

---

## Open Questions / Gaps

- Skill version negotiation protocol for multi-version manifests (deferred to P6).
- Hot-reload of skills without session restart (requires linear type transfer proof).
- Skill marketplace signature verification (out of scope for core).

---

## Contributor Deepening — Muse (2026-10-26)

### C28.7 Hook-count consistency (depth: is it 23?)

Three sources state the hook-moment count [VERIFIED — all read directly]: C28 lists 23 fixed moments with an enumerated algebra (count the algebra: Pre/Post pairs × 11 = 22 + SessionStart/SessionEnd/ErrorBoundary = 25 — NOT 23). C14.8 says "23 moments compiled" with a sketch showing 6 + "17 more" = 23. C30/C34 handoff protocols consume hook moments for session transitions. The algebra enumerates 25 names while the canonical count is 23 — two extra names exist somewhere. Frozen rule [PROPOSED — rule is my proposal; the count 23 (ADR-10) is the authority]:

1. The C28.2 algebra above is normative ONLY if it contains exactly 23 names. Audit: enumerate, number, and if 25, either (a) strike 2 names via C36 correction, or (b) amend ADR-10 to 25 with rationale. A manifest test (`cargo xtask skills-dag --verify`) that counts a different number than ADR-10 states is a Gate 2 failure, not a rounding error.
2. Every C30/C34 handoff transition and every C25 lifecycle transition must name its hook moment from the audited 23. A transition firing an unlisted moment is an unregistered capability (C16.10 same class).

### C28.8 Veto-storm deadlock (depth: vetoes compose badly)

Execution semantics: "A single Veto short-circuits remaining hooks for m and aborts." Composed across 23 moments × N skills, a veto at an early moment (e.g. PreToolCall) can starve every downstream moment's hooks of execution — including PostApproval audit hooks. Frozen ordering rule [PROPOSED — rule is my proposal]:

1. Audit/receipt hooks (PostApproval, PostRevocation, ErrorBoundary) are VETO-IMMUNE: they execute even when the action aborts. An aborted action without an audit receipt is a repudiation hole (C16 STRIDE): the veto itself must be receipted by a hook the veto cannot kill.
2. A skill whose veto rate exceeds a preregistered band (C06.16 judge-monitor analogue for hooks) is quarantined for review: chronic veto is either a broken skill or a policy fight — both need human eyes, not infinite abort loops.

What was NOT checked here: current hook implementation (outside workspace); veto-immunity in the present fire_hook sketch (verify at implementation).

---

## Source Detail Migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

### Skill Tree + Hooks (Original)

- **Skill registry** = single source of truth for all skills (built-in + user).
- **depends_on** edges form a DAG; topo-sorted at load.
- **23 hook moments** cover skill lifecycle, tool calls, subagent, holes, compaction, approval, session.
- **Compile-in manifest** generated at build; hand-edited JSON rejected.
- **Boot scan** (C14.6) auto-populates registry and validates bindings.
- **Brigade-scoped** (C06.12): skill fiber projected to lane's geodesic neighborhood.
- **Provenance**: each skill carries a revocation token; revoke consumes it.

---
