#!/usr/bin/env python3
"""Document-heavy flood part 2: code files (.py/.rs, no vendor/target).
Chunked by function/class/def boundaries (~2KB cap), judge-gated, fail-open
counted (code has symbols the judge may reject — those are reported, not lost).
Provenance carries file + chunk kind so lexical retrieval cites precisely."""

import json
import os
import re
import urllib.request

BASE = "http://127.0.0.1:8765"
REPO = os.path.dirname(os.path.abspath(__file__)) + "/.."
SKIP = {
    "CapabilityTreeSitter",
    "target",
    ".git",
    "node_modules",
    "__pycache__",
    "vendor",
}
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


def code_chunks(path, cap=2000, minlen=150):
    try:
        raw = open(path, encoding="utf-8", errors="replace").read()
    except OSError:
        return []
    # split on top-level def/fn/class/mod markers, keep meaty pieces
    parts = re.split(r"\n(?=(?:def |fn |class |pub fn |impl |mod |#\[))", raw)
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
    files = []
    for dirpath, dirnames, filenames in os.walk(REPO):
        dirnames[:] = [d for d in dirnames if d not in SKIP]
        for fn in filenames:
            if fn.endswith((".py", ".rs")):
                files.append(os.path.join(dirpath, fn))
    files.sort()
    print(f"code files: {len(files)}", flush=True)
    minted = coalesced = rejected = 0
    for f in files:
        rel = os.path.relpath(f, REPO)
        for c in code_chunks(f):
            try:
                r = call(
                    "map.ingest",
                    {
                        "text": f"[{rel}] {c[:1900]}",
                        "salience": 0.6,
                        "provenance": "doc-heavy-code",
                        "taint": "none",
                    },
                )
            except Exception as e:
                rejected += 1
                continue
            st = r["result"]["status"]
            if st == "ok" and r["result"].get("coalesced"):
                coalesced += 1
            elif st == "ok":
                minted += 1
            else:
                rejected += 1
        if (minted + coalesced + rejected) % 200 == 0:
            print(f"  ... m={minted} c={coalesced} r={rejected}", flush=True)
    print(f"minted={minted} coalesced={coalesced} rejected={rejected}", flush=True)
    for probe in [
        "symplectic_from_triple momentum",
        "map.lexical BM25 full corpus",
        "judge_text entropy floor",
        "calibrate_idf discriminative terms",
        "traverse form_residual triangle",
    ]:
        r = call("map.lexical", {"qtext": probe, "k": 1})
        h = r["result"]["hits"][0] if r["result"]["hits"] else {}
        print(
            f"probe {probe[:40]:40s} -> {(h.get('text', 'NONE'))[:65].replace(chr(10), ' ')} (bm25={h.get('bm25', 0):.1f})",
            flush=True,
        )
    s = call("map.snapshot", {})
    print("snapshot", s["result"]["digest"][:16], s["result"]["atoms"], flush=True)
    print("DOC-HEAVY-CODE DONE", flush=True)


if __name__ == "__main__":
    main()
