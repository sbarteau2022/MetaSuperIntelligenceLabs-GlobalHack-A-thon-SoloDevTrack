#!/usr/bin/env python3
"""stop_completion_gate.py — Stop. Converts pending warnings into a completion veto.

Date: 2026-10-30. Status: PROPOSED.
Reads: stop event on stdin. Stdout carries decision allow|continue plus a report_reason summary.
When vetoing, ARCHIVES (rather than deleting) the reported issues so a consecutive stop is not
vetoed by the same stale warnings — the trail survives in issues-archive.jsonl.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _engine import read_stdin_json, state_dir  # noqa: E402


def main() -> None:
    try:
        event = read_stdin_json()
    except RuntimeError as exc:
        print(
            json.dumps(
                {
                    "decision": "deny",
                    "report_reason": f"Stop-gate could not parse the event ({exc}).",
                }
            )
        )
        return
    try:
        directory = state_dir(str(event.get("workspace_root", ".")))
        pending = directory / "pending-issues.jsonl"
        issues = []
        if pending.exists():
            for line in pending.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if not line:
                    continue
                try:
                    issues.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
    except OSError as exc:
        print(
            json.dumps(
                {
                    "decision": "deny",
                    "report_reason": f"Stop-gate could not read the issue store ({exc}).",
                }
            )
        )
        return

    if not issues:
        print(json.dumps({"decision": "allow", "report_reason": None}))
        return

    summary = "; ".join(
        f"{i.get('path')}: [{i.get('rule_id')}] {i.get('message')}" for i in issues
    )
    try:
        archive = directory / "issues-archive.jsonl"
        with open(archive, "a", encoding="utf-8") as fh:
            for issue in issues:
                fh.write(json.dumps(issue) + "\n")
        pending.write_text("", encoding="utf-8")
    except OSError:
        pass
    print(
        json.dumps(
            {
                "decision": "continue",
                "report_reason": f"Completion vetoed: {len(issues)} pending warning(s). {summary} Fix them or record why each cannot be fixed, then stop again.",
            }
        )
    )


if __name__ == "__main__":
    main()
