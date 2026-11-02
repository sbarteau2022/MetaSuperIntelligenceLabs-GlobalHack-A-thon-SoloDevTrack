#!/usr/bin/env bash
# Holomorphic function-space math lock (housepaper). Forbids presenting
# massless-helicity sheaf cohomology / H^1(PT, O(-2h-2)) as runtime function
# space, and related forbidden claims, anywhere outside the lock doc and this script.
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
bad=0
files=$(grep -rl "sheaf cohomology\|tit_bench\|H^{1}(\\\\mathrm{PT}" . 2>/dev/null | grep -v node_modules | grep -v ".git/" || true)
while IFS= read -r f; do
  [ -z "$f" ] && continue
  norm="${f#./}"
  [ "$norm" = "docs/HOLOMORPHIC_FUNCTION_SPACE.md" ] && continue
  case "$norm" in scripts/verify/check_holomorphic_math_lock.sh) continue;; esac
  if grep -q "sheaf cohomology" "$f" && grep -qi "runtime function space\|runtime space" "$f"; then
    echo "FAIL: forbidden runtime-cohomology claim in $f" >&2
    bad=1
  fi
  if grep -Eq "MSE.{0,40}proves|proves.{0,40}H" "$f"; then
    echo "FAIL: possible MSE-proves-geometry claim in $f" >&2
    bad=1
  fi
  if grep -q "tit_bench validates Twister" "$f"; then
    echo "FAIL: forbidden tit_bench validation claim in $f" >&2
    bad=1
  fi
done <<< "$files"
if [ "$bad" -ne 0 ]; then exit 1; fi
echo "holomorphic math lock: ok"
