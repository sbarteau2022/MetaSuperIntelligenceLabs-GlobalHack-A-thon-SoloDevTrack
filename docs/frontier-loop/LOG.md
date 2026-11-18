# Frontier loop log: MTP head integration

## Step 1 — Extract and verify the MTP head from the main model: IN PROGRESS

**Command**: (running now)

**Observations from examination:**

- Config says `mtp_num_hidden_layers: 1`
- Current MTP file has 16 layers
- Main model has 40 layers with MTP layers at the end
- The MTP head is stored separately in `model-mtp.safetensors`

**Next action**: Extract the MTP head weights from the main model and compare to the standalone file.

## Step 1 — measured facts (2026-11-10)

- Premise in GOAL.md is wrong: `model-mtp.safetensors` holds **one** MTP layer (`language_model.mtp.layers.0`), 42 tensors, matching `mtp_num_hidden_layers: 1`. Not 16 layers.
- The MTP head is not in the main model. The 4 main shards hold `language_model.model.layers.0–39` only (40 layers, no `mtp` keys). `model.safetensors.index.json` maps all 42 `language_model.mtp.*` tensors to `model-mtp.safetensors`.
- Shapes: `mtp.fc.weight` BF16 [2048, 4096] (hidden + embedding concat); `mtp.layers.0.mlp.gate.weight` BF16 [256, 2048] (256-expert MoE router).
- So "extract the head from the main model's last layers" has no source tensors. Step 1's gate is trivially true of the standalone file, but the objective cannot be met as written (B1).
- `serve.sh` runs DSpark with a remote drafter (`Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK`) and does not load the local MTP file, so the local head may not be on the serving path.

**Gate status:** Step 1 objective not met as written. Blocked on a user decision.

## Step 1 (retargeted) — PASS 7/7 (2026-11-10)

Check script: scratch `check_mtp_step1.py` (not in repo).
- 42 MTP tensors in `model-mtp.safetensors`; index maps exactly those 42 to it.
- No MTP tensors in the four main shards.
- One MTP layer (`layers.0`), matching `mtp_num_hidden_layers: 1`.
- `mtp.fc` [2048, 4096] = [H, 2H]; router gate [256, 2048] = [E, H] (E=256).
- Norms: `pre_fc_norm_*`, `norm`, layer norms are [2048]; `q_norm`/`k_norm` are [256] = head_dim, matching main layers.

## Step 2 — PASS (2026-11-10)

Gate (pre-registered): top-1 agreement of the MTP head with the main model's top-1 two steps ahead >= 0.30. Chance 4e-6.

Implementation: `docs/frontier-loop/mtp_head.py`. Head = pre-fc norms, fc [2H->H], one full-attention MoE layer (layer idx 3, reused from mlx_lm qwen3_5), final norm, and the main model's `embed_tokens` and `lm_head`. MTP tensors are dequantized at load (bits/group inferred from shapes plus known input dims).

Gate test: `docs/frontier-loop/gates/step2_mtp_gate.py`. It first failed with ModuleNotFoundError (red), then passed after implementation.

Measured (one 512-token sample: GOAL.md + LOG.md, 510 positions, vocab 248320):
- MTP top-1 agreement with main model (t+2): 0.641 (gate >= 0.30)
- MTP top-1 accuracy vs true x_{t+2}: 0.476
- Main model top-1 accuracy vs true x_{t+2}: 0.539
- Chance: 4.03e-06

Caveats: one sample, teacher-forced (not a decode loop). Hidden input is the final-normed hidden state; the pre-norm variant was not tried. The causal mask is passed as "causal" and not separately verified.

## Step 3 — open

Needs: a speculative decode loop with the MTP head, with recurrent-state rollback for the GatedDeltaNet layers, and a measured baseline from the DSpark drafter (`Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK`, cached locally) on the same prompts. Not started.

## Step 3 — gate FAILED on speed (G3); G1 and G2 pass (2026-11-10)

