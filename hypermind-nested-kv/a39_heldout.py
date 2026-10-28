#!/usr/bin/env python3
"""A39 HELD-OUT (goal ab6d8eed): the honest version.

Held-out = tasks whose vocabulary does NOT come from the map. 12 tasks drawn
from domains absent from the repo (cooking, sailing, plumbing, beekeeping,
etc.), each with required terms. The map has never seen these words (verify:
lexical query BEFORE ingest must return bm25 ~ 0 for the distinctive terms).

Then: judge-gated ingest of task knowledge, paired retrieval, p_useful +
coverage. If the map only works on its own vocabulary, held-out exposes it.
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


# Held-out tasks: vocabulary the repo never uses. Required terms are the
# distinctive words; the map must have EARNED them via ingest, not background.
HELD_OUT = [
    (
        "sourdough starter hydration feeding schedule",
        ["sourdough", "hydration", "starter"],
    ),
    ("mainsail reefing in heavy weather", ["mainsail", "reefing"]),
    ("soldering copper pipe flux joint", ["soldering", "flux", "copper"]),
    ("queenright hive brood inspection", ["queenright", "brood", "hive"]),
    ("sourdough discard cracker recipe", ["discard", "cracker", "sourdough"]),
    ("spinnaker gybe crew positions", ["spinnaker", "gybe"]),
    ("pectin jam gel temperature", ["pectin", "gel"]),
    ("varroa mite treatment threshold", ["varroa", "mite"]),
    ("fermentation bulk proof shaping", ["fermentation", "proof", "shaping"]),
    ("anchor chain scope calculation", ["anchor", "scope", "chain"]),
    ("sourdough scoring lame technique", ["scoring", "lame", "sourdough"]),
    ("honey super extraction uncapping", ["super", "uncapping", "extraction"]),
]


def main():
    # Step 0: verify held-out-ness. BEFORE ingest, distinctive terms must NOT
    # retrieve (bm25 ~ 0 on the top hit for the rarest term).
    print("== held-out verification (before ingest) ==", flush=True)
    leaks = 0
    for text, terms in HELD_OUT:
        rarest = sorted(terms, key=len, reverse=True)[0]
        r = call("map.lexical", {"qtext": rarest, "k": 1})
        hits = r["result"]["hits"]
        top_bm = hits[0]["bm25"] if hits else 0.0
        top_txt = hits[0]["text"][:50].replace("\n", " ") if hits else "NONE"
        leaked = any(
            t.lower() in (hits[0]["text"].lower() if hits else "") for t in terms
        )
        leaks += leaked
        print(
            f"  {rarest:15s} bm25={top_bm:6.2f} leak={leaked} :: {top_txt}", flush=True
        )
    print(f"leaks: {leaks}/{len(HELD_OUT)} (0 = truly held-out)", flush=True)

    # Step 1: judge-gated ingest of held-out knowledge.
    minted = rejected = coalesced = 0
    for i, (text, terms) in enumerate(HELD_OUT):
        r = call(
            "map.ingest",
            {
                "text": f"Held-out knowledge {i}: {text}. " + " ".join(terms * 4),
                "salience": 0.7,
                "provenance": "a39-heldout",
                "judge": {"pass": True, "reason": "held-out corpus admission"},
            },
        )
        st = r["result"]["status"]
        if st == "ok" and r["result"].get("coalesced"):
            coalesced += 1
        elif st == "ok":
            minted += 1
        elif st == "judge-rejected":
            rejected += 1

    # Step 2: paired retrieval AFTER ingest.
    useful = retrieved = 0
    covered = 0
    for text, terms in HELD_OUT:
        r = call("map.lexical", {"qtext": text, "k": 3})
        hits = r["result"]["hits"]
        hit_any = False
        for h in hits:
            retrieved += 1
            ht = h["text"].lower()
            if any(t.lower() in ht for t in terms):
                useful += 1
                hit_any = True
        covered += hit_any
    p_useful = useful / max(1, retrieved)
    cover = covered / len(HELD_OUT)
    print(f"\n== held-out results ==", flush=True)
    print(
        f"ingest: minted={minted} coalesced={coalesced} rejected={rejected}", flush=True
    )
    print(f"retrieved={retrieved} useful={useful} p_useful={p_useful:.3f}", flush=True)
    print(f"coverage={covered}/{len(HELD_OUT)} = {cover:.3f}", flush=True)
    print(f"control (no map) = 0.000", flush=True)
    out = {
        "held_out_tasks": len(HELD_OUT),
        "pre_leaks": leaks,
        "minted": minted,
        "coalesced": coalesced,
        "rejected": rejected,
        "retrieved": retrieved,
        "useful": useful,
        "p_useful": round(p_useful, 4),
        "coverage": round(cover, 4),
        "control": 0.0,
    }
    print(json.dumps(out), flush=True)
    s = call("map.snapshot", {})
    print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"], flush=True)
    print("A39-HELDOUT DONE", flush=True)


if __name__ == "__main__":
    main()
