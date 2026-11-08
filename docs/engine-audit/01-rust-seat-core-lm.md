# 01 — Rust seat audit: `core_lm` (Qwen3.6-MoE, `qwen3_5_moe`)

Status as of 2026-11-05, repo branch `codex/consolidate-n3-globalmind-20261102`.
Scope: `core_lm/src/bin/serve_qwen.rs`, `qwen_forward.rs`, `qwen_model.rs`, `native_token_wire.rs`, `tokenizer.rs`, plus the modules they call (`model_config.rs`, `optiq.rs`, `ops.rs`, `cache/geometric.rs`, `geometry/bridge.rs`).

Verdict in one paragraph: the seat loads the right checkpoint and its expert-matvec path matches a full dequantize on real weights, but its forward pass is **not a Qwen3.6 forward pass**. Linear-attention layers are a made-up linear-attention variant with hard-coded convolution weights, not Gated DeltaNet. Full-attention layers have no output gate, no QK-norm, and a retrieval step that is an exact softmax only while the context is shorter than 64 tokens. Eight of the forty layers' projections are sized wrongly, the MoE router does not renormalise, the shared-expert sigmoid gate is missing, and the quantisation bit-widths come from a metadata file that this checkpoint does not contain. All 40 layers are affected: the 30 linear-attention layers have wrong `in_proj_qkv`, `in_proj_z` and `out_proj` shapes, and the 10 full-attention layers have a wrong `q_proj` shape. Its output should not be trusted as Qwen3.6 output until the items in §5 are fixed and gated against the reference.

## Reference anchors used

| Alias | Path |
|---|---|
| REF-QWEN35 | `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/mlx_lm/models/qwen3_5.py` |
| REF-GDN | `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/mlx_lm/models/gated_delta.py` |
| REF-NEXT | `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/mlx_lm/models/qwen3_next.py` |
| REF-ROPE | `/opt/homebrew/Cellar/omlx/0.7.0/libexec/lib/python3.11/site-packages/mlx_lm/models/rope_utils.py` |
| CKPT | `~/models/fused/Qwen3.6-MoE-Combined/` (`config.json`, `model.safetensors.index.json`, shards 1–4, `model-mtp.safetensors`) |

