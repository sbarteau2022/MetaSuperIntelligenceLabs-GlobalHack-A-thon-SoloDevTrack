#!/usr/bin/env bash
# Ten Commandments enforcement gate (Commandments I-X + domain extras)
# Exit non-zero on any failure. Emits why-receipt on failure.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

failures=()
run() {
  local name="$1"; shift
  if ! "$@" >/tmp/gate_out 2>&1; then
    failures+=("$name")
    echo "=== GATE FAIL: $name ==="
    cat /tmp/gate_out
    echo ""
    # why-receipt
    echo "WHY-RECEIPT: what ran=$name | command=$* | exit=$? | last output=$(tail -c 200 /tmp/gate_out) | expected=pass | evidenced cause=see above | unblocker=fix listed error"
  fi
}

# Structural commandments
[ -f TODO.md ] || { echo "FAIL: TODO.md missing (Commandment I)"; failures+=(todo); }
[ -f FAILURES.md ] || { echo "FAIL: FAILURES.md missing (Commandment IX)"; failures+=(failures); }
[ -f SUCCESSES.md ] || { echo "FAIL: SUCCESSES.md missing (Commandment IX)"; failures+=(successes); }
[ -f docs/report.tex ] || { echo "FAIL: report.tex missing (Commandment IX)"; failures+=(report); }
[ -f docs/citations.bib ] || { echo "FAIL: citations.bib missing (Commandment III)"; failures+=(citations); }

run "citations" bash scripts/verify/check_citations.sh
run "experiment_manifest" bash scripts/verify/check_experiment_manifest.sh
run "oracle_section" bash scripts/verify/check_report_oracle.sh
run "port_lock" bash scripts/verify/check_port_lock.sh
run "holomorphic_math_lock" bash scripts/verify/check_holomorphic_math_lock.sh
run "doc_metadata" bash scripts/verify/check_doc_metadata.sh

# Domain extra: no log files >100MB in repo
biglogs=$(find . -path ./node_modules -prune -o -type f -name "*.log" -size +100M -print 2>/dev/null)
if [ -n "$biglogs" ]; then
  echo "FAIL: oversized log files present"; echo "$biglogs"; failures+=(logs);
fi

# Commandment II: eval/test scripts clean (no uncommitted changes there)
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if [ -n "$(git diff --name-only -- scripts/eval tests 2>/dev/null)" ]; then
    echo "WARN: uncommitted changes in eval/test scripts (Commandment II)"
  fi
fi

if [ ${#failures[@]} -gt 0 ]; then
  echo "GATE RESULT: FAIL (${#failures[@]} checks)" >&2
  exit 1
fi
echo "GATE RESULT: PASS"
exit 0
