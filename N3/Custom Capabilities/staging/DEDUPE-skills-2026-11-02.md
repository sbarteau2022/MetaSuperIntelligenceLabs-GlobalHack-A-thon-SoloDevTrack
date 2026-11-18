# Skills dedupe receipt — 2026-11-02

Order: there should be only ONE set of anything right now. Dedupe and repoint.
Scope: skills only. Home base: ~/Desktop/N3/Custom Capabilities/
Method: sha256 verified before every move. Duplicates moved to Mac Trash ONLY
(mv to ~/.Trash) — nothing permanently deleted, nothing removed with rm.
Canonical copies were never moved. Servers untouched: SurrealDB pid 1709 on 8000,
oMLX pid 9681 on 8002. No AGENTS.md or README.md edits, no commits.
No changes to hypermind-nested-kv, CapabilityTreeSitter, or vendor skills.

## Counts

- Duplicate dirs removed to Trash: 12
- Symlinks created to canonical copies: 12
- Canonical skill copies verified intact after the run: 11
  (staging/run-workflow + 10 Custom Skills items)
- Failures / rollbacks: 0

## Set 1 — run-workflow (canonical: staging/run-workflow)

Canonical SKILL.md sha256:
82ed7c2e93d79fd95a01188c78539b6987f4c2a2dfc49c3b8991bebbe8925bf8

Removed to Trash, replaced with a symlink to the canonical dir:
1. ~/Desktop/GitHub/Dynanic-Hyperbolic-Neural-Graph/skills/system/run-workflow
   -> Trash: ~/.Trash/run-workflow-DHNG-dedupe-20261102
   -> Symlink: same original path -> staging/run-workflow
   Note: this copy was NOT git-tracked in its repo.
2. ~/Desktop/GitHub/Rust-TS-Tokio-Inference-Engine/skills/system/run-workflow
   -> Trash: ~/.Trash/run-workflow-RustTS-dedupe-20261102
   -> Symlink: same original path -> staging/run-workflow
   Note: this copy WAS git-tracked (skills/system/run-workflow/SKILL.md).
   Its repo working tree now shows the path as a symlink to the canonical
   copy. Content is unchanged and recoverable from Trash.

Both duplicates were hash-identical to the canonical copy before the move,
and the hash read through each new symlink after the move matched exactly.

## Set 2 — atoma-harness skills (canonical: Custom Skills)

Canonical = Custom Skills/<name>/SKILL.md, duplicate = Custom Plugins/atoma-harness/skills/<name>.
Each duplicate dir contained only SKILL.md and was tree-hash identical to the
canonical dir before the move. Each was moved to Trash and replaced with a
symlink to the canonical dir, so the plugin still resolves its skills and
there is only one set of CONTENT.

Canonical SKILL.md sha256 values (verified intact after the run):
- continuity:    82a151d4bd14a673dd879bf2027c74243418f4c394947e2f5afd5b75ebd4b7b0
- delivery:      18207bda579bf6a3df8165aeb333d9d47d6c90a4570a6050c064b141a96fefd1
- harness:       9ebd9c25ceb3821ff805a0741309e35c6be226c820bc583cae6e95e4f693ea70
- hole-verify:   54b428b4cc5be039c185f6ea7baf4135c06e1e990e16f429b91809aa543218a8
- incident:      9d1bc1c1b3558d1a4376953a9b3895c8e961b0a213a627366f5c6a2930bf7d63
- plan-judges:   f962b8edf456d0b0d858bdb807b25e260d1b5429b143ad554d280ce43d8e33c4
- pressure-test: cde09fc1209a0f9ba93fa51b951acfaa49b7bde06a00166b8b4e081d9bff0934
- release:       2ca3f0e6422bef41391f90490087613b89843fdee86fcbfaa2ab9da82fb99588
- research:      13996e530054d1228b3a985009aa9b6eeb11fcade47b645c801c33961472de88
- two-strike:    385f3d2cc18f6216e3d467b2d4506d7f8ff1f71b5ae7b605e20768d1b5ec9ba1

Trash destinations: ~/.Trash/<name>-atoma-plugin-dedupe-20261102 for each of
the 10 names above. Symlink target for each: Custom Skills/<name>.
Hash read through each new symlink matched the canonical hash exactly.

## Sets left alone, with reason

- Vendor/plugin skills: ~/.claude/skills (26 items incl. bailian-*),
  Desktop .agents/skills and .gemini/skills (empty) — out of scope per order.
- Whole tree: CapabilityTreeSitter (709 dirs) — design/reference tree, not a
  verified duplicate set; moving it would gut the repo.
- Kit-embedded: N3/web-development/kimi-frontend-kit/skills (14 items) —
  embedded in the kimi kit, not verified as a duplicate set against the home.
- Tool-managed repo skills: hackathon repo .agents/skills (4),
  EthicalIntelligenceProject .agents/skills (4), RAPIDAi .agents/skills (1) —
  not verified hash-identical to any home copy; moving them would break repo
  agent configs without a canonical target.
- Tool rules (.roo/rules-skill-writer x3) and design docs (MasterSkills/,
  MasterSkillsIndex, C28) — not standalone SKILL.md skills.
- The 4 newly staged skills (run-workflow, geodesic-ring-meta-gate,
  hyperbolic-lorentz-cpp20-kernel, polyglot-cwe-entropy-auditor) — verified
  unique against Custom Skills and outside skills; no duplicates found.
- outside skills (115 entries) — no hash-identical duplicate set against the
  home copies was found in the manifests or this run.

## Verification after the run

- Canonical staging/run-workflow SKILL.md hash unchanged: 82ed7c2e...8925bf8.
- All 10 Custom Skills SKILL.md hashes unchanged (listed above).
- All 12 Trash dirs present under ~/.Trash with -dedupe-20261102 suffixes.
- All 12 original paths now exist as symlinks and resolve to readable files.
- SurrealDB still listening on 8000 (pid 1709), oMLX still on 8002 (pid 9681).
