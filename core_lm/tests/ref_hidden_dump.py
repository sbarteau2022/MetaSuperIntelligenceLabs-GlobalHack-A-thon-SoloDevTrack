"""Oracle dump (not in the harness I/O path): reference hidden states for token 760 at position 0.
Writes tests/data/ref_hidden_760.bin = 41 x 2048 little-endian f32: embedding, then the output of each layer.
Produced by oMLX's bundled mlx_lm Qwen3.5-MoE (bf16 weights and activations, cast to f32 on write)."""
import os
import numpy as np
import mlx.core as mx
from mlx_lm import load

out = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data", "ref_hidden_760.bin")
model, _ = load("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined")
x = model.model.embed_tokens(mx.array([[760]]))
states = [np.array(x.astype(mx.float32)).reshape(-1)]
for layer in model.model.layers:
    x = layer(x, mask=None, cache=None)
    states.append(np.array(x.astype(mx.float32)).reshape(-1))
arr = np.stack(states).astype("<f4")
arr.tofile(out)
print("WROTE", out, arr.shape, "norms[0,1,29,40]:", [round(float(np.linalg.norm(arr[i])), 6) for i in (0, 1, 29, 40)])
