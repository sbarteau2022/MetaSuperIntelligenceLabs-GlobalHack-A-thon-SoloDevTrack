#!/usr/bin/env python3
"""Distill ceiling: single-pass near-lossless capacity of hypermind-mcp.

Single pass = one map.ingest per payload, no re-reads, no repair.
Near-lossless = exact-text query at matched salience returns the same atom top-1.
Capability = map.query top-k hit + map.attend overlap_with_exact_topk.

Writes nothing outside the repo: temp roots live under target/ceiling-tmp-*.
"""

import json
import os
import random
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
BIN = os.path.join(HERE, "target", "debug", "hypermind-mcp")
os.makedirs(os.path.join(HERE, "target"), exist_ok=True)


def spawn(root):
    env = dict(os.environ, HYPERMIND_ROOT=root)
    proc = subprocess.Popen(
        [BIN],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        bufsize=1,
        env=env,
    )
    return proc


def send(proc, req):
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    return json.loads(line) if line else None


def call(proc, tool, args=None, tid=1):
    resp = send(
        proc,
        {
            "jsonrpc": "2.0",
            "id": tid,
            "method": "tools/call",
            "params": {"name": tool, "arguments": args or {}},
        },
    )
    text = resp["result"]["content"][0]["text"]
    return json.loads(text)


def payload(i):
    # unique, fixed-width, collision-hostile: id, square, hashed marker
    return f"distill packet {i:05d} traj arc marker {i * i}::{(i * 2654435761) % 2**32} payload end"


