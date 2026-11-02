#!/usr/bin/env bash
# Commandment VIII: oracle/upper bound section must exist in report.tex
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if ! grep -q "Oracle / Theoretical Upper Bound" docs/report.tex; then
  echo "FAIL: oracle section missing" >&2
  exit 1
fi
echo "oracle section: present"
