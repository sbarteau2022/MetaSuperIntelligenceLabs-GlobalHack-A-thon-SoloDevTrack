---
name: kv-cache-and-continuity
description: Preserve radix prefix cache alignment, minimize Landauer thermodynamic token waste, and manage 60% context checkpoints.
depends_on: [runbook-plan-and-judges]
load_mode: lazy
---

# KV-Cache Physics, Radix Prefix Stability & 60% Continuity Checkpointing

Long-horizon agents thrive across 200+ steps only when marginal prefill cost per branch approaches zero.

## Five Binding Cache & Continuity Rules (Spec 15)

1. **Stable First, Volatile Last**:
   - Keep system identity, persona, canonicalized tool schemas, and eager skills anchored at the shallow prefix root (Tier 1, leak rate `0`).
   - Place volatile per-turn context (`cwd`, hole root, timestamps, file snippets) at the tail of the prompt so the shared radix prefix stays warm across branches.
2. **Path-Dependent Cache Identity**:
   - Every KV block identity is `H(parent_key, token_block, model_id, tokenizer_id, quantization_id)`. Never mix prefixes across incompatible model/quantization lanes.
3. **60% Context Window Checkpointing**:
   - Before reaching 88% auto-compaction, fold middle turns at ~60% capacity into a `BrainContinuityCheckpoint` while keeping the system prompt and original goal pinned.
   - Deposit a canonical 5W summary (`who`, `what`, `where`, `when`, `why`) into `mind.submit` at the end of every major milestone.
