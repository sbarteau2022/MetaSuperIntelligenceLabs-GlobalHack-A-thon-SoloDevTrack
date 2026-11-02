# Swarm Loop — the one runbook for long-horizon agentics

**Status.** CANONICAL working doc (2026-10-30, Stewart-directed). Compiled from 12 audited slots (§12 below) + the SwarmBrigade master skill. **English-only.** Bound by the engineering governance core: DECLARE, RETRIEVE, GATE, PARTITION, PROVE, RECORD, WAIVE-OR-OBEY.

**What it is.** The single loop for running 100+ step tasks as continuous batches: decompose once, admit waves of fibers, certify each wave, pressure-test across waves — **without ever setting the run down**. Blocked-continuous batching: wave N+1 is admitted while wave N's receipts are still being certified; the run never stops, only waves turn over.

**What it is not.** Not 12 separate runbooks. Not a daemon (no continuous-mode transport exists; waves run as bounded batches back-to-back). Not a second dispatcher (admission, execution, and gating live in `runbook.rs` / `engine.rs` / `gate.rs` — this doc is their operating order, not a replacement).

---

## 0. Calibration (DECLARE — say this before every run)

One line: goal, wave size, budget per wave, hardware, baseline path. Example: "Atlas audit, waves of 5 fibers, 40 packets/wave, Mac M3MAX, baseline = merge-base commit." No calibration line, no run.

---

## 1. The loop (one wave)

```
DECOMPOSE (once per run)
  → PREREGISTER (once per run; per-wave deltas appended, never rewritten)
  → ADMIT wave (ready_packets: Pending + depends_on all Passed + lane open + budget allows)
  → EXECUTE (capability-gate → call → judge → receipt, per call)
  → CERTIFY wave (meta recounts, reruns, greps; fiber PASS = REPORTED until checked)
  → PRESSURE-TEST (suites green; git clean of fiber writes; no silent loss)
  → CONDENSE (checkpoint, prune, verify same-next-action) if at step boundary
  → ADMIT next wave (no teardown, no restart of the run)
```

Waves overlap: certification of wave N runs while wave N+1 executes. The run ends when no packets remain Pending/Running AND the final pressure test is green — not when a wave finishes.

## 2. Admission (GATE — contracts before fan-out)

A packet is admitted iff ALL hold: leaf has a mappable executor (else it fails at plan time, never mid-run); `depends_on` all Passed; lane enabled with `running[lane] < max_concurrent`; backpressure allows; declared budget fits remaining. Wave size default 4–6 fibers with disjoint write scopes. Shared contracts exist on disk and are reviewed before any parallel fiber starts.

## 3. Execution (PARTITION — one mind per partition, front-to-back)

Each fiber: exactly one assigned task, read-only unless its task is the edit task, budgeted (word + tool-call caps). Every call returns a receipt (`receipt_sha256`); no receipt = the step didn't happen. Fiber writes outside assignment, exceeding budget, or drifting incoherent = kill and restart narrower. No second chances inside the same fiber.

## 4. Failure doctrine (core move — never retry identically)

- **R1 fail → re-approach with material change.** Identical retry is a violation.
- **R2 fail → escalate** packet + receipts + judge outputs with TTL; preregister a new approach before re-approaching.
- **Cost overrun >10% over declared** → escalation mandatory; orchestrator may revoke budget.
- **Layer-0 hard line:** judges, receipts, and claims are never deceived.
- Transport error (pipe broke) ≠ verdict (judge said no): transport gets one retry then escalates; verdicts never retry, they re-approach or escalate.

## 5. Degraded mode (keep the run alive)

Detect at the boundary → name it (`degraded:<component>`) → continue on the specified fallback → record receipt → escalate only on second failure. Degraded claims stay degraded: never quote full-system numbers from a degraded path. N3-unavailable = harness-local ledger only, no canonical-truth claim. Every branch writes a receipt or an explicit degradation record.

## 6. Continuity across waves (never set it down)

- **Checkpoint at step boundaries only, never mid-generation.** Classify first (pinned → anchors/locks → working set → reclaimable → dead); deterministic prune, model 5W draft, judge third; commit sha-chained checkpoint with kept/pruned/hash receipt; raw archived cold, never deleted.
- **Verify:** fresh run with only pinned + tail must take the same next action. If it can't free enough: explicit rejection or tier-2 degrade, written to audit.
- **Sequence everything:** push with sequence numbers at every step boundary; gap = explicit missing data; replay from last checkpoint, never silent.
- **Restart rule:** packets `running` at restart → mark `failed/restart_interrupted` → regenerate idempotent retry signatures → re-admit where deps satisfied → emit replayed events. No pre-restart packet stays `running`.

## 7. Wave certification (PROVE — checkers, not summaries)

