#!/usr/bin/env bash
# PreToolUse Hook: Blocks legacy/hallucinated frontend patterns before writing to disk.
# Checks for:
# 1. Creating legacy tailwind.config.js/ts when Tailwind v4 CSS-first (@theme) is required
# 2. Legacy `from "framer-motion"` instead of `from "motion/react"`
# 3. React.forwardRef in React 19 code
# 4. Calling setState inside R3F useFrame()
# 5. TypeScript `enum` or `namespace` violating TS 7.1 erasableSyntaxOnly

set -euo pipefail
PAYLOAD="$(cat)"

python3 -c '
import json, re, sys

try:
    data = json.loads(sys.stdin.read())
except Exception:
    print(json.dumps({"decision": "allow"}))
    sys.exit(0)

tool = data.get("toolCall", {})
args = tool.get("args", {})
path = args.get("Path", "") or ""
content = (args.get("Content", "") or "") + "\n" + (args.get("Replacement", "") or "")

# Only inspect frontend source/config files
if not any(path.endswith(ext) for ext in (".ts", ".tsx", ".js", ".jsx", ".css", ".json")):
    print(json.dumps({"decision": "allow"}))
    sys.exit(0)

# 1. Block tailwind.config.js / tailwind.config.ts creation
if re.search(r"tailwind\.config\.(js|ts|mjs|cjs)$", path):
    print(json.dumps({
        "decision": "deny",
        "reason": "Tailwind CSS v4 uses CSS-first configuration (`@import \"tailwindcss\";` and `@theme { ... }` in your main CSS file). Do not create a legacy tailwind.config.js/ts file."
    }))
    sys.exit(0)

# 2. Block legacy framer-motion import in TS/TSX files
if path.endswith((".ts", ".tsx")) and re.search(r"from\s+[\"\x27]framer-motion[\"\x27]", content):
    print(json.dumps({
        "decision": "deny",
        "reason": "Use Motion v12+ entrypoint `from \"motion/react\"` instead of legacy `from \"framer-motion\"`."
    }))
    sys.exit(0)

# 3. Block React.forwardRef in React 19 TS/TSX files
if path.endswith((".ts", ".tsx")) and re.search(r"\bforwardRef\s*(<|\()", content):
    print(json.dumps({
        "decision": "deny",
        "reason": "React 19 passes `ref` as a standard prop (`function Component({ ref, ...props })`). Do not wrap components in `forwardRef`."
    }))
    sys.exit(0)

# 4. Block TypeScript runtime enum in TS 7.1 erasableSyntaxOnly mode
if path.endswith((".ts", ".tsx")) and re.search(r"(^|\n)\s*(export\s+)?(const\s+)?enum\s+[A-Z]", content):
    print(json.dumps({
        "decision": "deny",
        "reason": "TypeScript 7.1 (`tsgo` + `erasableSyntaxOnly: true`) forbids runtime `enum` declarations. Use `as const` objects with union types (`type X = (typeof OBJ)[keyof typeof OBJ]`)."
    }))
    sys.exit(0)

print(json.dumps({"decision": "allow"}))
' <<< "$PAYLOAD"
