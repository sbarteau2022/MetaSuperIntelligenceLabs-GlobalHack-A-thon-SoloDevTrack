#!/usr/bin/env bash
# Commandment VI: every experiment manifest declares exactly one changed_var
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
found=0
if [ -d experiments ]; then
  for f in experiments/*/manifest.json; do
    [ -f "$f" ] || continue
    found=$((found+1))
    n=$(python3 -c "import json,sys; d=json.load(open('$f')); print(len(d.get('changed_var',[])))" 2>/dev/null || echo "INVALID")
    if [ "$n" != "1" ]; then
      echo "FAIL: $f has $n changed vars (need 1)" >&2
      exit 1
    fi
  done
fi
echo "experiment manifests: ok (checked $found)"
