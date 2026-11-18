#!/usr/bin/env bash
# Standalone Kimi Frontend & Frontier Stack Audit Script
# Compatible with both manual CLI execution and Kimi Code CLI (`exit 0` = pass, `exit 2` = block & feed stdout to agent).

set -euo pipefail
TARGET_DIR="${1:-.}"

python3 - "$TARGET_DIR" << 'EOF'
import os, re, sys

root = sys.argv[1]
errors = []

for dirpath, dirnames, filenames in os.walk(root):
    dirnames[:] = [d for d in dirnames if d not in ("node_modules", ".git", "dist", "build", ".next")]
    for fname in filenames:
        if not fname.endswith((".ts", ".tsx", ".css")):
            continue
        fpath = os.path.join(dirpath, fname)
        try:
            with open(fpath, "r", encoding="utf-8") as f:
                text = f.read()
        except Exception:
            continue

        rel = os.path.relpath(fpath, root)

        if fname.endswith((".ts", ".tsx")):
            if re.search(r"from\s+['\"]framer-motion['\"]", text):
                errors.append(f"[Motion v12] {rel}: Replace `from 'framer-motion'` with `from 'motion/react'`.")
            if re.search(r"\bforwardRef\s*(<|\()", text):
                errors.append(f"[React 19] {rel}: Replace `forwardRef` with direct `ref` prop.")
            if re.search(r"(^|\n)\s*(export\s+)?(const\s+)?enum\s+[A-Z]", text):
                errors.append(f"[TS 7.1 tsgo] {rel}: Replace runtime `enum` with `as const` object + union type.")
            if "useFrame(" in text and re.search(r"useFrame\s*\([^)]*\)\s*=>\s*\{[^}]*\bset[A-Z]\w*\(", text, re.S):
                errors.append(f"[R3F 60fps] {rel}: Detected React `setState` inside `useFrame`. Mutate refs or use `useStore.getState()` transiently.")
            if "// TODO" in text or "// add logic here" in text.lower():
                errors.append(f"[No-Stub Rule] {rel}: Contains unimplemented placeholder comment (`// TODO`).")

        if fname.endswith((".tsx", ".css")):
            if "transition-all" in text and ("ease-in-out" in text or "ease-linear" in text):
                errors.append(f"[Motion Physics] {rel}: Avoid `transition-all ease-in-out`. Use specific properties (`transition-[transform,opacity]`) and custom cubic-bezier/spring curves.")

if errors:
    print("❌ Kimi Frontier Frontend Audit Failed:")
    for e in errors:
        print(f"  - {e}")
    sys.exit(2)
else:
    print("✅ Kimi Frontier Frontend Audit Passed (0 violations).")
    sys.exit(0)
EOF
