---
name: hyperbolic-active-mind
description: 12-Basin Hyperbolic Active Mind continuity and specialist-carving contract: checkpoint status/save_note gating, five-domain carve allowlist, checkpoint path constant.
---

# hyperbolic-active-mind (interface)

## Purpose
Brain continuity contract: inspect or append verified milestones to the
anti-lobotomy checkpoint ledger; carve a specialist sub-map only for an allowlisted
domain. Everything-is-a-plugin stays a procedure; compiled behavior is the
action/domain/note contract.

## When to use
Brain checkpoint status, milestone appends, specialist carving for
coding_intensive / security_cyber / physics_math / legal_statutory /
medical_clinical. Pre-compaction checkpoint + pinned messages[0..1] keep the
Radix prefix cache warm.

## Out of scope
Live `:8088` round-trips and checkpoint file I/O (host-runtime, outside the
tool subset — pure decision/default layer only). `atoma-brain__checkpoint-status`
behavior is preserved by the Wave-1 port at
`N3/plugins/mcp-tools/checkpoint-status/` (already migrated, not duplicated here)
— VERIFIED present. `atoma-brain__carve-specialist` and `atoma-brain__dream-consolidate`
have NO callable schema anywhere in N3 (grep VERIFIED zero hits across N3 + MCP
configs) — listed as removed/non-ported claims in RECEIPT, procedure text retained.

## Type signature
Compiled gate: `tool.rs` (`brain-action-gate`, schema: `tool.json`).
- `brain_action_valid(action: String) -> bool` — "status" | "save_note"
- `brain_domain_valid(domain: String) -> bool` — five-domain allowlist
- `brain_gate(cfg: BrainAction) -> bool` — action AND domain valid
- `save_note_ok(action: String, note_present: boolean) -> bool` — save_note requires non-empty note
- `make_brain(action, domain, note_present) -> BrainAction`

## Source
Trash source: `skills-desktop-plugins-20261030/atoma-brain/skills/hyperbolic-active-mind.md`
(`e82f2872986279fffa04896730c86c4a97d5f74548bd5db124443863edbb0653` — see RECEIPT for exact sha). Shrunk to this interface
2026-10-31 (Agent A2). See `RECEIPT.md` for removed-claim proof.
