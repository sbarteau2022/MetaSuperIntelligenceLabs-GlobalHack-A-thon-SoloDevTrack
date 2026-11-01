# Claims Register — ingest linearization (2026-11-01)

- VERIFIED: baseline quadratic 3.6/14.2/56.5s (500/1K/2K); post-fix linear
  0.34/0.29/0.59/1.27s (1K/2K/4K/8K, ratios ~1x/2x/2.2x).
- VERIFIED: dedup-index bisection — 8K identical-text ingests in 0.87s
  (coalesce path O(1)), isolating the mint path as the quadratic.
- VERIFIED: profiler showed basin_append 80% of ingest (257us p50), growing
  with N despite 1 basin — traced to receipt format+hash over full member list.
- VERIFIED: post-lazy curve + 83 tests pass (72+3+8) + clippy clean.
- VERIFIED: incremental layout is a refold fixpoint (test); restore preserves
  layout + root + idf (test); fmt push == full format (test).
- VERIFIED: legacy Oct-28 snapshot does NOT parse with current binary
  (atoms: 0, sha matches). Pre-existing bincode positional fragility (silent
  `if let Ok` in both open paths predates this work); ledger replay is the
  designed fallback. New->new roundtrip verified by test.
- CORRECTION (dated 2026-11-01, replaces step-7 record): step-7 "80 tests
  (69+3+8)" was an arithmetic error. The bin held 68 then; correct count was
  68+3+8=79. Current count is 72+3+8=83.
- INFERRED: reweight/recover refold path still O(N^2) comparisons (cheap ops,
  no trig). Rare explicit ops, not ingest — recorded for step 8, not fixed here.
