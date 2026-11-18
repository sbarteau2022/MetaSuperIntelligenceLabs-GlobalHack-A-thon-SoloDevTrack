"""1-layer MTP head for Qwen3.6 (qwen3_5_moe), built from model-mtp.safetensors.

Architecture (from the tensor names and shapes in model-mtp.safetensors):
    e  = pre_fc_norm_embedding(embed_tokens(x_{t+1}))
    h  = pre_fc_norm_hidden(h_t)                      # h_t: main model's final-normed hidden state
    z  = fc(concat([e, h]))                           # [2H] -> [H]
    z  = one full-attention decoder layer (layers.0, MoE)   # layer_idx 3: (3+1) % 4 == 0
    z  = norm(z)
    logits = main model's lm_head(z)                  # shares the main model's output projection

The checkpoint's linears are quantized (affine). This module dequantizes them to bf16 at load time,
so the head runs as plain MLX ops. Per-tensor (bits, group_size) come from the shapes plus the known
input dimension of each module; the config has no quantization entries for the MTP paths.
"""

import json
import pathlib
import re

import mlx.core as mx
import mlx.nn as nn
from mlx.utils import tree_flatten

from mlx_lm.models.base import create_attention_mask
from mlx_lm.models.cache import KVCache
from mlx_lm.models.qwen3_5 import DecoderLayer, TextModelArgs

MTP_PREFIX = "language_model.mtp."
FULL_ATTENTION_LAYER_IDX = 3  # full_attention_interval = 4 -> layer 3 is full attention

# Input dimension of each quantized linear in the MTP layer (hidden=2048, moe_inter=512,
# attention input = num_heads*head_dim = 4096).
_IN_DIM = [
    (r"self_attn\.o_proj\.", 4096),
    (r"self_attn\.(q|k|v)_proj\.", 2048),
    (r"mlp\.shared_expert\.down_proj\.", 512),
    (r"mlp\.shared_expert\.(gate|up)_proj\.", 2048),
    (r"mlp\.shared_expert_gate\.", 2048),
    (r"mlp\.switch_mlp\.down_proj\.", 512),
    (r"mlp\.switch_mlp\.(gate|up)_proj\.", 2048),
]


def _in_dim(name: str) -> int:
    for pattern, dim in _IN_DIM:
        if re.search(pattern, name):
            return dim
    raise KeyError(f"no known input dim for quantized tensor {name}")


def _dequantize_group(weight: mx.array, scales: mx.array, biases: mx.array, in_dim: int) -> mx.array:
    """Recover (group_size, bits) from shapes: group = in/groups, bits = packed*32/in."""
    n_groups = scales.shape[-1]
    packed = weight.shape[-1]
    group_size = in_dim // n_groups
    bits = packed * 32 // in_dim
    assert group_size * n_groups == in_dim and bits * in_dim == packed * 32, (
        f"inconsistent quant shapes: weight {weight.shape} scales {scales.shape} in_dim {in_dim}"
    )
    assert group_size in (32, 64, 128) and bits in (2, 3, 4, 5, 6, 8), (group_size, bits)
    return mx.dequantize(weight, scales, biases, group_size=group_size, bits=bits).astype(mx.bfloat16)


def _read_mtp_tensors(repo: pathlib.Path) -> dict:
    """Return MTP tensors keyed without the 'language_model.mtp.' prefix, dequantized where needed."""
    raw = {
        key[len(MTP_PREFIX):]: tensor
        for key, tensor in mx.load(str(repo / "model-mtp.safetensors")).items()
        if key.startswith(MTP_PREFIX)
    }

    out = {}
    for key, tensor in raw.items():
        if key.endswith(".scales") or key.endswith(".biases"):
            continue
        if key.endswith(".weight") and tensor.dtype == mx.uint32:
            stem = key[: -len(".weight")]
            out[stem + ".weight"] = _dequantize_group(
                tensor, raw[stem + ".scales"], raw[stem + ".biases"], _in_dim(stem + ".")
            )
        else:
            out[key] = tensor.astype(mx.bfloat16)
    return out


class MTPHead(nn.Module):
    """Predicts the token after next from (main hidden state at t, token at t+1)."""

    def __init__(self, args: TextModelArgs):
        super().__init__()
        H = args.hidden_size
        self.pre_fc_norm_embedding = nn.RMSNorm(H, eps=args.rms_norm_eps)
        self.pre_fc_norm_hidden = nn.RMSNorm(H, eps=args.rms_norm_eps)
        self.fc = nn.Linear(2 * H, H, bias=False)
        self.layers = [DecoderLayer(args, layer_idx=FULL_ATTENTION_LAYER_IDX)]
        self.norm = nn.RMSNorm(H, eps=args.rms_norm_eps)
        # Set by load_mtp_head: shared with the main model.
        self.embed_tokens = None
        self.lm_head = None

    def make_cache(self):
        """One KV cache for the head's own attention layer; entries are added per committed position."""
        return [KVCache()]

    def hidden(self, hidden: mx.array, next_tokens: mx.array, cache=None) -> mx.array:
        """Final-normed head state (the input for the next chained draft step). No vocab projection."""
        e = self.pre_fc_norm_embedding(self.embed_tokens(next_tokens))
        h = self.pre_fc_norm_hidden(hidden)
        z = self.fc(mx.concatenate([e, h], axis=-1))
        layer_cache = cache[0] if cache is not None else None
        mask = create_attention_mask(z, layer_cache)
        z = self.layers[0](z, mask=mask, cache=layer_cache)
        return self.norm(z)

    def __call__(self, hidden: mx.array, next_tokens: mx.array, cache=None) -> mx.array:
        return self.lm_head(self.hidden(hidden, next_tokens, cache=cache))


def load_mtp_head(repo: pathlib.Path, model, dtype=mx.bfloat16) -> MTPHead:
    """Build the head from model-mtp.safetensors, sharing embed_tokens and lm_head with `model`.

    `dtype` must match the main model's activation dtype. A bf16 head running on fp16 activations
    measured 95 ms per token vs 2 ms in fp16 (Step 3 profiling), so the matched gate passes fp16.
    """
    repo = pathlib.Path(repo)
    cfg = json.loads((repo / "config.json").read_text())
    args = TextModelArgs.from_dict(cfg["text_config"])
    assert args.num_hidden_layers > 0 and cfg["text_config"]["mtp_num_hidden_layers"] == 1

    head = MTPHead(args)
    weights = _read_mtp_tensors(repo)

    # Map checkpoint names onto the module tree: layers.0.X -> layers.0.X, norm -> norm, etc.
    tree_names = {k for k, _ in tree_flatten(head.parameters())}
    missing = sorted(tree_names - set(weights))
    unexpected = sorted(set(weights) - tree_names)
    assert not missing and not unexpected, f"missing={missing[:5]} unexpected={unexpected[:5]}"

    head.load_weights(list(weights.items()), strict=True)
    head.set_dtype(dtype)
    mx.eval(head.parameters())

    lm = model.language_model
    head.embed_tokens = lm.model.embed_tokens  # quantized embedding, shared
    head.lm_head = lm.lm_head
    return head


def mtp_logits(head: MTPHead, hidden: mx.array, next_tokens: mx.array) -> mx.array:
    """hidden: [B, T, H] main-model hidden at t; next_tokens: [B, T] token at t+1 -> logits [B, T, V]."""
    return head(hidden, next_tokens)
