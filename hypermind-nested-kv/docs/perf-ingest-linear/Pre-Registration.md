# Pre-Registration — ingest linearization (written BEFORE the fix, 2026-11-01)

Predictions:
1. Scaling curve after fix: ~2x wall per doubling (linear total, ~0.15ms/ingest).
2. Full suite stays green, incl. ledger-root consistency + recover roundtrip
   (lazy receipts are byte-identical).
3. New tests: cover-all-atoms, refold-fixpoint, fmt-equality, restore-preserve.
4. Boot on 8K snapshot < 1s; recover on 8K works.
5. Extrapolation: 10K ≈ 1.6s, 100K ≈ 16s, 1M ≈ 3 min ingest-side.

Judges: wall-clock scaling runs, cargo test, cargo clippy, snapshot/recover probes.
