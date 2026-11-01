# Pre-Registration (written BEFORE verification runs, 2026-11-01)

Predictions:
1. `cargo test`: 57 unit + 3 + 8 integration pass (9 new unit tests added).
2. `cargo clippy --all-targets`: zero errors; pre-existing dead-code warnings remain.
3. Live scratch-namespace e2e: ingest shadow-ok, 2 links ok, neighbors 1-hop,
   path 2 hops, traverse via_mode=adjacency-direct, semantic ok (sim>0.01),
   causal rejected schema-error, db_status parity true.
4. Core DB after all e2e: 2,079 atoms / 2,079 addresses / 0 edges (probe
   residue deleted; incident rows 1-2 restored byte-identical by digest).
5. `map.db_status` parity_atoms=false on any fresh map (expected: memory has
   only new atoms; canonical cutover NOT claimed).

Judges: cargo test output, cargo clippy output, /sql query outputs, digest match.
