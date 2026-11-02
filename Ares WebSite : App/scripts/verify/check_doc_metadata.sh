#!/usr/bin/env bash
# N3 documentation standards: docs/*.md must carry metadata header.
set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
bad=0
for f in docs/*.md; do
  [ -f "$f" ] || continue
  for key in "title:" "document_type:" "version:" "date:" "status:"; do
    if ! grep -q "$key" "$f"; then
      echo "FAIL: $f missing metadata key '$key'" >&2
      bad=1
    fi
  done
done
if [ "$bad" -ne 0 ]; then exit 1; fi
echo "doc metadata: ok"
