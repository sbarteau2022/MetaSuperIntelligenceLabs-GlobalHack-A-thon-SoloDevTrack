#!/usr/bin/env bash
# SessionStart Hook: Detects active frontend stack (React 19, Tailwind v4, TS 7.1 tsgo)
# Reads JSON payload from stdin, outputs Antigravity injectSteps JSON to stdout.

set -euo pipefail
PAYLOAD="$(cat)"

WORKSPACE=$(echo "$PAYLOAD" | python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
    paths = data.get("workspacePaths") or []
    print(paths[0] if paths else "")
except Exception:
    print("")
' 2>/dev/null || echo "")

STACK_SUMMARY="Kimi Frontier Frontend Mode Active (React 19, Tailwind v4 OKLCH, TS 7.1 tsgo, Zustand v5 useShallow, Motion v12, R3F v9, Dockview/Monaco/CM6, Obsidian Force Graph & XYFlow v12)."

if [[ -n "$WORKSPACE" && -f "$WORKSPACE/package.json" ]]; then
  DETECTED=$(python3 -c "
import json
try:
    with open('$WORKSPACE/package.json') as f:
        pkg = json.load(f)
    deps = {**pkg.get('dependencies', {}), **pkg.get('devDependencies', {})}
    keys = ['react', 'tailwindcss', '@typescript/native-preview', 'typescript', 'zustand', 'motion', 'three', '@react-three/fiber', 'dockview-react', '@monaco-editor/react', '@codemirror/view', '@xyflow/react', 'elkjs', 'd3-force']
    found = [f'{k}:{deps[k]}' for k in keys if k in deps]
    print(', '.join(found) if found else 'No frontier deps installed yet')
except Exception:
    print('package.json parse skipped')
" 2>/dev/null || echo "")
  STACK_SUMMARY="$STACK_SUMMARY Detected workspace packages: [$DETECTED]."
fi

python3 -c '
import json, sys
msg = sys.argv[1]
print(json.dumps({
  "injectSteps": [
    {
      "systemMessage": {
        "systemMessage": msg
      }
    }
  ]
}))
' "$STACK_SUMMARY"
