# Skills staging manifest — 2026-11-02

Home base: `~/Desktop/N3/Custom Capabilities/` (Desktop N3 — staging only).
Future home `n3/hypermind` (clean no-sim Rust build, real graph + real nested KV
for forward-pass tensors) was NOT touched; migration comes later.
Servers untouched: SurrealDB pid 1709 on 8000, oMLX pid 9681 on 8002.
No AGENTS.md / README.md edits, no commits. No deletions anywhere in this run.
Staging folder already existed (created 08:29 CST by an earlier stopped run,
verified empty) and was reused, not recreated.

## Counts

- Standalone skill dirs found outside the home base: 6
- Moved into staging (verified: present at destination, gone from source,
  tree hash unchanged): 4
- Deduped — identical duplicate copies, NOT staged, NOT deleted: 2
- Held (listed below, not moved): see Held section
- Failures / rollbacks: 0

Home-base reference (already in place, not moved):
- `Custom Skills/`: 10 items
- `outside skills/`: 115 entries (100 dirs + 15 loose files)

## Moved (source -> staging/, tree hash = sha256 over relative paths + file contents)

1. `run-workflow`
   - From: `~/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/skills/system/run-workflow`
   - To: `Custom Capabilities/staging/run-workflow`
   - Files: 1 (`SKILL.md`, 10331 bytes)
   - SKILL.md sha256: 82ed7c2e93d79fd95a01188c78539b6987f4c2a2dfc49c3b8991bebbe8925bf8
   - Tree hash: 1fb714e2f50cf49aea58dae473009cd7feecf26a5c9cb733f4876febfeb45fa4

2. `geodesic-ring-meta-gate`
   - From: `~/Desktop/N3/design/autonomous_skills/geodesic-ring-meta-gate`
   - To: `Custom Capabilities/staging/geodesic-ring-meta-gate`
   - Files: 7 (SKILL.md, RECEIPT.md, skill.atoma_skill, tool.json, tool.rs, tool.ts, geodesic_ring_and_meta_gate.rs)
   - Tree hash: d438bb40494e636d1dd581726a74246c0e27a66c40aec3250da5347a9f9c5895

3. `hyperbolic-lorentz-cpp20-kernel`
   - From: `~/Desktop/N3/design/autonomous_skills/hyperbolic-lorentz-cpp20-kernel`
   - To: `Custom Capabilities/staging/hyperbolic-lorentz-cpp20-kernel`
   - Files: 7 (SKILL.md, RECEIPT.md, skill.atoma_skill, tool.json, tool.rs, tool.ts, hyperbolic_lorentz_kernel_cpp20.cpp)
   - Tree hash: 37945dc7dc146548cc68b2f264dc164e5224dcaa6641ebcae0a7e74a505947ed

4. `polyglot-cwe-entropy-auditor`
   - From: `~/Desktop/N3/design/autonomous_skills/polyglot-cwe-entropy-auditor`
   - To: `Custom Capabilities/staging/polyglot-cwe-entropy-auditor`
   - Files: 7 (SKILL.md, RECEIPT.md, skill.atoma_skill, tool.json, tool.rs, tool.ts, polyglot_cwe_and_entropy_auditor.py)
   - Tree hash: 7681cf2871ef8bf27af158b79316999d744a9662d96829aeba74ebb6ee40de83

All four were verified unique against every dir in `Custom Skills/` and
`outside skills/` (no tree-hash match) before moving.

## Deduped (identical content, one canonical copy staged, duplicates left in place)

- `run-workflow` — SKILL.md sha256 82ed7c2e93d79fd95a01188c78539b6987f4c2a2dfc49c3b8991bebbe8925bf8,
  byte-identical to the staged canonical copy. Left in place, NOT deleted, NOT staged twice:
  - `~/Desktop/GitHub/Dynanic-Hyperbolic-Neural-Graph/skills/system/run-workflow`
  - `~/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/skills/system/run-workflow`
- `Custom Skills/` 10 items (continuity, delivery, harness, hole-verify, incident,
  plan-judges, pressure-test, release, research, two-strike) are byte-identical
  (same tree hashes) to `Custom Plugins/atoma-harness/skills/` — already deduped
  copies already inside the home base. No action taken.

## Held (not moved, with reason)

- Whole tree: `CapabilityTreeSitter/` in the hackathon repo (709 dirs at depth <= 2)
  — design/reference tree, moving it would gut the repo. Held for a separate decision.
- Plugin-embedded: `Custom Plugins/atoma-harness/skills/` (10 items) — held; identical
  copies already live in `Custom Skills/`.
- Kit-embedded: `~/Desktop/N3/web-development/kimi-frontend-kit/skills/` (14 items:
  brutalist-frontend, cortex-react, cortex-typescript, design-system,
  design-taste-frontend, frontend-design, frontier-stack-2026, high-end-visual-design,
  ide-dockview-editors, kimi-frontend-orchestrator, minimalist-frontend, ui-styling,
  ui-ux-pro-max, webgl-r3f-graphs-dags) — held; embedded in the kimi kit.
- Vendor/plugin: `~/.claude/skills/` (26 items, incl. bailian-*) — held; moving breaks plugins.
- Vendor/plugin, empty: `~/Desktop/.agents/skills/` (0), `~/Desktop/.gemini/skills/` (0) — nothing to move.
- Tool-managed inside repos (held; moving breaks repo agent configs):
  - hackathon repo `.agents/skills/` (4: allocating-weights, swarm-brigade, tactical-orchestration, task-decomp)
  - `EthicalIntelligenceProject/.agents/skills/` (4: add-blog-post.md, add-component.md, add-page.md, styling.md)
  - `RAPIDAi/.agents/skills/` (1: cloudflare-rapidai)
  - `GrantIntelligence/.agents/skills/` (0), `GrantIntelligence/.gemini/skills/` (0)
- Tool rules, not SKILL.md skills (held):
  - repo `.roo/rules-skill-writer`, `~/Desktop/N3/.roo/rules-skill-writer`,
    `~/Desktop/Ares WebSite : App/.roo/rules-skill-writer`
- Design docs, not standalone skills (held; moving would gut repo docs):
  - hackathon repo `MasterSkills/` (README.md + SkillsC1.md, frozen taxonomy doc)
  - `MasterDesignDocument/MasterSkillsIndex`, `MasterDesignDocument/C28-Skill-Registry-Hooks`

## Failures

None. Every move verified before the next move started. No rollback needed.
