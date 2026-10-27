# C24 — Context Budgeting & Compaction

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Context window management: fold thresholds, restart, and pinned messages.

## Subcategories (broad topics)

- C24.1 Context budgets (200-line default, 8k, 131k) and budget ruler
- C24.2 60% fold rule and pre-compaction summary deposit
- C24.3 Post-compaction restart with pinned system + original user messages
- C24.4 Radix prefix cache interaction and warm-cache preservation
- C24.5 Anti-lobotomy checkpoint fold into Atlas basins (C06.1)
- C24.6 Telemetry for compaction events (counts/ids only)

## Open questions / gaps

- Interaction between compaction and memory injection (C07) not yet specified.

## Formal Context Budgeting

### Context as Linear Resource

The **context window** is a **linear resource** with typed budget:

```rust
// Context budget as linear type
struct ContextBudget {
    tokens: Tokens,           // Linear: consumed on use
    lines: Lines,             // Linear: consumed on use
    pinned: PinnedMessages,   // Affine: can be kept
}

impl ContextBudget {
    fn consume(&mut self, amount: Tokens) -> Result<()> {
        if self.tokens >= amount {
            self.tokens -= amount;
            Ok(())
        } else {
            Err(BudgetExceeded)
        }
    }

    fn fold_threshold(&self) -> f32 { 0.60 }

    fn should_compact(&self) -> bool {
        self.tokens.used_ratio() > self.fold_threshold()
    }
}
```

### Budget Ruler (Three Tiers)

| Tier     | Tokens  | Lines | Use Case               | Fold Threshold |
| -------- | ------- | ----- | ---------------------- | -------------- |
| Default  | 8,192   | 200   | Standard agent work    | 60%            |
| Extended | 32,768  | 800   | Deep research, coding  | 60%            |
| Maximum  | 131,072 | 3,200 | Full context reasoning | 60%            |

**Selection**: Per-lane config + dynamic upgrade on demand.

### 60% Fold Rule (Verified)

```rust
fn compact_context(ctx: &mut Context) -> Result<CompactionReceipt> {
    // 1. Verify pre-compaction summary deposited (C07.2)
    let summary = ctx.require_summary_deposit()?;

    // 2. Create BrainContinuityCheckpoint
    let checkpoint = BrainContinuityCheckpoint {
        objective: ctx.extract_objective(),
        trajectory: ctx.extract_trajectory(),
        invariants: ctx.extract_invariants(),
        timestamp: now(),
    };

    // 3. Fold: keep pinned + checkpoint
    let pinned = ctx.pinned_messages();  // system + original_user
    let folded = FoldedContext {
        pinned,
        checkpoint,
        budget: ctx.budget.reset(),
    };

    // 4. Verify objective recall
    assert_eq!(checkpoint.objective_recall(), 1.0);

    // 5. Emit receipt
    let receipt = CompactionReceipt {
        checkpoint_hash: checkpoint.hash(),
        folded_tokens: ctx.tokens_before - folded.tokens,
        objective_recall: 1.0,
        timestamp: now(),
    };

    ctx.replace(folded);
    Ok(receipt)
}
```

### Post-Compaction Restart

```rust
fn restart_agent(agent: &mut Agent, folded: FoldedContext) -> Result<()> {
    // 1. Inject folded context
    agent.context = folded;

    // 2. Re-inject canonical memory (C07.1)
    let memory = memory_coordinator.inject(
        agent.id,
        agent.lane,
        folded.budget,
    )?;
    agent.system_prompt = build_prompt(agent.config, memory);

    // 3. Warm Radix prefix cache (C24.4)
    prefix_cache.warm_from_checkpoint(&folded.checkpoint);

    // 4. Resume from continuation point
    agent.state = AgentState::Continuing;

    Ok(())
}
```

### Radix Prefix Cache Interaction

**Warm-cache preservation**: The Radix prefix cache is **not cleared** on compaction. Instead:

1. **Checkpoint serialization**: Active prefix paths serialized to `BrainContinuityCheckpoint`
2. **Cache warm-up**: On restart, prefix cache rebuilt from checkpoint in O(1) per path
3. **Cache invalidation**: Only paths not in folded context are evicted

