# N3 — Engineering Home

Plain-language guide to this folder. One home for Atoma engineering: rules, code, memory, and tools.

## What this is

`N3` is the single canonical place for engineering work. If it lives outside `N3`, it is not canonical. It holds governance, skills, runbooks, plugins, the local model runtime, and the gateway.

Live stores: Qdrant `research_library_v1` @ `127.0.0.1:6333` (30,550 points / 46,912 vectors, 1024d cosine + lexical sparse) holds all pre-prints, papers, memories — VERIFIED live this session. AtomaDB-RAG-PAMI (19G, 279,611 files per `PRUNE_MANIFEST_2026-11-01.txt:1`) removed from N3 2026-11-01 per `PRUNE_LEGACY_WAIVER_2026-11-01.md` — waiver claims `~/.Trash/N3-PRUNE-2026-11-01/` but Trash is not readable from this shell (Operation not permitted), recovery unverified here — use Finder Put Back.

## Start here

- [`AGENTS.md`](/Users/stewartbarteau/Desktop/N3/AGENTS.md) — how to work in this folder. Read first.
- [`ENGINEERING-GOVERNANCE.md`](/Users/stewartbarteau/Desktop/N3/ENGINEERING-GOVERNANCE.md) — 8 laws: declare, retrieve, forbid, gate, partition, prove, record, waive-or-obey.
- [`MIGRATION_AUDIT_2026-10-31.md`](/Users/stewartbarteau/Desktop/N3/MIGRATION_AUDIT_2026-10-31.md) — what moved, what is duplicate, what still needs owner decision.
- [`brief-tools-fetch.md`](/Users/stewartbarteau/Desktop/N3/brief-tools-fetch.md) — tool audit: backbone vs fail-as-is vs gaps.
- [`PRUNE_LEGACY_WAIVER_2026-11-01.md`](/Users/stewartbarteau/Desktop/N3/PRUNE_LEGACY_WAIVER_2026-11-01.md) — owner waiver + execution receipt for 19G legacy prune (AtomaDB, vault binaries, staging).

## Folders

- [`web-development/`](/Users/stewartbarteau/Desktop/N3/web-development) — Adapter #1. 10 area skills for frontend, backend, middleware, design, testing, data, API, devops, security. Includes gates, templates, docs.
- [`skills/`](/Users/stewartbarteau/Desktop/N3/skills) — 100+ skill packs. Harness judiciary and reusable behaviors.
- [`runbooks/`](/Users/stewartbarteau/Desktop/N3/runbooks) — How work runs. Plan, escalate, verify, deliver, research, release, incident. See [`INDEX.md`](/Users/stewartbarteau/Desktop/N3/runbooks/INDEX.md).
- [`plugins/`](/Users/stewartbarteau/Desktop/N3/plugins) — Domain plugins and compiled MCP tools.
- `core-lm/` — removed from N3 on 2026-11-01 by owner direction. The 466M directory is recoverable at `/Users/stewartbarteau/.Trash/core-lm-N3-2026-11-01` while the replacement execution and memory contracts are designed.
- [`gateway-3p/`](/Users/stewartbarteau/Desktop/N3/gateway-3p) — Local loopback service. Translates Claude Desktop requests to Rapid MLX.
- [`AtomaDB-RAG-PAMI` — REMOVED from N3 2026-11-01, legacy. Old Rust store (19G / 279,611 files). Literature migrated to Qdrant `research_library_v1` (30,550 points, VERIFIED live). Trash recovery path per waiver: `~/.Trash/N3-PRUNE-2026-11-01/AtomaDB-RAG-PAMI` — not verifiable from this shell, use Finder. Do not re-acquire without owner decision.](/Users/stewartbarteau/Desktop/N3/PRUNE_LEGACY_WAIVER_2026-11-01.md)
- [`design/`](/Users/stewartbarteau/Desktop/N3/design) — Specs and gates only. Vault binaries (`.atoma_map`, `.wal`, 421M) trashed 2026-11-01 per waiver.
- [`atoma-harness-marketplace/`](/Users/stewartbarteau/Desktop/N3/atoma-harness-marketplace) — Claude Code plugin: plan/judge discipline, continuity, escalation, release gates.
- [`kimi-frontend-kit/` — REMOVED top-level duplicate 2026-11-01. Canonical: `web-development/kimi-frontend-kit/` (14 gated interfaces + 56 port artifacts). Trash path per waiver: `~/.Trash/N3-PRUNE-2026-11-01/kimi-frontend-kit` — not verifiable from this shell, use Finder.](/Users/stewartbarteau/Desktop/N3/PRUNE_LEGACY_WAIVER_2026-11-01.md)
- [`cognitive-map/`](/Users/stewartbarteau/Desktop/N3/cognitive-map) — Sliced integration context and manifest.
- [`aie-packets/`](/Users/stewartbarteau/Desktop/N3/aie-packets) — Numbered task packets (`AIE-100` upwards).
- [`audit-queue/`](/Users/stewartbarteau/Desktop/N3/audit-queue) — Intake queue for audits and tool migrations. See [`QUEUE.md`](/Users/stewartbarteau/Desktop/N3/audit-queue/QUEUE.md).
- [`qdrant-points-20261101.jsonl` + `qdrant-ingest-20261101.log`](/Users/stewartbarteau/Desktop/N3/qdrant-ingest-20261101.log) — Seed receipt for the live Qdrant store. Do not delete.
- [`AI Research For Ingest/`](/Users/stewartbarteau/Desktop/N3/AI Research For Ingest) — Raw research waiting for ingest.

## Rules for working here

1. State intent in one line before acting.
2. Link canonical source, do not duplicate.
3. Small partitions, one owner front-to-back.
4. Paste checker output, never summarize.
5. Append corrections, never rewrite history.
6. Obey or hold a dated waiver recorded before work.

No bulk deletions. Trash archives stay until owner reviews per-item.
