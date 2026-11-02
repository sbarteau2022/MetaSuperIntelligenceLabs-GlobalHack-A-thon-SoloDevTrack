#!/usr/bin/env python3
"""Token-efficient tool_router for Zed threads — first-round smoke test.

Zed threads, per Stewart 2026-10-27, need ONLY:
  1. access to the RAG store, and
  2. deposit sessions in the HyperMap.
Everything else is lazy / search-only, never preloaded.

Exposed MCP surface: exactly 3 tools.
  route(task)            -> minimal tool/skill list + token estimate + reason
  search_catalog(kind, keyword) -> one-line summaries only, never full definitions
  execute(name, args)    -> runs one allowlisted tool, truncated output only

Token math: tokens ~= ceil(chars / 4). This is a rough, documented estimate,
not a provider token count. It is used only to enforce the cap, not to bill.
  MAX_INITIAL_TOKENS = 1500  (enforced in route(); route() never returns more)
  Router upfront (3 tool schemas): ~700 tokens (measured in smoke test)
  Default route (4 definitions): ~600-700 tokens (measured in smoke test)
  Full catalog upfront, for comparison:
    10 HyperMind schemas + 869-entry skills catalog (~72 KB) + all MCP
    servers = >20,000 tokens. The router saves >95% of that upfront.

RAG route (reused, not invented):
  POST {ATOMA_RAG_URL, default http://127.0.0.1:8787/rag/query}
  body {"text": ..., "collection": "papers"} — the AtomaApp backend route
  in backend/src/server.rs ("POST", "/rag/query"), which calls
  RagClient.query_collection() over ~/Desktop/AtomaDB-RAG-PAMI/data/library-db/.
  Collections allowed: papers, research, knowledge, models, platforms,
  build, engineering. If the backend is down, execute() returns an error
  receipt, never fake hits.

HyperMap route (reused, not invented):
  Spawns hypermind-mcp (stdio JSON-RPC, 10 map.* tools) per execute() call,
  with HYPERMIND_ROOT from the router environment. Default, when Zed runs
  it, is a router-owned map at ~/.config/zed/tool_router/hypermind — NOT
  the production map. Point HYPERMIND_ROOT at a production map only by
  explicit env change. No hooks/tails are added to HyperMind: hooks/tails
  belong in N^3, per Stewart 2026-10-27.

Receipts (taps doctrine):
  Appended to tool_router_receipts.jsonl next to this file (or, in Zed,
  ~/.config/zed/tool_router/tool_router_receipts.jsonl).
  Each line: ts, task_hash, arg_keys, args_sha, selected, reason,
  estimated_tokens, executed_tool, outcome, output_sha, output_chars.
  NEVER: task text, argument values, prompts, or full outputs.

This file is stdlib-only, like the other servers in ~/.config/zed/mcp/.
It is NOT part of the AtomaApp harness serving path: no Python worker
is added to the backend. Zed runs it as a context_server.
"""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

SERVER_NAME = "atoma-tool-router"
MAX_INITIAL_TOKENS = 1500
MAX_OUTPUT_CHARS = 4000
MAX_SEARCH_RESULTS = 10

RAG_URL = os.environ.get("ATOMA_RAG_URL", "http://127.0.0.1:8787/rag/query")
HYPERMIND_BIN = os.environ.get(
    "HYPERMIND_BIN",
    "/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/hypermind-nested-kv/target/release/hypermind-mcp",
)
HYPERMIND_ROOT = os.environ.get(
    "HYPERMIND_ROOT",
    str(Path.home() / ".config" / "zed" / "tool_router" / "hypermind"),
)
SKILLS_CATALOG = Path(
    os.environ.get(
        "ATOMA_SKILLS_CATALOG",
        "/Users/stewartbarteau/Desktop/AtomaApp/docs/skills-catalog.md",
    )
)
RECEIPT_FILE = Path(__file__).resolve().parent / "tool_router_receipts.jsonl"
LANE_MINT_SCRIPT = Path(
    "/Users/stewartbarteau/Desktop/GitHub/MetaSuperIntelligenceLabs-GlobalHack-A-thon-SoloDevTrack/hypermind-nested-kv/lane_mint.py"
)


