# 06 — mlx-vlm and mlx-lm audit (reference libraries)

Both are **on this machine**, as copies shipped inside oMLX 0.7.0 (alias **SP** = `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/`):
- `mlx_lm` **0.31.4.dev132+g94cdcae13** — `SP/mlx_lm/` (`mlx_lm-0.31.4.dev132+g94cdcae13.dist-info/METADATA:3`).
- `mlx_vlm` **0.7.1** — `SP/mlx_vlm/` (`mlx_vlm-0.7.1.dist-info/METADATA:3`; `mlx_vlm/__init__`-level `__version__ = "0.7.1"`).
- A second `mlx_lm-0.31.3` ships inside rapid-mlx (doc 03). Not diffed.
- Upstream repos not read online; these are the installed trees. Upstream: https://github.com/ml-explore/mlx-lm and https://github.com/Blaizzy/mlx-vlm (**[unverified]** as sources).

## 1. mlx-lm: the Qwen3.5 / Qwen3-Next reference

### 1.1 Qwen3.5 (`mlx_lm/models/qwen3_5.py`, 572 lines)
- `TextModelArgs` (`:26`), `norm_topk_prob: bool = True` (`:52`).
- `GatedDeltaNet` (`:87–206`): the authoritative linear layer. Components: `conv1d` depthwise (`:106`), `in_proj_qkv` sized `key_dim·2 + value_dim` (`:116`), `in_proj_b` (`:119`), `in_proj_a` (`:120–121`), `dt_bias` (`:122`) and `A_log` (`:125`), `self.norm` = `Qwen3NextRMSNormGated` imported as `RMSNormGated` (`:21`, built at `:127`), `out_proj` (`:129`). Forward `:133–206`: conv-state cache `:153`, q/k/v split `:170`, `normalize_qk` `:180`, `gated_delta_update` `:182`, gated norm `self.norm(out, z)` near `:200`.
- Text model: `Qwen3_5TextModel` (`:242`), `TextModel` (`:317`); weight `sanitize` transposes unsanitised conv1d (`:346–368`).
- Quantisation predicate: `mlp.gate` and `shared_expert_gate` are **always 8-bit, group 64** when `num_experts > 0` (`:371–381`), everything else default. This is the source of the 8-bit gates in the checkpoint.
- Cast predicate keeps `A_log` in fp32 (`:383–391`).
- Tensor-parallel `shard` (`:441+`) splits `dt_bias`, `A_log`, conv groups per rank (`:470–491`).

### 1.2 Gated DeltaNet kernels (`mlx_lm/models/gated_delta.py`, 647 lines)
- `compute_g` = `exp(-exp(A_log) · softplus(a + dt_bias))` (`:19–21`); `compute_lower_bound_g` (`:24–30`).
- `normalize_qk` — L2 with `inv_scale` folded into q (`:33–44`).
- Metal kernels: generic (`:47`), xtree comparator (`:152`), packed Dk=128 scalar-gate prefill specialisation (`:247`); env kill-switch `MLX_GDN_PACKED` default on (`:15`).
- Reference loop: `_gated_delta_step_ops` (`:385`), `gated_delta_ops` (`:557`), public entry `gated_delta_update` (`:605`).

### 1.3 Qwen3-Next (`mlx_lm/models/qwen3_next.py`, 525 lines) — the MoE/attention contract
- Attention `Qwen3NextAttention` (`:74`): `q_proj` produces queries and gate (`:82–102`), `q_norm`/`k_norm` (`:103–104`), RoPE (`:106–110`), output `o_proj(output · sigmoid(gate))` (`:151`).
- MoE `Qwen3NextSparseMoeBlock` (`:299`): `norm_topk_prob` (`:306`, applied `:332`), `shared_expert_gate = Linear(dim, 1)` (`:314`), `shared_y = sigmoid(shared_expert_gate(x)) * shared_y` (`:339`).
- Decoder layer (`:349`). These are the items the seat is missing (doc 01 §2).

### 1.4 Cache and batching (`mlx_lm/models/cache.py`, `mlx_lm/generate.py`)
- Cache classes: `_BaseCache` (`:154`), `ConcatenateKVCache` (`:195`), `QuantizedKVCache` (`:251`), `KVCache` (`:339`), `RotatingKVCache` (`:423`), **`ArraysCache` (`:596`)** — used for the GDN conv and recurrent state (hybrid caches), `ChunkedKVCache` (`:727`), `CacheList` (`:803`), `BatchKVCache` (`:880`), `BatchRotatingKVCache` (`:1101`), `TokenBuffer` (`:1462`), `PromptTrie` (`:1508`), `LRUPromptCache` (`:1599`) **[read as declarations]**.
- Generation: `stream_generate` (`mlx_lm/generate.py:658`), `generate` (`:763`), `BatchGenerator` (`:1512`) **[read as declarations]**.

## 2. mlx-vlm: the VLM and MTP side

### 2.1 Qwen3.5 in mlx-vlm (`mlx_vlm/models/qwen3_5/language.py`)
- `Qwen3_5RotaryEmbedding(MRoPERotaryEmbedding)` (`:26`), `Qwen3_5RMSNormGated` (`:48`), `Qwen3_5Attention` (`:850`), `Qwen3_5GatedDeltaNet` (`:1014`) which calls `gated_delta_update` (`:1135`), `Qwen3_5DecoderLayer` (`:1158`), `Qwen3_5Model` (`:1199`), `LanguageModel` (`:1370`) **[read as declarations]**. Imports `gated_delta_update` from its own `gated_delta` module (`:20`).
- Other families with linear attention: `mlx_vlm/models/qwen3_next/language.py` (exists), `mlx_vlm/models/qwen3_5_moe/` (exists) **[names only]**.

### 2.2 MTP tooling
- `mlx_vlm/split_mtp.py:1–8` — "Extract a model's native MTP tensors into a standalone MLX drafter", dispatching by `model_type` through an `MTP_SPLITTERS` registry **[read]**. This is the tool that produces the separate `model-mtp.safetensors` style drafter (the Qwen3.6 checkpoint in doc 01 has such a file).
- `mlx_vlm/speculative/`: `common.py` (`verify_forward` `:13`, RNG state copy/restore `:47–51`), `ddtree.py` (`DDTreeNode` `:27`, `build_ddtree` `:57`) **[read as declarations]**.
- Other mlx-vlm modules of note: `turboquant.py`, `kv_quant.py`, `apc.py` / `apc_coordinator.py` (automatic prefix caching), `moe_offload.py`, `generate/` **[names only]**.

## 3. Consequences for the seat (doc 01)

- The reference Qwen3.5 forward is available in two places (mlx-lm and mlx-vlm), and oMLX, rapid-mlx and Swift all build on the same design. The seat's linear attention is not a variant of this design; it is a different algorithm.
- Quantisation predicates here (`qwen3_5.py:371–381`) are the authoritative source for per-tensor bits. The seat should read them from `config.json` (see doc 01 §5.2), not from `optiq_metadata.json`.
- `norm_topk_prob=True` default (`qwen3_5.py:52`) and the shared-expert sigmoid (`qwen3_next.py:339`) are the two MoE facts the seat must implement.

## 4. Not verified

- Behaviour of `mlx_lm`'s `BatchGenerator` under speculative decoding (only declarations read).
- Upstream differences between the installed trees and `main` on GitHub (not diffed).
- `mlx_vlm` `split_mtp` output format compatibility with the Qwen3.6 `model-mtp.safetensors` in CKPT (not run).
