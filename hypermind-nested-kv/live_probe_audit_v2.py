#!/usr/bin/env python3
"""Probe audit v2: doc-correct AND topic-correct. The v1 audit demanded the
exact file; with section-chunked atoms the right TOPIC in the wrong file is
retrieval working, chunking splitting credit."""

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


# (probe, doc-must-contain, topic-must-contain-any)
CASES = [
    (
        "greedy top-k dominates stride sampling",
        "DEPTH_PROOF",
        ["top-k", "stride", "retained energy"],
    ),
    (
        "no-gradient invariant dW = 0",
        "DEPTH_PROOF",
        ["gradient", "frozen", "no-gradient", "dW"],
    ),
    (
        "O(1) id counter mind_max_id lazy vp rebuild",
        "Design.md",
        ["O(1)", "lazy", "counter", "rebuild"],
    ),
    (
        "Kuramoto mean-field 243x vs all-pairs",
        "MicrokernelBuildAndCertification",
        ["kuramoto", "mean-field", "243"],
    ),
    (
        "spiral-out radius NOT a corkscrew",
        "TorodialPhasePositionalEncoder",
        ["spiral", "corkscrew", "radius"],
    ),
    (
        "constraint network executable predicates",
        "AttentionEconomicsAndActiveInference",
        ["constraint", "predicate"],
    ),
    (
        "pre-registration A39",
        "Pre-Registration-A39",
        ["A39", "pre-registration", "prereg"],
    ),
    ("judge gate every minted atom", "PROTOCOL", ["judge"]),
    ("volume floor 8,644 atoms 104,312 arcs", "PROTOCOL", ["8,644", "floor", "volume"]),
    (
        "symplectic envelope traverse fail-explained",
        "SYMPLECTIC",
        ["traverse", "fail-explained", "symplectic"],
    ),
    ("map.reweight IDF calibrate", "main.rs", ["reweight", "IDF", "idf"]),
    ("Baseten grant hackathon", "grant", ["baseten", "grant", "hackathon"]),
]
doc_ok = top_ok = 0
for probe, want_doc, topics in CASES:
    r = call("map.query", {"qtext": probe, "k": 3})
    hits = r["result"]["hits"]
    top = hits[0]["text"] if hits else ""
    dh = want_doc.lower() in top[:200].lower()
    th = any(t.lower() in top.lower() for t in topics)
    doc_ok += dh
    top_ok += th
    flag = "DOC " if dh else ("TOPIC" if th else "MISS ")
    print(f"{flag} {probe[:45]:45s} -> {top[:60].replace(chr(10), ' ')}")
print(f"\ndoc-correct: {doc_ok}/{len(CASES)}  topic-correct: {top_ok}/{len(CASES)}")
s = call("map.snapshot", {})
print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"])
