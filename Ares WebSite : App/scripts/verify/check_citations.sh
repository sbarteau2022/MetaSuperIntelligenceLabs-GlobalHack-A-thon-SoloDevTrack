#!/usr/bin/env bash
# Commandment III: citations in report.tex must resolve in citations.bib
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
keys=$(grep -oE '\\cite\{[^}]*\}' docs/report.tex 2>/dev/null | tr ',' '\n' | sed 's/\\cite{//;s/}//' | sort -u || true)
if [ -z "$keys" ]; then
  echo "citations: ok (no citations present)"
  exit 0
fi
fail=0
while read -r key; do
  [ -z "$key" ] && continue
  if ! grep -q "@[a-zA-Z]*{${key}," docs/citations.bib; then
    echo "FAIL: unknown citation key: $key" >&2
    fail=1
  fi
done <<< "$keys"
if [ "$fail" -ne 0 ]; then exit 1; fi
echo "citations: ok"
