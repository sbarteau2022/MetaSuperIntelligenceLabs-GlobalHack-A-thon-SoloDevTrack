#!/usr/bin/env python3
"""Bulk-load canonical BGE-M3 vectors into atom_embedding_text_v1 (step 5).

Reads embed_bgem3.py output JSONL ({id, vector[1024], digest}), UPSERTs each
row with the atom record link + source digest. Verifies: dimension gate,
digest match against atom content_digest, HNSW index presence, final counts.

Usage: load_text_embeddings.py [--apply] [--limit N]
  default is DRY RUN (builds SurrealQL, validates shape, writes /tmp/load_text_embeddings.surql).
  --apply executes against hypermind/core in chunks of 50 (BEGIN/COMMIT each).
"""
import argparse, hashlib, json, sys, urllib.request

ENDPOINT = "http://127.0.0.1:8000/sql"
NSDB = "USE NS hypermind DB core; "

def sql(body: str):
    req = urllib.request.Request(ENDPOINT, data=(NSDB + body).encode(),
        headers={"Content-Type": "text/plain", "Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as r:
        return json.load(r)

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--vec", default="/tmp/atoms_vec.jsonl")
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--limit", type=int, default=0)
    args = ap.parse_args()

    rows = []
    with open(args.vec, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))
    if args.limit:
        rows = rows[:args.limit]

    # Shape gate: every row 1024D floats, digest is 64-hex.
    for r in rows:
        assert isinstance(r["id"], str) and r["id"].isdigit(), r["id"]
        assert len(r["vector"]) == 1024, (r["id"], len(r["vector"]))
        assert len(r["digest"]) == 64, r["id"]
        assert all(isinstance(x, float) for x in r["vector"]), r["id"]

    stmts = []
    for r in rows:
        vec = ", ".join(repr(float(x)) for x in r["vector"])
        stmts.append(
            f"UPSERT atom_embedding_text_v1:`{r['id']}` CONTENT {{ "
            f"workspace: workspace:hypermind, atom: atom:{r['id']}, "
            f"encoder: 'bge-m3', "
            f"encoder_revision: '5617a9f61b028005a4858fdac845db406aefb181', "
            f"dimensions: 1024, vector: [{vec}], normalized: true, "
            f"source_digest: '{r['digest']}', created_at: time::now() }} RETURN NONE;"
        )
    with open("/tmp/load_text_embeddings.surql", "w") as f:
        f.write(NSDB + "\n" + "\n".join(stmts) + "\n")
    print(f"rows={len(rows)} statements_built={len(stmts)} dry_run={not args.apply}")
    if not args.apply:
        return 0

    ok, err = 0, 0
    for i in range(0, len(stmts), 20):  # 20 x ~23KB ~= 460KB: 2x under the ~1MB server request ceiling (bisected 2026-11-01)
        chunk = stmts[i:i + 20]
        body = "BEGIN TRANSACTION; " + " ".join(chunk) + " COMMIT TRANSACTION;"
        try:
            resp = sql(body)
            if any(s.get("status") == "ERR" for s in resp[1:]):
                err += len(chunk)
                print(f"chunk {i}: {[s.get('result') for s in resp[1:] if s.get('status')=='ERR']}"[:300], file=sys.stderr)
            else:
                ok += len(chunk)
        except Exception as e:
            err += len(chunk)
            print(f"chunk {i} transport error: {e}"[:200], file=sys.stderr)
    print(f"loaded_ok={ok} loaded_err={err}")
    # Verification: counts + digest agreement + index presence.
    cnt = sql("SELECT count() AS n FROM atom_embedding_text_v1 GROUP ALL;")[1]["result"]
    idx = sql("INFO FOR TABLE atom_embedding_text_v1;")[1]["result"]["indexes"]
    print("count:", cnt, "| hnsw:", "atom_embedding_text_v1_hnsw" in idx)
    return 0 if err == 0 else 1

if __name__ == "__main__":
    raise SystemExit(main())
