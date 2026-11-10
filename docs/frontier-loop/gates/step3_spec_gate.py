"""Step 3 gate: speculative decoding with the 1-layer MTP head vs plain greedy and vs the DSpark drafter.

Pre-registered gate (set before the first measurement):
  G1 correctness (teacher-forced, revised after the first run): every spec token must be an argmax of the
     plain model's logits, given the spec prefix, up to a tie tolerance of 0.5 logits; gate >= 98% of tokens.
     Rationale: the model runs in bf16, and 1-token vs 2-token forwards differ by ~0.6 logits, so greedy
     trajectories can split at near-ties. Exact-match of free-running outputs is reported for information only.
     The first version of this gate used free-running exact match and failed at 5.5%; see LOG.md.
  G2 acceptance: MTP acceptance rate p_mtp = accepted drafts / drafts is >= p_dspark - 0.05.
     p_dspark is taken from the server's engine metrics as (accept_len - 1) / cap, with cap = 4.
     Pure-acceptance comparison is an approximation; see LOG.md for the caveat.
  G3 speed: MTP tokens/s > DSpark tokens/s on the same prompts.

The DSpark baseline is read from gates/step3_dspark_baseline.json, produced by bench_dspark_baseline.py
against the running serve.sh server. Missing baseline -> gate FAILS (not skipped).

Run with the mlx-dspark python:
  ~/.local/share/uv/tools/mlx-dspark/bin/python docs/frontier-loop/gates/step3_spec_gate.py
"""

import json
import pathlib
import sys

import mlx.core as mx
from mlx_lm import load

REPO = pathlib.Path("/Users/stewartbarteau/Desktop/Qwen3.6Cyber")
FRONTIER = REPO / "docs" / "frontier-loop"
GATES = FRONTIER / "gates"
sys.path.insert(0, str(GATES))
sys.path.insert(0, str(FRONTIER))

from step3_prompts import PROMPTS, MAX_NEW  # noqa: E402
from mtp_head import load_mtp_head  # noqa: E402
from spec_decode import greedy_decode, mtp_spec_decode  # noqa: E402

MATCH_GATE = 0.98
TIE_TOL = 0.5
ACCEPT_SLACK = 0.05
CAP = 4
BASELINE = GATES / "step3_dspark_baseline.json"


def teacher_forced_ok(model, ids, spec_tokens) -> float:
    """Fraction of spec tokens that are within TIE_TOL of the plain model's max logit given the spec prefix."""
    lm = model.language_model
    seq = mx.concatenate([ids, mx.array([spec_tokens])], axis=1)
    logits = lm.lm_head(lm.model(seq))[0]  # no cache: reference path
    T0 = ids.shape[1]
    ok = 0
    for j, tok in enumerate(spec_tokens):
        row = logits[T0 - 1 + j]
        if float(row[tok].item()) >= float(mx.max(row).item()) - TIE_TOL:
            ok += 1
    return ok / len(spec_tokens)


def main() -> int:
    model, tok = load(str(REPO))
    head = load_mtp_head(REPO, model)

    # Warm-up (kernel compile) so timings are steady-state.
    warm = tok.apply_chat_template(PROMPTS[0], add_generation_prompt=True, tokenize=True)
    greedy_decode(model, mx.array([warm]), 8)
    mtp_spec_decode(model, head, mx.array([warm]), 8)

    rows = []
    for i, msgs in enumerate(PROMPTS):
        ids = mx.array([tok.apply_chat_template(msgs, add_generation_prompt=True, tokenize=True)])
        plain, plain_s = greedy_decode(model, ids, MAX_NEW)
        spec, spec_s, stats = mtp_spec_decode(model, head, ids, MAX_NEW)

        n = min(len(plain), len(spec))
        common = 0
        for a, b in zip(plain[:n], spec[:n]):
            if a != b:
                break
            common += 1
        match = common / max(len(plain), 1)
        tf = teacher_forced_ok(model, ids, spec)
        p = stats["accepted"] / max(stats["drafted"], 1)
        rows.append(
            dict(prompt=i, plain_tps=len(plain) / plain_s, spec_tps=len(spec) / spec_s,
                 match=match, tf=tf, p_mtp=p, drafted=stats["drafted"], accepted=stats["accepted"])
        )
        print(f"prompt {i}: plain {len(plain) / plain_s:.1f} tok/s | MTP {len(spec) / spec_s:.1f} tok/s | "
              f"exact-prefix {match:.3f} | teacher-forced ok {tf:.3f} | p_mtp {p:.3f} ({stats['accepted']}/{stats['drafted']})")

    total_match = sum(r["tf"] for r in rows) / len(rows)
    p_mtp = sum(r["accepted"] for r in rows) / max(sum(r["drafted"] for r in rows), 1)
    spec_tps = sum(r["spec_tps"] for r in rows) / len(rows)
    plain_tps = sum(r["plain_tps"] for r in rows) / len(rows)

    if not BASELINE.exists():
        print(f"DSpark baseline missing: {BASELINE}")
        print("GATE FAIL (baseline not measured)")
        return 1
    base = json.loads(BASELINE.read_text())
    p_dspark, dspark_tps = base["p_dspark"], base["dspark_tps"]

    print(f"\nmean teacher-forced ok {total_match:.3f} (gate >= {MATCH_GATE}); exact-prefix mean {sum(r['match'] for r in rows) / len(rows):.3f} (info)")
    print(f"p_mtp {p_mtp:.3f} vs p_dspark {p_dspark:.3f} (gate >= {p_dspark - ACCEPT_SLACK:.3f})")
    print(f"tok/s: MTP {spec_tps:.1f} | plain greedy {plain_tps:.1f} | DSpark {dspark_tps:.1f} (gate MTP > DSpark)")

    g1 = total_match >= MATCH_GATE
    g2 = p_mtp >= p_dspark - ACCEPT_SLACK
    g3 = spec_tps > dspark_tps
    print(f"G1 correctness {'PASS' if g1 else 'FAIL'} | G2 acceptance {'PASS' if g2 else 'FAIL'} | "
          f"G3 speed {'PASS' if g3 else 'FAIL'}")
    ok = g1 and g2 and g3
    print("GATE", "PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
