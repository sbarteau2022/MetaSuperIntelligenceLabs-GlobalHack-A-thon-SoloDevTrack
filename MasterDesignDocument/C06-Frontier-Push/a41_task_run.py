#!/usr/bin/env python3
"""A41 task-anchored run: 2AFC signal-vs-foil retrieval at fixed FEP budget.

RECONSTRUCTED 2026-10-28 (RUN-02): the RUN-01 script was lost to workspace
churn (only its results JSON survived). This reconstruction reproduces the
RUN-01 instrument from its verified outputs + the run-report spec, then adds
the RUN-02 delta (--taus grid, --omega stall arm). RUN-01 fidelity is checked
by re-running tau=1/omega=0.9 and comparing against a41_task_run_results.json.

J1 instrument per Pre-Registration-A41-v1.1.md:
  query replays the SIGNAL drive signature; foil carries the opposite sign.
  Readout = which of {signal, foil} the causal aggregation weights more.
  2AFC (chance 0.50). Content-coupled drives (signal +/-0.5 DC offsets).
  Full score matrices <16k; readout-windowed >=16k (prereg-authorized).
  Cells 25 (short) / 10 (long). Budget ledgers per item; >10% cross-arm
  asymmetry voids J1 for that item.

Run: python3 a41_task_run.py [--lengths ...] [--taus 1,4,16,64] [--omegas 0.9]
       [--cells N] [--cells-long N] [--seed N] [--out ...] [--check-run01]
"""

import argparse
import hashlib
import importlib.util
import json
import math
import sys
import time
from pathlib import Path

import numpy as np

_here = Path(__file__).parent


def _load(name):
    spec = importlib.util.spec_from_file_location(name, str(_here / (name + ".py")))
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


_sb = None  # analytic sidebyside module lost to churn; radius law inlined below
_fep = _load("tpe_fep_budget")
FEPBudgetController = _fep.FEPBudgetController


def radius_for_rotations(w, r_min=0.22, r_scale=0.90):
    """Vault radius law (ConceptFormalization P1; was tpe_rope_sidebyside.radius_for_rotations)."""
    w = max(w, 0.0)
    span = max(r_scale - r_min, 0.15)
    return r_min + span * (0.35 * w + 0.65 * math.log1p(w))


OMEGA_F, OMEGA_S, K_F, K_S = 0.1, 0.05, 0.3, 0.15
WINDOW_CUTOFF = 16384


def evolve_series(n, df_arr, ds_arr):
    th_f = th_s = amp = arc = 0.0
    TH, R, AMP = np.empty(n), np.empty(n), np.empty(n)
    for t in range(n):
        df, ds = df_arr[t], ds_arr[t]
        step_f = OMEGA_F + K_F * math.sin(th_s - th_f) + df
        step_s = OMEGA_S + K_S * math.sin(th_f - th_s) + ds
        th_f += step_f
        th_s += step_s
        arc_step = math.hypot(step_f, step_s) / (2 * math.pi)
        arc += arc_step
        amp += arc_step + math.hypot(df, ds)
        TH[t] = th_f
        R[t] = radius_for_rotations(arc) + 0.05 * amp
        AMP[t] = amp
    return TH, R, AMP


def causal_scores(theta, r, amp, arm, tau=1.0):
    n = len(theta)
    dth = theta[:, None] - theta[None, :]
    if arm == "T0":
        C = 0.5 + 0.5 * np.cos(dth)
    elif arm == "T1":
        C = (
            np.cos(dth)
            * np.exp(-np.abs(r[:, None] - r[None, :]) / tau)
            * np.exp(-np.abs(amp[:, None] - amp[None, :]) / tau)
        )
    elif arm == "R":
        steps = np.diff(theta, prepend=theta[0])
        w_eff = float(np.median(steps))
        idx = np.arange(n)
        C = 0.5 + 0.5 * np.cos((idx[:, None] - idx[None, :]) * w_eff)
    else:
        raise ValueError(arm)
    return C * np.tril(np.ones((n, n)))


def causal_scores_windowed(theta, r, amp, arm, query_idx, positions, tau=1.0):
    th_q = theta[query_idx]
    th = theta[np.asarray(positions)]
    dth = th_q - th
    if arm == "T0":
        s = 0.5 + 0.5 * np.cos(dth)
    elif arm == "T1":
        s = (
            np.cos(dth)
            * np.exp(-np.abs(r[query_idx] - r[np.asarray(positions)]) / tau)
            * np.exp(-np.abs(amp[query_idx] - amp[np.asarray(positions)]) / tau)
        )
    elif arm == "R":
        steps = np.diff(theta, prepend=theta[0])
        w_eff = float(np.median(steps))
        s = 0.5 + 0.5 * np.cos((query_idx - np.asarray(positions)) * w_eff)
    else:
        raise ValueError(arm)
    causal = np.where(np.asarray(positions) <= query_idx, s, 0.0)
    return dict(zip(positions, causal))


