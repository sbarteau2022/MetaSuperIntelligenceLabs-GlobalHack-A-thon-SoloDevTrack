# 02 — oMLX 0.7.0 audit

Source read: `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/omlx/` (alias **OMLX** below). Installed locally; every claim here is from that tree at the line cited. Runtime behaviour was **not** exercised (no server started, per the port rule). Dist-info: `omlx-0.7.0.dist-info`.

Alias used below: `OMLX/` = that path.

## 1. What oMLX is, structurally

- A FastAPI app serving OpenAI-, Anthropic- and Responses-style APIs over MLX (`OMLX/server.py:1–30`, route table in §2).
- Engines: `BatchedEngine` for text LLMs (`OMLX/engine/batched.py:60`), `VLMBatchedEngine` for vision-language (`OMLX/engine/vlm.py:1810`), `DFlashEngine` (`OMLX/engine/dflash.py:351`), and non-streaming engines for embeddings, reranking, speech and audio (`OMLX/engine/*.py`).
- The token loop is **mlx-lm's `BatchGenerator`** with oMLX patches applied on top. oMLX constructs it at `OMLX/scheduler.py:3396` with `prefill_batch_size=1` and `completion_batch_size` from config (`:3402–3403`). The patched step is `_patched_generation_batch_step` (`:755`).
- oMLX depends on `mlx_lm` for text and `mlx_vlm` for VLM/TurboQuant: `OMLX/engine/batched.py` imports `mlx_lm.generate` / `stream_generate` (`OMLX/models/llm.py:163, 216`), and `OMLX/turboquant_kv.py:1–9` is a thin wrapper over `mlx_vlm.turboquant`.

## 2. Server (`OMLX/server.py`)

- Lifespan and engine pool: `lifespan` (`:433`), `get_engine` (`:1332`), `ServerState` (`:254`), `EngineType` (`:220`). Multi-model serving with LRU memory management is in the module docstring (`:1–30`) and `engine_pool.py` (4057 lines).
- Routes (`@app.*` lines): `GET /health` (`:2895`), `GET /api/status` (`:2945`), `GET /v1/models` (`:3320`), load/unload `POST /v1/models/{id}/load|unload` (`:3479`, `:3463`), `POST /v1/embeddings` (`:3522`), `POST /v1/rerank` (`:3670`), `POST /v1/completions` (`:3775`), `POST /v1/chat/completions` (`:3997`), `POST /v1/messages` (`:6460`, Anthropic), `POST /v1/messages/count_tokens` (`:6870`), `POST /v1/responses` (`:6996`), `GET /v1/responses/{id}` (`:8269`).
- Auth, middleware and error mapping: `verify_api_key` (`:323`), `ClientDisconnectTrackingMiddleware` (`:1291`), `scheduler_queue_full_handler` (`:973`), `prefill_memory_exceeded_handler` (`:1080`).
- Admission: `Scheduler.add_request` caps the waiting queue at `max(max_num_seqs × 4, 32)` (`OMLX/scheduler.py:9560, 9576`).

## 3. Scheduler and engine core

- `EngineConfig` (`OMLX/engine_core.py:256`), `EngineCore` (`:298`), `AsyncEngineCore` (`:1398`). The engine core runs the scheduler on its own thread and exposes async generators.
- `SchedulerConfig`: `max_num_seqs = 256` (`OMLX/scheduler.py:1674`), `completion_batch_size = 32` (`:1680`).
- `Scheduler` class at `:1902`; `step()` at `:13326` returns a `SchedulerOutput` (`:1739`). Adaptive concurrency cap: `_effective_max_num_seqs` (`:10714`) and the grow/shrink controller (`:13930–13939`).
- Continuous batching over rows: per-row sampler/logits-processor registration and realignment (`:590–709`), grammar rows (`:709`), and singleton KV filter/extract/extend helpers for batch removal (`:811–916`).
- Prefill is chunked and memory-gated (`_PrefillState` `:502`, `_PrefillEvictionNeeded` `:459`, prefill admission `_AdmissionEstimate` `:200`).
- Stop/abort/timeouts are handled in the scheduler (`_StopOutputState` `:257`).

## 4. KV cache: paged, prefix-shared, SSD-backed

**In-memory paged blocks** (`OMLX/cache/paged_cache.py`):
- Content-addressed blocks: `compute_block_hash` = SHA-256 over `model_name`, parent hash, token tuple, and extra keys (`:84–125`), i.e. a **chain hash** like vLLM (`:20, 93–94`).
- `CacheBlock` carries `block_hash` used both for prefix caching and SSD storage key (`:133–157`).

