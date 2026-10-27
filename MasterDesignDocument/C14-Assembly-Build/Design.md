# C14 — Assembly & Build Improvements

**Status.** PROPOSED (where to push harder). **Parent:** Master Design Document index.

## Purpose

How the app is built/assembled, independent of model science.

## Subcategories (broad topics)

- C14.1 Frontend chunking (code-split by route/panel; lazy-load CodeMirror/markdown)
- C14.2 Backend composition root (DI / declarative module list; remove unused methods or wire them)
- C14.3 Tool loop in lanes (plan → tool → observe → continue behind approval)
- C14.4 Streaming + TTFT (streaming default; propagate TTFT through receipts)
- C14.5 Open embedding default (bundled/local text-embedding lane or honestly labeled TF-hash baseline)
- C14.6 Skill tree auto-population (scan skills/ and global dirs at boot, idempotent)
- C14.7 N lanes not 3 (data-driven roster; per-agent rag_collection, tools allow-list, approval policy)
- C14.8 Hooks as compile-in manifest (23 moments compiled into binary manifest)
- C14.9 Test matrix as first-class artifact
- C14.10 Data-root portability (ATOMA_WORKSPACE_ROOT override; no baked user paths)
- C14.11 Release profile honesty (build chain documented in one script or split dev/release assets)
- C14.12 Governance file set per workstream adopted as repo structure

## Open questions / gaps

- Each item needs an owner and Day in C16 (verification gates) before sprint start.

---

## Source detail migrated from `01-Aers-SpecialOperations-Agentic-Lab-Design-Overview.md`

## 9. Assembly & build improvements (where I would push harder)

These are about _how the app is built/assembled_, not the model science. Each is a candidate
for a design-doc (or refactor) decision before the sprint, so we don't paint ourselves in.

1. **Frontend chunking.** `vite build` emits a single ~1.47 MB JS chunk. Enterprise assembly
   should code-split by route/panel (Settings, Swarm Board, Runs) and lazy-load CodeMirror and
   markdown; the current single chunk is the main LCP risk.
2. **Backend module as services, not one binary.** `main.rs` wires every subsystem by hand
   into `AppState`. Assembly should move to a small dependency-injected composition root (or
   at least a declarative module list) so new silos don't require editing main.rs; also remove
   the unused methods flagged by warnings (`is_enabled`, `workflow_of_packet`, etc.) or wire them.
3. **Tool loop in the lanes.** Today `/agent/run` is one completion per call; the agentic
   contract only exists in workflows. Assembly should make the lane runner a real tool loop
   (plan → tool call → observe → continue) behind an approval gate, instead of one-shot chat.
   Without this, "agent" is a label, not a capability.
4. **Streaming + TTFT.** Provider calls are non-streaming, so TTFT is unmeasured. Assembly
   should make streaming the default transport and propagate TTFT through telemetry receipts;
   blocking parsers are the single biggest latency honesty gap.
5. **Open embedding default.** Dense RAG only works when an embed endpoint exists. Ship a
   small bundled/local text-embedding lane (or a TF-hash baseline that is honestly labeled)
   so retrieval is never silently BM25-only in an "enterprise" build.
6. **Skill tree auto-population.** `/skills/tree` returns 0 nodes until skill-dir plugins are
   registered. Assembly should scan `skills/` and the global skill dirs at boot and register
   them idempotently, so the feature is live on day 1.
7. **N lanes, not 3.** `AgentConfig` hard-codes three slots. Make the roster data-driven (N
   agents with lane assignment) and carry `rag_collection`, tools allow-list, and approval policy
   per agent.
8. **Hooks as a compile-in manifest, not a JSON file.** Today `hooks.json` is hand-edited and
   `fire_event` never auto-runs handlers. Assembly should compile the 23 hook moments into the
   binary manifest (matching the deterministic pre/post gating story) so they can't drift.
9. **Test matrix as a first-class artifact.** 10 unit tests + 1 green Python E2E + 1 flaky
   browser E2E is thin for an enterprise ship. Add property tests on hole/merge/Merkle and a
   deterministic mock-lane agent loop test so reliability is _measured_, not asserted.
10. **Data-root portability.** Paths are absolute and user-specific (`/Users/.../AtomaApp`).
    Assembly should resolve everything relative to a discovered workspace root with a single
    `ATOMA_WORKSPACE_ROOT` override, and never bake a user path into defaults.
11. **Release profile honesty.** The binary bakes the frontend into `include_dir!`; changes
    require a full rebuild. Either document the required build chain (frontend build before
    cargo build --release) in one script, or split asset serving in dev vs. release.
12. **Governance file set per workstream.** Adopt the five-file set (Design, Pre-Registration,
    Claims Register, Successes, Failures) as repo structure, not prose, so claims are auditable.

MULTI-MODEL DISTILLATION - UNIFIED MEMORY MANIFOLD AS A HARNESS PLUG-IN

## Build System as Verified Artifact

### C14.1 Frontend Chunking: Dependency Graph Optimization

The **chunking strategy** solves:

