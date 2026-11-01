# Successes — Step 4 (2026-11-01)

1. Shared `rank_candidates()`: VP and SDB paths rank identically by construction.
2. `map.db_candidates` / `map.query_sdb` / `map.recall_bench` live, role-gated, receipted.
3. Battery: 300 ingests (300 shadow-ok, 5.7s wall incl. 900 HTTP round-trips),
   recall + latency reported per query.
4. 72 tests pass (61+3+8), clippy clean.
5. Scratch namespaces removed (scratch_step4, scratch_probe); core untouched.
