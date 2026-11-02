#!/usr/bin/env python3
"""audit.py — batch audit. Same rules-table.json as the live hooks. No drift by construction.

Date: 2026-10-30. Status: PROPOSED.
Usage: python3 skills/web-development/hooks/audit.py <dir> [--json]
Exit: 0 clean, 1 warnings only, 2 deny-rule hit, 3 usage/config error.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "scripts"))
from scripts._engine import load_rules, scan  # noqa: E402

EXCLUDE_DIRS = {
    ".git",
    "node_modules",
    "dist",
    "build",
    ".agent-state",
    "__pycache__",
    "kimi-frontend-kit",  # vendored reference, governed by its own rules
    "staging-2026-10-30",  # Stewart's unvetted drop (do-not-load per its README) — out of mandate scope
}
EXCLUDE_FILES = {"rules-table.json"}


def iter_files(root: Path):
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if path.name in EXCLUDE_FILES:
            continue
        if any(part in EXCLUDE_DIRS for part in path.parts):
            continue
        yield path


def main(argv: list[str]) -> int:
    args = [a for a in argv if a != "--json"]
    as_json = "--json" in argv
    if len(args) != 1:
        print("usage: audit.py <dir> [--json]", file=sys.stderr)
        return 3
    root = Path(args[0])
    if not root.is_dir():
        print(f"not a directory: {root}", file=sys.stderr)
        return 3
    try:
        rules = load_rules()
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 3

    findings: list[dict] = []
    scanned = 0
    for path in iter_files(root):
        try:
            content = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        scanned += 1
        try:
            rel = path.relative_to(root).as_posix() if path.is_absolute() else path.as_posix()
        except ValueError:
            # target outside cwd (e.g. audit run from hooks/ with absolute target):
            # fall back to path relative to the audited root, never crash.
            try:
                rel = path.relative_to(Path.cwd()).as_posix()
            except ValueError:
                rel = path.as_posix()
        for finding in scan(rel, content, rules, {"deny", "warn"}):
            findings.append({"path": rel, **finding})

    deny_hits = [f for f in findings if f["severity"] == "deny"]
    if as_json:
        print(json.dumps({"scanned": scanned, "findings": findings}, indent=2))
    else:
        print(
            f"audit: {scanned} files, {len(deny_hits)} deny, {len(findings) - len(deny_hits)} warnings"
        )
        for finding in findings:
            print(
                f"[{finding['severity'].upper()}] {finding['path']} ({finding['rule_id']}): {finding['message']}"
            )
    return 2 if deny_hits else (1 if findings else 0)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
