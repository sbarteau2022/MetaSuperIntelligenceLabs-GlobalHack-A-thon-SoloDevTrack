# Automaton-governed speculative decoding on frozen Muse Glimmer — 2026-11-04

Loop (`Ares WebSite : App/backend/src/harness/speculative.rs`): a prompt-lookup drafter (tokens that
followed the longest context suffix in the recalled atom text; no model) proposes up to 8 tokens ->
the single automaton `transition` admits/rejects/clarifies from measured state (prefix identity =
pinned artifact composite, confidence floor) -> admitted drafts go to the new native route
`/v1/verify-exact` (GVX2 -> GFO2 in core_lm: predict after context, then after each draft prefix,
stop at the first disagreement) -> the model's predictions are committed, the rejected tail rolls
back -> Q32.31 confidence moves 1/10 of the way toward `1 - rejected/drafted` via
`update_from_prediction_error`, which refuses unverified outcomes and takes the error only from the
verified model response. Every response receipt is recomputed over the request bytes and the artifact
digests must equal the pin. Control arm: same prompt, one verified greedy token per round, no proposals.

Hardware gate: `tests/speculative_live.rs` (ignored), isolated Metal `serve_glimmer` on 18111,
prompt "The capital of France is", 12 new tokens.
- Control: tokens [13796, 26, 1573, 373, 290, 2167, 26899, 1212, 6996, 310, 11698, 341], 12 rounds, 130.8 s.
- Speculative, on-topic atom: IDENTICAL tokens; 10 rounds; drafted 32, accepted 2, rolled back 30;
  confidence 0.6000 -> 0.4119 (4 calibrations); 96.4 s.
- Speculative, off-topic atom: IDENTICAL tokens; 12 rounds; drafted 7, accepted 0; confidence
  0.6000 -> 0.5400 (1 calibration); 106.5 s.

Reading: correctness holds (verification, not the drafter, decides; greedy output is unchanged) and the
automaton learns from rejection. The speedup is small (12 -> 10 rounds, ~26% wall time) because only
2 of 32 drafted tokens matched: Glimmer's continuation diverges from the atom text after "Paris".
Wall times are dominated by the verify route recomputing the whole context each round (no cache reuse
across calls), so absolute seconds are not a serving-latency claim.

Limits: one prompt, one run; the drafter is prompt lookup, not an MTP head (the native MTP seat on 8010
is not wired into this loop); the loop is not yet in `dispatch_mission_live` or any mission packet;
confidence is not persisted to Surreal here; the control-vs-speculative comparison measures rounds and
wall time only, no output-quality judging (item 6). Unit tests use a deterministic table model for control
flow only; the real-model equivalence is the hardware gate above.