**Prefix cache** (`OMLX/cache/prefix_cache.py`):
- `BlockAwarePrefixCache(CacheManager)` (`:176`) reuses blocks across requests; it has explicit support for **GDN (Gated DeltaNet) recurrent state checkpoints**: `set_gdn_checkpoint_loader` (`:312`), `set_exact_gdn_checkpoint_writer` (`:321`), `_commit_split_gdn_checkpoint` (`:429`), `_commit_exact_split_gdn_checkpoint` (`:461`), and `_validated_gdn_snapshot_layers` (`:377`). This is what makes prefix reuse correct for hybrid linear-attention models like Qwen3.5/3.6, where the recurrent state is not a KV slice.

**SSD tier** (`OMLX/cache/paged_ssd_cache.py`):
- Block-level **safetensors** serialisation "compatible with mlx-lm" (`:9`); blocks are written by a writer that does not call MLX (`_write_safetensors_no_mx`, `:927–979`, temp file then rename).
- Default block size 256 tokens (`:81`); block size is sized against RAM (`:98–139`).
- `PagedSSDBlockMetadata` (`:1026`), `PagedSSDCacheIndex` with a byte budget and `evict_until_size` (`:1118`, `:1233`), `GDNCheckpointMetadata` (`:1323`), `GDNCheckpointIndex` (`:1361`), `PagedSSDCacheManager(CacheManager)` (`:1623`).
- Budgets and switches (`OMLX/config.py`): `max_size = "100GB"` (`:121`), `hot_cache_max_size = "0"` (`:122`, disabled by default), `cache_dir` (`:120`, set from env at `:254`), `gdn_ssd_split_enabled` / `gdn_snapshot_storage` (`:123, 128–151`), env overrides `OMLX_GDN_SNAPSHOT_STORAGE`, `OMLX_GDN_SSD_SPLIT_ENABLED`, `OMLX_GDN_SSD_PENDING_MAX_SIZE`, `OMLX_GDN_SIDECAR_STATE_DTYPE` (`:233–249`). Default sidecar state dtype is fp32 (`:125`).
- Validation: `gdn_ssd_split_enabled` cannot be combined with hot-cache-only (`:373–377`).

**Compression**: `turboquant_kv.py` re-exports mlx-vlm's TurboQuant KV codecs and adds `BatchTurboQuantKVCache` for batching (`:1–9`). Scheduler has a `_turboquant_filter_singleton` path (`OMLX/scheduler.py:854`).

## 5. Speculative and MTP decoding

oMLX has **three** speculative paths: (a) is the text-LLM MTP path, (b) is the VLM MTP path, (c) is a block drafter (DFlash) for other families. Plus SpecPrefill (d).

**(a) Lightning MTP — text, patched into mlx-lm** (`OMLX/patches/mlx_lm_mtp/`):
- Entry: `apply_mlx_lm_mtp_patch()` (`OMLX/patches/mlx_lm_mtp/__init__.py:90`); global switches `set_mtp_active` / `is_mtp_active` (`:46, :60`) and depth `set_mtp_depth` / `get_mtp_depth` / `is_mtp_depth_fixed` (`:76–86`).
- Depth ceiling: `MAX_LIGHTNING_MTP_DRAFT_TOKENS = 8` (`OMLX/model_settings.py:37`), "the Lightning MTP runtime clamps deeper requests to this global ceiling".
- Batched drafting: `draft(batch, jobs)` (`OMLX/patches/mlx_lm_mtp/batched_head.py:90`), eligibility `eligible` (`:30`), per-row sampling `_sample_rows` (`:69`), head cache `_HeadCache` (`:14`).
- Rollback on rejection: `cache_rollback.py` arms undo (`:37`), wraps rotating caches (`:84`), stages functional updates (`:63`), applies the patch (`:186`). This is the piece intended to make rejected drafts leave attention caches unchanged (intent read from the module; not exercised here). The GDN recurrent state rollback is handled through the GDN checkpoint path in §4 and `OMLX/patches/qwen35_gdn_verify_fused.py` (see §6).
- MoE expert streaming keeps the MTP head resident for Qwen-family models (`OMLX/model_settings.py:39–42`, `MOE_OFFLOAD_MTP_MODEL_TYPES`).

**(b) VLM MTP** (`OMLX/speculative/vlm_mtp.py`):
- `VLMMTPDrafter` (`:70`), loader `load_vlm_mtp_drafter` (`:210`), proxies for the adapter (`:85–141`), `_MTPResetBindingProxy` (`:143`), decode loop `run_vlm_mtp_decode` (`:266`) that "stream[s] decoded tokens via mlx-vlm's MTP rounds".
- Routing: `_route_to_vlm_mtp` (`OMLX/scheduler.py:9763`); synthesized negative UIDs and `_VLMMTPDecodeState` (`:216–240`) bypass `BatchGenerator`.