Gate file: `docs/frontier-loop/gates/step3_spec_gate.py`. Baseline: `gates/bench_dspark_baseline.py` against serve.sh (port 8001), output `gates/step3_dspark_baseline.json`. Prompts: `gates/step3_prompts.py` (3 chat prompts, 128 new tokens, greedy).

### Results
| metric | MTP 1-layer spec | plain greedy | DSpark (serve.sh) | gate |
|---|---|---|---|---|
| tokens/s (mean of 3) | 94.8 | 85.3 | 97.5 | MTP > DSpark: **FAIL** |
| G1 teacher-forced correctness | 1.000 | — | — | >= 0.98: PASS |
| G2 acceptance, pre-registered | p_mtp 0.808 | — | p_dspark 0.478 | >= 0.428: PASS |

Per prompt MTP tok/s: 87.3, 99.8, 97.3. p_mtp per prompt: 0.73, 0.86, 0.84.

### Caveats that matter for reading the table
- G2 compares per draft slot. MTP drafts 1 token per round, so p_mtp = accepted/drafted. DSpark's cap is 4, and I used p = (accept_len − 1)/cap. The metric is not like-for-like. In tokens per target forward, DSpark gets accept_len 2.4–3.2 versus about 1.8 for MTP. The per-slot metric flatters MTP.
- Speed is not like-for-like either. The MTP and plain numbers come from an in-process Python harness, with bf16 KV and default settings. DSpark numbers come from the HTTP server, which runs with kv-bits 8, fp16 activations, and QWEN_* env settings (serve.sh). DSpark's tok/s includes HTTP overhead, so the fair speed comparison is not established by this run.
- Exact-prefix match with plain greedy is low (0.19 mean). That is expected for a bf16 model where 1-token and 2-token forwards differ by about 0.6 logits, so greedy paths split at near-ties. G1 therefore uses the teacher-forced check.

### Reflection (why the first attempt failed, and what changed)
1. Gate G1 as first written (free-running exact match, 98%) read 5.5%–19.5%. That looked like a broken loop. Teacher-forced checks on the plain path showed it was not. Cached 1-token decode matched the no-cache reference exactly (gap 0 to 0.25). The spec path had tokens 14 logits below argmax. So the bug was real in the spec path, not in the criterion.
2. Root cause: `_snapshot` copied `ArraysCache.state`, which is a tuple whose first element is a mutable list. The list was mutated in place, so the snapshot was not a snapshot. Restore-then-single-step differed from fresh by 8.25 logits. Fix: copy `c.cache`, `left_padding`, `lengths` explicitly. After the fix, restore is exact (diff 0.0) and the teacher-forced check is 1.000 on all prompts.
3. G1 criterion revised to teacher-forced with tie tolerance 0.5 logits. This is recorded in the gate's docstring, and the first run's numbers are kept above.

### Decision for the user (blocking)
MTP gives about 1.11x over plain greedy and is about 3% slower than the DSpark server on these prompts. Whether that counts as a fail depends on a fair DSpark comparison, which this harness cannot give. Options: (a) accept G3 as failed and stop the goal; (b) rerun both paths inside one process with matched KV/activation settings (kv-bits 8, fp16) and then decide; (c) try speed work on the MTP loop first (e.g., drop the extra head call on accept, or batch the head with verification).

## Step 3 — matched rerun (one process, same settings) — G3 FAILS (2026-11-10)

Supersedes the unmatched numbers above. Gate: `docs/frontier-loop/gates/step3_matched_gate.py`. Results: `gates/step3_matched_results.json`.

Setup, identical for all three methods, in one process: serve.sh QWEN_* environment, `long_context.install()`, target loaded with kv-bits 8, drafter `Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK` loaded via mlx_dspark's loader, the CLI's kernel patches, MTP head in fp16, cap 4, 128 new tokens, greedy, 3 prompts × 2 reps, methods interleaved per prompt.

| method | median tok/s | median tokens per main forward |
|---|---|---|
| plain greedy | 80.6 | 1.00 |
| MTP 1-layer spec | 90.4 | 1.51 |
| DSpark (drafter, cap 4) | 106.6 | 3.05 |

