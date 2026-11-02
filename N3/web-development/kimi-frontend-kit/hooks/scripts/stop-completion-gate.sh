#!/usr/bin/env bash
# Stop Hook: Prevents the agent from terminating early if `.kimi-audit.log` has unresolved stubs/anti-patterns.
# Follows Antigravity Stop hook contract (`{"decision": "continue", "reason": "..."}` or `{"decision": "allow"}`).

set -euo pipefail
PAYLOAD="$(cat)"

python3 -c '
import json, os, sys

try:
    data = json.loads(sys.stdin.read())
except Exception:
    print(json.dumps({"decision": "allow"}))
    sys.exit(0)

reason = data.get("terminationReason", "")
workspaces = data.get("workspacePaths") or []
ws = workspaces[0] if workspaces else ""

if reason == "NO_TOOL_CALL" and ws and os.path.isdir(ws):
    log_path = os.path.join(ws, ".kimi-audit.log")
    if os.path.isfile(log_path):
        with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
            issues = f.read().strip()
        # Remove log after reading once so we never infinite-loop
        try:
            os.remove(log_path)
        except OSError:
            pass
        if issues:
            print(json.dumps({
                "decision": "continue",
                "reason": f"Kimi Frontend Audit Gate caught unresolved issues before completion:\n{issues}\nPlease resolve these before finishing."
            }))
            sys.exit(0)

print(json.dumps({"decision": "allow"}))
' <<< "$PAYLOAD"
