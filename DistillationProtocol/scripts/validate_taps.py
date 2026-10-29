#!/usr/bin/env python3
"""Validate a teacher tap JSON before distillation (PROTOCOL §2, Pillar 2).

This does not generate taps; it enforces the contract that distill_reasoning_chain
expects so a bad tape fails before it reaches the engine.
"""

import json
import math
import sys


def main():
    if len(sys.argv) != 2:
        print("usage: validate_taps.py <taps.json>", file=sys.stderr)
        sys.exit(2)
    with open(sys.argv[1]) as f:
        data = json.load(f)
    errors = []
    for k in ("chain_id", "teacher_fingerprint", "total_teacher_layers", "steps"):
        if k not in data:
            errors.append(f"missing top-level '{k}'")
    steps = data.get("steps", [])
    if not steps:
        errors.append("steps must be non-empty")
    layers = data.get("total_teacher_layers")
    dims = set()
    prob_sums = []
    for i, s in enumerate(steps):
        for k in ("layer_index", "hidden_state", "next_op_probs"):
            if k not in s:
                errors.append(f"step[{i}] missing '{k}'")
                continue
        li = s.get("layer_index")
        if layers is not None and not (0 <= li < layers):
            errors.append(f"step[{i}] layer_index {li} out of [0,{layers})")
        hs = s.get("hidden_state", [])
        if not hs:
            errors.append(f"step[{i}] empty hidden_state")
        else:
            dims.add(len(hs))
        probs = s.get("next_op_probs", [])
        if not probs:
            errors.append(f"step[{i}] empty next_op_probs")
        else:
            sm = sum(probs)
            prob_sums.append(sm)
            if abs(sm - 1.0) > 1e-3:
                errors.append(f"step[{i}] next_op_probs sum={sm:.6f} (want ~1.0)")
            if any(p < 0 for p in probs):
                errors.append(f"step[{i}] negative probability")
    if len(dims) > 1:
        errors.append(f"inconsistent hidden dims across steps: {sorted(dims)}")
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        sys.exit(1)
    print(
        json.dumps(
            {
                "status": "ok",
                "steps": len(steps),
                "hidden_dim": sorted(dims)[0] if dims else None,
                "layers_covered": sorted({s["layer_index"] for s in steps}),
                "min_prob_sum": min(prob_sums) if prob_sums else None,
                "max_prob_sum": max(prob_sums) if prob_sums else None,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
PY