Gates: G1 correctness 1.000 (teacher-forced, PASS). G2 acceptance p_mtp 0.787 vs p_dspark 0.478 (PASS on the pre-registered per-slot metric). G3 speed FAIL: MTP 90.4 < DSpark 106.6.

### What the matched run found
1. A bf16 MTP head ran at 95 ms per token on fp16 activations. In fp16 it runs at 2 ms. The first matched attempt measured the MTP path at 14.6 tok/s because of this dtype mismatch, which was a bug in my harness, not a property of the head. The head now loads in fp16 (`load_mtp_head(..., dtype=mx.float16)`).
2. With the head fixed, MTP is 1.12x plain greedy. DSpark is 1.32x plain. The gap comes from tokens per forward: DSpark drafts 4 tokens per round and gets 3.05 per forward; MTP drafts 1 and gets 1.51.
3. The first unmatched run was not a fair comparison (see the caveats above). The matched run removes those confounds and still fails G3.

### Reflection
A single-draft MTP loop has a ceiling. With acceptance p, tokens per round is at most 1 + p, and with p≈0.8 that is about 1.8 before overhead. The loop overhead (one head call for the draft, one for the accept-path cache entry, and a reject re-run) takes the rest. DSpark wins on speed because it drafts a block. Making this head competitive would need multi-token drafting. The head is one recurrent layer, so it can be chained, but each extra draft depth will have lower acceptance. That is a new design, not a fix to this gate.

### Decision for the user (blocking)
G3 fails. Options:
(a) Stop the goal with G3 failed. The 1-layer MTP head is correct and accepted at 0.79, but it is slower than DSpark on this machine.
(b) Open a new step: chain the MTP head to draft k=2–4 tokens per round, and gate it against DSpark on the same harness.

## Step 4 — chained MTP drafts — gate FAILS on speed (G3); G1 passes (2026-11-10)

Protocol (GOAL.md Step 4): choose K in {2,3,4} on dev prompts (step3_prompts), gate the chosen K on held-out prompts (step4_prompts) against DSpark, same process, same settings (kv-bits 8, fp16 head, cap 4, 128 tokens, greedy). Files: `mtp_head.py` (`hidden()` split from `__call__`), `spec_decode.py` (`mtp_chain_decode`), `gates/step4_chain_gate.py`, `gates/step4_posthoc_heldout.py`.

### Red, then green
- The gate was written first and failed with ImportError (`mtp_chain_decode` missing).
- Correctness check on one prompt: all K match greedy output, teacher-forced 1.000, exact prefix 39/64 for K=3.

### Results
| | dev median tok/s | held-out median tok/s |
|---|---|---|
| plain greedy | 81.1 | 80.8 |
| DSpark (cap 4) | 106.6 | 87.2 |
| chain K=2 | 97.8 (chosen) | 77.1 |
| chain K=3 | 89.1 | 75.4 (post-hoc) |
| chain K=4 | 81.0 | 65.6 (post-hoc) |

- K* = 2 on dev. Held-out per-depth acceptance for K=2: 0.731 at depth 1, 0.648 at depth 2.
- G1 (held-out teacher-forced) = 1.000, PASS. G3 (chain_2 > DSpark on held-out): 77.1 < 87.2, FAIL.
- Post-hoc diagnostic: no chain depth beats DSpark on held-out. K=2 is also the best chain depth there, so the dev choice was right.

### What changed during the step
1. First version synced to host at every draft depth. Second version drafts lazily and syncs once per round. Held-out K=2 went 70 → 77 tok/s. Correctness did not change.
2. A profile of one round (K=2) on this machine: draft head calls about 0.75 ms each, draft lm_head about 1.3 ms each, the verify forward of 3 tokens 15.8 ms (plain 1 token is 10.9 ms), and the rejected re-run about 12 ms on each reject. Tokens per round were 1.35–1.6.

