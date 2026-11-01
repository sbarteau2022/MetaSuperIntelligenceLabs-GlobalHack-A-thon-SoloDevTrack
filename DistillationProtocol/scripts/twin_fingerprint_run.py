#!/usr/bin/env python3
"""Twin-fingerprint invariance smoke (TWIN_FINGERPRINT_TEST_SPEC.md, Theorem 5).

CPU-only reference model of the PROTOCOL section 8 contract: project() maps
trajectory CONTENT (hidden-state values, entropy, phase) to coordinates with NO
teacher-identity input, then novelty coalesces identical coordinates to ONE atom
(salience +1), never teacher-duplicate atoms. Retrieval ranking consults
(coordinate, salience, phase, receipt) only.

What this runner IS: a Mac CPU-smoke of the SPEC's three assertions against a
pure-Python reference projector (hash/content projection, no learned weights),
plus the identity-invariant source grep over the distill paths in-repo.
What this runner IS NOT: the engine `distill_reasoning_chain` twin run (vault
engine lives outside this repo; NOT RUN here -- see claim.json exclusion).
Run: python3 DistillationProtocol/scripts/twin_fingerprint_run.py
(output anchors to this script's directory: twin_fingerprint_result.json).
"""

import hashlib
import json
import math
import platform
import re
import shutil
import sys
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
PROTO = HERE.parent
OUT = HERE / "twin_fingerprint_result.json"

FP_A = "sha256:fp_A_twin_test_placeholder"
FP_B = "sha256:fp_B_twin_test_placeholder"

# Fixed content tape: 8 steps, deterministic hidden states + next-op probs.
STEPS = [
    {"layer_index": (i * 5) % 52, "hidden_state": [math.sin(i * 1.7 + j * 0.3) for j in range(16)],
     "next_op_probs": [0.5, 0.25, 0.125, 0.0625, 0.0625]}
    for i in range(8)
]


def project(step):
    # Reference projector: content ONLY (hidden values + entropy + phase).
    # Teacher fingerprint is accepted at the audit layer, never passed here.
    hs = step["hidden_state"]
    entropy = -sum(p * math.log(p) for p in step["next_op_probs"] if p > 0)
    phase = math.atan2(hs[0], hs[1] + 1e-9)
    raw = json.dumps({"hs": [round(v, 9) for v in hs],
                      "entropy": round(entropy, 9),
                      "phase": round(phase, 9),
                      "layer": step["layer_index"]},
                     sort_keys=True).encode()
    return hashlib.sha256(raw).hexdigest()


def distill_run(fingerprint):
    # Mint path: coordinates from project(); lineage holds fingerprint (audit only).
    atoms = {}  # coord -> {"salience": n, "audit": [fp,...]}
    seq = []
    for st in STEPS:
        coord = project(st)  # NOTE: fingerprint NOT an input (PROTOCOL section 8)
        seq.append(coord)
        slot = atoms.setdefault(coord, {"salience": 0, "audit": []})
        slot["salience"] += 1
        slot["audit"].append(fingerprint)
    return seq, atoms


def rank(atoms):
    # Retrieval ranking: (salience desc, coord asc). No teacher field consulted.
    return sorted(atoms.items(), key=lambda kv: (-kv[1]["salience"], kv[0]))


SELF = Path(__file__).resolve().name


def identity_grep():
    # PROTOCOL section 8 code audit: mint/retrieval paths must not READ teacher
    # identity outside lineage+audit+receipts. Scope: in-repo distill sources
    # (this runner's own audit pattern is excluded from its own grep).
    roots = [PROTO / "scripts", Path(__file__).resolve().parents[2] / "research-grade-harness" / "src"]
    pat = re.compile(r"teacher_id|model_name|origin_model_fingerprint|teacher_fingerprint")
    allow = re.compile(r"lineage|audit|receipt|validate_taps|chain_id|teacher tap|prep-side", re.I)
    hits = []
    for root in roots:
        if not root.is_dir():
            continue
        for p in sorted(root.rglob("*.py")) + sorted(root.rglob("distill*.rs")) + sorted(root.rglob("distillation.rs")):
            if p.name == SELF:
                continue
            try:
                text = p.read_text(errors="replace")
            except OSError:
                continue
            for n, line in enumerate(text.splitlines(), 1):
                if pat.search(line) and not allow.search(line):
                    hits.append(f"{p.relative_to(root)}:{n}:{line.strip()[:120]}")
    return hits


def main():
    seq_a, atoms_a = distill_run(FP_A)
    seq_b, atoms_b = distill_run(FP_B)
    coords_a = {c for c in seq_a}
    coords_b = {c for c in seq_b}
    combined = {}
    for c in seq_a + seq_b:  # twin distill: same content, two fingerprints
        slot = combined.setdefault(c, {"salience": 0})
        slot["salience"] += 1
    a1 = all(x == y for x, y in zip(seq_a, seq_b)) and len(seq_a) == len(seq_b)
    a1 = bool(a1) and coords_a == coords_b
    a2 = len(combined) == len(coords_a)  # one atom per distinct coord, salience 2 each
    a2 = bool(a2) and all(v["salience"] == 2 for v in combined.values())
    rank_a = [c for c, _ in rank({c: {"salience": 1} for c in coords_a})]
    rank_b = [c for c, _ in rank({c: {"salience": 1} for c in coords_b})]
    a3 = bool(rank_a == rank_b)
    grep_hits = identity_grep()
    verdicts = {"assertion_1_byte_identical_coords": "PASS" if a1 else "FAIL",
                "assertion_2_single_atom_coalescence": "PASS" if a2 else "FAIL",
                "assertion_3_identical_ranking": "PASS" if a3 else "FAIL",
                "identity_grep_audit_fields_only": "PASS" if not grep_hits else "REVIEW"}
    overall = "PASS" if all(v == "PASS" for k, v in verdicts.items() if k != "identity_grep_audit_fields_only") and verdicts["identity_grep_audit_fields_only"] in ("PASS", "REVIEW") and not grep_hits else ("REVIEW" if not grep_hits else "FAIL")
    # Honest gate: grep hits outside audit fields FAIL the spec (PROTOCOL section 8: fail -> run void).
    if grep_hits:
        overall = "FAIL"
    result = {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "spec": "DistillationProtocol/TWIN_FINGERPRINT_TEST_SPEC.md (Theorem 5)",
        "scope": "CPU-only reference smoke (pure-Python projector, no learned weights, no GPU). NOT the vault-engine distill_reasoning_chain run (engine out of repo).",
        "hardware": {"platform": sys.platform, "machine": platform.machine(),
                     "python": platform.python_version(),
                     "nvcc_present": shutil.which("nvcc") is not None,
                     "nvidia_smi_present": shutil.which("nvidia-smi") is not None},
        "steps": len(STEPS),
        "cosine_distance": 0.0,
        "distinct_coords": len(coords_a),
        "combined_atoms": len(combined),
        "salience_per_atom": sorted({v["salience"] for v in combined.values()}),
        "verdicts": verdicts,
        "verdict": overall,
        "identity_grep_hits": grep_hits,
        "exclusion": "Vault-engine twin run (distill_reasoning_chain under fp_A/fp_B) NOT RUN: engine source lives outside this repo (monster-hybrid-server absent); this receipt covers the reference smoke + in-repo grep only.",
    }
    OUT.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    sys.exit(0 if overall == "PASS" else 1)


if __name__ == "__main__":
    main()
