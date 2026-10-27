#!/usr/bin/env python3
"""Compute J1-J6 from A39 run files ALONE (no server, no re-run).

Usage:
  python3 analyze_a39.py results/control.jsonl results/treatment.jsonl \
      --corpus corpus_a39_frozen_v1.json --out results/a39_report.json

Rules (from prereg_a39_amendment.json):
  - Warm-up rows (warm_up=true) are excluded, and their presence in a
    measured file is reported as a warning.
  - J1 pass = judge_verdict == "pass" AND fallback_to_parent == false.
    Errors count in the denominator. Pass rate = passes / ALL rows.
  - J1 delta = treatment_pass_rate - control_pass_rate (percentage points).
    p-value by paired bootstrap, 10k resamples, as filed. Paired by task_id:
    only task_ids present in BOTH arms are counted, and the pairing is
    reported; unpaired tasks are listed, not silently dropped.
  - J2 retries = (attempt_count - 1) + backtrack_count. superposition_
    interventions is reported separately, never as retries.
  - J3 tokens = generated_token_count (expanded_tokens.len()). If a row
    only carries the old tokens_generated (emitted_ops.len()), it is
    flagged as legacy and excluded from J3, not silently used.
  - J4 latency = latency_ms, measured rows only.
  - J5 peak memory = peak_rss_bytes_sampled (sampled, not VRAM). If absent,
    J5 is reported unavailable, never estimated.
  - J6 requires a named second-machine run file (--replication <file>);
    without it, J6 is "not earned", not "failed".
  - The analyzer re-judges every row from its stored decoded_output_text
    using the frozen corpus judge, and reports any verdict mismatch.
  - Fallback rate per arm is always reported.
"""
import json, argparse, random, statistics, sys, os
sys.path.insert(0, os.path.expanduser("~/workspace/deterministic-judge"))
from judge import run_rubric

def load(path):
    return [json.loads(l) for l in open(path) if l.strip()]

