# Hardware gate runbook — fixed PPC

## Universal gate (Stewart, 2026-10-29)

No synthetic runs are allowed to ship. Ever.
A synthetic or simulated run is not ship evidence.
The fixed PPC must be run on hardware (this Mac) before anything ships.
Nothing from the VM prototype ships on VM evidence alone.

## Current state, verified 2026-10-29 from the Mac file system

- Repo: ~/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/research-grade-harness/
- src/rsi.rs: still the word-count score() — 0.5 plus 0.05 for improve/verified, plus 0.03 for optimize, cap 0.99. The G6 fix is still needed here.
- src/evolution.rs: already has the frontier fix (candidate_on_frontier = frontier.contains(&1)) plus a security floor. Do NOT reapply the old frontier fix from the VM audit copy; that copy was stale on this point.
- Cargo on this Mac, previously verified: 1.98.1.
- No rgah source was changed by staging this runbook.

## Hardware run, in order

1. Back up first, in Terminal on this Mac:
   cp src/rsi.rs /tmp/rsi.rs.bak-20261029
   cp src/evolution.rs /tmp/evolution.rs.bak-20261029

2. Port the fixed PPC to Rust, in src/rsi.rs, following the VM audit report (not the VM prototype code directly):
   - Add a probe function parameter to ProposeProbeCompose::run(), so quality comes from a real probe, not score().
   - Deduplicate proposals by Bundle::fingerprint() before probing.
   - Enforce the budget: do not record a probe that cannot be afforded.
   - Wire leakage_critic(), the 0.02 noise band, and regularized_adopt() into adoption. KEEP_BASE stays the default.
   - Return fingerprint, per-proposal passes, judge ids, and measured tokens in PpcMetrics.
   - Keep score() only as a labelled synthetic fallback, never as the adoption signal. Do not delete it silently; label it.

3. Run on this Mac, in Terminal:
   cd ~/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/research-grade-harness
   cargo test

4. Run the fixed PPC probe on this Mac with a real model signal (not the word score), on the 6 probe tasks plus 4 held-out tasks from the VM prototype, with deterministic last-number judges.
   - EXECUTED 2026-10-29 (VERIFIED): `src/bin/hardware_ppc.rs`, 8 probe + 4 held-out (task set widened: elementary items saturated, all bundles 8/8, then word problems added — still saturated). VM prototype task list not present in this repo; authored replacement set is documented in the receipt. Model: Qwen3.6-MoE-Combined via omlx :8000.

5. File the hardware receipt as plain text in this folder (no JSON, per the Mac no-JSON rule for receipts). The receipt must list: cargo test result, model used, per-bundle passes, tokens, regulators passed or failed, held-out result, and adopted bundle or KEEP_BASE.
   - FILED 2026-10-29 (VERIFIED): `HARDWARE-PPC-RECEIPT.txt` — cargo release green (18+10+8+1+10), real_probe=true, per-bundle 8/8, tokens, regulators (noise_band correctly failed=abstain), held-out 4/4, KEEP_BASE adopted. Verdict: honest abstention on measured signal; task-set ceiling noted with material-change escalation (harder tasks).

## Observability rule (Stewart, 2026-10-29)

The point of a harness run is that he can observe it.
Every run must leave a live status, a log, and a why-receipt on every failure: what ran, command, exit, last output, expected result, evidenced cause, unblocker.
A silent run, or a failure with no why, is itself a failure.
Apply this to the hardware run above: do not just report pass or fail. Leave the log and the why-receipt with the receipt.

## What counts as pass

- cargo test passes on this Mac.
- Fixed PPC decision is backed by real probe passes, not word scores.
- Held-out result is filed, not just probe result.
- No synthetic number is presented as a measured result.

## What does NOT count

- A VM run, a Python prototype run, or a simulated score, no matter how clean it looks.
- A cargo test pass on the old synthetic code alone. That proves the old code compiles, not that the fixed PPC works.
- Any claim that this runbook itself is a hardware run. It is staging only.
