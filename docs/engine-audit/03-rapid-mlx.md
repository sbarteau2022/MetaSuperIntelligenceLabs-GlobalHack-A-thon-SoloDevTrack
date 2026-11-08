# 03 — rapid-mlx audit

Local source: Homebrew cellar `/opt/homebrew/Cellar/rapid-mlx/1.0.0/` (alias **RAPID**), Python package `RAPID/libexec/lib/python3.15/site-packages/rapid_mlx/` (alias **RM**), with its own `mlx_lm-0.31.3` in the same site-packages. Binaries on PATH: `/opt/homebrew/bin/rapid-mlx`, `rapid-mlx-chat`, `rapid-mlx-bench`. Read locally; no server started; no port touched.

Status legend: **[read]** = read in the cited lines; **[vendor]** = claim from the project's own README, not reproduced; **[unverified]** = not read or not executed.

## 1. What it is

- Self-description: "open-source (Apache 2.0) OpenAI- and Anthropic-compatible LLM inference server for Apple Silicon, built on MLX, focused on reliable tool calling for coding agents" — `RAPID/README.md:13` **[read]**.
- Built on the same substrate as oMLX: `mlx_lm.BatchGenerator` continuous batching. The scheduler docstring says so explicitly — "using mlx-lm's BatchGenerator for efficient continuous batching", with "waiting queue", "running set" in the vLLM style — `RM/scheduler.py:1–12` **[read]**. It also installs a compatibility shim before importing `mlx_lm.generate` because of a thread-local stream issue on M5 single-stream GPUs — `RM/scheduler.py:~30–40` **[read]**; `RM/compiled_decode.py:1–25` repeats that rule.
- Endpoints named in the server docstring: `POST /v1/completions`, `POST /v1/chat/completions` (`RM/server.py:28–29`) **[read]**. Anthropic compatibility is claimed in the README (`RAPID/README.md:13`) **[read]**; the route implementation was not located.
- Launcher: `rapid-mlx launch <agent>` for Codex, Claude Code, OpenCode, etc. (`RAPID/README.md:7–9`, `:212`) **[read]**.

## 2. Qwen3.5 / Qwen3.6-specific kernels and decode

All four are in `RM/` and carry module docstrings:

- `gdn_prefill.py:1–10` — "Blocked-sequential Metal kernel for Gated DeltaNet prompt prefill". Claims the same sequential recurrence, restructured to cut ~32× redundant k/q traffic (~13 GB per 16k-token layer call) **[read; the traffic number is the author's estimate, unverified]**.
- `qwen35_fused_gdn_decode.py:1–12` — fuses the single-token GDN decode (causal conv, Q/K norm, recurrent update, gated RMSNorm) into one Metal launch. "Enrollment is model-local and fail-closed. A real Metal probe must reproduce the stock output, convolution cache, and FP32 recurrent state bit-for-bit" **[read]**. This is an explicit correctness gate against the stock path, which is the right pattern (see DESIGN-SPEC).
- `compiled_decode.py:1–25` — "Shape-stable, whole-step compiled replay for qualified width-one decode"; KV slab fixed-shape, hybrid `ArraysCache` threaded alongside; "never captures request state in a compiled closure" **[read]**. Licensed-notice file `COMPILED_DECODE_LICENSE` present.
- `model_aliases.py` — per-model capability flags including `supports_native_mtp` (`:509`, `:682–685`: native MTP is "AR-only"), `native_mtp_draft_model` (`:688–700`), `mtp_draft_model` (`:703`), `tensorfold_mtp` (`:435`, `:443–457`) **[read]**.

## 3. Speculative decoding and MTP

- CLI: `--enable-mtp` (`RM/cli_parser.py:1130`), `--mtp-num-draft-tokens` (`:1136`), `--mtp-optimistic` (`:1142`), `--mtp-sidecar` (`:1148`), and a speculative-method choice `none | dflash | mtp` (`:1120`) **[read]**. An example config string shows `{"method":"mtp","num_speculative_tokens":3}` (`:1092`) — a CLI convenience, not a model-I/O payload.
- Scheduler: `_mtp_controller_key` (`RM/scheduler.py:1005`), `_ensure_request_mtp_counter` (`:1032`), `_install_continuous_mtp_router` (`:1043`), `_install_mtp_vendored` (`:1586`), `_has_mtp_surface` (`:1655`) **[read]**. Comments at `:1699, 1718, 1762, 1834, 1839` refer to `_mtp_step` ("On every `_mtp_step` call…", "inside _mtp_step (the round-E fix)") — **the definition of `_mtp_step` was not located** by grep; it is likely patched or generated. Treat its body as **[unverified]**.
- Per-row seeded sampler: `RM/_seeded_sampler.py:25` says samplers are dispatched per row in `scheduler.py::_mtp_step` so that a row's RNG cannot corrupt another's; `:236` and `:387` describe the B=1 self-MTP priming draw **[read]**.
- Request-level policy: `RM/speculative/request_policy.py:1–12` — "Speculative decoding is configured once when an engine starts, but some request features can require the scheduler to use ordinary decoding for that request" **[read]**.
- Also present: `speculative/suffix_counter.py`, `speculative/tensorfold_families.py`, `speculative/tensorfold_glm53.py` (names only **[unverified]** contents).
- README benchmark table (`RAPID/README.md:25–33`) — Mac Studio M5 Max, 36 GB, Qwen3.5-9B / 3.8-27B / 3.6-35B, comparing to oMLX 0.7.0 and LM Studio, "default serving modes with Rapid's MTP text mode, oMLX's SSD cache". Example: Qwen3.6-35B decode 173.4 vs oMLX 145.8 tok/s **[vendor, unverified]**. The table is the project's own; no receipt in this repo.

## 4. Cache and memory

- Module names (not read in full): `RM/memory_cache.py`, `RM/singleton_cache_fastpath.py`, `RM/disk_caches.py`, `RM/cache/` **[unverified contents]**.
- `RM/scheduler.py` comments reference an "unnamed-model fallback" and a "corruption bug" around MTP state (`:1699`, `:1718`) — a sign that MTP state keying by model/sidecar is a known hazard (`_mtp_controller_key`, `:1005`).

## 5. Position relative to the other engines (facts from the code)

- Same batching substrate as oMLX (`mlx_lm.BatchGenerator`), so the batching primitives in doc 06 apply to both.
- Where it differs from oMLX as read: rapid-mlx is organised around **model aliases with capability flags** (`model_aliases.py`) and **request-level policy objects** (`request_policy.py`); oMLX is organised around **engines** (`BatchedEngine`, `VLMBatchedEngine`, `DFlashEngine`) and **a paged SSD cache** (doc 02 §4). rapid-mlx's visible cache files were not read, so its KV-cache design is **[unverified]**.
- Qwen3.5 handling: rapid-mlx hand-writes fused GDN prefill and decode Metal paths (`gdn_prefill.py`, `qwen35_fused_gdn_decode.py`) with fail-closed bit-exact probes. oMLX has the equivalent set of Qwen3.5 patches (doc 02 §6). Both are *patches over mlx-lm's correct GatedDeltaNet*, not reimplementations — the opposite of the seat's current state (doc 01).

## 6. Not verified

- Any performance number (README table is vendor-reported).
- `_mtp_step` definition and its acceptance/rollback path.
- Cache implementation details (`memory_cache.py`, `disk_caches.py`).
- Anthropic-route implementation.
- Whether the bit-exact probe in `qwen35_fused_gdn_decode.py` is exercised in CI — not read.
