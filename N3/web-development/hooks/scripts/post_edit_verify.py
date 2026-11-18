#!/usr/bin/env python3
"""post_edit_verify.py — PostToolUse. Appends warn-findings to the issue store. Never blocks.

Date: 2026-10-30. Status: PROPOSED.
Reads: post_write event on stdin. Stdout: {}. APPENDS to .agent-state/pending-issues.jsonl (never truncates).
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _engine import load_rules, read_stdin_json, scan, state_dir  # noqa: E402


def main() -> None:
    print(json.dumps({}))
    try:
        event = read_stdin_json()
    except RuntimeError:
        return
    path = event.get("path")
    content = event.get("content")
    if not isinstance(path, str) or not isinstance(content, str):
        return
    try:
        rules = load_rules()
    except RuntimeError:
        return
    findings = scan(path, content, rules, {"warn"})
    if not findings:
        return
    try:
        store = state_dir(str(event.get("workspace_root", "."))) / "pending-issues.jsonl"
        ts = datetime.now(timezone.utc).isoformat()
        with open(store, "a", encoding="utf-8") as fh:
            for finding in findings:
                fh.write(json.dumps({
                    "ts": ts,
                    "path": path,
                    "rule_id": finding["rule_id"],
                    "message": finding["message"],
                    "fix": finding.get("fix"),
                }) + "\n")
    except OSError:
        return


if __name__ == "__main__":
    main()
