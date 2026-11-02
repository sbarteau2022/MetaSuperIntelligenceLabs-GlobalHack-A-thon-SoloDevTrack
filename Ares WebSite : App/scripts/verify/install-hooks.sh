#!/usr/bin/env bash
# Install a pre-commit hook into the real git repo (parent) gating this project.
set -uo pipefail
PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TOPLEVEL="$(git -C "$PROJECT" rev-parse --show-toplevel 2>/dev/null)"
if [ -z "$TOPLEVEL" ]; then echo "not in a git repo"; exit 1; fi
HOOK="$TOPLEVEL/.git/hooks/pre-commit"
cat > "$HOOK" <<EOF
#!/usr/bin/env bash
# Auto-installed by Ares commandments (run from $PROJECT)
cd "$PROJECT" || exit 1
exec bash scripts/verify/gate.sh
EOF
chmod +x "$HOOK"
echo "installed pre-commit hook -> $HOOK"
