# Swarm phase 0: role-scoped sub-agents on one set of weights

Written 2026-10-07 before any run. Bars below are fixed now and are not moved after results.

## What is built
`spawn_agent(role, task)`: a coordinator delegates a self-contained task to a sub-agent of a role (coder, researcher,
analyst, librarian, writer). Each role has a fixed tool subset and a byte-stable preamble. All sub-agents use the same model
server (one set of weights) and its prefix cache. No new weights, no adapters, no model changes. Several `spawn_agent`
calls in one response run concurrently (at most 4 at once).

## Claims and bars
- **P1 prefix sharing.** For three sequential same-role spawns with different tasks, the server's prefix cache restores at
  least 70% of the prompt tokens of the first step of spawns 2 and 3 (read from `Prefix cache restore ... cached=C suffix=S`
  in the server log; share = C / (C + S)).
- **P2 correctness.** On the three-part task set below, the swarm arm answers all three parts correctly in at least 2 of 3 runs.
- **P3 not worse.** The swarm arm's correct-part count is at least the single-agent arm's in the same runs (paired by run).
- **Reported, no bar:** wall-clock of each arm and the speedup; concurrent versus sequential time for 3 analyst tasks
  (the server measured ~1.4x at two concurrent requests, so no speedup is promised).

## Task set (checkable, fixed)
1. analyst: exact sum of 40 numbers in `data.txt` (numbers generated from the run's timestamp; expected sum computed by the probe).
2. librarian: first author of "Linear Transformers Are Secretly Fast Weight Programmers", from the knowledge base. Expected: "Schlag".
3. researcher: fetch https://example.com and report the page's main heading. Expected: "Example Domain".

## Arms
- **S (swarm):** the coordinator (full belt plus `spawn_agent`) is asked to do all three and may delegate.
- **B (single agent):** the same coordinator without `spawn_agent` and without the delegation guidance.
Same model, same thinking mode (Auto), same task wording. n = 3 paired runs.

## Failure handling
A harness fault (crash, tool unavailable) is fixed and the run repeated with every bar unchanged; at most two cycles.
A bar missed is reported as missed.
