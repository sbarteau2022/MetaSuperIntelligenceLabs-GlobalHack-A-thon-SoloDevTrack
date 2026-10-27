# C06 — A40 Run Manifest v1 (FROZEN)

**Status.** [PROPOSED] Frozen run inputs for A40 under `Pre-Registration-A40-v1.1.md` (canonical hash `ac991e6e…`). **Date:** 2026-10-27.
**Rule.** This file is append-only once hashed. Any change = manifest v2 + new hash + new run id. ADR-19/ADR-22 apply.

## A40-RUN-01 (geometry + preservation + reuse, laptop hardware)

- **Run id.** `A40-RUN-01`. **Seed.** `0xA40C06`. **Decoding.** Greedy, temperature 0. **Batch order SHA-256.** Filed in run receipt at execution (computed from frozen input lists below).
- **Hardware (pinned, VERIFIED this machine).** Apple M5 Pro (18 cores: 6P+12E), arm64, 48 GB RAM, macOS 27.0.1 build 26A434, Darwin 27.0.0, Python 3.14.7, mlx 0.32.1. Disk: 412 GB free of 926 GB. `hardware` field logged per §8+C-A40-06.
- **Base model (REPORTED, verify at pull).** `mlx-community/Muse-Glimmer-30B-4bit` (Hub: 10,107 downloads, 18 likes, mlx + safetensors + apple-silicon tags, license Apache-2.0, base `meta-models/Muse-Glimmer-30B`). Tensor count 1,568 REPORTED from protocol — verified at pull per §2 step 1. Model REVISION pinned at pull time in run receipt (commit hash of downloaded snapshot).
- **Map source (VERIFIED by parse).** `live_brain.atoma_map` (`ATOMA_MAP_V1`, epoch 1, 8,644 atoms, 104,312 arcs, 5 lineage entries incl. `Muse-Glimmer-q4km-gs128-MLX`). Quad pool: 28,638 candidate (G,P) pairs, outdegree max 1493 / mean 3.98, 4,042 branching nodes — ≥120 quads / ≥4 depths feasible.
- **T_place / T_test split (C-A40-01).** Split by top-level branch root: even root ids → T_place (placement fitting), odd root ids → T_test (G1 triples). Placement script receives node features with ancestry stripped; triple sampler has no embedding access. Overlap audit (zero shared node ids) logged; any overlap VOIDS G1.
- **Quad construction (C-A40-02).** Per v1.1: P at depth d with parent G; C = child of P; U = child of G, U≠P; X = different top-level branch. 120 quads, ≥4 depths, ≥25/depth, frozen list in receipt.
- **M rubric (C-A40-03).** 1.0 zero-reprefill / 0.5 ≤10% / 0.0 else; floors M_C ≥ 10×M_A AND ≥ 10×M_B AND ≥ 0.50; void-if-M_A<0.05.
- **N + power (C-A40-04).** ≥150 base-correct/domain over 4 domains (factual-QA, summarization-faithfulness, instruction-following, code-repair); Wilson 95% CI; macro-average aggregation (C-A40-06).
- **Arms (C-A40-05).** A / B0 (Euclidean+map, NO Δ_H) / B / C + ungated D diagnostic. Δ_H effect = B−B0. Batch order identical across arms.
- **Repeatability (C-A40-07).** G1 test 5 + G4 as two separate OS processes, byte-compared; pins logged.
- **Replication + necessity (C-A40-08).** Second machine distinct SoC/RAM; Δ_H-removal ablation must collapse M + fail G1.
- **Stage 1 of this run (executable TODAY, no weights download).** Ingest manifest + execute the G1-equivalent geometry test on the REAL vault (`live_brain.atoma_map`) as the T_test-side dry run: quad sampling from odd branches, Lorentz-distance ordering, radius monotonicity, manifold validity, two-process repeatability. Gate thresholds applied exactly as filed. Verdict recorded below as the A40 test result for the native-map geometry leg.