def mean(xs):
    return statistics.mean(xs) if xs else None

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("control")
    ap.add_argument("treatment")
    ap.add_argument("--corpus", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--replication", default=None)
    ap.add_argument("--seed", type=int, default=0xA39)
    args = ap.parse_args()

    corpus = {t["task_id"]: t for t in json.load(open(args.corpus))["tasks"]}
    arms = {}
    warnings = []
    for name, path in (("control", args.control), ("treatment", args.treatment)):
        rows = load(path)
        warm = [r for r in rows if r.get("warm_up")]
        if warm:
            warnings.append(f"{name}: {len(warm)} warm-up rows present in measured file")
        rows = [r for r in rows if not r.get("warm_up")]
        # Re-judge from run file alone
        mismatches = []
        for r in rows:
            task = corpus.get(r["task_id"])
            if not task:
                warnings.append(f"{name}: {r['task_id']} not in frozen corpus")
                continue
            check = task["judge"]
            verdict, _, _ = run_rubric(
                r.get("decoded_output_text", ""),
                {"checks": [{"id": "a39", "check": check["check"], "target": check["target"],
                             **({"case_sensitive": check["case_sensitive"]} if "case_sensitive" in check else {}),
                             "required": True}]})
            if r.get("judge_verdict") and r["judge_verdict"] != verdict:
                mismatches.append(r["task_id"])
        arms[name] = {"rows": {r["task_id"]: r for r in rows}, "mismatches": mismatches}

    common = sorted(set(arms["control"]["rows"]) & set(arms["treatment"]["rows"]))
    unpaired = sorted((set(arms["control"]["rows"]) ^ set(arms["treatment"]["rows"])))
    if unpaired:
        warnings.append(f"unpaired task_ids (excluded from paired J1): {unpaired}")

    def is_pass(r):
        return r.get("judge_verdict") == "pass" and not r.get("fallback_to_parent", False)

    report = {}
    for name in ("control", "treatment"):
        rows = [arms[name]["rows"][t] for t in common]
        report[name] = {
            "n": len(rows),
            "j1_pass_rate": sum(is_pass(r) for r in rows) / len(rows) if rows else None,
            "fallback_rate": sum(bool(r.get("fallback_to_parent")) for r in rows) / len(rows) if rows else None,
            "j2_mean_retries": mean([(r.get("attempt_count", 1) - 1) + r.get("backtrack_count", 0) for r in rows]),
            "j2_interventions_mean": mean([r.get("superposition_interventions", 0) for r in rows]),
            "j3_mean_tokens": mean([r["generated_token_count"] for r in rows if "generated_token_count" in r]) if any("generated_token_count" in r for r in rows) else None,
            "j3_legacy_rows": sum(1 for r in rows if "generated_token_count" not in r),
            "j4_mean_latency_ms": mean([r.get("latency_ms", 0) for r in rows]),
            "j5_mean_peak_rss_bytes": mean([r["peak_rss_bytes_sampled"] for r in rows if r.get("peak_rss_bytes_sampled")]) if any(r.get("peak_rss_bytes_sampled") for r in rows) else None,
            "errors": sum(1 for r in rows if r.get("error")),
            "verdict_mismatches": arms[name]["mismatches"],
        }
    c, t = report["control"], report["treatment"]
    delta_pp = (t["j1_pass_rate"] - c["j1_pass_rate"]) * 100 if t["j1_pass_rate"] is not None and c["j1_pass_rate"] is not None else None
    # Paired bootstrap, 10k resamples, as filed, done correctly:
    # resampling pairs gives a distribution centred on the observed delta,
    # NOT on the null, so a naive P(d >= obs) is ~0.5 for any gap. The
    # null-centred (Hall) version shifts each bootstrap delta by -obs,
    # then counts P(shifted delta >= obs). Discordant pairs are also
    # reported, so the exact McNemar count is visible alongside it.
    rng = random.Random(args.seed)
    pairs = [(is_pass(arms["control"]["rows"][tid]), is_pass(arms["treatment"]["rows"][tid])) for tid in common]
    discordant_treatment_only = sum(1 for co, tr in pairs if tr and not co)
    discordant_control_only = sum(1 for co, tr in pairs if co and not tr)
    if pairs:
        obs = statistics.mean([1 if tr else 0 for _, tr in pairs]) - statistics.mean([1 if co else 0 for co, _ in pairs])
        ge = 0
        for _ in range(10000):
            sample = [rng.choice(pairs) for _ in pairs]
            d = statistics.mean([1 if tr else 0 for _, tr in sample]) - statistics.mean([1 if co else 0 for co, _ in sample])
            if d - obs >= obs:
                ge += 1
        p_value = (ge + 1) / 10001
    else:
        p_value = None
    report["j1_discordant"] = {"treatment_only": discordant_treatment_only,
                               "control_only": discordant_control_only}
    report["j1_delta_pp"] = delta_pp
    report["j1_p_value_bootstrap_10k"] = p_value
    report["j1_threshold"] = "delta >= +10 pp, p < 0.01"
    report["j1_passed"] = bool(delta_pp is not None and delta_pp >= 10 and p_value is not None and p_value < 0.01)
    for key, thresh in (("j2_mean_retries", -0.30), ("j3_mean_tokens", -0.25), ("j4_mean_latency_ms", -0.20), ("j5_mean_peak_rss_bytes", -0.15)):
        if c[key] is not None and t[key] is not None and c[key] != 0:
            report[key + "_delta_pct"] = (t[key] - c[key]) / c[key] * 100
            report[key + "_passed"] = (t[key] - c[key]) / c[key] <= thresh
        else:
            report[key + "_delta_pct"] = None
            report[key + "_passed"] = False
    report["j6"] = {"status": "not_earned_without_replication" if not args.replication else "replication_file_supplied_see_report",
                    "replication_file": args.replication}
    report["warnings"] = warnings
    report["paired_task_ids"] = common
    with open(args.out, "w") as f:
        json.dump(report, f, indent=2)
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    main()
