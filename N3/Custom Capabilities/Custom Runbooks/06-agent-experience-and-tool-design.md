---
name: agent-experience-and-tool-design
description: Design progressive-disclosure skills, bounded-output tools, and evidence-cited Propose-Probe-Compose (PPC) harness overlays.
depends_on: [kv-cache-and-continuity]
load_mode: lazy
---

# Agent Experience, Progressive Disclosure & PPC Evolution

Every token injected into a model request must earn its keep in pass rate and cache reuse.

## Progressive Disclosure & Tool Rules

1. **Start With Minimal Context**:
   - Keep root `SKILL.md` files under `4 KB` with a clear procedure, knowledge-source lookup order, and index table; load detailed references on demand.
   - Return bounded tool outputs with stable identifiers (`packet_id`, `hole_id`, file `path`) rather than dumping unpruned logs.
2. **Model Experience Contract**:
   - Every bundle or tool change must document: (a) what the model sees, (b) token cost, and (c) KV-cache prefix impact.
3. **Propose–Probe–Compose (PPC) Evolution**:
   - When evolving skills or harness overlays, separate **INV** (invariant process) and **DOM** (domain procedure) candidates, cite the exact trajectory/receipt moment motivating each edit, and gate adoption strictly on deterministic judges while preserving `KEEP_BASE` abstention.
