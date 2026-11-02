# Tooling consolidation manifest — 2026-11-02 (tooling agent)

Home base: `~/Desktop/N3/Custom Capabilities/` (Desktop N3 staging only).
Staging folder: `~/Desktop/N3/Custom Capabilities/staging/`
Future home `n3/hypermind` (clean no-sim Rust build, real graph + real nested KV):
NOTHING was staged there. Migration and Rust/TypeScript 7.1 conversion come later.
No servers touched (SurrealDB 8000, oMLX 8002 untouched). Nothing committed.
Method: sha256 content hash per file; dedupe by hash, not name alone.
Move rule: verify present-at-destination AND gone-from-source before the next move.
No move was rolled back, because no move was attempted (see below).

## Counts

- Inventoried outside the home: ~245 files plus 9 trees/dirs (see Held).
- Already home (canonical, not moved): 22 files.
- Moved into staging: 0 files.
- Confirmed exact duplicates found: 22 files (19 runbooks + 2 commands + 1 hooks file).
- Held: everything inventoried outside the home, for the reasons in the table.

## Why moved = 0 (honest result)

Every tooling/hook/rule/runbook candidate outside the home falls into exactly one
of these classes, and moving any of them would break a live loader or gut a repo:

1. Exact duplicate of something already home (Cline runbooks).
2. Live-loaded config (Cline globals — moving unloads them from Cline).
3. Repo-embedded code or design records (moving guts the repo).
4. Whole-kit tree (moving relocates a tree, it does not condense it).
5. Live launcher or possibly-live script (moving breaks the thing that calls it).

The consolidation already happened, in effect, by an earlier copy cycle: the
canonical copies live in the home. What remains is a decision, not more moving:
Stewart decides whether the Cline duplicate copies may be removed now that the
home copies are canonical. No content is lost either way.

## Already home (canonical copies, do not move)

- `Custom Tools/atoma-harness-commands/harness-run.md` — c5e445fa9d2b
- `Custom Tools/atoma-harness-commands/harness-status.md` — 685d1ae08a96
- `Custom Hooks/atoma-harness-hooks.json` — 1e472c6951fc
- `Custom Runbooks/` — 19 .md files (01–12 runbooks, 2026-09-14 controller,
  INDEX, README, engineering, landing-page-checklist, swarm-phase0-prereg,
  web-design). Full hash list was verified identical to the Cline copies.

## Dedupe findings (verified by sha256, 2026-11-02)

1. `~/Documents/Cline/Workflows/` (19 .md files) is BYTE-IDENTICAL, file for
   file, to `Custom Runbooks/` (19 .md files). Same names, same hashes.
   Canonical: the home copies. Cline copies: duplicates, held (live-loaded).
2. `Custom Tools/atoma-harness-commands/harness-run.md` and
   `harness-status.md` are BYTE-IDENTICAL to
   `Custom Plugins/atoma-harness/commands/` of the same names.
   Canonical: the Custom Tools copies. Plugin copies: plugin-embedded, held.
3. `Custom Hooks/atoma-harness-hooks.json` is BYTE-IDENTICAL to
   `Custom Plugins/atoma-harness/hooks/hooks.json`.
   Canonical: the Custom Hooks copy. Plugin copy: plugin-embedded, held.
4. This is the "recreated again" pattern in miniature: the same items were
   copied into the home, the plugin tree, and the Cline globals, and have
   since drifted apart as separate copies. No copy was found to differ in
   content — all checked copies are still identical.

## Held (not moved), with reason

| Item | Files | Reason held |
|---|---|---|
| `~/Documents/Cline/Workflows/` | 19 | Live-loaded Cline globals; exact dup of home runbooks |
| `~/Documents/Cline/Hooks/` | 195 | Live-loaded Cline globals; whole kit (compiled/, scripts/) |
| `~/Documents/Cline/Rules/` (AGENTS.md c39efcb2be4c, ENGINEERING-GOVERNANCE.md dcd17f42e1ec, global-engineering-law.md 54e662a2a300, opendesign-global-rules.md 91bffb8a978a) | 4 | Live-loaded Cline globals |
| Hackathon repo `MasterTooling/` (ToolingC01–C07 + INSTALLED-TOOL-CENSUS, README, TOOLING-AUDIT, TOOLING-CONDITIONS-DISPOSITION) | 11 | Repo-anchored design records; moving guts the repo |
| Hackathon repo `MasterDesignDocument/C22-Failure-Modes-Runbooks`, `C28-Skill-Registry-Hooks`, `C29-Tool-Registry-Code-Intel`, `MasterToolingIndex` | 4 dirs | Repo design docs |
| Hackathon repo `.roo/rules`, `.roo/rules-merge-resolver`, `.roo/rules-skill-writer` | 3 dirs | Repo Roo config; skills-adjacent, left for the skills agent |
| `hypermind-nested-kv/tools/` (embed_bgem3.py, load_text_embeddings.py) | 2 | Repo-embedded code (embed pipeline); do not touch hypermind |
| `Elle/tools/` (elle-mcp, ollama) | 2 dirs | Repo-embedded |
| `elle-worker/tools/` | 0 files (empty dir) | Nothing to move |
| `~/Desktop/N3/web-development/` incl. hooks/hooks.json, kimi-frontend-kit/hooks/hooks.json | whole tree | Whole-kit tree; candidate for later conversion, not staging |
| `~/Desktop/N3/gateway-3p/` | tree | Third-party gateway (vendor) |
| `~/Desktop/rapid-up.sh`, `~/Desktop/OpenDesign-fixed.command` | 2 | Live fleet launchers |
| `~/Desktop/N3/AGENTS.md`, `AGENTS.md.bak.20261101` | 2 | N3 root live rules; agents read AGENTS.md at project root |
| `~/Desktop/N3/mcp-config-backup-2026-10-30-21servers.json`, `-37servers.json` | 2 | Config backups, not tooling |
| `~/Desktop/N3/status-line-gateway.sh` (23 lines, Claude status line for gateway-3p) | 1 | Referenced in Claude history; possibly a live status-line command — held pending Stewart confirmation |

## Not touched

- Skills dirs (the parallel skills agent owns them; its staged items and
  `MANIFEST-skills-2026-11-02.md` in staging/ were left alone).
- `n3/hypermind` / `hypermind-nested-kv` as a destination: nothing staged there.
- `AGENTS.md`, `README.md` anywhere: not edited. Nothing committed.
- SurrealDB (8000), oMLX (8002), any server process: not touched, not restarted.

## Decisions needed from Stewart

1. Cline duplicate runbooks: remove the 19 Cline Workflows copies now that the
   home copies are verified canonical? Doing so unloads Cline global runbooks
   until Cline is re-pointed. Content is safe either way.
2. `status-line-gateway.sh`: move it into staging, or leave it at N3 root
   because a status-line command may still call it by that path?
3. `web-development/` kit tree: leave as a whole tree for the later
   Rust/TypeScript 7.1 conversion, rather than staging it piecemeal?