The reference is the mlx-lm copy shipped inside oMLX 0.7.0; mlx-lm version `0.31.4.dev132` per its dist-info. It is the same code family as [mlx-lm on GitHub](https://github.com/ml-explore/mlx-lm) (not read online; the local copy is what was audited).

## 1. Architecture as it really is

**Model shape, from CKPT `config.json` `text_config`** (read by a python receipt, §6):
- `hidden_size` 2048, `num_hidden_layers` 40, `full_attention_interval` 4, `layer_types` = 30 × `linear_attention` + 10 × `full_attention` (full layers at indices 3, 7, …, 39).
- Full attention: 16 query heads, 2 KV heads, `head_dim` 256, `attn_output_gate` **true**, `partial_rotary_factor` 0.25, `rope_theta` 1e7, `mrope_interleaved` true (`mrope_section` [11, 11, 10]).
- Linear attention: 16 key heads × 128, 32 value heads × 128, `linear_conv_kernel_dim` 4.
- MoE: 256 experts, top-8, `moe_intermediate_size` 512, one shared expert of 512; `vocab_size` 248320; `tie_word_embeddings` false; `eos_token_id` 248044.

**Checkpoint tensor layout** (`model.safetensors.index.json`, 1757 tensors; header reads in §6):
- Linear layer 0: `linear_attn.{in_proj_qkv, in_proj_z, in_proj_a, in_proj_b, out_proj}` (quantised), `linear_attn.{conv1d.weight, A_log, dt_bias, norm.weight}` (bf16/f32), plus `mlp.{gate, switch_mlp.{gate_proj,up_proj,down_proj}, shared_expert.{gate,up,down}_proj, shared_expert_gate}`.
- Full layer 3: `self_attn.{q_proj, k_proj, v_proj, o_proj}` (quantised), `self_attn.{q_norm, k_norm}.weight`.
- Non-layer: `language_model.model.embed_tokens.{weight,scales,biases}` (quantised), `language_model.model.norm.weight`, `language_model.lm_head.{weight,scales,biases}` (quantised).
- MTP head lives in `model-mtp.safetensors`; no `mtp.*` key appears in the index (receipt §6). The seat does not load it.

**Reference structure** (what a correct forward does):
- Linear layer = GatedDeltaNet: `in_proj_qkv` → depthwise causal `conv1d` (kernel 4) + SiLU → L2-norm of q/k with `q` scaled by `head_k_dim**-0.5` (REF-QWEN35:87–206, REF-GDN:33–44 `normalize_qk`); `beta` from `in_proj_b`, decay `g = exp(-exp(A_log)·softplus(a + dt_bias))` from `in_proj_a` (REF-GDN:19–21 `compute_g`); recurrent per-head matrix state update (REF-GDN:385 `_gated_delta_step_ops`, REF-GDN:557 `gated_delta_ops`, REF-GDN:605 `gated_delta_update`); gated RMSNorm with `z`; `out_proj` (REF-QWEN35:133–206).
- Full layer = Qwen3-Next attention: `q_proj` emits **2×** heads (query and gate), `q_norm`/`k_norm`, partial RoPE, softmax attention, output `o_proj(attn · sigmoid(gate))` (REF-NEXT:82, 103–104, 122–151).
- MoE: softmax router, top-k, **renormalise top-k weights** when `norm_topk_prob` (REF-NEXT:332; default `True` in REF-QWEN35:52); shared expert gated by `sigmoid(shared_expert_gate(x))` (REF-NEXT:314, 339).
- Quantisation: `mlp.gate` and `shared_expert_gate` are 8-bit, everything else 4-bit in this checkpoint (CKPT `config.json` `quantization` block; REF-QWEN35:375–376 quant predicate).

**What the seat actually does**, per token (`qwen_forward.rs:367–408`, `forward_with_permit`):
1. `embed` = single-row dequantise of `embed_tokens` (`qwen_forward.rs:69–76`, `qwen_model.rs:222–279`).
2. For each of 40 layers (`qwen_forward.rs:385–397`): `full_attention` or `linear_attention`, then `moe_mlp`. Both sublayers are residual.
3. Final `norm` (`qwen_forward.rs:399–400`), `lm_head` (`qwen_forward.rs:401–406`), full f32 logits (vocab 248320).

## 2. Layer-by-layer: what is wrong

### 2.1 Linear attention (`qwen_forward.rs:292–365`) is not Gated DeltaNet

| Item | Reference (Qwen3.6) | Seat | Evidence |
|---|---|---|---|
| Input projection rows | `in_proj_qkv` out = 2·2048 + 4096 = **8192** | `matvec_weight(…, h*3, h)` = **6144** rows | `qwen_forward.rs:305–310`; CKPT shape `[8192, 256]` U32 (§6) |
| Gate projection | `in_proj_z` out = **4096** (32 v-heads × 128) | `matvec_weight(…, h, h)` = **2048** | `qwen_forward.rs:311–316`; CKPT `[4096, 256]` |
| `out_proj` | in = **4096**, out = 2048 | `matvec(…, h, h)`: in = 2048 | `qwen_forward.rs:356–361`; CKPT `[2048, 512]` U32 = 4096 inputs at 4-bit (R3, R4) |
| `in_proj_a`, `in_proj_b` (decay and beta gates) | used | **never loaded** | `qwen_forward.rs` has no reference to them; CKPT lists them for layer 0 |
| `A_log`, `dt_bias` | used in `g` | **never loaded** | none in `qwen_forward.rs`; CKPT lists them |
| `conv1d.weight` [8192, 4, 1] | trained depthwise kernel of width 4 | **hard-coded** taps `0.55, 0.25, 0.12, 0.08`, 3 history slots | `qwen_forward.rs:17`, `331–333` |
| Q/K/V split | separate per-head q, k, v | element-wise: `q_raw=qkv[d]`, `k_raw=qkv[h+d]`, `v_raw=qkv[2h+d]` | `qwen_forward.rs:322–325` |
| Recurrent state | per head matrix `S ∈ R^{Dv×Dk}` with delta rule | per-element **vector** `s_num[d]`, `z_den[d]`, `γ=0.96` decay, ELU+1 features | state at `qwen_forward.rs:18–20`; update `347–350`; feature map `336–345`; `gamma` `321` |
| Normalisation | gated RMSNorm (`norm.weight`) on the output | none | `qwen_forward.rs:363` multiplies by `silu(z)` only |
| Output | `out_proj(gated_norm(o, z))` | `out_proj(o) ⊙ silu(z)` | `qwen_forward.rs:356–364` |

The layer is a different architecture with the same tensor names. The header comment at `qwen_forward.rs:1–5` ("geometric memory", "no k_cache") shows the design intent, not the reference. Note also `LinearAttnLayerState::conv_history: [[f32;3]; 2048]` (`qwen_forward.rs:17`) — the history is indexed by `d % 2048` (`:327`), which only works because `h = 2048`; it would silently alias on any other width.

### 2.2 Full attention (`qwen_forward.rs:78–212`) is wrong in four ways

1. **No output gate.** `q_proj` in the checkpoint is `[8192, 256]` U32 (§6) = 2 × 16 × 256. The seat asks for `nh * hd` = 4096 rows (`qwen_forward.rs:102–107`), so it reads the wrong half of the weight matrix and never applies `sigmoid(gate)` (`REF-NEXT:151`; CKPT `attn_output_gate: true`).
2. **No QK-norm.** `q_norm`, `k_norm` are not loaded or applied (`qwen_forward.rs:94–125`; reference `REF-NEXT:103–104, 130–131`).
3. **RoPE pairing.** `apply_rope` rotates *adjacent* pairs `(x[2i], x[2i+1])` (`ops.rs:99–112`, frequency denominator `rotary_dim` at `ops.rs:103`). The reference builds RoPE with `initialize_rope(...)` and no `traditional` argument (`REF-NEXT:106–110`), so the default `traditional: bool = False` applies (`REF-ROPE:77`); MLX's `traditional=False` is the split-half convention. This is a mismatch to verify with a layer-by-layer oracle (not executed here).
4. **Attention is retrieval, not softmax over the sequence.** Each kv head owns a `GeometricMemoryMap` (`qwen_forward.rs:45–50`, 1,000,000-atom cap at `:49`). Per query head the seat calls `geo.retrieve(&q_lv, top_k=64, false)` (`qwen_forward.rs:150, 159`), which:
   - in `cache/geometric.rs:259` checks whether a VP-tree index exists. `write()` sets `index = None` at `geometric.rs:207` and `:226`; `rebuild()` (`:168`) is never called on the qwen path (only `lib.rs:183` and tests call it). So retrieval is always the **linear scan** in `geometric.rs:259–281`;
   - ranks **all atoms** by Minkowski inner product of Picard-reduced addresses (`geometric.rs:254, 270`), ascending, and keeps the first 64 (`:277–281`). The ranking key is geometric proximity in H³ × T², not `q·k`;
   - then the seat computes exact `q·k/√hd` over the stored key bytes of those ≤64 atoms (`qwen_forward.rs:164–190`) and softmaxes them (`:192`).
   
   So: **for contexts of up to 64 positions** the candidate set is every prior key of that kv head (the current token is written before it is read, `qwen_forward.rs:132–146`), and the result is an exact softmax over the past. **Beyond 64 positions it is an approximation** whose candidate set is chosen by the Lorentz embedding, not by key similarity. Either way it is not the reference attention, because of items 1–3. The cost is `O(N log N)` per query head per token (full sort of all N atoms, `geometric.rs:270`), so prefill of T tokens is `O(T² log T)` per layer.
5. **Dropped loss.** `f32_to_atom_address` returns `(lv, phase, _loss)` and the loss is discarded (`qwen_forward.rs:137`). `phase` is stored (`geometric.rs:106`) but never read by `retrieve`.
6. **Memory growth.** Every write also pushes an `AuditEntry` with a `SystemTime` (`geometric.rs:209–214`) and appends 2048 bytes of payload per kv head per token (`qwen_forward.rs:138–145`). With 10 full layers × 2 kv heads = 20 maps (`qwen_forward.rs:45–50`) there is no eviction until 1,000,000 atoms per map.

### 2.3 MoE (`qwen_forward.rs:214–290`)

- **Router not renormalised.** Softmax over all 256 logits (`:237`), top-8 indices (`:238`), and the top-k probabilities are used as-is (`:249, 256`). The reference renormalises top-k when `norm_topk_prob` (`REF-NEXT:332`; default true in `REF-QWEN35:52`; the checkpoint config omits the key, so the default applies).
- **Shared-expert gate missing.** The shared expert output is added unconditionally (`:285–287`). The reference multiplies it by `sigmoid(shared_expert_gate(x))` (`REF-NEXT:339`). CKPT has `shared_expert_gate` as `[1, 512]` U32 at 8-bit (§6); the seat never reads it.
- **Expert path (verified).** The routed experts use `expert_matvec` (`:250–254`), which slices only the routed expert's rows of the stacked `[256, rows, packed]` tensors (`qwen_model.rs:386–447`). Real-weight parity test passes (§6, receipt R2): `expert_matvec == full dequant` for expert 5, relative error < 1e-3.
- `top_k_indices` sorts all 256 scores per token (`ops.rs:116–120`). Correct but wasteful; a partial select would do.

### 2.4 Quantisation and weight loading (`qwen_model.rs`, `optiq.rs`)

- **Bit-width source is wrong for this checkpoint.** `WeightManager::load` reads per-tensor `bits` and `group_size` only from `optiq_metadata.json` (`qwen_model.rs:126–144`). That file is **not present** in CKPT (directory listing, §6). Every tensor therefore falls back to `bits = 4` (`qwen_model.rs:177`) and `group = 64` (`:124`).
- The checkpoint's own `config.json` `quantization` block says `mlp.gate` and `shared_expert_gate` are **8-bit** (group 64), and the MTP tensors are 5/6/8-bit. `ModelConfig` never parses that block (`model_config.rs:38–66` raw structs have no `quantization` field).
- Receipt (§6, R4): for `mlp.gate`, `packed_u32_per_row = 512` over `in = 2048` ⇒ **8 bits**. The seat dequantises it as 4 bits. The router logits are therefore wrong from layer 0 on. The `switch_mlp` experts (4-bit, R4) and the embedding (4-bit, R4) are correct.
- `dequantize` (`optiq.rs:118–211`) is a correct generic affine decoder: `w = q·scale + bias` per group of `group_size` consecutive flat elements (`optiq.rs:103–116`), supports U32 bit-packing that crosses word boundaries (`:166–183`) and U8 (`:186–207`). It is not the problem.
- `get_weight` caches dequantised tensors only when `retain_cache` or ≤ 65,536 elements (`qwen_model.rs:215`). Norms (f32, small) are cached; quantised matrices use the fused path.
- The Metal fast path is taken only for `U32` with 4- or 6-bit (or a generic kernel) and aligned pointers (`qwen_model.rs:310–360`); otherwise `qmatvec_fused_zero_copy` (`:362–376`). Which path ran is not logged.

### 2.5 Other forward details

- `rmsnorm` is the plain form `x·w/√(mean(x²)+eps)` (`ops.rs:45–63`). The reference Qwen3.5 `q_norm`/`k_norm` are Gemma-style (SGLang: `GemmaRMSNorm`, `sglang/.../qwen3_5.py:1402`); whether the stored weights include the `+1` offset is **unverified** and must be checked against the checkpoint values before any norm is wired in.
- `forward` calls `immune_gate.authorize_forward_pass` for every token (`qwen_forward.rs:410–417`); this is a compliance hook, not part of the model.
- `reset_memory` rebuilds 20 geometric maps and all linear states (`qwen_forward.rs:419–427`). `complete` calls it on every request (`serve_qwen.rs:53`), so the seat is stateless per request, which is what the wire doc claims (`serve_qwen.rs:10, 15–16`).

## 3. Wire protocol (`serve_qwen.rs`, `native_token_wire.rs`)

**Transport.** Hand-written HTTP/1.1 over `std::net::TcpListener` (`serve_qwen.rs:148`), one connection at a time in a single accept loop (`:151–231`). Header read cap 1 MiB (`:170`), body cap 16 MiB (`:192`). `Connection: close` on every reply (`:92`). No threads: a long `complete` blocks every other request. This is deliberate ("sequential", `serve_qwen.rs:3`) because the model is `&mut` (`:51`).

**Loopback only.** The bind host must be `127.0.0.1` or `localhost` (`serve_qwen.rs:128–131`). Default bind `127.0.0.1:8003` (`:106`). Note the standing port key reserves `8003` for "the next model server" and forbids using it for another service without Stewart's sign-off; the default here collides with that reservation. This document does not touch any port.

**Endpoints** (`serve_qwen.rs:205–226`):
- `GET /health` → `ready`.
- `POST /v1/tokenize-native` — `GTK2` request (`native_token_wire.rs:98–111`: magic, u32 length, u8 `add_bos`, UTF-8 text ≤ 16 KiB) → `GTO2` reply (`:113–133`: magic, count, u32 ids, 32-byte tokenizer digest, 32-byte receipt = SHA-256 over request‖reply-without-receipt). **`add_bos` is parsed and then ignored** (`serve_qwen.rs:208` calls `tok.encode(&req.text)` only).
- `POST /v1/detokenize-native` — `GDT2` → `GDR2` (`native_token_wire.rs:39–96`). The reply has a receipt but no tokenizer digest.
- `POST /v1/complete-native` — `QNC1` request: `u32 max_new` (1..=128), `u32 n` (1..=1024), `n × u32` prompt ids (`serve_qwen.rs:30–49`). Reply `QNR1`: `u32 n_out`, ids, 32-byte receipt = SHA-256(request‖"QNR1"‖n_out‖ids) (`:77–88`). Greedy argmax, EOS stops (`:59–73`). The prompt is prefilled one token at a time (`:55–57`); there is no batched prefill.

**Receipts bind the request.** Each reply hashes the request bytes it answers (`native_token_wire.rs:128–131`, `serve_qwen.rs:84–86`). The unit tests check tamper rejection on the tokenize and detokenize frames (`native_token_wire.rs:165–221`). These tests exist; their last run is **not** recorded here (see §6).

**No JSON in the model I/O path.** JSON is parsed only at load time (`model_config.rs:1` header, `tokenizer.rs:1` header, `qwen_model.rs:94–101`). Requests and replies are typed bytes. This satisfies the boundary rule for the seat itself.

**Tokenizer identity.** `tokenizer_digest` = SHA-256 of the whole `tokenizer.json` file (`serve_qwen.rs:139–141`), so a reply can be bound to a tokenizer file. `complete` does not return the digest.

## 4. Tokenizer (`tokenizer.rs`)

- Loads vocab, merges (both `[a,b]` pairs and `"a b"` strings), added tokens and the first `Split` regex of the pre-tokenizer from `tokenizer.json` (`tokenizer.rs:136–183`). Regex by `fancy_regex` (`:4`) so lookaheads work.
- Added tokens (e.g. `<|im_start|>`) are split out first, longest-match at the earliest position (`:186–217`).
- Byte-level BPE: `bytes_to_unicode` table (`:112–133`), greedy lowest-rank merge loop (`:218–259`). Complexity is `O(L²)` per pre-token; fine for prompts ≤ 1024 ids.
- `decode` drops unknown ids silently and uses `from_utf8_lossy` (`:298–309`), so partial multi-byte sequences in streamed output become replacement characters. The detok endpoint inherits this.
- If the pre-tokenizer has no `Split` (`:274–281`), a whitespace splitter is used instead of the byte-level one — a silent fallback that would mis-tokenise a checkpoint with a different pre-tokenizer layout.
- Tests: two synthetic merge tests (`:321–352`) and a Glimmer prompt parity test that **skips** when its fixture is missing (`:355–392`). There is no Qwen3.6 fixture test in this file; Qwen tokenisation parity is therefore **unverified**.

## 5. Known-wrong list (blocking) — fix order

1. Read `q_proj`/`k_proj`/`v_proj`/`o_proj` and `in_proj_*`/`out_proj` shapes from the index, not from constants. Assert shapes at load.
2. Load bit-widths from `config.json` `quantization` (per-layer overrides) and drop the `optiq_metadata.json` dependency, or add the file. Fix `mlp.gate` and `shared_expert_gate` to 8-bit.
3. Replace linear attention with GatedDeltaNet per REF-QWEN35:87–206 and REF-GDN:385–605 (conv1d weights from the checkpoint, `A_log`, `dt_bias`, `in_proj_a/b`, per-head matrix state, gated RMSNorm).
4. Full attention: split `q_proj` into query and gate, add `q_norm`/`k_norm`, apply `sigmoid(gate)` before `o_proj`, switch RoPE to the reference pairing, and replace retrieval with exact causal softmax (or a documented, tested approximation).
5. MoE: renormalise top-k when `norm_topk_prob`; add `shared_expert_gate` sigmoid.
6. Gate every change with a layer-by-layer oracle against REF-QWEN35 on real weights (the parent session's "layer-0 oracle" is the right tool; it is not in this repo tree as a passing test).

## 6. Performance facts and receipts

**Receipts run for this audit (2026-11-05, no servers started, no ports opened):**

| ID | Command / source | Result |
|---|---|---|
| R1 | `cargo check --bins --offline` in `core_lm/` | `Finished` with exit 0; 2 warnings in the lib (`cargo fix` suggestions). Log: scratchpad `core_lm_check.log`. |
| R2 | `cargo test --offline --test qwen_expert_matvec -- --ignored` (real weights, `~/models/fused/Qwen3.6-MoE-Combined`) | `expert_matvec_matches_full_dequant_for_a_routed_expert ... ok`, 17.45 s. Proves the expert-only path equals the full dequant for `switch_mlp.gate_proj`, expert 5, relative error < 1e-3. It does **not** test the 8-bit router or any attention layer. |
| R3 | python: safetensors header read of the shards named in the index | Shapes: `in_proj_qkv` U32 `[8192,256]`; `in_proj_z` U32 `[4096,256]`; `out_proj` U32 `[2048,512]`; `conv1d.weight` BF16 `[8192,4,1]`; `A_log` BF16 `[32]`; `in_proj_a` U32 `[32,256]`; `q_proj` (layer 3) U32 `[8192,256]`; `k_proj` U32 `[512,256]`; `mlp.gate` U32 `[256,512]`; `switch_mlp.gate_proj` U32 `[256,512,256]`; `switch_mlp.down_proj` U32 `[256,2048,64]`; `shared_expert_gate` U32 `[1,512]`; `embed_tokens` U32 `[248320,256]`; `lm_head` U32 `[248320,256]`. |
| R4 | python arithmetic on R3 shapes (bits = 32·packed_cols / in_dim) | `mlp.gate` 8.0; `shared_expert_gate` 8.0; `switch gate_proj` 4.0; `switch down_proj` 4.0; `embed` 4.0. Seat's rows vs checkpoint: `q_proj` 4096 vs 8192; `in_proj_qkv` 6144 vs 8192. |
| R5 | `ls` of CKPT | no `optiq_metadata.json`; `model-mtp.safetensors` present; index has no `mtp.*` keys. |

**Performance claim (parent session): ~95 s/token before, ~0.25 s/token warm after the expert-only fix.** Not reproduced here. What is verified: the fix is `expert_matvec` (`qwen_model.rs:386–447`, used at `qwen_forward.rs:250–254`), it is in commit `02deb172` ("Final commit of GPT run: all current tree", found with `git log -S expert_matvec`), and its parity is R2. The commit message has no timing receipt, and the before-state was not located in history. Treat 95 s / 0.25 s as **unverified** until a timed run on this machine writes a receipt.

Costs visible in the code, independent of that claim:
- Linear-attention and full-attention matvecs are `O(rows × cols)` per token per layer on the quantised weights, one call per projection (`qwen_forward.rs:102–119`, `305–316`), plus one `expert_matvec` per routed expert ×3 (`:250–254`) and the shared expert ×3 (`:261–284`).
- Full attention retrieval: `O(N log N)` per query head per token (§2.2 item 4). This grows with context and is the first thing that will dominate beyond a few hundred tokens.
- Prefill is token-by-token (`serve_qwen.rs:55–57`); no chunked or batched prefill.
- Every `forward` allocates many `Vec<f32>` (e.g. `qwen_forward.rs:94–100`, `:164–190`).

## 7. What this seat is good for, and what it is not

- Good: a typed, receipted, loopback-only *harness* for a Rust forward path; expert-matvec parity; tokenizer wire; deterministic greedy decode once the forward is correct.
- Not good: any claim that its completions are Qwen3.6 completions; any latency number taken from it as Qwen3.6 latency; any comparison against oMLX, rapid-mlx or SGLang output until §5 items 1–5 pass.

## 8. Open items (not verified)

- Whether `q_norm`/`k_norm` stored weights carry a `+1` offset (§2.5).
- RoPE pairing convention (§2.2 item 3) — read from the reference, not executed.
- Qwen tokenizer parity against a Qwen3.6 fixture (§4).
- Timing claim in §6.
- Full-attention behaviour beyond 64 positions (§2.2 item 4) — read from code, not executed.
