#!/usr/bin/env python3
"""pre_tool_guard.py — PreToolUse. Deny-list on write content. FAILS CLOSED.

Date: 2026-10-30. Status: PROPOSED.
Reads: pre_write event on stdin. Stdout: {"decision","report_reason","rule_id","fix"}. Exit 0 always.
First deny-rule match wins. Schema errors deny (fail closed), never allow.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _engine import load_rules, read_stdin_json, scan  # noqa: E402


def deny(reason: str, rule_id: str, fix: str | None = None) -> None:
    print(json.dumps({"decision": "deny", "report_reason": reason, "rule_id": rule_id, "fix": fix}))
    sys.exit(0)


def main() -> None:
    try:
        event = read_stdin_json()
    except RuntimeError as exc:
        deny(f"Pre-guard could not parse the hook event ({exc}). Failing closed.", "schema-error",
             "Fix the harness event payload to match hooks/HOOK_EVENT_SCHEMA.md.")

    path = event.get("path")
    content = event.get("content")
    if not isinstance(path, str) or not path:
        deny("Pre-guard received no path. Failing closed.", "schema-error",
             "Provide the root-relative path in the hook event.")
    if not isinstance(content, str):
        deny(f"Pre-guard received no content for {path}. Failing closed.", "schema-error",
             "Provide the full post-write content in the hook event.")

    try:
        rules = load_rules()
    except RuntimeError as exc:
        deny(f"Pre-guard could not load the rule table ({exc}). Failing closed.", "schema-error",
             "Repair skills/web-development/hooks/rules-table.json.")

    findings = scan(path, content, rules, {"deny"})
    if findings:
        first = findings[0]
        deny(
            f"Blocked {path}: {first['message']} Matched: {first['matched']!r}.",
            first["rule_id"],
            first.get("fix"),
        )

    print(json.dumps({"decision": "allow", "report_reason": None, "rule_id": None, "fix": None}))


if __name__ == "__main__":
    main()
