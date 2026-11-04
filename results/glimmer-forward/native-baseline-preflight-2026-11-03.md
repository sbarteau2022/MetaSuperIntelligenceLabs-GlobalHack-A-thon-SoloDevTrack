# Native Glimmer baseline preflight, 2026-11-03

Command: `cargo run --release --bin glimmer_forward_preflight -- manifests/muse-glimmer-candidate.manifest --native-baseline` from `globalmind-runtime/`.

Observed exit: 1. Frozen artifact rehash passed; roster digest `187597bb49578ac31e250da2240b6960d0af5487399c0df77d46ee19771b5c04`, input digest `5ec5680c736f3d4aaee64827e67e8ef037a34d4e5f9ce570f66385a563b6c465`. Position mode: native Glimmer RoPE/NoPE; TPPE memory address not exercised. Stopped because `GLIMMER_NATIVE_FORWARD_BIN` is unset. No inference or mind round trip was run.

The baseline mode isolates the trained model's native position rule from the unpromoted TPPE Q/K ablation. A pinned native Glimmer executor and real forward receipt remain required.

## Follow-up on 2026-11-03: local native baseline only

The existing ignored `core_lm/target/release/generate_glimmer` executable was SHA-256
`ada4357a801161d54efebf6196aab9aec326fc9d7a55e7553fd79fd22a85dfbe`.
With that exact path and hash supplied as `GLIMMER_NATIVE_FORWARD_BIN` and
`GLIMMER_NATIVE_FORWARD_SHA256`, the release preflight exited 0. It rehashed
the frozen model artifacts and reported the same roster and input digests above.
It explicitly reported `tppe_memory_address=not_exercised` and
`inference_not_run=true`.

Separately, `GLIMMER_STOP_AFTER_PREFILL=1 ./core_lm/target/release/generate_glimmer
'The capital of France is' 1` exited 0 on the local Metal path. Prompt IDs
were `[200000, 954, 7963, 323, 11698, 373]`; final prefill argmax was token
`13796` (`Paris`), logit `17.3479`, after a reported 5.6 s prefill. This proves
one frozen-weight local baseline prompt, not a universal decoder, exact tensor
K/V page, Surreal recall, automaton admission, or full harness forward pass.
