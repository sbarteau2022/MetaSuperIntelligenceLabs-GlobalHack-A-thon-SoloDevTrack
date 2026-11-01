# Successes — ingest linearization (2026-11-01)

1. Ingest O(N^2) -> O(1) amortized: 8K in 1.3s (was 7.2s; was-56s at 2K pre-fix).
2. Four compounding fixes, each semantics-preserving, each tested.
3. 5 new tests (cover, fixpoint, fmt-equality, restore, guard pre-existed).
4. 83 tests pass, clippy clean. Boot 8K 0.5s, recover 8K ok.
5. No SurrealDB impact (shadow paths untouched); core DB not touched by this work.