def est_tokens(text: str) -> int:
    """Rough token estimate: ceil(chars / 4). Documented, not a provider count."""
    return (len(text) + 3) // 4


def sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", errors="ignore")).hexdigest()[:16]


def log_receipt(task: str, selected, reason: str, est: int, executed: str, outcome: str, args: dict, output: str) -> None:
    try:
        rec = {
            "ts": int(time.time()),
            "task_hash": sha(task),
            "arg_keys": sorted(args.keys()),
            "args_sha": sha(json.dumps(args, sort_keys=True)),
            "selected": selected,
            "reason": reason,
            "estimated_tokens": est,
            "executed_tool": executed,
            "outcome": outcome,
            "output_sha": sha(output),
            "output_chars": len(output),
        }
        with open(RECEIPT_FILE, "a", encoding="utf-8") as f:
            f.write(json.dumps(rec, sort_keys=True) + "\n")
    except Exception:
        pass


# --- Catalog index: name, one-line summary, trigger keywords, full schema ---
# Full schemas are returned ONLY by route() for the selected few, never by
# tools/list and never by search_catalog(). Search returns summaries only.
CATALOG = {
    "rag.query": {
        "summary": "RAG query over the library store (BM25 + dense, AtomaApp /rag/query).",
        "triggers": ["rag", "paper", "papers", "research", "find", "lookup", "search", "knowledge", "cite", "reference"],
        "schema": {
            "type": "object",
            "properties": {
                "text": {"type": "string"},
                "collection": {"type": "string", "enum": ["papers", "research", "knowledge", "models", "platforms", "build", "engineering"]},
            },
            "required": ["text"],
        },
    },
    "hypermap.query": {
        "summary": "HyperMap k-NN recall in hyperbolic H3 x T2 (map.query).",
        "triggers": ["recall", "remember", "hypermap", "hyper", "map", "previous", "session", "trajectory"],
        "schema": {
            "type": "object",
            "properties": {"qtext": {"type": "string"}, "k": {"type": "integer"}},
            "required": ["qtext"],
        },
    },
    "hypermap.attend": {
        "summary": "HyperMap nested-KV attention, depth-gated recall (map.attend).",
        "triggers": ["attend", "depth", "nested", "attention", "deep recall"],
        "schema": {
            "type": "object",
            "properties": {"qtext": {"type": "string"}, "k": {"type": "integer"}},
            "required": ["qtext"],
        },
    },
    "hypermap.ingest": {
        "summary": "Deposit a session / walk into the HyperMap (map.ingest).",
        "triggers": ["deposit", "save", "store", "ingest", "session", "walk", "trajectory", "lock in"],
        "schema": {
            "type": "object",
            "properties": {
                "text": {"type": "string"},
                "provenance": {"type": "string"},
                "salience": {"type": "number"},
            },
            "required": ["text"],
        },
    },
    "hypermap.health": {
        "summary": "HyperMap counts, capacity, geometry status (map.health).",
        "triggers": ["health", "status", "counts", "capacity"],
        "schema": {"type": "object", "properties": {}},
    },
    "lane.mint": {
        "summary": "Mint exactly one gated atom from a completed Zed-thread lane run into the live HyperMap (:8765). Only if verdict=passed; otherwise held, never minted. Text-action level only, not weight-level.",
        "triggers": ["mint", "lane run", "lock in run", "trajectory mint", "mint run"],
        "schema": {
            "type": "object",
            "properties": {
                "run_id": {"type": "string"},
                "lane": {"type": "string"},
                "verdict": {"type": "string"},
                "output_summary": {"type": "string"},
                "prompt": {"type": "string"},
                "steps": {"type": "array", "items": {"type": "string"}},
                "tool_calls": {"type": "array", "items": {"type": "string"}},
            },
            "required": ["run_id", "lane", "verdict"],
        },
    },
    # Lazy-only: searchable, never in the default route.
    "hypermap.create": {"summary": "Create/initialize a HyperMap under the workspace root (map.create).", "triggers": ["create map", "init map"], "schema": {"type": "object", "properties": {"subpath": {"type": "string"}}}},
    "hypermap.compact": {"summary": "Anti-lobotomy continuity checkpoint, no destructive rewrite (map.compact).", "triggers": ["compact", "checkpoint"], "schema": {"type": "object", "properties": {"objective": {"type": "string"}, "recent_state": {"type": "string"}}}},
    "hypermap.checkpoint": {"summary": "Read or write the continuity checkpoint (map.checkpoint).", "triggers": ["checkpoint"], "schema": {"type": "object", "properties": {"action": {"type": "string", "enum": ["get", "set"]}, "note": {"type": "string"}}}},
    "hypermap.snapshot": {"summary": "Atomic snapshot + WAL append, returns sha256 (map.snapshot).", "triggers": ["snapshot"], "schema": {"type": "object", "properties": {}}},
    "hypermap.recover": {"summary": "Verify snapshot hash and replay; rebuild from mind lines if needed (map.recover).", "triggers": ["recover", "replay"], "schema": {"type": "object", "properties": {}}},
    "hypermap.export_passport": {"summary": "Export a portable map passport (map.export_passport).", "triggers": ["passport", "export"], "schema": {"type": "object", "properties": {}}},
}

