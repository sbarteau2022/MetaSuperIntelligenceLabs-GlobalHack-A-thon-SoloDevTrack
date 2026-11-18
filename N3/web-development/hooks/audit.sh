#!/usr/bin/env sh
# audit.sh — thin wrapper so CI and humans run the same gate one way.
# Date: 2026-10-30. Usage: sh skills/web-development/hooks/audit.sh [dir] [--json]
# Exit codes mirror audit.py: 0 clean, 1 warnings, 2 deny hit.
set -eu
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET="${1:-$ROOT}"
exec python3 "$ROOT/skills/web-development/hooks/audit.py" "$TARGET" ${2:-}
