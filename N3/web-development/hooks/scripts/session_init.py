#!/usr/bin/env python3
"""session_init.py — SessionStart. Detects the stack, injects constraints. NEVER blocks.

Date: 2026-10-30. Status: PROPOSED.
Reads: common envelope on stdin. Writes: {"system_message","detected"} on stdout. Exit 0 always.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from _engine import read_stdin_json  # noqa: E402


def main() -> None:
    try:
        event = read_stdin_json()
    except RuntimeError:
        event = {}
    root = Path(event.get("workspace_root", "."))

    detected: dict = {}
    pkg = root / "package.json"
    if pkg.exists():
        detected["package_json"] = True
        try:
            data = json.loads(pkg.read_text(encoding="utf-8"))
            deps = {**data.get("dependencies", {}), **data.get("devDependencies", {})}
            detected["deps"] = sorted(deps.keys())
            scripts = data.get("scripts", {})
            for gate in ("build", "test", "lint", "typecheck", "audit"):
                if gate in scripts:
                    detected.setdefault("gates", []).append(gate)
        except (OSError, json.JSONDecodeError):
            detected["package_json_unreadable"] = True

    constraints = [
        "Global logic: skills/web-development/GLOBAL-LOGIC.md. Binding rules: RULES.md.",
        "Skill files load on trigger only; RULES.md loads always.",
        "Every generative task declares BRIEF + dials before generating (GLOBAL-LOGIC.md §7).",
        "No secrets in files. No stubs without a tracked issue. Semantic tokens only in components.",
    ]
    if "audit" in detected.get("gates", []):
        constraints.append("Batch audit available via the project's audit script before completion.")

    print(json.dumps({"system_message": " | ".join(constraints), "detected": detected}))


if __name__ == "__main__":
    main()