DEFAULT_ROUTE = ["rag.query", "hypermap.query", "hypermap.attend", "hypermap.ingest"]
ALLOWLIST = set(CATALOG.keys())


def tool_defs(names):
    out = []
    for n in names:
        c = CATALOG[n]
        out.append({"name": n, "summary": c["summary"], "inputSchema": c["schema"], "estimated_tokens": est_tokens(json.dumps(c["schema"]))})
    return out


def do_route(task: str, task_kind: str = "") -> dict:
    text = (task + " " + task_kind).lower()
    selected = list(DEFAULT_ROUTE)
    reasons = ["default Zed-thread route: RAG + HyperMap only"]
    # Task-gating: add a lazy tool only if the task actually mentions it,
    # and only as a suggestion with its summary — route() still caps tokens.
    for name, c in CATALOG.items():
        if name in selected:
            continue
        if any(t in text for t in c["triggers"]):
            selected.append(name)
            reasons.append(f"task mentions '{name}' trigger")
    defs = tool_defs(selected)
    total = sum(d["estimated_tokens"] for d in defs)
    if total > MAX_INITIAL_TOKENS:
        # Cap enforcement: fall back to summaries only, never exceed the cap.
        defs = [{"name": d["name"], "summary": CATALOG[d["name"]]["summary"], "estimated_tokens": est_tokens(CATALOG[d["name"]]["summary"])} for d in defs]
        total = sum(d["estimated_tokens"] for d in defs)
        reasons.append("capped: full schemas truncated to summaries to stay under 1500 tokens")
    return {"selected": selected, "definitions": defs, "estimated_tokens": total, "cap_tokens": MAX_INITIAL_TOKENS, "under_cap": total <= MAX_INITIAL_TOKENS, "reasons": reasons}


def do_search(kind: str, keyword: str) -> dict:
    kw = keyword.lower()
    results = []
    if kind in ("tool", "both"):
        for name, c in CATALOG.items():
            if kw in name.lower() or kw in c["summary"].lower() or any(kw in t for t in c["triggers"]):
                results.append({"kind": "tool", "name": name, "summary": c["summary"]})
    if kind in ("skill", "both"):
        # Search the on-disk catalog by keyword; return one-line matches only.
        # The full catalog (~72 KB, 869 entries) is never sent to the model.
        try:
            lines = SKILLS_CATALOG.read_text(encoding="utf-8", errors="ignore").splitlines()
        except Exception:
            lines = []
        for line in lines:
            if kw in line.lower():
                results.append({"kind": "skill", "name": line.strip()[:160], "summary": "skill catalog match; load full skill.md only on explicit selection via execute()"})
                if len(results) >= MAX_SEARCH_RESULTS:
                    break
        if not results or kind == "skill":
            # Curated fallback index (names + triggers only, no full text).
            curated = [
                ("runbook-plan-and-judges", "plan a runbook DAG of packets with judges"),
                ("agent-experience-and-tool-design", "design tools and agent experience"),
            ]
            for name, desc in curated:
                if kw in name or kw in desc:
                    results.append({"kind": "skill", "name": name, "summary": desc})
    return {"results": results[:MAX_SEARCH_RESULTS], "note": "summaries only; full definitions load only on explicit selection"}


