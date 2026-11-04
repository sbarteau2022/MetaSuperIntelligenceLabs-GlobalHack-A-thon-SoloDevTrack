# Governed 30-step run — what it is and is not (2026-11-04)

Raw audit output: `governed-run-30-2026-11-04.txt`. Test: `Ares WebSite : App/backend/tests/native_glimmer_kv_live.rs::governed_run_30_steps_audit` (ignored).

Ran: the harness `Engine::run_sequential_live` (the engine behind `/api/harness/mission/live`) over a 33-packet plan —
30 real `mind.recall` steps plus 3 must-fail steps (undeclared tool, unsupported tool `shell.exec`, malformed args) —
with capability gate, per-atom `Contains` judges, receipt log, and a recording executor wrapping `NativeGlimmerExecutor`
(real `globalmind-mcp` child -> native tokenize -> exact K/V hit/miss -> continue-exact) on an isolated frozen Muse Glimmer
server. A disposable SurrealKV was restarted at step 15 while the run continued.

Audited with zero failures: 30 passed; the 3 negatives escalated and never reached the executor; every bound receipt
validated; model input bytes equal the exact atom text; recall evidence carried the atom's content digest; K/V pattern was
exactly 6 prefills then 24 hits; each atom's generation was identical across all its steps; one artifact composite throughout;
PASS receipts carried the executor's chain hashes in order; mission receipt chain verified; 96 telemetry writes.
Latency 7.4 s min / 7.6 s median / 30.2 s max (max = first-touch prefill/restart), 271 s wall, 120 generated tokens.

NOT claimed:
- Not the HTTP route: the live backend (4810) was not rebuilt/restarted and no permit/governance Surreal (8000) was used.
- Telemetry went to a local OTLP sink, not HyperDX; the operator's secrets.env was not read.
- Atoms are 6 hand-made witnesses, not distilled knowledge; step outputs are generated token IDs (no detokenizer) and the
  judge checks the recall evidence, so answer quality / knowledge-use is NOT judged (handoff item 5 "with vs without atoms").
- The speculative loop and MTP head were not in this run; acceptance/rollback was audited only in the separate item-5 gate.
- Page-digest roster is held by the test across the restart, not recovered from the DB.
- 30 steps, one run, one checkpoint; token cost is generated-token count and energy proxy only.
