# Q1.5 candidate-v2 tensor K/V requalification — 2026-11-02

Status: PASS for the frozen Qwen3.6 Layer-3 reconstruction and its exact-model tensor K/V contract. This does not certify full-model execution, cross-model K/V reuse, or TPPE quality.

The test uses `tppe-layer3-candidate-v2`, the candidate manifest and contract, the candidate numeric configuration, and pure toroidal phase rotation plus radial scale at the Q/K boundary. It no longer uses the retired RoPE-frequency hybrid.

The earlier apparent hang was a debug-profile run inside `Manifest::verify_artifacts()`, which streams SHA-256 over every rostered artifact before model loading or database startup. A worker-thread sample showed SHA-256 compression; the main test-harness thread was waiting normally. The debug diagnostic was interrupted. This was not evidence of a SurrealDB stall.

The optimized command completed: `cargo test --release --test tensor_kv_qualification -- --nocapture --test-threads=1`. Result: 1 passed, 0 failed, 42.64 s. Full roster rehash: 33.84 s. Model binary load: 39.86 ms. Isolated SurrealDB boot and schema: 6.32 s. `cargo check --tests -q` passed after adding phase markers.

The hardware receipt at `results/tensor-kv/tensor-kv-qualification-report.txt` records all seven gates: session creation, two real K/V tensor pages, process restart, byte-for-byte and SHA-256 equality, injected decode with exact hidden/logit parity across 248,320 logits, 20/20 identity misses and 11/11 rejected invalid writes, plus hardware latency and memory measurements. The database was isolated from the quarantined port 8000 instance.

The phase markers remain in the test so a future slow roster hash is visible. No live mind database was changed. The old reference-v1 receipt must not be substituted for this candidate-v2 receipt.
