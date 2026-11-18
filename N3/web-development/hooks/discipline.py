#!/usr/bin/env python3
"""discipline.py — the conformance gate that makes the discipline REQUIRED, not advisory.

Date: 2026-10-30. Status: PROPOSED.
Usage: python3 skills/web-development/hooks/discipline.py [--json]
Exit: 0 pass (required before any skill/tool/hook/rule/template change claims done),
      1 warnings (fix or record with dated reason),
      2 FAIL (missing file, broken skeleton, invalid rule table — blocks completion).

What it checks (each maps to a RULES.md / GLOBAL-LOGIC.md law):
  D1  required top-level governance files exist
  D2  every area ships SKILL.md in the mandated skeleton (full-tier §§1-10, flat-tier Conventions)
  D3  every SKILL.md carries frontmatter name+description, a Date, and a Status
  D4  hooks layer complete: config, schema, rule table, 5 gate scripts, batch audit
  D5  rules-table.json valid: every rule has id/scope/compilable pattern/severity/message/fix
  D6  templates inventory present; no floating `latest` version pins
  D7  docs/ five-file set present
  D8  language rule table parses, merges cleanly, no duplicate ids across both tables
  D9  frontend language skills complete: 11 languages/*.md carry Title+Done-means+Rules+Pitfalls+Versions+Date+Status and a matching plugin.json id
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent  # skills/web-development/

AREAS_FULL = [
    "frontend-dev",
    "backend-systems",
    "middleware",
    "ui-ux-flow",
    "frontend-design-language",
    "testing-qa",
]
AREAS_FLAT = ["database-data", "api-integrations", "devops-deployment", "security-auth"]

FULL_SECTIONS = [
    "When to Use / Out of Scope",
    "Banned Patterns",
    "Calibration",
    "System Spec",
    "Reference Patterns",
    "Scripts & Data",
    "Execution Protocol",
    "Pre-Output Checklist",
    "Chaining & References",
    "Corrections",
]

TOP_FILES = [
    "GLOBAL-LOGIC.md",
    "RULES.md",
    "BEST-PRACTICES.md",
    "EDGE-CASES.md",
    "README.md",
    "package.json",
    ".gitignore",
]
HOOK_FILES = [
    "hooks/hooks.json",
    "hooks/HOOK_EVENT_SCHEMA.md",
    "hooks/rules-table.json",
    "hooks/rules-languages.json",
    "hooks/audit.py",
    "hooks/audit.sh",
    "hooks/scripts/_engine.py",
    "hooks/scripts/session_init.py",
    "hooks/scripts/pre_tool_guard.py",
    "hooks/scripts/post_edit_verify.py",
    "hooks/scripts/stop_completion_gate.py",
]
DOC_FILES = [
    "docs/Design.md",
    "docs/Pre-Registration.md",
    "docs/CLAIMS_REGISTER.md",
    "docs/Successes.md",
    "docs/Failures.md",
]
TEMPLATE_FILES = [
    "templates/middleware/request-id.ts",
    "templates/middleware/error-envelope.ts",
    "templates/middleware/validate.ts",
    "templates/middleware/PIPELINE.md",
    "templates/api/openapi.yaml",
    "templates/backend/env.example",
    "templates/frontend/base.css",
    "templates/testing/release-checklist.md",
    "templates/devops/docker-compose.yml",
    "templates/database/migration-template.sql",
    "templates/security/secrets-policy.md",
]


def main() -> int:
    fails: list[str] = []
    warns: list[str] = []

    # D1 — top-level governance files
    for rel in TOP_FILES:
        if not (ROOT / rel).is_file():
            fails.append(f"D1 missing top-level file: {rel}")

    # D2/D3 — SKILL.md skeletons
    for area in AREAS_FULL:
        path = ROOT / area / "SKILL.md"
        if not path.is_file():
            fails.append(f"D2 missing full-tier skill: {area}/SKILL.md")
            continue
        text = path.read_text(encoding="utf-8")
        heads = [m.group(1).strip() for m in re.finditer(r"^##\s+(.+)$", text, re.M)]
        idx = 0
        for required in FULL_SECTIONS:
            found = next(
                (
                    i
                    for i in range(idx, len(heads))
                    if required.lower() in heads[i].lower()
                ),
                None,
            )
            if found is None:
                fails.append(
                    f"D2 {area}/SKILL.md: section missing or out of order: '{required}'"
                )
                break
            idx = found + 1
        if (
            "name:" not in text.split("---")[1]
            if text.startswith("---") and text.count("---") >= 2
            else True
        ):
            warns.append(f"D3 {area}/SKILL.md: frontmatter name not found")
        if "description:" not in (
            text.split("---")[1]
            if text.startswith("---") and text.count("---") >= 2
            else text[:400]
        ):
            warns.append(f"D3 {area}/SKILL.md: frontmatter description not found")
        if "Date:" not in text:
            warns.append(f"D3 {area}/SKILL.md: no Date line")
        if "Status:" not in text:
            warns.append(f"D3 {area}/SKILL.md: no Status line")

    for area in AREAS_FLAT:
        path = ROOT / area / "SKILL.md"
        if not path.is_file():
            fails.append(f"D2 missing flat-tier skill: {area}/SKILL.md")
            continue
        text = path.read_text(encoding="utf-8")
        if not re.search(r"^##\s+Conventions", text, re.M | re.I):
            fails.append(
                f"D2 {area}/SKILL.md: flat-tier skill needs a '## Conventions' section"
            )
        if "Date:" not in text:
            warns.append(f"D3 {area}/SKILL.md: no Date line")

    # D4 — hooks layer
    for rel in HOOK_FILES:
        if not (ROOT / rel).is_file():
            fails.append(f"D4 missing hooks file: {rel}")

    # D5 — rule table validity
    table = ROOT / "hooks/rules-table.json"
    if table.is_file():
        try:
            data = json.loads(table.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            fails.append(f"D5 rules-table.json is not valid JSON: {exc}")
            data = {}
        seen: set[str] = set()
        for i, rule in enumerate(data.get("rules", [])):
            rid = rule.get("id", f"<rule {i} without id>")
            if rid in seen:
                fails.append(f"D5 duplicate rule id: {rid}")
            seen.add(rid)
            for field in ("scope", "pattern", "severity", "message", "fix"):
                if field not in rule:
                    fails.append(f"D5 rule {rid}: missing field '{field}'")
            if rule.get("severity") not in ("deny", "warn"):
                fails.append(f"D5 rule {rid}: severity must be deny|warn")
            try:
                re.compile(rule.get("pattern", ""))
            except re.error as exc:
                fails.append(f"D5 rule {rid}: pattern does not compile: {exc}")

    # D8 — language rule table (merged union with D5)
    seen_union: set[str] = set()
    for table_name in ("hooks/rules-table.json", "hooks/rules-languages.json"):
        table_path = ROOT / table_name
        if not table_path.is_file():
            if table_name.endswith("rules-languages.json"):
                warns.append("D8 rules-languages.json absent — base table only")
            else:
                fails.append(f"D8 missing {table_name}")
            continue
        try:
            table_data = json.loads(table_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            fails.append(f"D8 {table_name} is not valid JSON: {exc}")
            continue
        for rule in table_data.get("rules", []):
            rid = rule.get("id", f"<{table_name} rule without id>")
            if rid in seen_union:
                fails.append(f"D8 duplicate rule id across tables: {rid}")
            seen_union.add(rid)
            for field in ("scope", "pattern", "severity", "message", "fix"):
                if field not in rule:
                    fails.append(f"D8 {table_name} rule {rid}: missing field '{field}'")
            try:
                re.compile(rule.get("pattern", ""))
            except re.error as exc:
                fails.append(f"D8 {table_name} rule {rid}: pattern does not compile: {exc}")

    # D6 — templates + no floating pins
    for rel in TEMPLATE_FILES:
        if not (ROOT / rel).is_file():
            fails.append(f"D6 missing template: {rel}")
    for rel in ("templates/devops/docker-compose.yml", "templates/api/openapi.yaml"):
        path = ROOT / rel
        if path.is_file() and re.search(r":latest\b", path.read_text(encoding="utf-8")):
            fails.append(
                f"D6 {rel}: floating ':latest' pin is forbidden (GLOBAL-LOGIC.md §8)"
            )

    # D7 — five-file set
    for rel in DOC_FILES:
        if not (ROOT / rel).is_file():
            fails.append(f"D7 missing governance doc: {rel}")

    # D9 — frontend language skills complete (SWARM D scope: 11 files only)
    D9_IDS = [
        "zustand",
        "vite",
        "codemirror",
        "monaco",
        "tailwind",
        "react-three-fiber",
        "react-flow",
        "dockview",
        "html",
        "wasm",
        "tauri",
    ]
    lang_dir = ROOT.parent / "languages"
    plugin_ids: set[str] = set()
    plugin_file = lang_dir / "plugin.json"
    if plugin_file.is_file():
        try:
            plugin_ids = {
                str(entry.get("id"))
                for entry in json.loads(plugin_file.read_text(encoding="utf-8")).get(
                    "languages", []
                )
                if entry.get("id")
            }
        except json.JSONDecodeError as exc:
            fails.append(f"D9 languages/plugin.json is not valid JSON: {exc}")
    else:
        fails.append("D9 missing languages/plugin.json")
    d9_results: dict[str, str] = {}
    for lid in D9_IDS:
        md = lang_dir / f"{lid}.md"
        if not md.is_file():
            fails.append(f"D9 missing language skill: languages/{lid}.md")
            d9_results[lid] = "FAIL: file missing"
            continue
        text = md.read_text(encoding="utf-8")
        missing = [
            label
            for label, ok in (
                ("Title", text.lstrip().startswith("#")),
                ("Done-means", "Done means" in text),
                ("## Rules", bool(re.search(r"^##\s+Rules", text, re.M))),
                ("## Pitfalls", bool(re.search(r"^##\s+Pitfalls", text, re.M))),
                ("## Versions", bool(re.search(r"^##\s+Versions", text, re.M))),
                ("Date:", "Date:" in text),
                ("Status:", "Status:" in text),
            )
            if not ok
        ]
        if lid not in plugin_ids:
            missing.append("plugin.json entry")
        if missing:
            fails.append(f"D9 languages/{lid}.md: missing {', '.join(missing)}")
            d9_results[lid] = f"FAIL: missing {', '.join(missing)}"
        else:
            d9_results[lid] = "pass"

    as_json = "--json" in sys.argv[1:]
    if as_json:
        print(json.dumps({"fails": fails, "warns": warns, "d9": d9_results}, indent=2))
    else:
        print(f"discipline: {len(fails)} fail, {len(warns)} warnings")
        for item in fails:
            print(f"[FAIL] {item}")
        for item in warns:
            print(f"[WARN] {item}")
        for lid in D9_IDS:
            print(f"[D9] {lid}: {d9_results.get(lid, 'not checked')}")
    return 2 if fails else (1 if warns else 0)


if __name__ == "__main__":
    sys.exit(main())
