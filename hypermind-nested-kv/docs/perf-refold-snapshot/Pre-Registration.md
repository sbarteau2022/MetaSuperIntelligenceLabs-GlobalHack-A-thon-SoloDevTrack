# Pre-Registration (written BEFORE the work, 2026-11-01)

Predictions:
1. Synthetic spread refold (all-singleton worst case): pre-fix superlinear
   (10/19/53/320/1936ms for 500-8K); post-fix near-linear, basins identical.
2. Brute-force equivalence test passes on spread corpus (member sets equal).
3. Full suite stays green (incl. mirror + ledger-root + recover tests).
4. Container: magic/version/length verified; foreign/truncated/future-version
   files classify correctly; recover on 8K linear (0.36/0.72s for 4K/8K).
5. 20K ledger replay boots in <10s.

Judges: scaling probes, cargo test, clippy, container tests, replay drill.
