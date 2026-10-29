#!/usr/bin/env python3
"""Proof 2: the model-agnostic harness walk (goal ce19e81a).

Simulates ANY model in the harness (no model-specific code anywhere — only
map tools). The walk: reason over the map (euclid + lexical), store each
step's conclusion in the nested KV tree, ingest the walk trace itself as an
atom. Proves: (a) the mind is persistent (KV + atoms survive across steps),
(b) continuous (each step reads prior steps' KV), (c) model-agnostic (the
harness speaks only map tools; swap the reasoner, nothing changes).
"""
import json, urllib.request

BASE = "http://127.0.0.1:8765"
_tid = [0]
def call(tool, args):
    _tid[0] += 1
    body = json.dumps({"jsonrpc": "2.0", "id": _tid[0], "method": "tools/call",
                       "params": {"name": tool, "arguments": args}}).encode()
    req = urllib.request.Request(BASE, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        resp = json.loads(r.read())
    return json.loads(resp["result"]["content"][0]["text"])

def reason_step(step, question, prior_keys):
    # 1. Read prior steps' conclusions from the KV tree (continuity).
    priors = {}
    for k in prior_keys:
        f = call("map.kv_fetch", {"key": k})
        assert f["result"]["status"] == "hit", f"walk broken: prior {k} missing: {f}"
        priors[k] = f["result"]["value"]
    # 2. Reason over the map: euclid (flat) + lexical (docs).
    e = call("map.read_euclid", {"qtext": question, "k": 3, "dim": 9})
    l = call("map.lexical", {"qtext": question, "k": 2})
    # 3. Conclude (the reasoner — swappable; here: top evidence synthesis).
    top_e = e["result"]["hits"][0]
    top_l = l["result"]["hits"][0] if l["result"]["hits"] else {"text": "none", "bm25": 0}
    conclusion = (f"step{step} re {question[:40]}: euclid[{top_e['id']}] "
                  f"lex[{top_l.get('bm25',0):.1f}] priors={len(priors)}")
    # 4. Store conclusion in the nested KV tree (persistent memory).
    key = f"reasoning/walk/step-{step}"
    s = call("map.kv_store", {"key": key, "value": conclusion,
                              "produced_by": [top_e["id"]]})
    assert s["result"]["status"] == "stored", s
    # 5. Ingest the walk trace itself as an atom (the map learns the walk).
    a = call("map.ingest", {"text": f"walk trace step {step}: {question} -> {conclusion}",
                            "salience": 0.6, "provenance": "multimodel-walk"})
    assert a["result"]["status"] in ("ok",), a
    atom_id = a["result"].get("atom_id", a["result"].get("coalesced") and "coalesced")
    print(f"  step {step}: kv={key} atom={atom_id} priors_read={len(priors)}", flush=True)
    return key

def main():
    print("== multimodel walk: 5 steps, any-model harness ==", flush=True)
    keys = []
    questions = [
        "symplectic traverse fail-explained remedy",
        "hunger ledger priority distill order",
        "salience normalization shell distance",
        "judge gate mint-on-pass entropy floor",
        "nested KV cache hit accumulation",
    ]
    # Seed step 0 so step 1 has continuity.
    s = call("map.kv_store", {"key": "reasoning/walk/step-0",
                              "value": "walk start: multimodel mind online",
                              "produced_by": []})
    assert s["result"]["status"] == "stored"
    keys.append("reasoning/walk/step-0")
    for i, q in enumerate(questions, 1):
        keys.append(reason_step(i, q, keys))
    # Verify persistence: every step's KV readable, every trace ingested.
    print("== persistence audit ==", flush=True)
    for k in keys:
        f = call("map.kv_fetch", {"key": k})
        assert f["result"]["status"] == "hit", k
    print(f"  all {len(keys)} walk KV entries readable", flush=True)
    r = call("map.lexical", {"qtext": "walk trace step multimodel", "k": 10})
    traces = [h for h in r["result"]["hits"] if "walk trace step" in h["text"]]
    print(f"  walk trace atoms ingested: {len(traces)}", flush=True)
    assert len(traces) >= 5, "walk not ingested in real time"
    s = call("map.snapshot", {})
    print(f"snapshot {s['result']['digest'][:16]} atoms={s['result']['atoms']}", flush=True)
    print("MULTIMODEL-WALK DONE", flush=True)

if __name__ == "__main__":
    main()
