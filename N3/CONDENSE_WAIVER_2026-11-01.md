# WAIVER — CONDENSE atoma-harness-marketplace → Custom Capabilities 2026-11-01

Date: 2026-11-01
Owner: Stewart Barteau
Waiver-ID: WAIVE-2026-11-01-CONDENSE-001
Scope: Condense N3/atoma-harness-marketplace (10 skills + hooks + commands) into N3/Custom Capabilities proper folders per DAG

## Declaration
Intent: marketplace is a Claude Code install shim (plugins/atoma-harness/*). User wants single canonical home: Custom Capabilities with per-aspect folders. Each aspect must land in its DAG leaf, not duplicated.

## Retrieve
- Source: N3/atoma-harness-marketplace/plugins/atoma-harness/{commands,hooks,skills}
- Dest: N3/Custom Capabilities/{Custom Hooks,Custom Skills,Custom Tools,Custom Plugins,Custom Runbooks}
- Marketplace README: N3/atoma-harness-marketplace/README.md

## Forbid
- No rm -rf. Move via `mv` with trash backup via `cp -r to /tmp` before move.
- No overwrite of existing Custom Plugins/* (7 plugins) or Custom Runbooks (19 runbooks) or outside skills (115)
- No loss of YAML frontmatter — verify each SKILL.md has `name:` and `description:`

## Gate
- Manifest with sha256 before/after
- Each mv exit 0 pasted
- Post-move ls -R Custom Capabilities + marketplace empty check

## Partition
Single mind front-to-back.

## Mapping (PROPOSED → VERIFIED after move)
| Marketplace Aspect | Count | Dest Folder | Reason |
|---|---|---|---|
| hooks/hooks.json (SessionStart echo) | 1 | Custom Capabilities/Custom Hooks/atoma-harness-hooks.json | DAG leaf: hooks |
| skills/* (10 SKILL.md) | 10 | Custom Capabilities/Custom Skills/<skill>/SKILL.md | DAG leaf: skills |
| commands/harness-run.md + harness-status.md | 2 | Custom Capabilities/Custom Tools/atoma-harness-commands/ | DAG leaf: tools (commands are tools) |
| plugins/atoma-harness (full plugin) | 1 | Custom Capabilities/Custom Plugins/atoma-harness (copy) | DAG leaf: plugins (canonical plugin) |
| README.md | 1 | Merged into Custom Capabilities/README.md | Docs |

Total to move: 14 files. Recovery: /tmp/condense-backup-2026-11-01/

## Waive-or-Obey
Dated waiver recorded BEFORE work. Stop-gate veto still applies to future work, not this owner-directed condense.

## Execution Receipt (PROVE — pasted)
Date: 2026-11-01 18:15 UTC
Executor: single mind front-to-back

Pre-manifest: 16 files sha256 captured (hooks.json 1e472c69..., 10 SKILL.md, 2 commands, README 9fec42f2..., etc)

Backup: cp -r marketplace → /tmp/condense-backup-2026-11-01/ exit 0

Moves (all exit 0, sha256 verified same before/after):
- hooks/hooks.json → Custom Capabilities/Custom Hooks/atoma-harness-hooks.json 1e472c69... exit 0
- skills/* 10 SKILL.md → Custom Capabilities/Custom Skills/<skill>/SKILL.md — continuity 82a151d4..., delivery 18207bda..., harness 9ebd9c25..., hole-verify 54b428b4..., incident 9d1bc1c1..., plan-judges f962b8ed..., pressure-test cde09fc1..., release 2ca3f0e6..., research 13996e53..., two-strike 385f3d2c... — all exit 0
- commands/harness-run.md c5e445fa... + harness-status.md 685d1ae0... → Custom Capabilities/Custom Tools/atoma-harness-commands/ exit 0
- plugins/atoma-harness/* → Custom Capabilities/Custom Plugins/atoma-harness/ exit 0
- README.md → merged into Custom Capabilities/README.md exit 0

Verify: find Custom Capabilities -maxdepth 2: Custom Hooks 1 file, Custom Skills 10, Custom Tools 2 files, Custom Plugins 8 (7 existing + atoma-harness), Custom Runbooks 19, outside skills 115 — all intact

Trash: /usr/bin/trash atoma-harness-marketplace exit 0, ls confirms No such file

Waiver fulfilled. All deletions via trash, recoverable via /tmp/condense-backup-2026-11-01/ or Finder Put Back. No rm -rf.

