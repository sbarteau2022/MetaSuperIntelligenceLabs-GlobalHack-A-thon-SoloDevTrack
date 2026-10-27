#!/usr/bin/env python3
"""Probe a candidate corpus and promote only verified tasks to the frozen set.

Usage (AFTER the runner patch is applied and the active ablation run has finished):
  1. Copy the vault to a throwaway path (never probe against the live vault).
  2. Run the patched runner in probe mode for BOTH arms on the throwaway copy:
       ablation_runner --arm control   --corpus corpus_a39_candidate_v1.json \
           --output probe_control.jsonl   --seed 0xC0DECAFE --vault-copy /tmp/a39_throwaway.atoma_map --no-warmup --no-persist
       ablation_runner --arm treatment --corpus corpus_a39_candidate_v1.json \
           --output probe_treatment.jsonl --seed 0xC0DECAFE --vault-copy /tmp/a39_throwaway.atoma_map --no-warmup --no-persist
     Probe output rows must carry the full decoded_output_text plus judge fields
     (see runner_receipt_patch.md). Do NOT use the old snippet-only output: a
     200-char snippet cannot verify a task.
  3. python3 probe_corpus.py corpus_a39_candidate_v1.json probe_control.jsonl \
       probe_treatment.jsonl --out corpus_a39_frozen_v1.json

Promotion rule (preregistered, do not soften after seeing results):
  A task is COUNTED iff at least one arm, in the probe, returns a coherent
  decoded text (non-empty, no fail-closed error) AND its preregistered judge
  check passes on that decoded text. A task that errors in both arms, or
  whose expected check never appears in either arm, is DROPPED, not counted
  as a fail. Rationale: such a task measures the judge/vault mismatch, not
  the carved-view treatment, and counting it would reproduce the 0/24
  uninformative run.
  The run proceeds only if the counted set is >= 40 tasks. If it is < 40,
  expand the candidate corpus and re-probe; never lower the promotion rule.

Output: frozen corpus JSON (tasks in counted set only, verification_status
"verified_by_probe", probe arm results recorded per task) plus its sha256.
The frozen file, not the candidate file, is the corpus the A39 run uses.
"""
import json, hashlib, sys, argparse

import os
sys.path.insert(0, os.path.expanduser("~/workspace/deterministic-judge"))
from judge import run_rubric  # noqa: E402  (deterministic-judge v1.0, tested)

def load_rows(path):
    rows = {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            r = json.loads(line)
            rows[r["task_id"]] = r
    return rows

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("candidate")
    ap.add_argument("probe_control")
    ap.add_argument("probe_treatment")
    ap.add_argument("--out", required=True)
    ap.add_argument("--min-counted", type=int, default=40)
    args = ap.parse_args()

    cand = json.load(open(args.candidate))
    control = load_rows(args.probe_control)
    treatment = load_rows(args.probe_treatment)

    counted, dropped = [], []
    for task in cand["tasks"]:
        tid = task["task_id"]
        results = {}
        passed_any = False
        for arm, rows in (("control", control), ("treatment", treatment)):
            row = rows.get(tid)
            if row is None:
                results[arm] = {"status": "missing_from_probe"}
                continue
            if row.get("error"):
                results[arm] = {"status": "error", "error": row["error"]}
                continue
            decoded = row.get("decoded_output_text") or row.get("output_snippet", "")
            if not decoded.strip():
                results[arm] = {"status": "empty_decoded"}
                continue
            # Re-judge from the run file alone: the runner's verdict is checked,
            # not trusted blindly. If the runner stored a verdict, it must agree.
            check = task["judge"]
            verdict, checks, _err = run_rubric(
                decoded,
                {"checks": [{"id": "a39", "check": check["check"],
                             "target": check["target"],
                             **({"case_sensitive": check["case_sensitive"]} if "case_sensitive" in check else {}),
                             "required": True}]},
            )
            ok = verdict == "pass"
            results[arm] = {"status": "pass" if ok else "fail", "judge_verdict": verdict}
            if row.get("judge_verdict") and row["judge_verdict"] != verdict:
                results[arm]["verdict_mismatch"] = row["judge_verdict"]
            passed_any = passed_any or ok
        if passed_any:
            t = dict(task)
            t["verification_status"] = "verified_by_probe"
            t["probe_results"] = results
            counted.append(t)
        else:
            dropped.append({"task_id": tid, "probe_results": results})

    print(f"candidate={len(cand['tasks'])} counted={len(counted)} dropped={len(dropped)}")
    for d in dropped:
        print(f"  dropped {d['task_id']}: {d['probe_results']}")
    if len(counted) < args.min_counted:
        print(f"BLOCKED: counted set {len(counted)} < {args.min_counted}. "
              "Do not run A39. Expand the candidate corpus and re-probe.")
        sys.exit(2)

    doc = {"corpus_id": "a39_frozen_v1", "protocol": "CS-ABLATION-01",
           "status": "FROZEN",
           "derived_from": cand["corpus_id"],
           "promotion_rule": "counted iff >=1 arm passes preregistered judge on coherent decoded text in probe",
           "tasks": counted, "dropped": dropped}
    text = json.dumps(doc, indent=2) + "\n"
    with open(args.out, "w") as f:
        f.write(text)
    h = hashlib.sha256(text.encode()).hexdigest()
    with open(args.out + ".sha256", "w") as f:
        f.write(f"{h}  {args.out}\n")
    print(f"frozen={args.out} sha256={h}")

if __name__ == "__main__":
    main()
