# Design — refold linearization + versioned snapshots (2026-11-01)

Item 1 — refold O(N^2) -> O(N log N)-shaped:
- Root cause chain (audited): (a) per-seed full uncovered scan, (b) per-seed
  O(uncovered) result filter, (c) float-unsound tree from degenerate-split
  guard, (d) shell-tree/metric mismatch (fold gate is raw-hyperbolic).
- Fix: metric-matched raw VP tree (built+queried with hyperbolic_dist) +
  EXACT range search (same metric both sides) + ball-side intersection with
  BTreeSet uncovered + deterministic-shuffle vps. Absorbed SET identical to
  full scan -> receipts byte-identical (proven by brute-force equivalence).
- Pruning padded with 1e-9 eps (lorentz_inner cancellation can violate the
  triangle inequality at 1e-13 scale near boundaries; padding never misses,
  emission stays exact).

Item 2 — versioned snapshot container:
- Layout MAGIC(8)="HM3SNAP\\0" | VERSION u32 LE (=1) | LEN u64 LE | bincode.
- Reader classifies: Missing / Current / LegacyRaw / Unusable(reason).
  Unusable is LOUD (exact reason); unknown versions refuse (never misparse).
- Legacy raw bincode: best-effort parse, else ledger replay (unchanged path).
- 20K ledger replay boots in 4.7s (one-time upgrade cost; 1M ≈ 4-8 min).
