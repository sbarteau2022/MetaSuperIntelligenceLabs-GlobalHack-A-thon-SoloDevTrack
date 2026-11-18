#!/usr/bin/env python3
"""AIE curriculum manifest builder (Model Home cognitive map, Launch gate 6).

Scans a directory of AIE module packets (markdown) and emits manifest.json:
per module {module_id, course, concept_ids (from Lesson headings), dimensions
(Blueprint nine, keyword-derived, never all-nine), rubric criteria refs,
prerequisites (explicit refs + canonical order), source_path, sha256}.

Usage (Stewart, Mac):
  python3 build_manifest.py "/Users/stewartbarteau/Desktop/Model Home/workspace/aie-packets" manifest.json

Dimensions are the Blueprint §C nine: TRACE SPEC LOOP TEST DEBUG FOUND
CONTEXT COMPOSE SEC. A dimension is attached only when keyword evidence in
lesson/lab headings + rubric criteria reaches threshold — modules with no
evidence are flagged, not silently labelled (Blueprint: do not mechanically
label all modules with all nine).
"""
import hashlib
import json
import re
import sys
from pathlib import Path

DIMENSION_KEYWORDS = {
    "TRACE": ["bytecode", "interpreter evaluates", "trace", "execution", "predict, on paper", "predict the"],
    "SPEC": ["contract", "signature", "specification", "protocol", "typing", "mypy", "interface"],
    "LOOP": ["loop", "iterator", "generator", "iteration", "comprehension", "enumerate"],
    "TEST": ["pytest", "property-based", "hypothesis", "test", "verify", "assertion", "falsif"],
    "DEBUG": ["debug", "traceback", "logging", "profil", "error", "locate", "defect"],
    "FOUND": ["interpreter", "values, names", "mutab", "binding", "object identity", "fundamental"],
    "CONTEXT": ["codebase", "reading real", "reading code", "reading history", "review", "context", "documentation"],
    "COMPOSE": ["packaging", "ci ", "continuous integration", "github actions", "build", "compose", "architecture", "package", "repository", "merge strategy", "branch"],
    "SEC": ["security", "secret", "supply chain", "vulnerab", "dependency audit", "token"],
}
MODULE_ID_RE = re.compile(r"(AIE-\d{3})-M(\d{2})")
REF_RE = re.compile(r"AIE-(\d{3})(?:-M(\d{2}))?(?:\s*[–-]\s*M?(\d{2}))?")
LESSON_RE = re.compile(r"^###\s+Lesson\s+(\d+)\s*[—-]\s*(.+)$", re.M)
RUBRIC_HEAD_RE = re.compile(r"^###\s+Rubric.*$", re.M)
TABLE_ROW_RE = re.compile(r"^\|\s*\*\*(.+?)\*\*\s*\|", re.M)


def slug(s: str) -> str:
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")[:48]


def expand_prereq_refs(text: str, course: str):
    """Explicit AIE refs in a Prerequisite line, ranges expanded."""
    refs = []
    for m in REF_RE.finditer(text or ""):
        c, mod, rng_end = m.group(1), m.group(2), m.group(3)
        if mod is None:
            refs.append(f"AIE-{c}")
            continue
        start = int(mod)
        end = int(rng_end) if rng_end else start
        for n in range(start, end + 1):
            refs.append(f"AIE-{c}-M{n:02d}")
    return refs


