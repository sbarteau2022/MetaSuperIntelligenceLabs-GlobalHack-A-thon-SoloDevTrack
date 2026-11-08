"""Oracle dump (not in the harness I/O path): reference intermediates of layer 31's full attention at
position 0 for the reference input tests/data/ref_hidden_760.bin[31]. Writes qg (q_proj output, [head][q|gate]),
v (v_proj output) and gated (attention output * sigmoid(gate), before o_proj) as f32 LE files."""
import os, sys
import numpy as np
import mlx.core as mx
from mlx_lm import load

LAYER = 31
here = os.path.dirname(os.path.abspath(__file__))
ref = np.fromfile(os.path.join(here, "data", "ref_hidden_760.bin"), dtype="<f4").reshape(41, 2048)
model, _ = load("/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined")
A = model.model.layers[LAYER].self_attn
x = mx.array(ref[LAYER]).reshape(1, 1, 2048).astype(mx.bfloat16)
xn = model.model.layers[LAYER].input_layernorm(x)
qg = A.q_proj(xn)                       # [1,1,8192]
v = A.v_proj(xn)                        # [1,1,512]
nh, hd, nkv = 16, 256, 2
qgh = qg.reshape(nh, 2 * hd)
gate = qgh[:, hd:]                       # [16,256]
vh = v.reshape(nkv, hd)
mix = mx.repeat(vh, nh // nkv, axis=0)   # position 0: softmax over one key -> value passes through
gated = (mix * mx.sigmoid(gate.astype(mx.float32)).astype(mix.dtype)).reshape(-1)
for name, arr in [("qg", qg.reshape(-1)), ("v", v.reshape(-1)), ("gated", gated)]:
    out = os.path.join(here, "data", f"ref_l{LAYER}_{name}.bin")
    np.array(arr.astype(mx.float32)).astype("<f4").tofile(out)
    print("WROTE", name, arr.shape)