$$\min_{\text{chunks}} \sum_{c \in \text{chunks}} \text{size}(c) \cdot \text{load\_prob}(c)$$

subject to:

- **Route-level splitting**: Each route/panel = separate chunk
- **Lazy loading**: CodeMirror, Markdown, Swarm Board loaded on demand
- **Shared deps**: React, UI libs in vendor chunk
- **LCP budget**: Critical path < 170KB gzipped

**Vite config** (generated from route manifest):

```typescript
// vite.config.ts — generated from routes.yaml
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: (id) => {
          if (id.includes("codemirror")) return "codemirror";
          if (id.includes("markdown")) return "markdown";
          if (id.includes("swarm-board")) return "swarm-board";
          if (id.includes("node_modules")) return "vendor";
        },
      },
    },
  },
});
```

### C14.2 Backend Composition Root: Typed Dependency Injection

The **composition root** is a **typed module graph**:

```rust
// composition_root.rs — compile-time verified
#[derive(ModuleGraph)]
struct AppModules {
    #[provides]
    config: Config,

    #[provides]
    event_ledger: EventLedger,

    #[provides]
    mind_store: MindStore,

    #[provides]
    rag_engine: RagEngine,

    #[provides]
    workflow_engine: WorkflowEngine,

    #[provides]
    hole_substrate: HoleSubstrate,

    #[provides]
    tool_registry: ToolRegistry,

    #[provides]
    skill_registry: SkillRegistry,

    #[provides]
    mcp_runtime: McpRuntime,

    #[provides]
    model_serving: ModelServing,

    #[provides]
    telemetry: Telemetry,

    #[provides]
    compliance: Compliance,
}

// Main just wires the graph
fn main() -> Result<()> {
    let modules = AppModules::build()?;
    let app_state = modules.into_app_state();
    run_server(app_state)
}
```

**Verification**: `cargo check` fails if:

- Cycle in dependency graph
- Missing required dependency
- Unused module (dead code)
- Version mismatch between modules

### C14.3 Tool Loop: Agent as State Machine

The **lane runner** is a **Mealy machine** with typed transitions:

```rust
enum AgentState {
    Planning(Plan),
    AwaitingApproval(ToolCall),
    Executing(ToolCall),
    Observing(ToolResult),
    Continuing,
    Done(Result),
}

impl AgentState {
    fn step(self, input: AgentInput) -> Result<(Self, AgentOutput)> {
        match (self, input) {
            (Planning(plan), AgentInput::PlanApproved)
                => Ok((AwaitingApproval(plan.next_tool()), Output::ToolCall)),
            (AwaitingApproval(call), AgentInput::Approved(receipt))
                => Ok((Executing(call), Output::Executing)),
            (Executing(call), AgentInput::Result(result))
                => Ok((Observing(result), Output::Observed)),
            (Observing(result), AgentInput::Continue)
                => Ok((Continuing, Output::Continue)),
            (Continuing, AgentInput::NextStep)
                => self.plan.next().map(|p| (Planning(p), Output::Planning))
                    .unwrap_or((Done(Ok(())), Output::Done)),
            _ => Err(InvalidTransition),
        }
    }
}
```

### C14.4 Streaming + TTFT: Measured Transport

**Streaming protocol** with TTFT propagation:

```rust
// Streaming response with TTFT telemetry
struct StreamResponse {
    ttft_ms: u64,           // Measured at first token
    tokens: TokenStream,    // Async stream
    receipt: Receipt,       // Includes TTFT
}

impl StreamResponse {
    async fn from_provider(req: Request) -> Result<Self> {
        let start = Instant::now();
        let mut stream = provider.stream(req).await?;

        let first_token = stream.next().await?;
        let ttft = start.elapsed().as_millis() as u64;

        Ok(Self {
            ttft_ms: ttft,
            tokens: stream::once(async move { first_token }).chain(stream),
            receipt: Receipt::new(ttft, ...),
        })
    }
}
```

### C14.5 Open Embedding Default: Honest Baseline

**Embedding options** (all honestly labeled):

| Option    | Model  | Size  | Quality  | Honesty Label                    |
| --------- | ------ | ----- | -------- | -------------------------------- |
| BM25      | —      | 0     | Baseline | "BM25 only"                      |
| TF-Hash   | Custom | 50MB  | Medium   | "TF-Hash baseline (no semantic)" |
| MiniLM-L6 | ONNX   | 90MB  | Good     | "Local MiniLM-L6"                |
| BGE-Small | ONNX   | 130MB | Better   | "Local BGE-Small"                |
| Remote    | API    | —     | Best     | "Remote: {endpoint}"             |

**Default**: BM25 (NF6). User opts into local/remote via config.

### C14.6 Skill Tree: Boot-Time Idempotent Scan

```rust
fn populate_skill_tree() -> SkillTree {
    let mut tree = SkillTree::new();

    // Local skills
    for entry in scan_dir("skills/")? {
        let skill = load_skill(entry)?;
        tree.register(skill)?;
    }

    // Global skills (~/.agents/skills)
    for dir in global_skill_dirs() {
        for entry in scan_dir(dir)? {
            let skill = load_skill(entry)?;
            tree.register_idempotent(skill)?;  // Skip if already registered
        }
    }

    tree.validate_topo_sort()?;  // Fail fast on cycles
    tree
}
```

