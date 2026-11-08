"""Diagnostic oracle only (not in the harness I/O path): per-layer hidden-state norms for one
token from oMLX's mlx_lm Qwen3.5-MoE, to compare with core_lm's RGAH_DEBUG_LAYERS output."""
import mlx.core as mx
from mlx_lm import load

model, tok = load("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined")
x = model.model.embed_tokens(mx.array([[760]]))
print(f"EMB {float(mx.sqrt(mx.sum(x * x))):.6f}")
for i, layer in enumerate(model.model.layers):
    x = layer(x, mask=None, cache=None)
    kind = "lin" if layer.is_linear else "full"
    print(f"L{i} {kind} {float(mx.sqrt(mx.sum(x * x))):.6f}")
