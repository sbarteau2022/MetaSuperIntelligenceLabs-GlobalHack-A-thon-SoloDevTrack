#!/usr/bin/env python3
"""Ablation sweep stub for distillation parameters (PROPOSED, simulated).

Models band allocation + greedy-vs-stride retained energy on a Zipf spectrum.
NOT a measurement on a real teacher checkpoint (see DISTILLATION_* not-checked).
"""

import json
import math


def zipf_energy(N, s, k):
    # returns retained energy for taking top-k of Zipf spectrum E_i = 1/i^s
    es = [1.0 / (i**s) for i in range(1, N + 1)]
    es.sort(reverse=True)
    top = sum(es[:k])
    total = sum(es)
    return top / total


def stride_energy(N, s, k):
    es = [1.0 / (i**s) for i in range(1, N + 1)]
    # uniform stride sample
    stride = max(1, N // k)
    idx = list(range(0, N, stride))[:k]
    return sum(es[i] for i in idx) / sum(es)


models = {
    "Muse-Glimmer": {"L": 52, "B_options": [64, 96, 128]},
    "Nemotron-30B": {"L": 52, "B_options": [64, 96, 128]},
    "Qwen3.6-MoE": {"L": 40, "B_options": [48, 64, 96]},
}

N = 1000
s = 1.0
rows = []
for name, info in models.items():
    L = info["L"]
    n_early = math.floor(0.34 * L)
    n_mid = math.floor(0.34 * L)
    n_late = L - n_early - n_mid
    for B in info["B_options"]:
        Be, Bm, Bl = round(0.20 * B), round(0.50 * B), round(0.30 * B)
        ke, km, kl = (
            max(1, round(Be / n_early)),
            max(1, round(Bm / n_mid)),
            max(1, round(Bl / n_late)),
        )
        rows.append(
            {
                "model": name,
                "L": L,
                "B": B,
                "bands_layers": {"early": n_early, "mid": n_mid, "late": n_late},
                "band_budget": {"early": Be, "mid": Bm, "late": Bl},
                "k_per_layer": {"early": ke, "mid": km, "late": kl},
                "walks_max": L - 1,
                "greedy_retained": zipf_energy(N, s, max(1, B // L)),
                "stride_retained": stride_energy(N, s, max(1, B // L)),
            }
        )

# greedy vs stride on canonical probe
probe = {
    "N": 1000,
    "k": 64,
    "s": 1.0,
    "greedy": zipf_energy(1000, 1.0, 64),
    "stride": stride_energy(1000, 1.0, 64),
}
probe["ratio"] = probe["greedy"] / probe["stride"]

out = {"probe": probe, "rows": rows}
with open("DistillationProtocol/ablation_sweep_results.json", "w") as f:
    json.dump(out, f, indent=2)
print(json.dumps(probe, indent=2))
print("rows:", len(rows))
for r in rows:
    print(
        r["model"],
        "B=" + str(r["B"]),
        "k=",
        r["k_per_layer"],
        "walks_max=",
        r["walks_max"],
    )
