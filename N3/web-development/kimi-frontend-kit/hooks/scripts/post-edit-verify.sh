#!/usr/bin/env bash
# PostToolUse Hook: Runs background static checks after file edits and logs diagnostics.
# Antigravity PostToolUse contract requires returning `{}` on stdout.

set -euo pipefail
PAYLOAD="$(cat)"

python3 -c '
import json, os, sys

try:
    data = json.loads(sys.stdin.read())
    tool = data.get("toolCall", {})
    args = tool.get("args", {})
    path = args.get("Path", "") or ""
    workspaces = data.get("workspacePaths") or []
    ws = workspaces[0] if workspaces else ""
    if ws and os.path.isdir(ws) and path.endswith((".ts", ".tsx", ".css")):
        log_path = os.path.join(ws, ".kimi-audit.log")
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            src = f.read()
        warnings = []
        if "// TODO" in src or "// FIXME" in src:
            warnings.append(f"{path}: Contains TODO/FIXME stub comment")
        if "transition-all" in src and ("ease-in-out" in src or "linear" in src):
            warnings.append(f"{path}: Uses generic transition-all ease-in-out/linear instead of custom spring/cubic-bezier")
        with open(log_path, "w", encoding="utf-8") as lf:
            lf.write("\n".join(warnings))
except Exception:
    pass

print("{}")
' <<< "$PAYLOAD"
