"""Oracle dump (not in the harness I/O path): for one layer, the reference residual after the attention
block (h = x + attn(x)) for the reference input x = tests/data/ref_hidden_760.bin[layer], plus the
reference layer output for checking. Writes tests/data/ref_h{layer}.bin (2048 f32 LE)."""
import os, sys
import numpy as np
import mlx.core as mx
from mlx_lm import load

layer = int(sys.argv[1])
here = os.path.dirname(os.path.abspath(__file__))
ref = np.fromfile(os.path.join(here, "data", "ref_hidden_760.bin"), dtype="<f4").reshape(41, 2048)
model, _ = load("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined")
L = model.model.layers[layer]
x = mx.array(ref[layer]).reshape(1, 1, 2048).astype(mx.bfloat16)
a = L.self_attn(L.input_layernorm(x), None, None) if not L.is_linear else L.linear_attn(L.input_layernorm(x), None, None)
h = x + a
out = h + L.mlp(L.post_attention_layernorm(h))
h_np = np.array(h.astype(mx.float32)).reshape(-1).astype("<f4")
out_np = np.array(out.astype(mx.float32)).reshape(-1).astype("<f4")
h_np.tofile(os.path.join(here, "data", f"ref_h{layer}.bin"))
rel = np.linalg.norm(out_np - ref[layer + 1]) / np.linalg.norm(ref[layer + 1])
print(f"WROTE layer={layer} h_norm={np.linalg.norm(h_np):.6f} ref_out_recon_rel={rel:.6f}")