def call_hypermap(tool: str, args: dict) -> dict:
    name = "map." + tool.split(".", 1)[1]
    reqs = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "tool-router", "version": "0.1"}}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": name, "arguments": args}},
    ]
    env = dict(os.environ)
    env["HYPERMIND_ROOT"] = HYPERMIND_ROOT
    p = subprocess.run(
        [HYPERMIND_BIN], input="\n".join(json.dumps(r) for r in reqs) + "\n",
        capture_output=True, text=True, timeout=60, env=env,
    )
    lines = [json.loads(l) for l in p.stdout.splitlines() if l.strip().startswith("{")]
    if len(lines) < 2:
        raise RuntimeError(f"hypermind-mcp returned {len(lines)} lines: {p.stdout[:400]} {p.stderr[:400]}")
    res = lines[1]
    if "error" in res:
        raise RuntimeError(str(res["error"]))
    return res.get("result", {})


def call_rag(args: dict) -> dict:
    text = args.get("text", "")
    collection = args.get("collection", "papers")
    if collection not in ("papers", "research", "knowledge", "models", "platforms", "build", "engineering"):
        raise ValueError(f"no such collection {collection}")
    req = urllib.request.Request(
        RAG_URL, data=json.dumps({"text": text, "collection": collection}).encode(),
        headers={"Content-Type": "application/json"}, method="POST",
    )
    with urllib.request.urlopen(req, timeout=30) as r:
        v = json.load(r)
    # Truncate hits to save tokens: top 3, 800 chars each, with hit count.
    assembled = v.get("assembled", []) if isinstance(v, dict) else []
    hits = []
    for a in assembled[:3]:
        hits.append({"doc_id": a.get("doc_id"), "text": (a.get("text") or "")[:800]})
    return {"hits_returned": len(hits), "hits_total": len(assembled), "hits": hits}


def do_lane_mint(args: dict) -> dict:
    """Gated Zed-thread mint: delegates to lane_mint.py, which talks to
    the live hypermind-mcp on :8765. Passed runs mint exactly one atom;
    anything else is held by lane_mint.py and never minted. Receipts
    here store only hashes/keys, never prompt text (router doctrine)."""
    import tempfile

    run = {
        "run_id": str(args.get("run_id", "")),
        "lane": str(args.get("lane", "")),
        "verdict": str(args.get("verdict", "")),
        "output_summary": str(args.get("output_summary", "")),
        "prompt": str(args.get("prompt", "")),
        "steps": list(args.get("steps", []) or []),
        "tool_calls": list(args.get("tool_calls", []) or []),
    }
    if not run["run_id"] or not run["lane"]:
        raise ValueError("lane.mint requires run_id and lane")
    safe = "".join(c if c.isalnum() or c in "-_" else "_" for c in run["run_id"])[:64]
    tmp = Path(tempfile.gettempdir()) / f"lane_mint_{safe}.json"
    tmp.write_text(json.dumps(run), encoding="utf-8")
    cmd = [sys.executable, str(LANE_MINT_SCRIPT), "--run", str(tmp)]
    if run["verdict"] == "passed":
        cmd.append("--mint")
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
    try:
        tmp.unlink()
    except Exception:
        pass
    if proc.returncode != 0:
        raise RuntimeError(f"lane_mint failed: {proc.stdout[:500]} {proc.stderr[:500]}")
    return {"tool": "lane.mint", "verdict": run["verdict"],
            "minted": run["verdict"] == "passed",
            "output": proc.stdout.strip()[:1000]}


