# A39 canonical pieces — staged, not yet applied to the server tree

Built 2026-10-27, after Stewart's 0/24 uninformative run and his list of
six required pieces. Nothing here was run against the live server or vault,
because Stewart's 1:37pm control run is active on the Mac. Applying the
runner patch or probing the vault while that run is active would contaminate
it. The order after that run finishes is at the bottom.

## Sign-off, 2026-10-27
Alignment SIGNED: **B — throwaway pre-ingest** (Stewart's instruction
2026-10-27 14:01 CDT to make the best A/B decision; rationale in
`prereg_a39_amendment.json`). Fallback policy is NOT signed by that
decision and remains recommended-not-signed (fallback counts as fail).
This pack is distinct from `A39_CORPUS.json` in the same folder: that
file is the 40-task intervention-threshold workload from the earlier
A39 pilot; this pack is the judge/instrumentation/preregistration set
for a judge-verified paired run. Do not conflate the two corpora.

## Files

- `corpus_a39_candidate_v1.json` + `.sha256` — (1) 48 tasks, 16 per
  domain, 6 original tasks preserved, 42 new. Each task carries its
  preregistered judge (token_contains, case-insensitive, or
  last_number_eq for arithmetic). Status is CANDIDATE, not frozen, on
  purpose: a task that has never been seen in decoded output cannot be
  honestly frozen. Hash: `1add5f405ba6d168070fbbf32195b9eb3556a28974f3df0610d823160822aedd`.
- `probe_corpus.py` — (1) promotion script. Probe both arms on a vault
  COPY, re-judge from the probe file, drop tasks neither arm passes,
  freeze only if the counted set is >= 40. Tested on synthetic probe
  output: 48 candidates -> 42 counted, 6 dropped, frozen hash computed.
- `prereg_a39_amendment.json` — (2) alignment, (5) fallback, J1-J6
  definitions, instrumentation, judge rule, cold-start/replication.
  Status PROPOSED, not filed: a preregistration needs Stewart's sign-off.
  Recommended: alignment B (throwaway pre-ingest), fallback counts as fail.
- `judge_rust.rs` — (4) canonical judge in Rust, std only, mirrors the
  tested Python judge (token_contains, normalized, last_number_eq,
  sha256). Place at `orchestrator/src/judge.rs`. NOT compiled here: no
  rustc in this VM; cargo check on the Mac is step 2 below.
- `runner_receipt_patch.md` — (3) + (4) exact patch: receipt fields,
  warm-up removal, judge call sites (old lines ~299 and ~400), full
  decoded text in AblationResult, RSS sampler, gate-counter TODO.
  Two honest non-inventions in it: no `tool_call_count` (no external
  tools exist in this transformer; gates/backtracks are named as such),
  and no discrete `peak_vram_bytes` (unified Metal has no such counter;
  J5 is sampled RSS + separate metal kernel telemetry).
- `analyze_a39.py` — (3)+(4) J1-J6 from run files alone, paired by
  task_id, re-judges every row, errors in the denominator, fallback
  reported separately. Tested on synthetic paired runs; a bootstrap
  bug found in testing (p ~0.56 for a 28-point gap) was fixed to the
  null-centred version (p = 0.0002 for 12-0 discordant pairs) before delivery.
- `replicate_a39.sh` — (6) cold-start + replication script. Second
  machine is NOT named here: none was verified from the Mac in this
  session, and the only other copy found is on the same Mac. Replication
  is earned only when a named second machine produces a run file with
  the same corpus hash and judge version.

## What was verified in this session (and what was not)

Verified from the Mac tree (read-only):
- Filed prereg CS-ABLATION-01: 200 tasks promised, manifest hash
  TO_BE_COMPUTED, corpus on disk = 6 tasks. Filed metrics: pass delta
  >= +10pp p<0.01 bootstrap 10k, retries -30%, tokens -25%, latency
  -20%, VRAM -15%, cross-hardware M3_Max/M2_Ultra.
- Fallback already exists in code (continuous_active_mind.rs ~2791):
  on "superposition fail-closed" only, re-execute on parent map,
  `fallback_to_parent=true`. The old runner ignored that flag.
- Decoded text is transducer egress from emitted ops (byte-level decode
  plus `<atoma.xxx>` mnemonics), so a keyword appearing in decoded text
  depends on the walked trajectory spelling it — which is why task
  verification by probe, not by assertion, is the only honest freeze.
- No J1-J6 list was found in the filed prereg or Claims_Register; the
  J-mapping in the amendment is inferred from the filed metrics and is
  flagged as such for Stewart to correct.

NOT verified, and not claimed:
- No task in the candidate corpus has been seen in live decoded output.
  The corpus is a candidate until the probe runs.
- The Rust judge and runner patch are uncompiled (no rustc here).
- No second machine, no cold-start run, no CC-013 evidence.

## Order after the active run finishes
1. Stewart signs alignment + fallback (options in the delivery message).
2. Apply `runner_receipt_patch.md` + `judge_rust.rs` to the Mac tree,
   `cargo check --release --bin ablation_runner`, run `judge.rs` tests.
3. Probe on a vault copy (`probe_corpus.py`), freeze corpus (>= 40).
4. Run A39 via `replicate_a39.sh` on the frozen corpus, cold start.
5. `analyze_a39.py` on the run files; claim or correctly fail J1-J6
   from that report alone.
