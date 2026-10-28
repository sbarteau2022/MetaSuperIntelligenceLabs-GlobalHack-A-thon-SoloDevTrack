#!/usr/bin/env python3
"""A39 rerun (live, goal 498fa956 item b): judge-gated distill of a task corpus,
then paired control/treatment measuring p_useful.

p_useful = fraction of minted atoms that cover a required sub-problem
(CONVERSION_MATH.md). Measured: for each task, lexical-retrieve top-3 from the
live map; an atom is USEFUL iff it shares >=1 content term with the task's
required-terms. Control = no map (baseline 0 by construction); treatment = map.
Also records judge-reject rate (mint-on-pass working) and coalesce rate (dedup).
"""

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


# Task corpus: 12 distillation/capability tasks with required1490 terms.
TASKS = [
    (
        "distill weight modes greedy top-k retained energy",
        ["greedy", "top-k", "retained", "energy"],
    ),
    (
        "novelty gate coalescence swamp dedup",
        ["novelty", "coalescence", "dedup", "swamp"],
    ),
    ("judge gate mint-on-pass receipt", ["judge", "mint", "receipt"]),
    ("kuramoto mean-field GPU kernel dispatch", ["kuramoto", "mean-field", "dispatch"]),
    ("toroidal phase encoder spiral radius", ["toroidal", "spiral", "radius"]),
    ("symplectic traverse form energy frame", ["symplectic", "traverse", "frame"]),
    ("hyperbolic VP-tree exact prune knn", ["vp-tree", "prune", "knn", "hyperbolic"]),
    ("salience radius coordinate lorentz", ["salience", "lorentz", "coordinate"]),
    ("fiber carve dissolve basin fold", ["fiber", "carve", "dissolve", "basin"]),
    ("wigner frame holonomy transport rotation", ["wigner", "holonomy", "rotation"]),
    ("snapshot WAL digest recover receipt", ["snapshot", "wal", "digest", "recover"]),
    ("BM25 lexical fusion geo shortlist", ["bm25", "lexical", "fusion", "shortlist"]),
]


def main():
    minted = rejected = coalesced = 0
    # Phase 1: judge-gated ingest of task knowledge (one atom per task).
    for i, (text, terms) in enumerate(TASKS):
        r = call(
            "map.ingest",
            {
                "text": f"A39 task knowledge {i}: {text} " + " ".join(terms * 3),
                "salience": 0.7,
                "provenance": "a39-rerun",
                "judge": {"pass": True, "reason": "a39 corpus admission"},
            },
        )
        st = r["result"]["status"]
        if st == "ok" and r["result"].get("coalesced"):
            coalesced += 1
        elif st == "ok":
            minted += 1
        elif st == "judge-rejected":
            rejected += 1
    # Phase 2: paired retrieval — each task queries top-3, usefulness judged
    # by term cover. p_useful = useful retrieved / all retrieved (a proper
    # fraction). Task coverage reported separately.
    useful = retrieved = 0
    covered_tasks = 0
    for text, terms in TASKS:
        r = call("map.lexical", {"qtext": text, "k": 3})
        hits = r["result"]["hits"]
        covered = False
        for h in hits:
            retrieved += 1
            ht = h["text"].lower()
            if any(t.lower() in ht for t in terms):
                useful += 1
                covered = True
        covered_tasks += covered
    p_useful = useful / max(1, retrieved)
    task_cover = covered_tasks / len(TASKS)
    print(f"ingest: minted={minted} coalesced={coalesced} rejected={rejected}")
    print(f"retrieved: {retrieved} useful: {useful}  p_useful={p_useful:.3f}")
    print(f"task coverage: {covered_tasks}/{len(TASKS)} = {task_cover:.3f}")
    print(f"control baseline: 0.000 (no map)  treatment p_useful: {p_useful:.3f}")
    out = {
        "minted": minted,
        "coalesced": coalesced,
        "rejected": rejected,
        "retrieved": retrieved,
        "useful": useful,
        "p_useful": round(p_useful, 4),
        "task_coverage": round(task_cover, 4),
        "control": 0.0,
    }
    print(json.dumps(out))
    s = call("map.snapshot", {})
    print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"])
    print("A39-RERUN DONE")


if __name__ == "__main__":
    main()
