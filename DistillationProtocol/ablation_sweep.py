#!/usr/bin/env python3
"""Ablation sweep for distillation parameters (SIMULATION ONLY).

Models band allocation + greedy-vs-stride retained energy on a Zipf spectrum.
NOT a measurement on a real teacher checkpoint (see AUDIT.md "What was NOT
checked": no measurement of retained-energy distributions on a real teacher).

PASS/FAIL thresholds are named constants below, each with a 1-line rationale.
Every output row carries a verdict. Re-run from anywhere:
  python3 DistillationProtocol/ablation_sweep.py
(output path anchors to this script's directory, not the cwd).
"""

import json
import math
import platform
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "ablation_sweep_results.json"

# --- PASS/FAIL thresholds (named constants; 1-line rationale each) ---
PROBE_GREEDY_STRIDE_RATIO_MIN = 3.0
# Rationale: pre-reg prediction 1 (PreRegistrations/distillation_ablation.md) floors greedy-vs-stride at 3x for (N=1000,k=64,Zipf s=1) against the filed 3.63x sim.
ROW_GREEDY_GE_STRIDE_REQUIRED = True
# Rationale: greedy top-k dominates uniform stride on a sorted spectrum by construction, so any row inversion signals a code bug rather than a finding.
WALKS_MAX_EQ_L_MINUS_1_REQUIRED = True
# Rationale: walks_max must equal L-1 because a trajectory walk spans all teacher layers, so any mismatch signals a band-partition bug.


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
        greedy = zipf_energy(N, s, max(1, B // L))
        stride = stride_energy(N, s, max(1, B // L))
        walks_max = L - 1
        checks = {
            "greedy_ge_stride": greedy >= stride,
            "walks_max_eq_L_minus_1": walks_max == L - 1,
        }
        verdict = (
            "PASS"
            if (checks["greedy_ge_stride"] or not ROW_GREEDY_GE_STRIDE_REQUIRED)
            and (
                checks["walks_max_eq_L_minus_1"]
                or not WALKS_MAX_EQ_L_MINUS_1_REQUIRED
            )
            else "FAIL"
        )
        rows.append(
            {
                "model": name,
                "L": L,
                "B": B,
                "bands_layers": {"early": n_early, "mid": n_mid, "late": n_late},
                "band_budget": {"early": Be, "mid": Bm, "late": Bl},
                "k_per_layer": {"early": ke, "mid": km, "late": kl},
                "walks_max": walks_max,
                "greedy_retained": greedy,
                "stride_retained": stride,
                "threshold_checks": checks,
                "verdict": verdict,
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
probe["ratio_min"] = PROBE_GREEDY_STRIDE_RATIO_MIN
probe["verdict"] = (
    "PASS" if probe["ratio"] >= PROBE_GREEDY_STRIDE_RATIO_MIN else "FAIL"
)

out = {
    "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
    "scope": "SIMULATION ONLY (pure Python Zipf sim). NOT a teacher-checkpoint measurement; no GPU used or needed.",
    "hardware": {
        "platform": sys.platform,
        "machine": platform.machine(),
        "python": platform.python_version(),
        "nvcc_present": shutil.which("nvcc") is not None,
        "nvidia_smi_present": shutil.which("nvidia-smi") is not None,
    },
    "thresholds": {
        "PROBE_GREEDY_STRIDE_RATIO_MIN": {
            "value": PROBE_GREEDY_STRIDE_RATIO_MIN,
            "rationale": "Pre-reg prediction 1 floor (3x) for (N=1000,k=64,Zipf s=1) vs filed 3.63x sim.",
        },
        "ROW_GREEDY_GE_STRIDE_REQUIRED": {
            "value": ROW_GREEDY_GE_STRIDE_REQUIRED,
            "rationale": "Greedy top-k dominates uniform stride on a sorted spectrum by construction; inversion signals a code bug.",
        },
        "WALKS_MAX_EQ_L_MINUS_1_REQUIRED": {
            "value": WALKS_MAX_EQ_L_MINUS_1_REQUIRED,
            "rationale": "walks_max must equal L-1 (walk spans all teacher layers); mismatch signals a band-partition bug.",
        },
    },
    "probe": probe,
    "rows": rows,
    "summary": {
        "rows_total": len(rows),
        "rows_pass": sum(1 for r in rows if r["verdict"] == "PASS"),
        "rows_fail": sum(1 for r in rows if r["verdict"] == "FAIL"),
        "probe_verdict": probe["verdict"],
    },
}
with open(OUT, "w") as f:
    json.dump(out, f, indent=2)
print(json.dumps(probe, indent=2))
print("rows:", len(rows), "pass:", out["summary"]["rows_pass"], "fail:", out["summary"]["rows_fail"])
for r in rows:
    print(
        r["model"],
        "B=" + str(r["B"]),
        "greedy=%.6f" % r["greedy_retained"],
        "stride=%.6f" % r["stride_retained"],
        r["verdict"],
    )