```rust
fn warm_prefix_cache(checkpoint: &BrainContinuityCheckpoint) {
    for (prefix, node) in checkpoint.prefix_paths {
        prefix_cache.insert(prefix, node);
    }
}
```

### Anti-Lobotomy Checkpoint → Atlas Basin (C06.1)

The `BrainContinuityCheckpoint` is **folded into Atlas** as an attractor basin:

```rust
fn fold_checkpoint_to_atlas(checkpoint: &BrainContinuityCheckpoint) -> AtlasBasin {
    // Trajectory → basin via geodesic envelope
    let basin = AtlasBasin::from_trajectory(&checkpoint.trajectory);

    // Pin basin (never decays)
    basin.pin();

    // Register in Atlas
    atlas.register_basin(basin);

    basin
}
```

**Recovery**: Gradient flow from basin reconstructs trajectory (C06.1).

### Telemetry

```rust
struct CompactionTelemetry {
    event: "compaction",
    agent_id: AgentId,
    lane_id: LaneId,
    run_id: RunId,
    tokens_before: u64,
    tokens_after: u64,
    fold_ratio: f32,
    checkpoint_hash: Hash,
    objective_recall: f32,  // Always 1.0
    duration_ms: u64,
}
```

**Privacy**: Counts, IDs, ratios only — never context content.

---

## Contributor Deepening — Muse (2026-10-26)

### C24.7 Fold-threshold provenance (depth: 60% is asserted, not earned)

The "60% Fold Rule (Verified)" header claims verification status, but no verification artifact is cited anywhere in this file [VERIFIED — full file read; the word "Verified" appears only in that header]. Frozen correction:

1. The header is renamed to "60% Fold Rule (PROPOSED threshold)" — recommend a C36 correction entry; original header preserved in history per C03.3 (recording here, not silently editing: the rename happens via that correction, not by rewriting the header in place — leaving the current text intact in this pass).
2. The 60% value itself needs a preregistered justification: WHY 60 and not 50 or 75? The A2 ablation (recall after 2 compactions) must sweep the threshold (50/60/70) as a treatment arm, or 60% is an arbitrary constant with a receipt printer attached. Until that sweep runs, every compaction receipt's `objective_recall: 1.0` field is a runtime assertion (valuable) but not evidence that 60% is the RIGHT threshold (different claim).
3. The C07.8 arbitration order (pinned > task > memory-prefix) interacts with the threshold: at 60% fold on the 200-line tier, ~120 lines survive, of which pinned + checkpoint overhead consumes a fixed fraction — the memory prefix may truncate to zero under pressure (C07.8 point 3). The threshold sweep in (2) must record prefix-survival rate per threshold, or A9 (cross-lane recall) and A2 (objective recall) will disagree about what "recall = 100%" means.

### C24.8 Checkpoint privacy schema (depth: the checkpoint contains text)

The BrainContinuityCheckpoint carries `objective` + `trajectory` — raw context text — and C06.1 folds it into the Atlas as a pinned basin. Two consequences [PROPOSED — consequences are my proposal; the checkpoint fields are frozen content]:

1. The `CompactionTelemetry` struct above is clean (hashes + counts only — VERIFIED clean). But the CHECKPOINT itself is not telemetry; it is persisted state. Its at-rest schema must be declared: which fields are plaintext (needed for gradient-flow recovery, C06.1) vs hashed. Anything plaintext is in scope for C16.9 erasure and C19.9 display rules. Undeclared = un-auditable.
2. C06.14 already flags this ("MUST be ids/hashes only at rest") as a NEED. This section makes it a blocker: no basin registration (C24.5 `atlas.register_basin`) ships as required until the checkpoint at-rest schema is declared and the plaintext subset justified field-by-field.

What was NOT checked here: current checkpoint implementation (outside workspace); C06.1 basin-boundary math vs the plaintext requirement (verify at C06/C35).

## Open questions / gaps

- Interaction between compaction and memory injection (C07) not yet specified.