### Reflection
A round costs about 20 ms, and one round yields about 1.5 tokens. That caps throughput near 75–80 tok/s, which matches the measurement. DSpark's block drafter yields 2.5–3 tokens per forward, so it stays ahead even with a more expensive verify. Two things would have to change to beat it: acceptance at depth 2 would need to rise far above 0.65, or the rejected re-run (about 5 ms per round on average) would need to go away. The re-run exists because the recurrent layers cannot be trimmed. Removing it means keeping intermediate recurrent states from the verify forward, which the installed mlx_lm does not return. The verify cost alone (15.8 ms per round) still leaves little room at 1.5 tokens per round. The ceiling analysis says this is not a tuning gap.

### Decision for the user (blocking)
Step 4 gate failed. Options:
(a) Stop the goal. MTP-based speculation, 1-layer head, does not beat DSpark on this machine. Record G1 and G2 pass and G3 fail.
(b) Pursue the recurrent-state change in mlx_lm (return intermediate recurrent states from the verify forward) to remove the rejected re-run. The ceiling analysis says this may not be enough to beat DSpark.

## Correction to the Step 4 reflection (2026-11-10)

The Step 4 reflection said a round "yields about 1.5 tokens" and that this capped throughput near 75–80 tok/s. That was wrong. 1.5 was tokens per **main-model forward**, and the forward count included the rejected re-runs. Recomputed from `step4_chain_results.json` (held-out, K=2): tokens per **round** were 2.13–2.33 (mean 2.19). There were 23–35 re-runs per 128 tokens, on 40–58% of rounds. Measured on this machine, a re-run costs 11.0 ms (1 token) to 13.3 ms (2 tokens), and a 3-token block costs 15.7 ms. So re-runs took about 19–25% of wall time. Removing them predicts roughly 100–110 tok/s, not a ceiling at 75–80. The "not a tuning gap" conclusion did not hold.

## Step 5 — chained drafts with DSpark's hybrid rollback — PASS (2026-11-10)

Mechanism: mlx_dspark's `Target` already solves the recurrent-state problem (target.py, "speculative verify + rollback"). During the block forward, `_capture_linear()` records each gated-DeltaNet layer's recurrence inputs and conv input. `rollback(cache, n_rejected, accepted)` rebuilds each recurrent state at the accept point with the recurrence kernel only, lazily, without a second model forward. It also trims KV. New function: `spec_decode.mtp_chain_decode_rollback(target, head, ids, n, K)`. It calls `rollback(cache, K - a, draft[:a])`, so keep = a + 1, and takes the next hidden and pending from block index a.

Order of work: the function was written before the Step 5 gate. The first check compared it token for token with the re-run decoder: identical on the first prompt (64 tokens, K=2), with zero re-run forwards.

### Gate (Step 4 protocol, unchanged): `gates/step5_rollback_gate.py` → `step5_rollback_results.json`
| | dev median tok/s | held-out median tok/s |
|---|---|---|
| plain greedy | 80.0 | 82.7 |
| DSpark (cap 4) | 107.6 | 88.8 |
| rchain K=2 | 115.9 | 97.7 |
| rchain K=3 | 116.4 (chosen) | 94.3 |
| rchain K=4 | 104.5 | 89.5 |

K* = 3 on dev, by 0.5 tok/s over K=2. G1 held-out teacher-forced = 1.000 PASS. G3: 94.3 > 88.8 PASS. Per-depth acceptance at K=3: 0.723, 0.670, 0.600.

### Confirmation: `gates/step5_confirm.py` → `step5_confirm_results.json`
- Held-out, 5 reps × 3 prompts, interleaved. Median tok/s: DSpark 89.6, rchain K=2 99.1, rchain K=3 94.8. The chain was faster than DSpark in 15/15 matched pairs for both K.
- Token identity of the rollback decoder against the re-run decoder: 7/12 (prompt × K) runs identical over 128 tokens. All 5 divergences are at near-ties: at the first differing token, both candidates are within 0.141 logits of the no-cache reference's top logit, and two are exact ties. That is fp16 numeric noise between the two rebuild paths (kernel rebuild vs full re-forward), not state corruption. G1 stays 1.000.

