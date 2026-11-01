# Design — ingest O(N^2) -> O(1) amortized (2026-11-01)

Mandate: no step 8 until the quadratic is ironed out.

Measured baseline (pre-fix, debug build): 500→3.6s, 1000→14.2s, 2000→56.5s
(4x per doubling = O(N^2); projected 1M ≈ 163 days).

Root causes found (in order):
1. Dedup full-scan: re-tokenized + re-sorted EVERY atom's text per ingest.
2. Basin full refold per ingest (rebuild_basins): O(N) trig-heavy compares.
3. Receipt reformat: format!("{:?}", members) is O(members) per ingest.

Fixes (all semantics-preserving, receipts bit-identical):
1. Atom gains cached `topic: [f64;32]` + `toks_hash: u64` (serde defaults;
   backfilled once at open). Dedup via HashMap<toks_hash, ids> (same coalesce
   rule, bucketed). Index built O(N) at open, O(1) per ingest.
2. basin_append: incremental, domain-scoped, centroid-gated (same fold rule);
   running sums (sum_coord/phase/salience/topic) make refresh O(1); receipts
   bit-stable (member-list hash unchanged).
3. Receipt/member strings cached (members_fmt); refresh is O(1) amortized.
4. Receipt/root recompute moved LAZY (basins_dirty flag): ingest never touches
   receipts/root (its output doesn't expose them); carve/dissolve/snapshot/
   export refresh first. Bytes identical to eager — recover() verifies.
5. Snapshot persists idf; open/recover adopt it (no recalibrate/reembed);
   basins/hunger/kv/checkpoint/epoch restored as-is (no refold on boot).
   Legacy path (no stored basins/idf): one-time calibrate + single refold.
