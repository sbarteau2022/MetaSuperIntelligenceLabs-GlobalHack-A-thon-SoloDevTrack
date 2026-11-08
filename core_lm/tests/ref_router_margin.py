"""Oracle diagnostic (not in the harness I/O path): for given layers, feed the reference's own input
hidden state, compute the reference router's top-8 experts and the score margin at the cut
(8th vs 9th). A small margin means bf16 rounding can flip the expert set, which explains
large but correct-looking layer errors."""
import os, sys
import numpy as np
import mlx.core as mx
from mlx_lm import load

here = os.path.dirname(os.path.abspath(__file__))
ref = np.fromfile(os.path.join(here, "data", "ref_hidden_760.bin"), dtype="<f4").reshape(41, 2048)
model, _ = load("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined")
for l in [int(a) for a in sys.argv[1:]]:
    L = model.model.layers[l]
    x = mx.array(ref[l]).reshape(1, 1, 2048).astype(mx.bfloat16)
    if L.is_linear:
        a = L.linear_attn(L.input_layernorm(x), None, None)
    else:
        a = L.self_attn(L.input_layernorm(x), None, None)
    h = x + a
    g = L.mlp.gate(L.post_attention_layernorm(h)).astype(mx.float32)
    p = np.array(mx.softmax(g, axis=-1)).reshape(-1)
    order = np.argsort(-p)
    s8, s9 = p[order[7]], p[order[8]]
    print(f"LAYER {l} kind={'lin' if L.is_linear else 'full'} top8={sorted(order[:8].tolist())} "
          f"p8={s8:.6f} p9={s9:.6f} margin_rel={(s8 - s9) / s8:.5f}")