### Fresh prompts, pre-registered: `gates/step5_fresh_check.py` → `step5_fresh_results.json`
The Step 4 held-out set had informed the decision to pursue this step, so it was no longer fully held out. Three new prompts were written and the pass rule was fixed in the docstring before any run (K* = 3 from dev; rchain_3 median > DSpark; G1 >= 0.98). 3 reps, interleaved.
- Median tok/s: plain 80.4, DSpark 109.9, rchain K=2 116.3, rchain K=3 118.5.
- rchain_3 faster than DSpark in 9/9 matched pairs; rchain_2 in 8/9. G1 = 1.000. **PASS.**

### Caveats
- One machine (M5-class GPU; mlx_dspark's small-M and multirow kernels are gated off on it), one session, greedy decoding, 128-token outputs, three prompts per set.
- The decoder uses private mlx_dspark internals (`_capture_linear`, `_stash`, `_spec_width`) from version 0.20.3. A package update can break it silently, which is why the token-identity check exists.
- Nothing is wired into serve.sh. The server still runs DSpark.
- The margin is modest: +6% to +11% over DSpark depending on prompt set and K. K=2 and K=3 trade places across sets, so the dev choice of K is within noise.

## Erratum: Step 5 statistics (verification, 2026-11-10)

A three-lens verification (correctness, measurement, protocol) found that the Step 5 summary overstated the evidence:
- "15/15" and "9/9" are 3 independent prompts per set, each replayed. Replays are deterministic: forwards, accepted, rollbacks and tokens match across repetitions. The repetitions only measure timing noise. One-sided sign test: p = 0.125 per set.
- Order was fixed (DSpark, then rchain_2, then rchain_3) and never counterbalanced. Within-process drift between repetitions was up to about 6%.
- Fresh prompt 2 is a tie within noise (rchain_3 +0.4% to +3.9%; rchain_2 −3.2% in one replay). The fresh pass rests on prompts 0 and 1.
- The token-identity check (7/12 identical to the re-run decoder) is not bit-exact. "Bit-for-bit" in target.py's comment describes the verify forward's own state, not a committed forward.
- The Step 4 ceiling line at the top of this log ("1.5 tokens per round") is superseded by the correction above. Tokens per round were 2.13–2.33.

Divergence evidence, from `gates/step5_divergence.py` (run in this session; output recorded here). At the first token where the re-run and rollback decoders differ, `gap_*` is how far the chosen token is below the no-cache reference argmax:

| set | prompt | K | first diff | re-run token | rollback token | gap re-run | gap rollback |
|---|---|---|---|---|---|---|---|
| dev | 1 | 2 | 39 | 35169 | 321 | 0.000 | 0.016 |
| dev | 2 | 2 | 35 | 321 | 3055 | 0.000 | 0.047 |
| held-out | 1 | 2 | 19 | 321 | 748 | 0.000 | 0.109 |
| held-out | 1 | 3 | 79 | 3134 | 11870 | 0.141 | 0.000 |
| held-out | 2 | 3 | 48 | 15156 | 61446 | 0.000 | 0.000 |

So all five divergences are near-ties: the candidate that was not the top token is at most 0.141 logits below it. In one case both candidates are exact ties (gap 0.000). The earlier wording "two are exact ties" was wrong.

## Wiring the MTP decoder into the server (2026-11-10)

What is wired (default, `serve.sh`): `DSPARK_DECODER` defaults to `mtp`. Set `DSPARK_DECODER=dspark` to roll back. The server is `docs/frontier-loop/serving/run_server_mtp.py`. It runs the same install steps as `run_server.py`, in the same order, and replaces two things:
- `load_drafter` (both the module and the server's binding) returns an empty `nn.Module` stub. The DSpark drafter is not loaded, so the server uses less memory. The stub supplies the prefix-cache ctx factory.
- `speculative_generate` becomes `mtp_serve.mtp_speculative_generate`, with the same keyword interface and the same `GenResult`.

`mtp_serve.py` behavior:
- Greedy or sampled, no penalties, no logprobs: MTP chain (K = 3, `MTP_K`), with prompt-lookup drafts (n-gram, up to 32 tokens, `MTP_LOOKUP`, `MTP_LOOKUP_LONG`) when the suffix matches. One verify forward per round. Rejected tails roll back through mlx_dspark's hybrid capture. Sampled requests use speculative-sampling acceptance with a one-hot proposal.
- Penalties or logprobs: sequential decoding with the server's sampling helpers, still keeping the head cache current.
- Prefix caching: the head's KV cache rides in the prefix-cache ctx slot (`HeadCtx`, the CtxCache protocol). A reuse is valid only if the head cache length equals the reused length; otherwise the request is a cold prefill, which is correct and slower.
- Stop strings, EOS, `max_tokens`, streaming, progress and checkpoint hooks, and the active-KV budget guard (`record_cache`) all run on this path.

Bugs found and fixed while wiring (each would have hidden the result):
1. The server binds `load_drafter` at import, so patching only the cli module left the real DSpark drafter loaded. Symptom: every reuse fell back to a cold prefill (2.0 s for a 4k prompt, against 0.017 s warm). Fix: patch `server.load_drafter` too.
2. The drafter stub must be an `nn.Module`: the server walks its modules when choosing kernels.
3. Telemetry sized its arrays by the MTP cap, so lookup rounds of up to 32 drafts crashed the request. Fix: report each round's real draft count and source `lookup`.

Verification:
- `test_mtp_serve.py`, all passed: T1 greedy teacher-forced 1.000 on 3 prompts; T2 prefix reuse at the stable boundary, 1.000; T3 sampled run valid; T4 logprobs length, and penalties change output; T5 stop string; T6 mismatched reuse falls back to a correct cold run; T7 1158-token prompt, chunked prefill, partial reuse at a mid-prompt rung, 1.000; T8 copy-heavy prompt with lookup drafts, 1.000 with 12 lookup rounds.
- `http_smoke.py` against the default server: 6/6 checks pass.
- `soak_http.py`: 12 turns of a growing conversation, greedy and sampled mixed. 0 errors, no tracebacks. Cached tokens grow turn by turn, and TTFT stays at 0.06–0.18 s.

Benchmarks over HTTP, same machine, same settings. Each server was run alone:

| workload | MTP (default) | DSpark (previous default) |
|---|---|---|
| short greedy, 256 tokens (3 prompts × 2 reps): e2e tok/s / decode tok/s / TTFT | 109.5 / 139.0 / 0.50 s | 102.6 / 106.3 / 0.71 s |
| sampled (T 0.7), short: e2e tok/s | 102.6 | 86.3 |
| long prompt, 6.5k tokens, ~100 out: TTFT / e2e tok/s | 3.68 s / 25.0 | 3.84 s / 24.2 |
| copy-heavy, 4.1k prompt, 512 out, cold: e2e tok/s | 101.7 | 105.3 |
| copy-heavy, same prompt again (prefix hit): e2e tok/s / TTFT | 166.6 / 0.71 s | 181.9 / 0.84 s |

Read the table honestly:
- MTP is faster on short and sampled requests (+7% to +19%) and at parity on long prompts.
- DSpark is ahead on copy-heavy requests, by 3% cold and 9% on a prefix hit. The cause is not isolated yet.
- All short-prompt numbers are three prompts, two reps. The copy and long rows are single runs.
- Production runs up to 16384 output tokens. None of these benchmarks is that long.

Not done: nothing has been validated beyond these runs. Two gaps remain. Long-output (thousands of tokens) and multi-day behavior are untested. Tool-call and reasoning parsing are inherited from the server, but not checked end to end.

## Why DSpark led on copy-heavy requests — traced and fixed (2026-11-10)

Method: `serving/trace_copy.py` reads the server's `/rounds` telemetry for one cold copy-heavy request (4.1k prompt, 512 output) and summarizes per source. First round dropped (its time includes prefill). Files: `serving/trace_*.json`.

| path | decode tok/s | model-draft rounds | lookup rounds | lookup tok/round | lookup ms/round |
|---|---|---|---|---|---|
| MTP, 3–4-gram index | 175.8 | 17 × 3.0 tok, 27 ms | 51 | 9.0 | 48 |
| DSpark | 196.4 | 19 × 3.0 tok, 31 ms | 40 | 11.2 | 50 |
| MTP, 4–5-gram index | 194.6 / 195.5 (2 runs) | 19 × 3.2 tok, 28 ms | 39 | 11.5 | 53 |

Finding: round cost was equal or lower on the MTP path, so kernels and per-round overhead were not the gap. The gap was the lookup policy. mlx_dspark's hybrid mode builds its n-gram index with `min_n=4, max_n=5` (its source comment: trigrams fire on 4–10% of chat rounds and forgo a productive model-draft round). I had used the pure-lookup default of 3–4-gram, which fired more often with worse drafts (lookup rounds 51 vs 40, 8.5 of 14.4 accepted vs DSpark's 10.2 of 15.3). `mtp_serve` now defaults to a 4–5-gram index (`MTP_NGRAM_MIN`, `MTP_NGRAM_MAX`) and clamps lookup drafts to the remaining token budget, as DSpark does. Decode throughput now matches DSpark (195 vs 196 tok/s).

End-to-end on this prompt, MTP is 103.5 and 107.8 tok/s against DSpark's 109.8. The remainder is the first round (prefill plus first token): 2.14–2.32 s against 2.07 s. Not chased. Short greedy and sampled requests are unchanged within noise (107.9 and 98.9 e2e tok/s vs 109.5 and 102.6 before). The in-process suite still passes (8/8).

Single traces per path (one DSpark, three MTP); the point is the per-round mechanism, not a margin.

This also answers the "dynamic loop" question for now: the loop already switches draft source per round (lookup vs MTP chain). What it lacks is DSpark's cap controller and a verify-cost model. Whether those pay off is open; the copy gap was a threshold setting, not a missing mechanism.

## Drafter policy A/B (automaton vs fixed), preregistered — FAILED (2026-11-10)

Prereg: `serving/ab_policy_prereg.md` (written before the run). Harness: `serving/ab_policy.py`, one process, arms interleaved, 2 reps. Results: `serving/ab_policy_results.json`. Policy: `serving/policy.py` (choose lookup or MTP depth 1–3 by (1+E[accepted])/round_ms, online EMA, probe every 10 rounds). Default remains `fixed`.

| workload | fixed tok/s | automaton tok/s | ratio |
|---|---|---|---|
| short greedy | 113.1 | 111.5 | 0.99 |
| sampled | 93.6 | 97.7 | 1.04 |
| copy-heavy | 111.4 | 99.8 | 0.90 |
| long 2k | 61.4 | 62.6 | 1.02 |
| mean | 94.9 | 92.9 | 0.98 |

- P1 correctness: PASS (worst teacher-forced 1.000).
- P2 mean: FAIL (92.9 < 94.9).
- P3 no-regress: FAIL (copy 0.90 < 0.97).

Why it likely failed (not yet tested): on copy the policy chose lookup in only 19–35 of about 100 rounds, so it under-used the source that gave the best throughput. The lookup expected-acceptance estimate is one learned count, not conditioned on draft length (lookup drafts range 1–32), so it mis-scores long lookup drafts. The forced probes also cost rounds. The fix is a length-conditioned estimate, but changing the policy after seeing this result would be a new hypothesis and needs a new prereg. Not done.

Hyperbolic addressing (nested K/V): not tested. The served configuration holds one prefix slot, so there is nothing to address among; the exact-token prefix match already selects it. The hackathon design's own status says live attention-cache reuse is not certified (native forward-pass tensor I/O not implemented). Exact attention K/V is kept, as the design requires for output identity.

## Nested hyperbolic K/V + attention-economics A/B — FAILED quality gate (2026-11-10)

Design and prereg were written before implementation/model execution: `serving/HYPERBOLIC_CACHE_AB_DESIGN.md` and `serving/ab_hyperbolic_cache_prereg.md`. The experiment keeps exact-model MLX tensor pages, addresses candidate prefixes with a two-level Poincare-ball coordinate, and minimizes the preregistered recall-vs-recompute free energy over retrieval depth and 25/50/75% reuse fractions. It is an isolated harness; production exact-prefix behavior is unchanged.

Pressure Strike 1: the 3D hash projection alone aliased an unrelated short prompt and failed closed incorrectly. Before any model A/B, similarity was changed to nested-H3 proximity multiplied by a token-set Jaccard guard. The rerun passed 6/6 hostile cases. This correction is recorded in the preregistration.

Authoritative A/B (cache restore included in query timing), 4 related pairs, 96 greedy output tokens:

| case | exact wall | hyper wall | speed ratio | hyper reference agreement | first token |
|---|---:|---:|---:|---:|---|
| scheduler | 1.642 s | 1.117 s | 0.68 | 1.0000 | match |
| database | 1.702 s | 1.171 s | 0.69 | 0.96875 | match |
| network | 1.691 s | 1.237 s | 0.73 | 1.0000 | match |
| compiler | 1.610 s | 1.130 s | 0.70 | 0.98958 | match |

Median exact 1.667 s; median hyperbolic 1.151 s (31% lower). Intended source ranked first in 4/4. Both unrelated controls chose recompute. Exact control agreement was 1.000 in 4/4. Gates H1, H2, H4, H5, H6 pass. H3 fails because database is 0.96875 < 0.98. Overall FAIL; no promotion.

Interpretation: approximate nested-cache reuse is technically possible and the measured speed headroom is large enough to justify more research, but the current free-energy policy is too aggressive at 75% reuse. The full-forward quality oracle is post-timing and not deployable online. See `serving/HYPERBOLIC_CACHE_AB_ADVERSARIAL_REVIEW.md` for every edge disposition.

## Matched Euclidean K/V cache A/B — FAILED quality gate (2026-11-10)

Preregistered before implementation in `serving/EUCLIDEAN_CACHE_AB_DESIGN.md` and `serving/ab_euclidean_cache_prereg.md`. This held workload, coordinates, lexical guard, free-energy policy, anchors, snapshots, timing, and oracle fixed; only Poincare distance became L2 distance.

| case | Hyper exact | Hyper cache | Hyper agreement | Euclidean exact | Euclidean cache | Euclidean agreement |
|---|---:|---:|---:|---:|---:|---:|
| scheduler | 1.642 s | 1.117 s | 1.0000 | 1.637 s | 1.145 s | 1.0000 |
| database | 1.702 s | 1.171 s | **0.96875** | 1.711 s | 1.175 s | **0.96875** |
| network | 1.691 s | 1.237 s | 1.0000 | 1.684 s | 1.226 s | 1.0000 |
| compiler | 1.610 s | 1.130 s | 0.98958 | 1.583 s | 1.117 s | 0.98958 |
| **median** | **1.667 s** | **1.151 s** | — | **1.661 s** | **1.160 s** | — |
| **speed reduction** | — | **31.0%** | — | — | **30.1%** | — |
| **quality gate** | — | **FAIL (H3)** | — | — | **FAIL (E3)** | — |

Median exact 1.661 s; Euclidean 1.160 s (30.1% lower). E1, E2, E4, E5, and E6 passed; E3 failed. The Euclidean arm chose the same 75% anchors and had the same quality pattern as nested hyperbolic on these cases. Therefore the curvature choice was not the cause of drift here; the aggressive reuse depth is the active suspect. Production remains exact-prefix. Full dispositions: `serving/EUCLIDEAN_CACHE_AB_ADVERSARIAL_REVIEW.md`.
