# Forward recall and active-inference gates

- [ ] Measure cache-hit and memory effects of the versioned radix key under a frozen workload; only identity correctness was tested.
- [ ] Connect `ForwardRecall` to a real model executor and verify the consumed K/V page against a baseline forward on the same token tape. Current bridge test proves only the read/join boundary.
- [ ] Verify full-model activation and logit parity on frozen artifacts; Layer-3 component evidence cannot certify the whole model.
- [ ] Build Rust-native Qwen VL query embedding parity against the stored 1024D ingest recipe, then test held-out brain-library retrieval.
- [ ] Correct `fep.rs` coefficient units and beta-limit interpretation; verify a physical energy calibration before using joules in a score.
- [ ] Make `PolicyRanker` honor an explicit resource budget and quality floor; current execution ranking ignores its `landauer_cost`.
- [ ] Specify a quality-floor and resource-budget controller with measured-cost calibration before promoting AttentionEconomics as policy. The current reward/free-energy equivalence is refuted in `report.tex`.
- [ ] Independently review the math and the live counterexample; no agent vote was run because spawning agents requires owner approval.
- [ ] Measure recall candidate scan latency at current corpus size before claiming fast graph traversal; the current read path ranks candidates client-side.
