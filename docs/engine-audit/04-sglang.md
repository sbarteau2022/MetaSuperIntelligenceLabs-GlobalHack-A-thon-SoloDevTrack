# 04 — SGLang audit (Qwen3.5 / Qwen3.6 MoE and speculative decoding)

Local source: `/Users/stewartbarteau/src/sglang` (alias **SG**), `python/sglang/srt/…` (alias **SRT**). HEAD `df13d5dce31abfeff0e701a62e8c39a136758e6c`, 2026-10-20, "[Feat] sgl-router: Route prefill buckets by uncached tokens (#45103)" — from `git log -1` **[read]**. Read locally; not run.

Legend as in doc 03: **[read]**, **[vendor]**, **[unverified]**.

## 1. Position

- SGLang is a CUDA/ROCm-first serving stack. Backend flags in the Qwen3.5 model code are `_is_cuda`, `_is_hip`, `_is_xpu`, `_is_cpu` (`SRT/models/qwen3_5.py:1569–1582`) **[read]**. No Metal/MLX backend was found in that file. Whether SG has any Apple Silicon path was **not** verified beyond that.
- So SGLang is a *design reference* for the scheduler, radix cache and speculative machinery; it is not a runnable peer on this Mac.

## 2. Qwen3.5 / Qwen3.6 model code (`SRT/models/qwen3_5.py`)

- Linear layers: `Qwen3_5GatedDeltaNet` (`:355`); the recurrent op goes through `RadixLinearAttention` (`:103` import, `:489` construction with `num_q_heads`, `num_k_heads`, `num_v_heads`, `head_q_dim`, `head_k_dim`, `layer_id`) **[read]**. The layer uses a `conv1d` weight view and `dt_bias` sharded across attention TP (`:~470–489`) **[read]**. The actual kernel dispatch lives behind `RadixLinearAttention` (`SRT/layers/radix_linear_attention.py`, not read).
- Linear decoder layer: `Qwen3_5LinearDecoderLayer` (`:1119`) **[read]**.
- Full attention: `Qwen3_5AttentionDecoderLayer` (`:1259`); `attn_output_gate = getattr(config, "attn_output_gate", True)` (`:1312–1313`); the q projection is widened to `total_num_heads * (1 + attn_output_gate)` (`:1332`) and `q_heads = num_attention_heads * (2 if attn_output_gate else 1)` (`:1706–1707`) **[read]**. Partial RoPE via `get_rope(..., partial_rotary_factor=…)` (`:1305, :1316–1322`) **[read]**.
- `q_norm` and `k_norm` are `GemmaRMSNorm` (`:1402`) **[read]** — the Gemma-style `(1+w)` form. That is a concrete reference for the open question in doc 01 §2.5.
- Fused paths for CUDA and other backends: `:1437–1582` (fused qk-norm + RoPE + gate kernels with `has_gate`, `mrope_axis_map`) **[read]**.
- Model classes: `Qwen3_5ForCausalLM` (`:1679`), `Qwen3_5MoeForCausalLM` (`:2039`), `Qwen3_5ForConditionalGeneration` (`:2348`), `Qwen3_5MoeForConditionalGeneration` (`:2560`) **[read]**.
- Checkpoint loading skips MTP tensors in the main model: `if "mtp" in name:` (`:1980`, `:2177`, `:2444`, `:2746`) **[read]**; the MTP head is a separate class in `SRT/models/qwen3_5_mtp.py:94` (`Qwen3_5ForCausalLMMTP`) and `SRT/models/qwen3_next_mtp.py:40` (`Qwen3NextForCausalLMMTP`) **[read]**.
- Linear-attention cache layout is documented by `mamba_ssm_dtype` in the checkpoint config and by the hybrid pool files in §4.

## 3. Speculative decoding (`SRT/speculative/`)

- Registry: `spec_registry.py` — `is_frozen_kv_mtp` (`:78`), reserved alias `NEXTN → EAGLE` (`:181–183`) **[read]**.
- Families present (names read via `ls`, contents largely **[unverified]**): `eagle_*` (`eagle_worker_v2.py`, `eagle_info.py`, `eagle_utils.py`, CUDA-graph runners), `frozen_kv_mtp_*` (`frozen_kv_mtp_worker_v2.py`, `frozen_kv_mtp_info.py`, `frozen_kv_mtp_utils.py`, `frozen_kv_mtp_cuda_graph_runner.py`), `dflash_*` (`dflash_worker_v2.py`, `dflash_info*.py`, `dflash_disaggregation.py`), `standalone_worker_v2.py`, `cpp_ngram/`, `adaptive_spec_params.py`, `adaptive_runtime_state.py`, `base_spec_worker.py`, `spec_utils.py` **[read as names]**.
- `frozen_kv_mtp` is the MTP-with-frozen-KV variant; the file names suggest a verify step that reuses the target's KV (not read in full — **[unverified]**).
- `dp_spec_prefill_coordination.py` and `decoupled_spec_io.py` indicate speculative prefill is coordinated across data-parallel ranks **[names only, unverified]**.

## 4. Caches

- Radix tree prefix cache: `SRT/mem_cache/radix_cache.py` — `RadixKey` (`:59`), `TreeNode` (`:250`), `RadixCache(BasePrefixCache)` (`:281`) **[read as declarations]**.
- Hybrid / Mamba state pools: `SRT/mem_cache/memory_pool.py` and `SRT/mem_cache/pool_host/mamba.py` (class names not pinned in this pass) **[names only]**. These are where recurrent state for linear-attention layers would live; the exact pairing with the radix tree was **not** read.
- Adaptive speculative parameters: `SRT/speculative/adaptive_spec_params.py` (name only).

## 5. What is worth copying (facts → design relevance)

1. **One radix/prefix tree keyed by tokens, with a second state pool for recurrent layers.** The split exists in code (`radix_cache.py` + `memory_pool.py`/`mamba.py`). oMLX reaches the same place with checkpoint blocks (doc 02 §4).
2. **Speculative workers are first-class and per-family** (`eagle_worker_v2.py`, `frozen_kv_mtp_worker_v2.py`, `dflash_worker_v2.py`) behind a registry (`spec_registry.py`). This is the structure a unified engine needs for "many MTP styles behind one loop".
3. **Gemma-style q/k norm** and gated q-projection are implemented explicitly for Qwen3.5 (`qwen3_5.py:1312–1332, 1402`). Use as the reference for the seat's fix list.

## 6. Not verified

- Any SGLang benchmark on Apple Silicon (not applicable here; no MLX backend found).
- Runtime behaviour of any of the speculative workers.
- `RadixLinearAttention` kernel selection and state layout.
- `mamba.py` pool layout.
