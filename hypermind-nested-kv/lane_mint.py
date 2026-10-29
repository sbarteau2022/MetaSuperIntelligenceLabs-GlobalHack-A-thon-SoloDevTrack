#!/usr/bin/env python3
"""Lane trajectory minting, gated — 2026-10-28.

Starts minting from Ares lane runs WITHOUT touching the serving path
(:8787 backend, lanes, or the global_active_mind vault are not modified).

What it does, in order:
1. Reads one completed lane run as JSON (from /agent/run output, or a
   hand-written file with the same fields).
2. GATE: mints ONLY if verdict == "passed" (madmind-core judges passed).
   Anything else is written to the held file, never minted.
3. Mints exactly ONE atom per passed run via hypermind-mcp map.ingest
   (:8765), with provenance = run_id + lane. No per-token, per-step,
   or per-doc flooding — live-distill-02 minted 1,603 doc atoms and
   the probe audit came back 0/12 correct, so volume is not the goal.
4. Appends the full trajectory record to live-map/lane_trajectories.jsonl
   (text-level trajectory: prompt hash, steps, tool calls, judge verdict).
   This is NOT a weight-level TeacherTapRecord — lanes return text only,
   so no hidden-state claim is made here.

Usage:
  python3 lane_mint.py --run run.json            # dry run, mints nothing
  python3 lane_mint.py --run run.json --mint     # gated mint, one atom
  python3 lane_mint.py --self-test               # dry-run gate check, no mint

Run JSON fields: run_id, lane, prompt, output_summary, verdict,
steps (list of strings), tool_calls (list of strings). Missing verdict
is treated as NOT passed.
"""

import argparse
import hashlib
import json
import os
import sys
import urllib.request

BASE = "http://127.0.0.1:8765"
HERE = os.path.dirname(os.path.abspath(__file__))
TRAJ_PATH = os.path.join(HERE, "live-map", "lane_trajectories.jsonl")
HELD_PATH = os.path.join(HERE, "live-map", "lane_trajectories_held.jsonl")

_tid = [0]


def call(tool, args):
    _tid[0] += 1
    body = json.dumps(
        {"jsonrpc": "2.0", "id": _tid[0], "method": "tools/call",
         "params": {"name": tool, "arguments": args}}
    ).encode()
    req = urllib.request.Request(BASE, data=body,
                                 headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        resp = json.loads(r.read())
    return json.loads(resp["result"]["content"][0]["text"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", help="path to lane run JSON")
    ap.add_argument("--mint", action="store_true", help="actually mint (gated)")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        run = {"run_id": "self-test-2026-10-28", "lane": "test",
               "prompt": "self test", "output_summary": "gate check only",
               "verdict": "failed", "steps": [], "tool_calls": []}
    elif args.run:
        with open(args.run) as f:
            run = json.load(f)
    else:
        ap.error("--run or --self-test required")

    passed = run.get("verdict") == "passed"
    prompt_hash = hashlib.sha256(
        run.get("prompt", "").encode()).hexdigest()[:16]
    record = {
        "run_id": run.get("run_id", ""),
        "lane": run.get("lane", ""),
        "prompt_sha256_16": prompt_hash,
        "output_summary": run.get("output_summary", ""),
        "verdict": run.get("verdict", ""),
        "steps": run.get("steps", []),
        "tool_calls": run.get("tool_calls", []),
        "level": "text-action trajectory (not weight-level)",
    }

    if not passed:
        os.makedirs(os.path.dirname(HELD_PATH), exist_ok=True)
        with open(HELD_PATH, "a") as f:
            f.write(json.dumps(record) + "\n")
        print(f"HELD, not minted: verdict={run.get('verdict')} "
              f"run={run.get('run_id')} -> {HELD_PATH}")
        return 0

    os.makedirs(os.path.dirname(TRAJ_PATH), exist_ok=True)
    with open(TRAJ_PATH, "a") as f:
        f.write(json.dumps(record) + "\n")

    if not args.mint:
        print(f"DRY RUN, not minted: passed run {run.get('run_id')} "
              f"trajectory recorded -> {TRAJ_PATH}")
        print("Re-run with --mint to mint one atom.")
        return 0

    text = (f"Lane trajectory {run.get('run_id')} lane={run.get('lane')} "
            f"verdict=passed steps={len(record['steps'])} "
            f"tools={len(record['tool_calls'])} "
            f"summary={run.get('output_summary', '')}")
    result = call("map.ingest", {
        "text": text,
        "provenance": f"lane-run:{run.get('run_id')}:{run.get('lane')}",
        "salience": 1.0,
        "taint": "lane-trajectory-text-level",
    })
    print(f"MINTED 1 atom: run={run.get('run_id')} result={result}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
