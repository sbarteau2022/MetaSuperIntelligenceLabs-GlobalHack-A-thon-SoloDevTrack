# Native Glimmer baseline preflight, 2026-11-03

Command: `cargo run --release --bin glimmer_forward_preflight -- manifests/muse-glimmer-candidate.manifest --native-baseline` from `globalmind-runtime/`.

Observed exit: 1. Frozen artifact rehash passed; roster digest `187597bb49578ac31e250da2240b6960d0af5487399c0df77d46ee19771b5c04`, input digest `5ec5680c736f3d4aaee64827e67e8ef037a34d4e5f9ce570f66385a563b6c465`. Position mode: native Glimmer RoPE/NoPE; TPPE memory address not exercised. Stopped because `GLIMMER_NATIVE_FORWARD_BIN` is unset. No inference or mind round trip was run.

The baseline mode isolates the trained model's native position rule from the unpromoted TPPE Q/K ablation. A pinned native Glimmer executor and real forward receipt remain required.