Per wave, meta-only: recount fiber numbers, re-run key commands, grep absence claims. Pressure test: crate suites + observability suite + harness tests green; `git status` clean of fiber writes; every loss lands in a receipt or ledger row. Certify with numbers (pass/fail counts, latencies with hardware + workload + comparison) or don't certify. A recorded failure beats an asserted success.

## 8. Research gate (no execution without research)

No tool call runs on an unresearched packet. The log must show at least one ContextInject (planning happened) or the engine refuses with the missing terms named. Research is per-packet: new wave, new information need, new ContextInject.

## 9. Hooks (23 moments, fixed)

23 hook moments fixed at compile time; no runtime registration. `Veto(reason)` aborts the enclosing action with receipt; one veto short-circuits the rest. Audit hooks (PostApproval, PostRevocation, ErrorBoundary) are veto-immune. Algebra-vs-canonical count drift (25 vs 23) is open — until resolved, 23 is the enforced number.

## 10. Ingest discipline (when a wave pulls sources)

One source at a time, serialized manifest apply, no DB writes during a pull. Verify per source (chunk count + retrieval smoke) before the next starts. Judge-gated: no pass, no atom. Per-packet mint only on pass; teacher identity in lineage only, never in coordinates.

## 11. Kill rules (ship-blocker gates)

- Veto-immune audit hooks REQUIRED before any veto-storm drill is executable.
- Kill-switch drill: live flip under load with `switch_ms`, `dropped_receipts`, `atlas_readable`, `toggled_event` — idle drill rejected, QA-witnessed at P5, fail blocks phase exit. No passing drill artifact exists today: **no veto-storm or kill-switch drill runs until the drill artifacts are filed.**
- Force-close is Admin-gated, emits receipt, time-boxed with review.

## 12. Source audit (where every line came from — saved for removal clearance)

| # | Source | Scope | Status |
|---|---|---|---|
| 1 | C22 `Design.md` C22.1 OOM | detect → halve inflight → kill-switch → evict max-x0 unpinned → receipt; verify pressure <0.7 in 60s | PROPOSED text, unexecuted; LRU wording known-wrong (fix: least-salient per C05) |
| 2 | C22 `Design.md` C22.2 breaker | isolate lane → backoff ×3 → probe/30s → auto-close on 3 pass; force-close admin-only | PROPOSED text, unexecuted |
| 3 | C22 `Design.md` C22.3 hole-merge | auto-reject on Merkle mismatch → preserve both → diff → approve-force/reject → re-merge | PROPOSED text, unexecuted |
| 4 | C22 `Design.md` C22.4 packet-restart | mark failed/restart_interrupted → retry signatures → re-admit on deps → replayed events | PROPOSED text; mechanism live in `runbook.rs` |
| 5 | C22 `Design.md` C22.5 compaction-restart | summary deposit → checkpoint → fold pinned+checkpoint → restart → recall == 1.0 | PROPOSED text; primitives live (`map.compact/checkpoint`) |
| 6 | C22 `Design.md` C22.8/C22.9 + `Rules.md` | last_drilled per runbook, contained-before-cascade order; cut-set probs placeholders, CS4 first, force-close Admin-gated | RULE, enforced by §7/§11 |
| 7 | `ResearchAgenticListy.md` | ingest runbook: one source at a time, no DB writes during pull, verify-before-next | PROPOSED, needs nod |
| 8 | `HARDWARE-GATE-RUNBOOK.md` | no synthetic ships; fixed-PPC on hardware; live log + why-receipt per failure | EXECUTED 2026-10-29, receipt filed |
| 9 | `src/runbook.rs` | Packet/Plan types, MadMind judgeless-reject, Kahn cycle check, ready_packets admit | LIVE code, tested |
| 10 | `N^3-Wiring-and-Fallbacks.md` §§3–5 | 8-row fallback matrix + detect/name/continue/record/escalate SOP + invariants | SPEC, wire-proofed 2026-10-29 |
| 11 | `Research-Grade-Agentic-Harness-Design.md` §§10–12 + `gate.rs` | R1/R2 doctrine, >10% rule, Layer-0, research-gate (ContextInject or refuse) | SPEC + live `gate.rs` |
| 12 | `N^3-Algorithms-Workflows-SOP.md` + RAG §8–10 + `LONGTASK-BENCH-PRE-REG.md` | packet lifecycle, 40-packet wave shape + redteam leaf, condensing + active-context protocol | SPEC + pre-reg, redteam unrun |

**Compaction note (no sprawl):** this file replaces the 12 slots above as the working loop. Sources stay on disk until Stewart clears removal — nothing deleted by this write.

---

## Failure doctrine (whole loop)

Leaf with no mappable tool → stop, name the hunger gap. Fiber wild spend/scope breach/incoherence → kill, restart narrower. Dissolve void → quarantine, never force-merge. Missing receipt mid-plan → halt branch, re-run leaf. Bar missed → reported as missed, two fix cycles max, bars unchanged.