**(c) DFlash drafter** (`OMLX/speculative/dflash_drafter.py`):
- Block drafter that "replaces the embedded MTP head as the draft source inside `omlx.patches.mlx_lm_mtp`" (`:1–12`), with per-row sliding-window context caches (`:40–68`), `DFlashDrafter` (`:170`), and `DFlashEngine` (`OMLX/engine/dflash.py:351`). Draft model is mlx-vlm's `DFlash2DraftModel` / `DFlashDraftModel`.

**(d) SpecPrefill** (`OMLX/specprefill/`): draft/target/planning/policy modules (`draft.py`, `target.py`, `planning.py`, `policy.py`). Not audited in depth.

## 6. Qwen3.5 / Qwen3.6 model path in oMLX

- **Model code**: text Qwen3.5 comes from mlx-lm (`mlx_lm/models/qwen3_5.py`, audited in doc 06); VLM from `mlx_vlm/models/qwen3_5*`. oMLX does not reimplement GatedDeltaNet; it inherits mlx-lm's correct version (REF-QWEN35:87–206) and then patches hot paths.
- **Qwen3.5-specific patches** (`OMLX/patches/`), each a module docstring:
  - `qwen35_gdn_chunked.py:1` — "Route Qwen3.5/3.6 Gated DeltaNet prefill to an optimized Metal kernel" (147 lines).
  - `qwen35_gdn_prework.py:1` — "Fused GDN prework for Qwen3.5/3.6 verify and selected decode paths" (2097 lines).
  - `qwen35_gdn_verify_fused.py:1` — "Fused Qwen3.5 GDN verify recurrence with deferred state commit" (535 lines). Deferred commit is what lets a rejected verify leave the recurrent state untouched.
  - `qwen35_moe_routed_decode.py:1` — "Fused routed and shared experts for one-token Qwen3.5-MoE-family decode" (872 lines).
  - `qwen35_verify_qmm.py:1` — "Verify-shape quantized-matmul kernels for native MTP" (1579 lines).
  - `qwen35_fa256_attention.py:1` — "Route Qwen3.5/3.6 head_dim=256 prefill attention to a steel FA kernel" (376 lines).
  - `qwen35_ragged_decode.py` (122 lines), `qwen35_moe_router.py`, `qwen35_moe_gate_up.py`, `qwen35_moe_weighted_sum.py`, `qwen35_packed_linear.py`, `qwen35_q4_mlp.py`, `qwen35_oq_a8.py` (not read in full).
- **Compiled decode** is also present in rapid-mlx (doc 03) but not checked here for oMLX.
- Checkpoint support for Qwen3.6 MoE + MTP: `patches/mlx_vlm_mtp/`, `patches/qwen3_6_nested_visual.py` (not read in full).

## 7. What this means for the unified engine (facts only)

1. The closest existing design to "unified loop with MTP + prefix + SSD" is here: chain-hashed blocks (`paged_cache.py:84`), SSD block store with index and budget (`paged_ssd_cache.py:1118–1233`), GDN-aware prefix checkpoints (`prefix_cache.py:312–461`), MTP drafting with rollback (`patches/mlx_lm_mtp/`).
2. Scheduling is per-step over rows with explicit singleton/filter helpers (`scheduler.py:754–916`) rather than one fused step function; the MTP route is a second decode path that bypasses `BatchGenerator` (`scheduler.py:216–240`, `:9763`). A unified engine must not keep two loops.
3. Correctness of GDN rollback depends on deferred-commit kernels (`qwen35_gdn_verify_fused.py`) and on checkpointing recurrent state at block boundaries (`prefix_cache.py:429`). The checkpoint granularity is a design parameter, not a constant.
4. Nothing in oMLX computes prediction error, homogeneity or isotropic suppression; those are the project's own terms (see DESIGN-SPEC).

## 8. Not verified

- Runtime behaviour, throughput, hit rates, SSD write amplification: not measured.
- `scheduler.py` step internals beyond the cited regions; `engine_pool.py` LRU policy; `specprefill/*`; `patches/qwen35_*` beyond the docstrings cited.
- Whether `_patched_generation_batch_step` (`:755`) is the only patched step under MTP; read only the signature region.
- Upstream oMLX repository was not consulted online; the audit is of the installed 0.7.0 wheel/tree.