def parse_module(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    sha = hashlib.sha256(path.read_bytes()).hexdigest()
    m = MODULE_ID_RE.search(path.name) or MODULE_ID_RE.search(text[:200])
    course, mod_num = m.group(1), int(m.group(2))
    module_id = f"{course}-M{mod_num:02d}"

    title_m = re.search(r"^#\s+.+?[—-]\s*(.+)$", text, re.M)
    title = title_m.group(1).strip() if title_m else module_id
    course_line = re.search(r"^\*\*Course:\*\*\s*(.+)$", text, re.M)
    prereq_line = re.search(r"^\*\*Prerequisite:\*\*\s*(.+)$", text, re.M)

    concepts = []
    for lm in LESSON_RE.finditer(text):
        cid = f"{module_id}-L{lm.group(1)}"
        concepts.append({"concept_id": cid, "label": lm.group(2).strip(),
                         "module_id": module_id, "kind": "lesson"})

    rubric = []
    rm = RUBRIC_HEAD_RE.search(text)
    if rm:
        section = text[rm.start(): rm.start() + 6000]
        for row in TABLE_ROW_RE.finditer(section):
            label = row.group(1).strip()
            if label.lower() not in ("dimension",):
                rubric.append({"criterion_id": f"{module_id}-R-{slug(label)}",
                               "label": label, "module_id": module_id})

    hay = (title + "\n" + "\n".join(c["label"] for c in concepts) + "\n"
           + "\n".join(r["label"] for r in rubric) + "\n"
           + "\n".join(re.findall(r"^###\s+Lab.*$", text, re.M))).lower()
    dimensions, dim_evidence = [], {}
    for dim, kws in DIMENSION_KEYWORDS.items():
        hits = [kw for kw in kws if kw in hay]
        if len(hits) >= 2 or (hits and dim in ("TEST",) and "pytest" in hay):
            dimensions.append(dim)
            dim_evidence[dim] = hits
    dimensions.sort()

    prereqs = sorted(set(expand_prereq_refs(prereq_line.group(1) if prereq_line else "", course)))
    return {
        "module_id": module_id,
        "course": course,
        "course_order": int(course.split("-")[1]),
        "module_order": mod_num,
        "title": title,
        "course_label": course_line.group(1).strip() if course_line else course,
        "concept_ids": [c["concept_id"] for c in concepts],
        "concepts": concepts,
        "dimensions": dimensions,
        "dimension_status": "candidate_keyword_evidence_NOT_authored",
        "dimension_evidence": dim_evidence,
        "flags": ([] if dimensions else ["NO_DIMENSION_EVIDENCE"])
                 + ([] if concepts else ["NO_LESSON_CONCEPTS"])
                 + ([] if rubric else ["NO_RUBRIC_TABLE"]),
        "rubric_criteria": rubric,
        "prerequisites": prereqs,
        "source_path": str(path),
        "sha256": sha,
    }


def main() -> int:
    src = Path(sys.argv[1])
    out = Path(sys.argv[2])
    files = sorted(src.rglob("AIE-*.md")) if src.is_dir() else [src]
    modules = [parse_module(f) for f in files if "_GATE" not in f.name]

    # Canonical order prerequisites: previous module in same course.
    by_course = {}
    for mod in modules:
        by_course.setdefault(mod["course"], []).append(mod)
    for mods in by_course.values():
        mods.sort(key=lambda x: x["module_order"])
        for prev, cur in zip(mods, mods[1:]):
            if prev["module_id"] not in cur["prerequisites"]:
                cur.setdefault("order_prerequisites", []).append(prev["module_id"])
    for mod in modules:
        mod.setdefault("order_prerequisites", [])

    manifest = {"version": "aie-manifest-v1", "module_count": len(modules), "modules": modules}
    out.write_text(json.dumps(manifest, indent=2), encoding="utf-8")

    # Validation report
    ids = {m["module_id"] for m in modules}
    graph = {m["module_id"]: [p for p in m["prerequisites"] + m["order_prerequisites"] if p in ids]
             for m in modules}
    # Kahn acyclicity check on full graph incl. external-free edges
    indeg = {k: 0 for k in graph}
    for k, ps in graph.items():
        for p in ps:
            indeg[k] += 0  # edges p -> k
    order, stack = [], [k for k, ps in graph.items() if not ps]
    seen = set(stack)
    adj = {k: [] for k in graph}
    for k, ps in graph.items():
        for p in ps:
            adj[p].append(k)
    while stack:
        n = stack.pop()
        order.append(n)
        for nxt in adj[n]:
            graph[nxt].remove(n)
            if not graph[nxt] and nxt not in seen:
                seen.add(nxt)
                stack.append(nxt)
    acyclic = len(order) == len(ids)
    report = {
        "modules": len(modules),
        "all_have_sha256": all(len(m["sha256"]) == 64 for m in modules),
        "flagged_no_dimensions": [m["module_id"] for m in modules if "NO_DIMENSION_EVIDENCE" in m["flags"]],
        "flagged_no_rubric": [m["module_id"] for m in modules if "NO_RUBRIC_TABLE" in m["flags"]],
        "external_prereq_refs": sorted({p for m in modules for p in m["prerequisites"] if p not in ids}),
        "prereq_chain_acyclic": acyclic,
        "concept_count": sum(len(m["concepts"]) for m in modules),
        "rubric_criterion_count": sum(len(m["rubric_criteria"]) for m in modules),
    }
    Path(str(out) + ".validation.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["all_have_sha256"] and acyclic else 1


if __name__ == "__main__":
    raise SystemExit(main())