def do_execute(name: str, args: dict, task: str = "") -> dict:
    if name not in ALLOWLIST and not name.startswith("skill."):
        raise ValueError(f"tool not allowlisted: {name}")
    if name == "lane.mint":
        out = do_lane_mint(args)
    elif name == "rag.query":
        out = call_rag(args)
    elif name.startswith("hypermap."):
        out = call_hypermap(name, args)
    elif name.startswith("skill."):
        slug = name[len("skill."):]
        # Load a skill file only on explicit selection, truncated.
        candidates = [
            Path(f"/Users/stewartbarteau/Desktop/AtomaApp/skills/{slug}.md"),
            Path(f"/Users/stewartbarteau/Desktop/skills/{slug}.md"),
        ]
        found = next((c for c in candidates if c.is_file()), None)
        if not found:
            raise ValueError(f"skill file not found for {slug}; use search_catalog to find a skill first")
        out = {"skill": slug, "path": str(found), "text_truncated": found.read_text(encoding="utf-8", errors="ignore")[:2000]}
    else:
        raise ValueError(f"unhandled allowlisted tool {name}")
    out_text = json.dumps(out)[:MAX_OUTPUT_CHARS]
    log_receipt(task, [name], "execute", est_tokens(out_text), name, "ok", args, out_text)
    return {"tool": name, "result_truncated": out, "truncation_chars": MAX_OUTPUT_CHARS}


TOOLS = [
    {"name": "route", "description": "Route a Zed-thread task to the minimal tool/skill list, with token estimate and reason. Default: RAG + HyperMap only. Full definitions load only for the selected few, under a 1500-token cap.", "inputSchema": {"type": "object", "properties": {"task": {"type": "string"}, "task_kind": {"type": "string"}}, "required": ["task"]}},
    {"name": "search_catalog", "description": "Search tools or skills by keyword. Returns one-line summaries only, never full definitions.", "inputSchema": {"type": "object", "properties": {"kind": {"type": "string", "enum": ["tool", "skill", "both"]}, "keyword": {"type": "string"}}, "required": ["keyword"]}},
    {"name": "execute", "description": "Execute one allowlisted tool (rag.query, hypermap.*, lane.mint, or skill.<slug>). Output truncated to 4000 chars. Unknown tools are refused.", "inputSchema": {"type": "object", "properties": {"name": {"type": "string"}, "args": {"type": "object"}, "task": {"type": "string"}}, "required": ["name"]}},
]


def emit(rid, payload):
    print(json.dumps({"jsonrpc": "2.0", "id": rid, "result": payload}), flush=True)


def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except Exception:
            continue
        method = req.get("method")
        rid = req.get("id")
        if method == "initialize":
            emit(rid, {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}, "serverInfo": {"name": SERVER_NAME, "version": "0.1.0"}})
        elif method == "tools/list":
            emit(rid, {"tools": TOOLS})
        elif method == "tools/call":
            params = req.get("params", {})
            name = params.get("name")
            args = params.get("arguments", {}) or {}
            try:
                if name == "route":
                    out = do_route(args.get("task", ""), args.get("task_kind", ""))
                    log_receipt(args.get("task", ""), out["selected"], "; ".join(out["reasons"]), out["estimated_tokens"], "route", "ok", args, json.dumps(out))
                elif name == "search_catalog":
                    out = do_search(args.get("kind", "both"), args.get("keyword", ""))
                elif name == "execute":
                    out = do_execute(args.get("name", ""), args.get("args", {}) or {}, args.get("task", ""))
                else:
                    raise ValueError(f"unknown tool {name}")
                payload = {"content": [{"type": "text", "text": json.dumps(out, indent=2)}], "isError": False}
            except Exception as e:
                payload = {"content": [{"type": "text", "text": json.dumps({"error": str(e)})}], "isError": True}
            emit(rid, payload)
        elif method in ("notifications/initialized", "notifications/cancelled"):
            continue
        else:
            if rid is not None:
                print(json.dumps({"jsonrpc": "2.0", "id": rid, "error": {"code": -32601, "message": "method not found"}}), flush=True)


if __name__ == "__main__":
    main()
