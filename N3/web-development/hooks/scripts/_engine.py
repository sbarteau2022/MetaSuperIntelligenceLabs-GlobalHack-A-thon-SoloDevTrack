"""Shared rule-table engine for all web-development hooks.

Date: 2026-10-30. Status: PROPOSED.
One table (rules-table.json), three severities, zero drift between live hooks and batch audit.
"""

from __future__ import annotations

import fnmatch
import json
import re
import sys
from pathlib import Path

HOOKS_DIR = Path(__file__).resolve().parent.parent
RULES_TABLE = HOOKS_DIR / "rules-table.json"
RULES_LANGUAGES = HOOKS_DIR / "rules-languages.json"
STATE_DIR_NAME = ".agent-state"


def _load_table(path: Path) -> list[dict]:
    try:
        with open(path, encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"cannot load {path.name}: {exc}") from exc
    return data.get("rules", [])


def load_rules() -> list[dict]:
    rules = _load_table(RULES_TABLE)
    if RULES_LANGUAGES.is_file():
        try:
            rules = rules + _load_table(RULES_LANGUAGES)
        except RuntimeError as exc:
            print(f"warning: {exc} — proceeding with base table only", file=sys.stderr)
    else:
        print(
            "warning: rules-languages.json absent — proceeding with base table only",
            file=sys.stderr,
        )
    seen: set[str] = set()
    for rule in rules:
        rid = rule.get("id", "<missing-id>")
        if rid in seen:
            raise RuntimeError(f"duplicate rule id across tables: {rid}")
        seen.add(rid)
    compiled = []
    for rule in rules:
        try:
            rx = re.compile(rule["pattern"], re.MULTILINE)
        except re.error as exc:
            raise RuntimeError(f"bad regex in rule {rule.get('id')}: {exc}") from exc
        compiled.append({**rule, "_rx": rx})
    return compiled


def _scope_match(path: str, pattern: str) -> bool:
    # fnmatch treats "**/" as requiring a directory part, so "b.py" never
    # matches "**/*.py". Match the full path AND the bare filename: a rule
    # scoped to a file type must fire regardless of directory depth.
    # (Bug found 2026-10-30 by synthetic pre-guard test; see Failures.md.)
    from os.path import basename
    # NOTE: str.lstrip chars-strips ("**/*.py".lstrip("**/") == ".py") — must use removeprefix.
    bare = pattern.removeprefix("**/") if pattern.startswith("**/") else pattern
    if fnmatch.fnmatch(path, pattern) or fnmatch.fnmatch(basename(path), bare):
        return True
    # directory-component match: "**/hooks/**" must also hit "hooks/audit.py"
    # (leading "**/" requires a slash fnmatch won't conjure) and "a/hooks/b".
    parts = path.replace("\\", "/").split("/")
    return any(fnmatch.fnmatch(part, seg.strip("*") or "*") for part in parts for seg in pattern.split("/") if seg.strip("*"))


def rule_applies(rule: dict, path: str) -> bool:
    if any(_scope_match(path, exc) for exc in rule.get("exclude", [])):
        return False
    return any(_scope_match(path, scope) for scope in rule.get("scope", ["**"]))


def scan(
    path: str, content: str, rules: list[dict], severities: set[str]
) -> list[dict]:
    """Return one finding per (rule) on first match. Findings are dicts with rule_id, message, fix, matched."""
    findings = []
    for rule in rules:
        if rule.get("severity") not in severities:
            continue
        if not rule_applies(rule, path):
            continue
        match = rule["_rx"].search(content)
        if match:
            findings.append(
                {
                    "rule_id": rule["id"],
                    "severity": rule["severity"],
                    "message": rule["message"],
                    "fix": rule.get("fix"),
                    "matched": match.group(0)[:120],
                }
            )
    return findings


def state_dir(workspace_root: str) -> Path:
    d = Path(workspace_root) / STATE_DIR_NAME
    d.mkdir(parents=True, exist_ok=True)
    return d


def read_stdin_json() -> dict:
    raw = sys.stdin.read()
    try:
        return json.loads(raw) if raw.strip() else {}
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"malformed hook stdin: {exc}") from exc
