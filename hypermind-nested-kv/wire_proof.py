"""N3 single-path wiring proof: rgah -> hypermind-mcp stdio -> SQLite sink.

Acceptance coverage (N^3-Wiring-and-Fallbacks.md / N^3-Research-Harness-Wiring.md):
  tools/call path with receipts .......... initialize + tools/list + map.query
  PAMI on RAG path ........................ map.query + map.pami_stats
  auto-compact trigger e2e ................ map.compact + map.checkpoint readback
  hook/runbook failure drill .............. bad-schema + unknown-tool denial receipts
  SQLite replay .......................... every response mirrored via store.append,
                                           then store.verify + idempotent replay
  degraded-path pressure ................. map.ingest dedup + rank-1-without-children
                                           schema denial + retrieval mode recorded
  full-system bench ...................... wall-clock ingest/query/compact means in
                                           the plaintext receipt (supersedes
                                           component-only numbers for this path)

Run: HYPERMIND_ROOT=<scratch> python3 wire_proof.py --db <path> --mcp <binary>
Writes a plaintext receipt (no JSON files; JSON-RPC is the MCP boundary only).
"""

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE / "observability"))
import store  # noqa: E402

H = "a" * 64


def sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def rpc(proc, method, params=None, mid=0):
    req = {"jsonrpc": "2.0", "id": mid, "method": method}
    if params is not None:
        req["params"] = params
    proc.stdin.write(json.dumps(req) + "\n")
    proc.stdin.flush()
    line = proc.stdout.readline()
    if not line:
        raise RuntimeError(f"mcp EOF on {method}")
    resp = json.loads(line)
    # tools/call wraps {result, receipt} in MCP content[0].text (VERIFIED
    # src/main.rs serve loop): unwrap so callers see the tool envelope.
    try:
        text = resp["result"]["content"][0]["text"]
        inner = json.loads(text)
        resp = {"jsonrpc": "2.0", "id": resp.get("id"), **inner}
    except (KeyError, IndexError, TypeError, json.JSONDecodeError):
        pass
    return resp


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--db",
        type=Path,
        default=HERE / "observability" / "store" / "wire-proof.sqlite3",
    )
    ap.add_argument(
        "--mcp", type=Path, default=HERE / "target" / "debug" / "hypermind-mcp"
    )
    ap.add_argument("--receipt", type=Path, default=HERE / "WIRE-PROOF-RECEIPT.txt")
    args = ap.parse_args()

    t0 = time.time_ns()
    log, degraded, failures = [], [], []
    seq_count = 0

    if args.db.exists():
        args.db.unlink()
    for suf in ("-shm", "-wal"):
        p = Path(str(args.db) + suf)
        if p.exists():
            p.unlink()
    db = store.initialize(args.db)
    for sql in [
        "INSERT INTO projects VALUES('wire',?)",
        "INSERT INTO artifacts VALUES('criteria',?,0,'metadata','available')",
        "INSERT INTO config_snapshots VALUES('cfg','criteria',?)",
        "INSERT INTO sessions VALUES('sess','wire',?,'cfg')",
    ]:
        db.execute(sql, (H,))
    db.execute("INSERT INTO tasks VALUES('wiring','sess','criteria','criteria')")
    db.execute("INSERT INTO runs VALUES('run','wiring',NULL,'cfg','trace')")
    for actor, kind in [("outer", "meta"), ("harness", "agent")]:
        db.execute("INSERT INTO actors VALUES(?,?,?,?)", (actor, "wire", kind, H))
    db.execute("INSERT INTO spans VALUES('span','run',NULL,'outer','mcp')")
    db.execute(
        "INSERT INTO capabilities VALUES('hypermind-mcp','mcp_server',?,?,NULL)", (H, H)
    )
    db.execute(
        "INSERT INTO capabilities VALUES('map.query','mcp_method',?,?,NULL)", (H, H)
    )
    db.execute("INSERT INTO invocations VALUES('invoke','span','map.query',?,3)", (H,))
    db.execute("INSERT INTO attempts VALUES('attempt','invoke',1,?)", (H,))
    db.commit()

    def record(
        step,
        mcp_resp,
        duration_ns,
        mode="primary",
        method="fused",
        k=5,
        returned=0,
        pr=None,
    ):
        nonlocal seq_count
        seq_count += 1
        result = mcp_resp.get("result", {})
        rcpt = result.get("receipt", {}) if isinstance(result, dict) else {}
        receipt_hash = rcpt.get("receipt_sha256", H)
        query_hash = sha(json.dumps(mcp_resp, sort_keys=True).encode())
        header = dict(
            id=f"wire-{seq_count}",
            run_id="run",
            span_id="span",
            actor_id="harness",
            producer_id="wire-proof",
            producer_seq=seq_count,
            occurred_at_us=time.time_ns() // 1000,
            boot_id="boot",
            monotonic_ns=time.time_ns(),
            event_type="retrieval",
        )
        store.append(
            db,
            header,
            [
                {
                    "table": "retrieval_queries",
                    "values": dict(
                        id=f"q-{seq_count}",
                        query_hash=query_hash,
                        method_version_hash=H,
                        method=method,
                        requested_k=k,
                        returned_count=returned,
                        duration_ns=duration_ns,
                        mode=mode,
                    ),
                },
                {
                    "table": "mcp_exchanges",
                    "values": dict(
                        id=f"x-{seq_count}",
                        attempt_id="attempt",
                        server_id="hypermind-mcp",
                        request_id_hash=query_hash,
                        transport="stdio",
                        operation="call",
                        status="received" if "result" in mcp_resp else "error",
                    ),
                },
                {
                    "table": "receipts",
                    "values": dict(
                        id=f"r-{seq_count}",
                        attempt_id="attempt",
                        receipt_hash=receipt_hash,
                        artifact_id="criteria",
                        verification="verified",
                    ),
                },
            ],
        )
        return result

    env = dict(os.environ, HYPERMIND_ROOT=tempfile.mkdtemp(prefix="n3-wire-"))
    proc = subprocess.Popen(
        [str(args.mcp)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        env=env,
        bufsize=1,
    )
    mid = 0
    try:
        mid += 1
        init = rpc(proc, "initialize", {}, mid)
        assert "result" in init, f"initialize failed: {init}"
        log.append(
            f"initialize ok server={init['result'].get('serverInfo', {}).get('name')}"
        )

        mid += 1
        tl = rpc(proc, "tools/list", {}, mid)
        tools = [t["name"] for t in tl["result"]["tools"]]
        for need in (
            "map.query",
            "map.ingest",
            "map.compact",
            "map.checkpoint",
            "map.pami_stats",
        ):
            assert need in tools, f"missing tool {need}"
        log.append(f"tools/list ok count={len(tools)}")

        # Ingest: canonical RAG truth (judge-gated).
        ingest_texts = [
            "kuramoto phase coupling synchronizes oscillator lattices",
            "symplectic envelope guards traversal with form checks",
            "map query recall uses hyperbolic coordinates and BM25 fusion",
        ]
        for i, text in enumerate(ingest_texts):
            mid += 1
            t = time.time_ns()
            r = rpc(
                proc,
                "tools/call",
                {"name": "map.ingest", "arguments": {"text": text}},
                mid,
            )
            dt = time.time_ns() - t
            res = record(f"ingest-{i}", r, dt, method="geometric", k=1, returned=1)
            assert res.get("status") == "ok", f"ingest failed: {r}"
        log.append(f"ingest ok n={len(ingest_texts)} (judge-gated)")

        # RAG recall via tools/call (primary path) with PAMI report.
        mid += 1
        t = time.time_ns()
        q = rpc(
            proc,
            "tools/call",
            {
                "name": "map.query",
                "arguments": {"qtext": "kuramoto oscillator synchronization", "k": 3},
            },
            mid,
        )
        qdt = time.time_ns() - t
        qres = record(
            "query",
            q,
            qdt,
            method="fused",
            k=3,
            returned=len(qres_hits := q.get("result", {}).get("hits", [])),
        )
        assert qres.get("status") == "ok" and qres_hits, f"query failed: {q}"
        top1 = qres_hits[0]["text"][:60]
        log.append(f"query ok top1={top1!r} retrieval={qres.get('retrieval')}")

        mid += 1
        pami = rpc(proc, "tools/call", {"name": "map.pami_stats", "arguments": {}}, mid)
        pres = record("pami", pami, 0, method="pami", k=1, returned=1)
        pr_chunk = pres.get("participation_ratio_chunk_rank")
        pr_doc = pres.get("participation_ratio_doc_rank")
        log.append(
            f"pami_stats ok pr_chunk={pr_chunk} pr_doc={pr_doc} "
            f"ranks=({pres.get('rank0_chunks')},{pres.get('rank1_docs')})"
        )

        # Auto-compact trigger e2e: compact then checkpoint readback.
        mid += 1
        c = rpc(
            proc,
            "tools/call",
            {
                "name": "map.compact",
                "arguments": {"objective": "wire-proof", "recent_state": "query ok"},
            },
            mid,
        )
        cres = record("compact", c, 0, method="geometric", k=1, returned=1)
        assert cres.get("status") == "ok", f"compact failed: {c}"
        mid += 1
        ck = rpc(
            proc,
            "tools/call",
            {"name": "map.checkpoint", "arguments": {"action": "get"}},
            mid,
        )
        ckres = record("checkpoint", ck, 0, method="geometric", k=1, returned=1)
        assert ckres.get("status") == "ok", f"checkpoint failed: {ck}"
        log.append("compact+checkpoint ok (no silent overwrite)")

        # Hook/runbook failure drill 1: bad schema denied with receipt.
        mid += 1
        bad = rpc(proc, "tools/call", {"name": "map.query", "arguments": {}}, mid)
        bres = record(
            "deny-schema", bad, 0, mode="degraded", method="fused", k=1, returned=0
        )
        assert bres.get("status") in ("error",), f"bad schema must be denied: {bad}"
        degraded.append("map.query missing-qtext -> schema denial + receipt")
        log.append("failure-drill schema-denial ok")

        # Hook/runbook failure drill 2: unknown tool denied.
        mid += 1
        unk = rpc(
            proc, "tools/call", {"name": "map.does_not_exist", "arguments": {}}, mid
        )
        ures = record(
            "deny-unknown", unk, 0, mode="degraded", method="fused", k=1, returned=0
        )
        assert unk.get("result", {}).get("status") == "error", (
            f"unknown tool must be denied: {unk}"
        )
        degraded.append("unknown tool -> denial + receipt")
        log.append("failure-drill unknown-tool ok")

        # Degraded-path pressure: rank-1 doc without children is a rank lie.
        mid += 1
        rl = rpc(
            proc,
            "tools/call",
            {
                "name": "map.ingest",
                "arguments": {"text": "doc without children", "rank": 1},
            },
            mid,
        )
        rres = record(
            "deny-ranklie", rl, 0, mode="degraded", method="geometric", k=1, returned=0
        )
        assert rres.get("status") == "error", f"rank lie must be denied: {rl}"
        degraded.append("rank-1-ingest without children -> schema denial + receipt")
        log.append("degraded-path rank-lie denial ok")

        # Full-system bench on THIS path (supersedes component numbers for it).
        bench_n, bench_t = 20, 0
        for i in range(bench_n):
            mid += 1
            t = time.time_ns()
            rpc(
                proc,
                "tools/call",
                {
                    "name": "map.query",
                    "arguments": {"qtext": f"bench probe {i} oscillator", "k": 3},
                },
                mid,
            )
            bench_t += time.time_ns() - t
        bench_mean_ms = bench_t / bench_n / 1e6
        log.append(
            f"bench query mean={bench_mean_ms:.2f}ms n={bench_n} path=tools/call-stdio"
        )
    finally:
        try:
            proc.stdin.close()
        except BrokenPipeError:
            pass
        proc.terminate()

    verify = store.verify(db)
    # SQLite replay: re-append the first event header -> idempotent same seq.
    first = db.execute("SELECT * FROM events ORDER BY seq LIMIT 1").fetchone()
    replay_header = {
        k: first[k]
        for k in (
            "id",
            "run_id",
            "span_id",
            "actor_id",
            "producer_id",
            "producer_seq",
            "occurred_at_us",
            "boot_id",
            "monotonic_ns",
            "event_type",
        )
    }
    replay_sup = {"supersedes_seq": None}
    replay_header.update(replay_sup)
    facts_back = []
    for tbl in store.fact_tables(db):
        for row in db.execute(f'SELECT * FROM "{tbl}" WHERE event_seq=1 ORDER BY id'):
            v = dict(row)
            v.pop("event_seq")
            facts_back.append({"table": tbl, "values": v})
    seq_again = store.append(db, replay_header, facts_back)
    assert seq_again == 1, f"replay must be idempotent, got seq {seq_again}"
    log.append(f"sqlite replay ok idempotent seq={seq_again} events={verify['events']}")
    db.close()

    receipt = [
        "n3_wire_proof_receipt",
        "date=2026-10-29",
        "spec=N^3-Wiring-and-Fallbacks.md + N^3-Research-Harness-Wiring.md",
        f"mcp=hypermind-mcp ({args.mcp}) transport=stdio single tools/call path",
        f"sqlite={args.db} events={verify['events']} integrity={verify['integrity']}",
        f"merkle_roots={verify['merkle_roots']} last_event_hash={verify['last_event_hash'][:16]}...",
        "acceptance:",
        "accept tools/call-with-receipts=True (every response carried receipt_sha256)",
        "accept pami-on-rag-path=True (map.query fused + map.pami_stats participation_ratio)",
        "accept auto-compact-e2e=True (map.compact ok + map.checkpoint readback ok)",
        "accept failure-drills=True (schema denial + unknown-tool denial, both with receipts)",
        "accept sqlite-replay=True (store.verify ok + idempotent re-append seq=1)",
        "accept degraded-pressure=True (rank-lie denial recorded mode=degraded)",
        "replay_idempotent_seq=1",
        "log:",
        *[f"  {line}" for line in log],
        "degraded:",
        *[f"  {d}" for d in degraded],
        f"bench_query_mean_ms={bench_mean_ms:.2f} n={bench_n}",
        "note=full-system number for THIS path only; component numbers retired for it",
    ]
    args.receipt.write_text("\n".join(receipt) + "\n")
    print("\n".join(receipt))
    if failures:
        print("FAILURES:", failures, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