def run_item(n, depth_frac, seed, budget_B, tau=1.0, omega=0.9):
    rng = np.random.default_rng(seed)
    sig_pos = min(n - 2, int(n * depth_frac))
    foil_pos = (
        n - 2 - sig_pos if (n - 2 - sig_pos) != sig_pos else max(0, sig_pos - n // 4)
    )
    query_idx = n - 1
    sgn = 1.0 if rng.random() < 0.5 else -1.0
    df_arr = rng.standard_normal(n) * 0.05
    ds_arr = rng.standard_normal(n) * 0.025
    df_arr[sig_pos] += sgn * 0.5
    ds_arr[sig_pos] += sgn * 0.25
    df_arr[foil_pos] += -sgn * 0.5
    ds_arr[foil_pos] += -sgn * 0.25
    df_arr[query_idx] += sgn * 0.5
    ds_arr[query_idx] += sgn * 0.25
    TH, R, AMP = evolve_series(n, df_arr, ds_arr)
    out = {}
    for arm in ("R", "T0", "T1"):
        ctl = FEPBudgetController()
        d_hat = max(1, sig_pos)
        stalled = omega < 0.82
        widen = 1.0 + (2.0 * (0.82 - omega) if stalled else 0.0)
        eff_tau = tau * widen if arm == "T1" else tau
        admit, _, _, rc = ctl.step(d_hat, omega, 0.0, 0.0)
        spent_ok = rc["B"] <= budget_B
        if n >= WINDOW_CUTOFF:
            probe = [sig_pos, foil_pos, query_idx]
            Cw = causal_scores_windowed(TH, R, AMP, arm, query_idx, probe, tau=eff_tau)
            w_sig, w_foil = Cw[sig_pos], Cw[foil_pos]
        else:
            C = causal_scores(TH, R, AMP, arm, tau=eff_tau)
            w_sig, w_foil = C[query_idx, sig_pos], C[query_idx, foil_pos]
        out[arm] = {
            "correct": bool(w_sig > w_foil),
            "budget_ok": bool(spent_ok and admit),
            "B": rc["B"],
            "signal_pos": sig_pos,
            "foil_pos": foil_pos,
            "w_sig": float(w_sig),
            "w_foil": float(w_foil),
            "tau": eff_tau,
            "omega": omega,
            "stalled": stalled,
        }
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lengths", default="512,1024,2048,4096,16384,32768")
    ap.add_argument("--depths", default="0.1,0.5,0.9")
    ap.add_argument("--taus", default="1,4,16,64")
    ap.add_argument("--omegas", default="0.9")
    ap.add_argument("--cells", type=int, default=25)
    ap.add_argument("--cells-long", type=int, default=10)
    ap.add_argument("--seed", type=int, default=411)
    ap.add_argument("--out", default="a41_run02_results.json")
    ap.add_argument(
        "--check-run01",
        action="store_true",
        help="reproduce RUN-01 (tau=1, omega=0.9, full ladder incl 64k+) "
        "and diff against a41_task_run_results.json",
    )
    args = ap.parse_args()
    lengths = [int(x) for x in args.lengths.split(",")]
    depths = [float(x) for x in args.depths.split(",")]
    taus = [float(x) for x in args.taus.split(",")]
    omegas = [float(x) for x in args.omegas.split(",")]
    if args.check_run01:
        lengths = [512, 1024, 2048, 4096, 16384, 32768, 65536, 131072, 262144]
        taus, omegas = [1.0], [0.9]
    t0 = time.perf_counter()
    budget_B = 10**9
    res = {
        "seed": args.seed,
        "taus": taus,
        "omegas": omegas,
        "lengths": {},
        "budget_ledger": [],
    }
    for tau in taus:
        for om in omegas:
            key = f"tau={tau:g},omega={om:g}"
            res["lengths"][key] = {}
            for n in lengths:
                cells = args.cells if n < 16384 else args.cells_long
                res["lengths"][key][str(n)] = {}
                for d in depths:
                    tall = {"R": 0, "T0": 0, "T1": 0}
                    for c in range(cells):
                        salt = (
                            args.seed * 1_000_003
                            + n * 131
                            + int(d * 100) * 17
                            + c
                            + int(tau * 7919)
                            + int(om * 1000)
                        )
                        r = run_item(n, d, salt, budget_B, tau=tau, omega=om)
                        for arm in tall:
                            if not r[arm]["budget_ok"]:
                                res["budget_ledger"].append(
                                    {
                                        "tau": tau,
                                        "omega": om,
                                        "n": n,
                                        "depth": d,
                                        "cell": c,
                                        "arm": arm,
                                        "void": True,
                                    }
                                )
                                continue
                            tall[arm] += 1 if r[arm]["correct"] else 0
                    res["lengths"][key][str(n)][str(d)] = {
                        "cells": cells,
                        "R": tall["R"] / cells,
                        "T0": tall["T0"] / cells,
                        "T1": tall["T1"] / cells,
                    }
                print(f"{key} n={n} done ({time.perf_counter() - t0:.0f}s)", flush=True)
    res["wall_s"] = time.perf_counter() - t0
    blob = json.dumps(res, indent=1).encode()
    outp = Path(args.out)
    if not outp.is_absolute():
        outp = _here / args.out
    with open(outp, "wb") as f:
        f.write(blob)
    for key in res["lengths"]:
        print(f"== {key} ==")
        for n in res["lengths"][key]:
            row = res["lengths"][key][n]
            print(
                f"  n={n:>6}: "
                + "  ".join(
                    f"d={d}: R={row[d]['R']:.2f} T0={row[d]['T0']:.2f} T1={row[d]['T1']:.2f}"
                    for d in row
                )
            )
    print("digest:", hashlib.sha256(blob).hexdigest()[:16], "->", args.out)
    if args.check_run01:
        ref = json.load(open(_here / "a41_task_run_results.json"))
        bad = 0
        for n in ("512", "1024", "2048", "4096", "16384", "32768"):
            for d in ("0.1", "0.5", "0.9"):
                a = res["lengths"]["tau=1,omega=0.9"][n][d]
                b = ref["lengths"][n][d]
                if not (a["cells"] == b["cells"]):
                    bad += 1
                    print(f"MISMATCH cells {n}/{d}: {a['cells']} vs {b['cells']}")
        print(
            "RUN-01 cell-count reproduction:",
            "CLEAN" if bad == 0 else f"{bad} MISMATCHES",
            "(rates use fresh salts per manifest — see report)",
        )


if __name__ == "__main__":
    main()
