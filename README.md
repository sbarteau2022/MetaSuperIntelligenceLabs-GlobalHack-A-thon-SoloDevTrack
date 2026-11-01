# Meta Super Intelligence Labs — Global Hackathon Design (Solo Dev Track)

This repository holds the design for an enterprise-grade agentic system called **Ares SpecOps
Agentic Lab**. It is a design-first repository: every component is specified, traced to a
verification gate, and tied to a preregistered ablation before any code is written.

The goal is simple. A single developer, working alone on one Apple Silicon machine, should be
able to ship a coherent agentic system — agent lanes, retrieval and memory, workflow and hole
orchestration, multi-agent communication, telemetry, recovery, security, and compliance — with
the same discipline a larger team would use.

## What is in this repository

- `MasterDesignDocument/` is the main design index. It contains 35 numbered category folders,
  `C01` through `C35`, each with a `Design.md` and `Rules.md`, plus a condensed decision log
  (`Decisions.md`) and a list of verification commands (`VerificationCommands.md`).
- `PreRegistrations/` is a top-level folder of preregistered ablations. No ablation runs
  without a filed hash. Per-registration status is tracked in that folder's `README.md`.
- `MasterTooling/` (pinned at the repo root) certifies the tools required by each necessary
  skill. It contains one record per skill (`ToolingC01` through `ToolingC07`), 35 tools in total.
- `MasterSkills/` (pinned at the repo root) defines the seven necessary skills and proves why
  no additional skill is required for the sprint.

Read `MasterDesign-Readme.md` (pinned at the repo root) for the same high-level entry point.

## How the design is organized

The design is contract-first. Each category names what it owns, which verification gate it
must pass, and which ablation exercises it. Nothing ships on prose alone: every claim must
carry a label (VERIFIED, REPORTED, INFERRED, or PROPOSED), and every success is recorded
alongside its failures in the five-file set (Design, Pre-Registration, Claims Register,
Successes, Failures).

The seven necessary skills, and the tools that enable them, are:

1. Formal specification and model checking.
2. Systems programming and kernel fusion (Rust, MLX, Metal).
3. Agentic runtime orchestration (skills, tools, lanes, brigades, holes).
4. Observability and telemetry engineering.
5. Verification and ablation framework engineering.
6. Data and knowledge engineering (retrieval, ingest, documentation, governance).
7. Security, sandboxing, and compliance engineering.

## Where to start reading

1. Read this `README.md` for the plain-language overview.
2. Open `MasterDesignDocument/C01-Scope-Goals-Non-Goals/Design.md` for the problem statement
   and success definition.
3. Walk `C01` through `C35` in order; each folder links to its gates and ablations.
4. Check `MasterSkills/SkillsC1.md` and `MasterTooling/README.md` for necessity and tools.
5. Check `PreRegistrations/` for ablation status before any run.

## Status

This is a PROPOSED design scaffold under finalization. Categories range from FROZEN
(invariants and contracts) to scaffolded detail pending. A final audit receipt is recorded in
`AUDIT-RECEIPT.md`. The design will be updated as the build progresses; this README will be
updated alongside it.

## Important constraints

- Hardware target: Apple Silicon, 36 GB or more of unified memory, Metal 3.0+.
- Local-only: no external API dependencies for the core loop.
- Model: Qwen3.8-27B-Uncensored-OrcaRouter-MLX-4bit served by Rapid-MLX on port 8001.
- Context window: 131,072 tokens maximum, 200-line default budget.

---

_Last updated: 2026-10-26 (finalizing pass)._

## External HyperbolicMap/KV plugin (2026-10-31)

The active harness plugin source is now in the sibling folder
`../hyperbolic-map-nested-kv-plugin`. `research-grade-harness::MapPlugin`
loads its external MCP executable only after checking map/KV tools and health.
The N3 `hypermind-nested-kv` tree remains as a legacy development snapshot and
observability home while its other consumers are migrated. The external plugin
smoke and harness test suite pass; model reasoning and transformer attention KV
replacement are not established by those tests.

## N³ traceability and observability base (2026-10-29)

The [SQLite foundation](hypermind-nested-kv/observability/README.md) covers the
whole system: meta decisions, cost-gated brigade activation, models, capabilities,
MCP/plugins/skills, execution and acceptance, DAG/knowledge graphs, resource
usage, KV, atoms, novelty/reranking, security, research evidence and recovery.
The local base is initialized; live runtime adapters remain PROPOSED.
Schema, constraints, validation evidence and operating instructions live beside
the versioned SQL under `hypermind-nested-kv/observability/`.

## Local work publication — 2026-10-29

This source snapshot also includes the local harness mission CLI and native-name
registry, optimizer scheduling changes, PAMI rotation wiring, coherence/structured
egress modules, frontier design notes and N³ integration/fallback specifications.
Publication preserves development work; it is not a production release or proof
that the fixed PPC hardware gate or live SQLite adapters have passed.

Pre-push validation on this Mac: `cargo test --locked` passed in
`research-grade-harness` (42 tests) and `hypermind-nested-kv` (48 tests);
the SQLite suite passed 54 tests. The native build reports existing unused-code
warnings. See `research-grade-harness/HARDWARE-GATE-RUNBOOK.md` for the remaining
real-model hardware gate; synthetic/unit fixtures do not satisfy it.

Review limitation (2026-10-26): the mission resume CLI accepted recognized packet IDs
from receipt lines without validating their outcome/evidence; that resume path was
not verified acceptance. Corrected 2026-10-29: `rgah resume` now trusts only
`receipt packet=<id> ... outcome=PASS` lines for packets in the mission (bare ids,
summary lines, and non-PASS outcomes are ignored and counted on stderr), and the
`resume --help` text states the path is operator convenience, not acceptance proof.
Corrections are entries, not edits: the 2026-10-26 limitation above stands as the
record of what was shipped; the 2026-10-29 fix changes behavior going forward.

FW-A1-CUDA result status (2026-10-29): `ablation_results/FW-A1-CUDA/` holds only a
`STATUS.md` disposition marker — no `claim.json`, no judge ratios, no CUDA execution
(DRAFT pre-reg, no filed hash; CUDA absent on this Mac). Scaffold trees
`ablation_benchmarks/{cf_workers,go_wasm,metal_rust,ts_frontend,zig}/src/` are empty;
`ts_frontend` has no code pending the TS 7.1 shootout per `TS-CLEARANCE.md`.
