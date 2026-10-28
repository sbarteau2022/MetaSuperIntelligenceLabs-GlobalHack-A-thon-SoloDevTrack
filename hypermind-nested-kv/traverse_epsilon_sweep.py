#!/usr/bin/env python3
"""Item 3: sweep traversal epsilons on live atoms; preregister per-budget values."""
import json, urllib.request, itertools

BASE = "http://127.0.0.1:8765"
_tid = [0]
def call(tool, args):
    _tid[0] += 1
    body = json.dumps({"jsonrpc": "2.0", "id": _tid[0], "method": "tools/call",
                       "params": {"name": tool, "arguments": args}}).encode()
    req = urllib.request.Request(BASE, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as r:
        resp = json.loads(r.read())
    return json.loads(resp["result"]["content"][0]["text"])

h = call("map.health", {})
print("live atoms:", h["result"]["atoms"])
# sample pairs: top-k neighbors of several probes (realistic traversals)
probes = ["judge gate", "Kuramoto mean-field", "toroidal phase", "distillation v1",
          "pre-registration A39", "constraint network", "id counter", "Novak"]
pairs = []
for p in probes:
    r = call("map.query", {"qtext": p, "k": 4})
    ids = [x["id"] for x in r["result"]["hits"]]
    pairs += [(ids[i], ids[j]) for i in range(len(ids)) for j in range(i+1, len(ids))]
pairs = list(dict.fromkeys(pairs))[:40]
print(f"pairs: {len(pairs)}")
FORM_TOLS = [1e-12, 1e-9, 1e-6, 1e-3]
FRAME_EPSS = [1e-9, 1e-6, 1e-3, 1e-2]
for ft, fe in itertools.product(FORM_TOLS, FRAME_EPSS):
    disp = fx_form = fx_frame = 0
    for a, b in pairs:
        t = call("map.traverse", {"from_atom": a, "to_atom": b, "mlir_op": 26,
                                  "energy_budget": 100.0, "form_tol": ft, "frame_eps": fe})
        v = t["result"]["verdict"]
        if v == "dispatch": disp += 1
        else:
            rsn = t["result"]["reason"]
            if "form" in rsn: fx_form += 1
            if "frame" in rsn: fx_frame += 1
    print(f"form_tol={ft:<8} frame_eps={fe:<8} dispatch={disp:>3}/{len(pairs)} form-fail={fx_form:>3} frame-fail={fx_frame:>3}")
print("SWEEP DONE")
