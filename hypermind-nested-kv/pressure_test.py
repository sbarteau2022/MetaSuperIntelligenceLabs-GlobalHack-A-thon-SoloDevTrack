#!/usr/bin/env python3
"""Pressure / correctness harness for hypermind-mcp (hypermind-nested-kv)."""

import json
import os
import subprocess
import sys
import tempfile
import time

BIN = os.path.join(os.path.dirname(__file__), "target", "debug", "hypermind-mcp")
ROOT = tempfile.mkdtemp(prefix="hypermind-pt-")
os.environ["HYPERMIND_ROOT"] = ROOT
proc = subprocess.Popen(
    [BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1
)


def send(req):
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    # read one line
    line = proc.stdout.readline()
    return json.loads(line) if line else None


def call(tool, args=None, tid=1):
    resp = send(
        {
            "jsonrpc": "2.0",
            "id": tid,
            "method": "tools/call",
            "params": {"name": tool, "arguments": args or {}},
        }
    )
    text = resp["result"]["content"][0]["text"]
    return json.loads(text)


# init
print("init:", send({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {}}))

# 1) ingest N atoms with distinct texts
N = 200
t0 = time.time()
ids = []
for i in range(N):
    r = call(
        "map.ingest",
        {
            "text": f"atom-{i} retrieval target payload number {i} with unique marker {i * i}",
            "salience": 0.5,
            "provenance": "pressure-test",
            "taint": "none",
        },
    )
    assert r["result"]["status"] == "ok", r
    ids.append(r["result"]["atom_id"])
ingest_ms = (time.time() - t0) * 1000
print(f"ingested {N} atoms in {ingest_ms:.1f} ms ({N / ingest_ms * 1000:.0f} atoms/s)")

# 2) query and check top hit contains the marker
t0 = time.time()
target_text = "atom-42 retrieval target payload number 42 with unique marker 1764"
r = call("map.query", {"qtext": target_text, "k": 5})
q_ms = (time.time() - t0) * 1000
hits = r["result"]["hits"]
print(f"query {q_ms:.1f} ms; top hit text = {hits[0]['text'][:60] if hits else None}")
assert any("42" in h["text"] for h in hits[:3]), "expected atom-42 in top-3"
assert "receipt" in r

# 3) snapshot
r = call("map.snapshot", {})
snap_digest = r["result"]["digest"]
print("snapshot digest:", snap_digest, "atoms:", r["result"]["atoms"])
assert r["result"]["status"] == "ok"

# 4) recover round-trip
r2 = call("map.recover", {})
print("recover:", r2["result"])
assert r2["result"]["status"] == "ok"
assert r2["result"]["atoms"] == N

# 5) health
h = call("map.health", {})
print("health:", h["result"])

# 6) checkpoint
cp = call(
    "map.compact",
    {"objective": "pressure-test objective", "recent_state": "state-after-ingest"},
)
print("compact:", cp["result"]["status"])
assert cp["receipt"]["receipt_sha256"]

# 7) export passport
pp = call("map.export_passport", {})
print(
    "passport sha:",
    pp["result"]["passport_sha256"],
    "atoms:",
    pp["result"]["passport"]["atom_count"],
)
assert pp["result"]["passport"]["atom_count"] == N

# 8) sustained write pressure: ingest 500 more
t0 = time.time()
for i in range(500):
    call("map.ingest", {"text": f"pressure-{i} bulk load entry {i}", "salience": 0.3})
bulk_ms = (time.time() - t0) * 1000
print(f"bulk 500 in {bulk_ms:.1f} ms ({500 / bulk_ms * 1000:.0f} atoms/s)")

# 9) query under load
t0 = time.time()
r = call("map.query", {"qtext": "pressure-250 bulk load", "k": 3})
print(
    f"query under load {(time.time() - t0) * 1000:.1f} ms; top={r['result']['hits'][0]['text'][:40] if r['result']['hits'] else None}"
)

send({"jsonrpc": "2.0", "id": 999, "method": "shutdown", "params": {}})
proc.wait(timeout=10)
print("PASS")
print("ROOT:", ROOT)