### C14.8 Hooks: Compile-Time Manifest

```rust
// hooks_manifest.rs — generated by build.rs
pub const HOOK_MANIFEST: &[HookMoment] = &[
    HookMoment { name: "pre_tool_call", veto: true,  receipt: true },
    HookMoment { name: "post_tool_call", veto: false, receipt: true },
    HookMoment { name: "pre_hole_create", veto: true,  receipt: true },
    HookMoment { name: "post_hole_create", veto: false, receipt: true },
    HookMoment { name: "pre_hole_merge", veto: true,  receipt: true },
    HookMoment { name: "post_hole_merge", veto: false, receipt: true },
    // ... 17 more moments
];

// Fire hook — compile-time verified
pub fn fire_hook<T: HookHandler>(moment: &str, ctx: &mut HookContext) -> Result<()> {
    const MOMENTS: &[&str] = &[/* all 23 names */];
    assert!(MOMENTS.contains(&moment), "Unknown hook moment: {}", moment);

    T::handle(moment, ctx)
}
```

### C14.10 Data Root: Workspace Discovery

```rust
fn discover_workspace_root() -> PathBuf {
    // Priority order:
    // 1. ATOMA_WORKSPACE_ROOT env var
    // 2. .atoma_workspace marker file (walk up from cwd)
    // 3. Cargo.toml with [workspace] (walk up)
    // 4. Git root (walk up)
    // 5. Fallback: $HOME/.atoma
}
```

### C14.11 Release Profile: Single Build Script

```bash
#!/bin/bash
# build_release.sh — single source of truth

set -euo pipefail

# 1. Frontend
cd frontend
npm ci
npm run build
cd ..

# 2. Backend
cargo build --release --locked

# 3. Verify assets baked correctly
cargo run --release --bin verify_assets

# 4. Generate build receipt
./scripts/generate_build_receipt.sh

# 5. Package (dev)
# ./scripts/package_tauri.sh  # Phase 6
```

### C14.12 Governance Files: Repo Structure

```
/workstream-<name>/
  Design.md
  Pre-Registration.md
  Claims_Register.md
  Successes.md
  Failures.md
```

**Enforced by CI**: `cargo test --test governance_structure`

## Owner/Day Assignment

| Item                    | Owner         | Day | Gate                       |
| ----------------------- | ------------- | --- | -------------------------- |
| C14.1 Frontend chunking | Frontend lead | D1  | LCP < 2.5s                 |
| C14.2 Composition root  | Backend lead  | D1  | `cargo check` clean        |
| C14.3 Tool loop         | Agent lead    | D3  | Mock-lane test pass        |
| C14.4 Streaming         | Backend lead  | D2  | TTFT measured              |
| C14.5 Embedding default | RAG lead      | D4  | BM25 default, local opt-in |
| C14.6 Skill auto-pop    | Skills lead   | D1  | Tree non-empty at boot     |
| C14.7 N lanes           | Config lead   | D3  | Migration test pass        |
| C14.8 Hooks manifest    | Backend lead  | D1  | 23 moments compiled        |
| C14.9 Test matrix       | QA lead       | D2  | Property tests green       |
| C14.10 Data root        | Platform lead | D1  | Portability smoke pass     |
| C14.11 Build script     | Release lead  | D1  | Single script builds all   |
| C14.12 Governance       | All leads     | D1  | CI enforces structure      |

---

## Contributor Deepening — Muse (2026-10-26)

### C14.13 Owner/Day/Gate consistency (depth: the assignment table is a claim)

The Owner/Day table above cites "C16 (verification gates)" in its Open question but C16 is Security/Threat/Compliance — the gates live in C15. Recording the correction without rewriting history [PROPOSED — correction text is my proposal; the misreference is VERIFIED in the current file]: the Open question should read "Each item needs an owner and Day in C15 (verification gates) [C16 → C15 correction] before sprint start." Recommend a C36 correction entry; original line preserved per C03.3.

Frozen assignment rule [PROPOSED — rule is my proposal]:

- Every C14 row gains its Day-gate (G1–G8, C15.9) alongside the artifact check: e.g. C14.4 Streaming → G3/G4 schedule + C15.2 TTFT evidence. A row with an artifact check but no Day gate is unscheduled work — invisible to the sprint plan (C32).
- Owner roles ("Backend lead" etc.) must resolve to named humans before Day 1; role names without names are PROPOSED staffing, not assignments. Unnamed rows are flagged at G1.
- C14.5 embedding default interacts with C09.8 trigger 4 (embed endpoint added/changed → re-run A3): flipping the default after Day 4 invalidates A3 EVIDENCED status. Freeze the default at P2 exit or schedule the re-run.

What was NOT checked here: C32 day assignments vs this table's Day column (verify at C32); whether role names map to the lane roster (C34) or to humans outside it.

## Open questions / gaps

- Each item needs an owner and Day in C16 (verification gates) before sprint start.
