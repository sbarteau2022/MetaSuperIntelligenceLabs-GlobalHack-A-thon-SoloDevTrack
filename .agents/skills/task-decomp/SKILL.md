---
name: task-decomp
description: Decompose a user goal into a task tree with explicit information needs per leaf, applying the min-info rule (necessary at depth, not merely sufficient). Use when turning any request into executable leaves before orchestration.
---

# TaskDecomp

Decompose `goal → sub-tasks → leaf actions`, each leaf stating its
information need explicitly. A leaf that can't name what it needs is not
ready to execute — decompose further.

## When to use

Every user query, before any tool runs. No leaf executes without a stated
information set. This is the Meta stage of `ORCHESTRATION-PROTOCOL.md` §1.

## Procedure

1. **Write the goal in one sentence.** If you can't, ask the user (don't guess).
2. **Decompose to leaves.** Each leaf is one verb + one object
   ("retrieve the traverse contract for atom 1142", not "handle traversal").
   Depth: stop when a leaf maps to exactly one tool call.
3. **State each leaf's information need I(L).** Required sub-problems U(L),
   then the minimal set covering them:
   `I*(L) = smallest I with cov(I) ≥ θ AND depth(I) ≥ δ(L)`.
   - `θ` (coverage threshold): default 0.8 — four of five sub-problems
     covered; the fifth is the agent's judgment call, not a lookup.
   - `δ(L)` (depth): reasoning hops required (1 = lookup, 3 = A→B→C chain).
     Set from the leaf verb: retrieve = 1, traverse-chain = hops, synthesize = 2+.
4. **Apply the necessity test.** For each leaf, name the ONE fact whose
   absence voids the leaf. No load-bearing fact = leaf too big — split it.
   This is necessary-vs-sufficient: sufficient lets a good agent succeed;
   necessary means even a perfect agent must fail without it. Ship necessary.
5. **Depth-budget check.** Never pay depth-3 cost for a depth-1 leaf
   (over-retrieval); never starve a depth-3 leaf with depth-1 budget
   (under-retrieval — the `mlir_op#26` failure mode: disconnected facts,
   no connecting path). `map.attend`'s data-dependent `depth_k` is the
   live version of this gate.
6. **Emit the task tree.** Goal, sub-tasks, leaves with (U(L), I*(L), θ, δ,
   necessity fact, assigned tool). This tree is the input to
   `tactical-orchestration`.

## Worked miniature

Query: "why did traverse 1142→1305 fail?"
- Leaf 1 (retrieve): need the traverse receipt for the pair (depth 1,
  necessity = the receipt itself; without it, no diagnosis).
- Leaf 2 (diagnose): need form/energy/frame residuals (depth 1,
  necessity = the `reason` field; "over budget" vs "frame shear" decide
  the fix).
- Leaf 3 (remedy): need the hunger row + budget history (depth 2:
  receipt → ledger, necessity = hits count; 1 hit = fluke, 13 = pattern).

## Failure doctrine

- A leaf with no mappable tool: stop the tree, name the missing capability
  (this becomes a hunger gap — see the Hunger Ledger rules). Do NOT
  approximate with the wrong tool.
- Reference: `ORCHESTRATION-PROTOCOL.md` §2 (min-info rule).

## Native-stack note

Decomposition is a reasoning discipline, not a runtime — it produces the
tree that `tactical-orchestration` executes. The tree itself is data
(JSON at the MCP boundary, native structs inside Rust). Keep the tree
small: leaves map 1:1 to tool calls, so tree size bounds execution cost.
