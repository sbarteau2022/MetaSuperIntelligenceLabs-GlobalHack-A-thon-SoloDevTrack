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
