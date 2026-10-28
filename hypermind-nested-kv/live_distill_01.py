#!/usr/bin/env python3
"""Live distill step 1: ingest the repo's own design docs into the live map.
Chunked per-section so each atom is one retrievable claim; one pass; fail-closed."""

import json
import re
import subprocess
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
    with urllib.request.urlopen(req, timeout=60) as r:
        resp = json.loads(r.read())
    return json.loads(resp["result"]["content"][0]["text"])


FILES = [
    "../DistillationProtocol/PROTOCOL.md",
    "../DistillationProtocol/CONVERSION_MATH.md",
    "../DistillationProtocol/DISTILLATION_V1_V2_DEPTH_PROOF.md",
    "../HyperMindNestedKV-Certification.md",
]


def chunks(path):
    raw = open(path).read()
    parts = re.split(r"\n(?=#{1,3} )", raw)
    out = []
    for p in parts:
        p = p.strip()
        if len(p) < 200:
            continue
        out.append(p[:2000])
    return out


def main():
    total = 0
    for f in FILES:
        for c in chunks(f):
            r = call(
                "map.ingest",
                {
                    "text": f"[{f.split('/')[-1]}] {c}",
                    "salience": 0.5,
                    "provenance": "live-distill-01",
                    "taint": "none",
                },
            )
            assert r["result"]["status"] == "ok", r
            total += 1
    print(f"ingested {total}")
    # verify: query each doc's headline back
    for probe in [
        "teacher-invariance",
        "retained energy per budget",
        "CERTIFIED WITH CORRECTIONS",
        "greedy top-k dominates",
    ]:
        r = call("map.query", {"qtext": probe, "k": 3})
        hits = r["result"]["hits"]
        print(f"probe {probe!r:45s} -> top: {hits[0]['text'][:80] if hits else None}")
        assert hits and r["receipt"]["receipt_sha256"], probe
    s = call("map.snapshot", {})
    print("snapshot", s["result"]["digest"], s["result"]["atoms"])
    assert s["result"]["status"] == "ok"
    print("LIVE-DISTILL-01 PASS")


if __name__ == "__main__":
    main()
