#!/usr/bin/env bash
# Wait for a Bailian finetune job or deployment to reach a terminal/ready state.
# Emits one line per status change (for use under the Monitor tool) and exits
# when the target state is reached.
#
# Usage:
#   wait.sh finetune <job-id>            # exits on SUCCEEDED/FAILED/CANCELED/PARTIALLY_SUCCEEDED
#   wait.sh deploy   <deployed-model>    # exits on RUNNING/FAILED/STOPPED
#
# Runs under bash (not zsh) on purpose: `status` is a read-only built-in in
# zsh, so writing a status-poll loop in zsh will crash with "read-only
# variable". Bash has no such constraint.
set -uo pipefail

kind="${1:-}"
id="${2:-}"
if [ -z "$kind" ] || [ -z "$id" ]; then
  echo "usage: wait.sh <finetune|deploy> <id>" >&2
  exit 2
fi

case "$kind" in
  finetune)
    cmd=(bl finetune get --job-id "$id" --output json)
    done_states=(SUCCEEDED FAILED CANCELED PARTIALLY_SUCCEEDED)
    interval=30
    ;;
  deploy)
    cmd=(bl deploy get --deployed-model "$id" --output json)
    done_states=(RUNNING FAILED STOPPED)
    interval=15
    ;;
  *)
    echo "unknown kind: $kind (expected finetune|deploy)" >&2
    exit 2
    ;;
esac

prev=""
while true; do
  s=$("${cmd[@]}" 2>/dev/null || true)
  st=$(printf '%s' "$s" | grep -o '"status": "[^"]*"' | head -1 | cut -d'"' -f4)
  [ -z "$st" ] && st="UNKNOWN"

  if [ "$st" != "$prev" ]; then
    echo "$kind $id -> $st"
    prev="$st"
  fi

  for d in "${done_states[@]}"; do
    if [ "$st" = "$d" ]; then
      echo "terminal: $st"
      exit 0
    fi
  done

  sleep "$interval"
done
