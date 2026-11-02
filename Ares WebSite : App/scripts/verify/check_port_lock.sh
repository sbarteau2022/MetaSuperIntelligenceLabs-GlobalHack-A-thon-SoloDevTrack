#!/usr/bin/env bash
# Port lock from ROADMAP.md / SYSTEM_WORKFLOW_LOCK.md
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if ! grep -q "8000 — SurrealDB ONLY" ROADMAP.md; then
  echo "FAIL: port 8000 lock missing from ROADMAP.md" >&2
  exit 1
fi
# 6331 must be declared stale, never used
if grep -R "6331" src 2>/dev/null | grep -v "stale" >/dev/null 2>&1; then
  echo "FAIL: stale port 6331 referenced in src/" >&2
  exit 1
fi
echo "port lock: ok"