def run_N(N, nq=100, salience=0.5, seed=7):
    root = tempfile.mkdtemp(prefix=f"ceiling-{N}-", dir=os.path.join(HERE, "target"))
    proc = spawn(root)
    send(proc, {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}})
    t0 = time.time()
    ids = []
    for i in range(N):
        r = call(
            proc,
            "map.ingest",
            {
                "text": payload(i),
                "salience": salience,
                "provenance": "ceiling",
                "taint": "none",
            },
            tid=1000000 + i,
        )
        assert r["result"]["status"] == "ok", r
        ids.append(r["result"]["atom_id"])
    ingest_s = time.time() - t0
    # snapshot + sizes
    r = call(proc, "map.snapshot", {}, tid=1)
    assert r["result"]["status"] == "ok", r
    digest = r["result"]["digest"]
    snap_path = os.path.join(root, ".hypermind", "map.snapshot.bin")
    snap_bytes = os.path.getsize(snap_path) if os.path.exists(snap_path) else -1
    mind_path = os.path.join(root, ".hypermind", "db", "mind.jsonl")
    mind_bytes = os.path.getsize(mind_path) if os.path.exists(mind_path) else -1
    # exact queries at matched salience
    rng = random.Random(seed)
    sample = rng.sample(range(N), min(nq, N))
    top1 = 0
    top3 = 0
    margins = []
    t0 = time.time()
    attend_overlap_sum = 0
    attend_n = 0
    for idx in sample:
        q = payload(idx)
        r = call(proc, "map.query", {"qtext": q, "k": 5}, tid=2)
        hits = r["result"]["hits"]
        hids = [h["id"] for h in hits]
        want = ids[idx]
        if hids and hids[0] == want:
            top1 += 1
        if want in hids[:3]:
            top3 += 1
        if len(hits) >= 2:
            margins.append(hits[0]["score"] - hits[1]["score"])
    q_ms = (time.time() - t0) * 1000 / max(1, len(sample))
    # attend overlap on a subsample (exact path computed inside)
    for idx in sample[:20]:
        r = call(proc, "map.attend", {"qtext": payload(idx)}, tid=3)
        attend_overlap_sum += r["result"].get("overlap_with_exact_topk", 0)
        attend_n += 1
    # recover round-trip
    r2 = call(proc, "map.recover", {}, tid=4)
    rec_ok = r2["result"]["status"] == "ok" and r2["result"]["atoms"] == N
    h = call(proc, "map.health", {}, tid=5)
    send(proc, {"jsonrpc": "2.0", "id": 999, "method": "shutdown", "params": {}})
    try:
        proc.wait(timeout=10)
    except Exception:
        proc.kill()
    return {
        "N": N,
        "ingest_s": round(ingest_s, 2),
        "atoms_per_s": round(N / ingest_s, 1),
        "top1": top1,
        "top3": top3,
        "nq": len(sample),
        "top1_rate": round(top1 / max(1, len(sample)), 4),
        "top3_rate": round(top3 / max(1, len(sample)), 4),
        "q_ms": round(q_ms, 3),
        "margin_min": round(min(margins), 6) if margins else None,
        "margin_med": round(sorted(margins)[len(margins) // 2], 6) if margins else None,
        "attend_overlap_mean": round(attend_overlap_sum / max(1, attend_n), 3),
        "snap_bytes": snap_bytes,
        "mind_bytes": mind_bytes,
        "bytes_per_atom_snap": round(snap_bytes / N, 1) if snap_bytes > 0 else None,
        "digest": digest[:16],
        "recover_ok": rec_ok,
        "health_atoms": h["result"]["atoms"],
    }


def cross_salience(N=500):
    """Half atoms at 0.9, half at 0.1; query uses qs=0.5 always -> measures radius coupling loss."""
    root = tempfile.mkdtemp(prefix="ceiling-xsal-", dir=os.path.join(HERE, "target"))
    proc = spawn(root)
    send(proc, {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}})
    ids_hi = []
    ids_lo = []
    for i in range(N):
        s = 0.9 if i % 2 == 0 else 0.1
        r = call(
            proc, "map.ingest", {"text": payload(i), "salience": s}, tid=2000000 + i
        )
        (ids_hi if s == 0.9 else ids_lo).append((i, r["result"]["atom_id"]))
    rng = random.Random(11)
    res = {}
    for name, lst in (("hi09", ids_hi), ("lo01", ids_lo)):
        samp = rng.sample(lst, min(50, len(lst)))
        t1 = t3 = 0
        for i, want in samp:
            r = call(proc, "map.query", {"qtext": payload(i), "k": 5}, tid=3)
            hids = [h["id"] for h in r["result"]["hits"]]
            if hids and hids[0] == want:
                t1 += 1
            if want in hids[:3]:
                t3 += 1
        res[name] = {
            "top1": round(t1 / max(1, len(samp)), 4),
            "top3": round(t3 / max(1, len(samp)), 4),
            "n": len(samp),
        }
    send(proc, {"jsonrpc": "2.0", "id": 999, "method": "shutdown", "params": {}})
    try:
        proc.wait(timeout=10)
    except Exception:
        proc.kill()
    return res


def near_dup():
    """10 near-duplicate texts differing by one trailing token; count rank inversions."""
    root = tempfile.mkdtemp(prefix="ceiling-neardup-", dir=os.path.join(HERE, "target"))
    proc = spawn(root)
    send(proc, {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}})
    base = "near duplicate retrieval probe payload alpha beta gamma"
    texts = [f"{base} variant {i}" for i in range(10)]
    ids = []
    for t in texts:
        r = call(
            proc, "map.ingest", {"text": t, "salience": 0.5}, tid=3000000 + len(ids)
        )
        ids.append(r["result"]["atom_id"])
    exact = 0
    for t, want in zip(texts, ids):
        r = call(proc, "map.query", {"qtext": t, "k": 3}, tid=4)
        hids = [h["id"] for h in r["result"]["hits"]]
        if hids and hids[0] == want:
            exact += 1
    send(proc, {"jsonrpc": "2.0", "id": 999, "method": "shutdown", "params": {}})
    try:
        proc.wait(timeout=10)
    except Exception:
        proc.kill()
    return {"exact_top1": exact, "n": len(texts)}


if __name__ == "__main__":
    sweep = [int(x) for x in sys.argv[1:]] or [200, 1000, 2000]
    out = []
    for N in sweep:
        print(f"--- N={N} ---", flush=True)
        r = run_N(N)
        print(json.dumps(r, indent=1), flush=True)
        out.append(r)
        if r["top1_rate"] < 0.95:
            print(f"STOP: top1 {r['top1_rate']} < 0.95 at N={N}", flush=True)
            break
    print("--- cross-salience ---", flush=True)
    xs = cross_salience()
    print(json.dumps(xs, indent=1), flush=True)
    print("--- near-dup ---", flush=True)
    nd = near_dup()
    print(json.dumps(nd, indent=1), flush=True)
    print(json.dumps({"sweep": out, "cross_salience": xs, "near_dup": nd}, indent=1))
