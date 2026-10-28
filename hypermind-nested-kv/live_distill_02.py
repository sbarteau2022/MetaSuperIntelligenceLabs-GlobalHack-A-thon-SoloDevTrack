#!/usr/bin/env python3
"""Live distill 02: flood the live map with ALL repo design knowledge.

Every .md outside CapabilityTreeSitter/target/.git, chunked per-section
(~2KB cap), one pass, fail-closed. Provenance = relative path so every
atom cites its source. Skips files already covered by distill-01's 4 docs
only if their content hash matches (they won't — chunking differs; the
novelty gate doesn't exist yet, so distill-01's 28 atoms will double —
accepted and counted, dedup is a later gate).
"""

import json
import os
import re
import urllib.request

BASE = "http://127.0.0.1:8765"
REPO = os.path.dirname(os.path.abspath(__file__)) + "/.."
SKIP_DIRS = {"CapabilityTreeSitter", "target", ".git", "node_modules", "__pycache__"}
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


def md_files():
    out = []
    for dirpath, dirnames, filenames in os.walk(REPO):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in filenames:
            if fn.endswith(".md"):
                out.append(os.path.join(dirpath, fn))
    return sorted(out)


def chunks(path, cap=2000, minlen=200):
    try:
        raw = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return []
    parts = re.split(r"\n(?=#{1,4} )", raw)
    out = []
    for p in parts:
        p = p.strip()
        if len(p) < minlen:
            continue
        while len(p) > cap:
            cut = p.rfind("\n", 0, cap)
            cut = cut if cut > cap // 2 else cap
            out.append(p[:cut])
            p = p[cut:].strip()
        out.append(p)
    return out


def main():
    files = md_files()
    print(f"files: {len(files)}", flush=True)
    total = 0
    fails = []
    for f in files:
        rel = os.path.relpath(f, REPO)
        for c in chunks(f):
            try:
                r = call(
                    "map.ingest",
                    {
                        "text": f"[{rel}] {c}",
                        "salience": 0.5,
                        "provenance": "live-distill-02",
                        "taint": "none",
                    },
                )
            except Exception as e:
                fails.append((rel, repr(e)))
                continue
            if r["result"]["status"] != "ok":
                fails.append((rel, r["result"]))
                continue
            total += 1
        if total % 500 == 0:
            print(f"  ... {total} atoms", flush=True)
    print(f"ingested {total} atoms, fails {len(fails)}", flush=True)
    for rel, e in fails[:10]:
        print("FAIL", rel, str(e)[:120])
    # verify across knowledge areas
    for probe in [
        "toroidal phase encoder spiral-out",
        "Kuramoto mean-field",
        "judge gate",
        "id counter O(1)",
        "pre-registration A39",
        "tooling audit",
        "salience-stratified budget",
        "Novak",
    ]:
        r = call("map.query", {"qtext": probe, "k": 3})
        hits = r["result"]["hits"]
        top = hits[0]["text"][:70].replace("\n", " ") if hits else None
        print(f"probe {probe!r:38s} -> {top}")
    s = call("map.snapshot", {})
    print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"])
    print("LIVE-DISTILL-02 DONE")


if __name__ == "__main__":
    main()
