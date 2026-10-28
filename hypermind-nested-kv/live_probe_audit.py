#!/usr/bin/env python3
"""Probe audit at 1,631 atoms: how often does the right DOC win top-1?"""

import json
import urllib.request

BASE = "http://127.0.0.1:8765"
_tid = [0]


def call(tool, args):
    _tid[0] += 1
    body = json.dumps(
        {
            "jsonrpc": "2.0",
            "id": _tid[0],
            "method": "tools/call",
            "params": {"name": tool, "arguments": args},
        }
    ).encode()
    req = urllib.request.Request(
        BASE, data=body, headers={"Content-Type": "application/json"}
    )
    with urllib.request.urlopen(req, timeout=120) as r:
        resp = json.loads(r.read())
    return json.loads(resp["result"]["content"][0]["text"])


CASES = [
    ("greedy top-k dominates stride sampling", "DISTILLATION_V1_V2_DEPTH_PROOF"),
    ("no-gradient invariant dW = 0", "DISTILLATION_V1_V2_DEPTH_PROOF"),
    ("O(1) id counter mind_max_id lazy vp rebuild", "Design.md"),
    ("Kuramoto mean-field 243x vs all-pairs", "MicrokernelBuildAndCertification"),
    ("spiral-out radius NOT a corkscrew", "TorodialPhasePositionalEncoder"),
    (
        "constraint network executable predicates",
        "AttentionEconomicsAndActiveInference",
    ),
    ("pre-registration A39", "Pre-Registration-A39"),
    ("judge gate every minted atom", "PROTOCOL.md"),
    ("volume floor 8,644 atoms 104,312 arcs", "PROTOCOL.md"),
    ("cross-salience 0.9 query 0.5 recall", "Certification.md"),
    ("id counter O(1) append lazy rebuild", "C06"),
    ("metallib pinned binary", "HyperbolicMicrokernelPlugin"),
]
ok = 0
for probe, want in CASES:
    r = call("map.query", {"qtext": probe, "k": 3})
    hits = r["result"]["hits"]
    top = hits[0]["text"][:60].replace("\n", " ") if hits else "NONE"
    hit = want.lower() in (hits[0]["text"][:200].lower() if hits else "")
    ok += hit
    print(f"{'HIT ' if hit else 'MISS'} want~{want[:40]:40s} got: {top}")
print(f"\n{ok}/{len(CASES)} doc-correct top-1")
s = call("map.snapshot", {})
print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"])
