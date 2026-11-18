# Kimi Frontend Kit — Roo Cline scaffold

Source kit, do not copy it into the project — point at it:
`../N3/web-development/kimi-frontend-kit/`

This file is loaded by Roo Cline from `.roo/rules/` on every session in this
repo. It makes the kit's frontend tools available to the Cline agent as
rules, checklists, and gate scripts — not as MCP servers. The kit has no MCP
server, so nothing was added to `.roo/mcp.json`.

## The 5-stage loop (from `workflows/KIMI_OK_COMPUTER_LOOP.md`)
1. Brief read + 3-dial calibration
2. Token spec (OKLCH)
3. Subagent decomposition (`subagents/subagent_definitions.json`)
4. Zero-stub implementation — no dead clicks, every control mutates live state
5. Browser vision QA

## Skills in the kit (13)
Each skill is a `SKILL.md` + `tool.json` + `tool.ts` + `tool.rs` + `RECEIPT.md`
in `../N3/web-development/kimi-frontend-kit/skills/<name>/`. Read the SKILL.md
before using a skill; run its tool gate before shipping anything it built.

- brutalist-frontend
- cortex-react
- cortex-typescript
- design-system — tokens, slides, validators in `scripts/`
- design-taste-frontend
- frontend-design
- frontier-stack-2026
- high-end-visual-design
- ide-dockview-editors
- kimi-frontend-orchestrator — the QA gate: calibration done, types pinned,
  zero-stub, browser verified, 5 interaction states per primitive
- minimalist-frontend
- ui-styling
- ui-ux-pro-max
- webgl-r3f-graphs-dags

## Templates
`templates/package.json`, `templates/tsconfig.json`, `templates/vite.config.ts`
— start a new frontend from these, not from a blank Vite scaffold.

## Hooks (manual, not auto-run by Roo)
`hooks/hooks.json` is Claude-format (SessionStart / PreToolUse / PostToolUse /
Stop). Roo does not auto-run it. Run the scripts by hand at the same points:
- `hooks/scripts/session-init.sh` at session start
- `hooks/scripts/pre-tool-guard.sh` before write/edit
- `hooks/scripts/post-edit-verify.sh` after write/edit
- `hooks/scripts/audit-frontend.sh` for a full audit
- `hooks/scripts/stop-completion-gate.sh` before finishing

## Atoma overrides — these win over the kit's generic guidance
- Palette lock: slate (#26262b / #2e2e33 / #34343a, hairlines #45454d),
  white + amber #f59e0b main, grey #71717a secondary only. Cyan/blue/purple
  out. Do not let the kit's generic token spec override this on Atoma work.
- Site source for the current Atoma site is the Framer export + marketing
  copy in this Ares workspace — not a React clone. The Nemotron React
  clone (`src/components/dispatch`, `src/framer`, `public/clone`) was trashed
  2026-11-04 and must not be rebuilt on a guess.
- Atoma Global Engineering Law in `.roo/rules/rules.md` still governs:
  quality over speed, gates before ship, no synthetic ships, deletions to
  Trash, landed means live in the running app.
