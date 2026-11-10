"""Step 5 gate: chained MTP drafts with DSpark's hybrid rollback (no rejected re-run), vs DSpark.

Same protocol as Step 4 (GOAL.md): choose K in {2,3,4} on DEV prompts by median tokens/s, then gate the chosen K
on HELD-OUT prompts against DSpark, in one process with matched settings.
  G1 correctness: teacher-forced, within 0.5 logits of the reference argmax, >= 0.98 on held-out.
  G3 speed: median tokens/s on held-out, chain(K*) > DSpark.
Reported only: held-out throughput for every K (diagnostic), per-depth acceptance, rollback counts.

Run:  ~/.local/share/uv/tools/mlx-dspark/bin/python docs/frontier-loop/gates/step5_rollback_gate.py
Writes gates/step5_rollback_results.json.
"""

import json
import pathlib
import statistics
import sys

GATES = pathlib.Path(__file__).resolve().parent
FRONTIER = GATES.parent
sys.path.insert(0, str(GATES))
sys.path.insert(0, str(FRONTIER))

import mlx.core as mx  # noqa: E402

from step3_prompts import PROMPTS as DEV_PROMPTS, MAX_NEW  # noqa: E402
from step4_prompts import HELDOUT_PROMPTS  # noqa: E402
from step3_matched_gate import setup, reference_ok, REPO  # noqa: E402
from mtp_head import load_mtp_head  # noqa: E402
from spec_decode import greedy_decode, mtp_chain_decode_rollback  # noqa: E402

KS = (2, 3, 4)
CAP = 4
REPS = 2
G1_GATE = 0.98
RESULTS = GATES / "step5_rollback_results.json"


def run_set(name, prompts_msgs, methods, encode_prompt, tok, target, drafter, lookup, speculative_generate, head):
    model = target.model
    rows = []
    for rep in range(REPS):
        for i, msgs in enumerate(prompts_msgs):
            content = msgs[0]["content"]
            ids = encode_prompt(tok, content, use_chat=True)
            for m in methods:
                if m == "plain":
                    out, secs = greedy_decode(model, mx.array([ids]), MAX_NEW, make_cache=target.make_cache)
                    toks, n, forwards, extra = out, len(out), len(out), {}
                elif m == "dspark":
                    res = speculative_generate(target, tok, drafter, content, max_new_tokens=MAX_NEW,
                                               max_draft_tokens=CAP, lookup_drafts=lookup, lookup_long_draft=32,
                                               confidence_threshold=0.0, temperature=0, top_p=1, top_k=1, seed=0,
                                               apply_chat_template=True, on_text=None)
                    toks, secs, n, forwards = res.token_ids, res.seconds, res.num_tokens, res.target_forwards
                    extra = {"accept_len": res.mean_accept_len}
                else:
                    K = int(m.split("_")[1])
                    out, secs, st = mtp_chain_decode_rollback(target, head, mx.array([ids]), MAX_NEW, K)
                    toks, n, forwards = out, len(out), st["forwards"]
                    extra = {"K": K, "drafted": st["drafted"], "accepted": st["accepted"],
                             "rollbacks": st["rollbacks"], "rounds": st["rounds"],
                             "accepted_by_depth": st["accepted_by_depth"], "reached_depth": st["reached_depth"]}
                row = dict(set=name, rep=rep, prompt=i, method=m, tokens=n, seconds=secs, tps=n / secs,
                           forwards=forwards, tok_per_fwd=n / forwards, **extra)
                if m.startswith("rchain"):
                    row["g1"] = reference_ok(model, ids, toks[:n])
                rows.append(row)
                print(json.dumps({k: (round(v, 3) if isinstance(v, float) else v) for k, v in row.items()}))
    return rows


def med(rows, method, key="tps"):
    return statistics.median(r[key] for r in rows if r["method"] == method)


def main() -> int:
    target, tok, drafter, speculative_generate, encode_prompt, lookup, target_repo, drafter_repo = setup()
    model = target.model
    head = load_mtp_head(REPO, model, dtype=mx.float16)

    ids0 = encode_prompt(tok, DEV_PROMPTS[0][0]["content"], use_chat=True)
    greedy_decode(model, mx.array([ids0]), 8, make_cache=target.make_cache)
    mtp_chain_decode_rollback(target, head, mx.array([ids0]), 8, 2)
    speculative_generate(target, tok, drafter, DEV_PROMPTS[0][0]["content"], max_new_tokens=8,
                         max_draft_tokens=CAP, lookup_drafts=lookup, apply_chat_template=True, on_text=None,
                         temperature=0, top_p=1, top_k=1, seed=0)

    dev_methods = ["plain", "dspark"] + [f"rchain_{K}" for K in KS]
    dev_rows = run_set("dev", DEV_PROMPTS, dev_methods, encode_prompt, tok, target, drafter, lookup,
                       speculative_generate, head)
    dev_tps = {K: med(dev_rows, f"rchain_{K}") for K in KS}
    K_star = max(KS, key=lambda K: (dev_tps[K], -K))
    print(f"\nDEV median tok/s: plain {med(dev_rows, 'plain'):.1f} dspark {med(dev_rows, 'dspark'):.1f} "
          + " ".join(f"rchain_{K} {dev_tps[K]:.1f}" for K in KS) + f"  -> K* = {K_star}")

    held_methods = ["plain", "dspark"] + [f"rchain_{K}" for K in KS]
    held_rows = run_set("heldout", HELDOUT_PROMPTS, held_methods, encode_prompt, tok, target, drafter, lookup,
                        speculative_generate, head)
    chain_name = f"rchain_{K_star}"
    g1_val = statistics.mean(r["g1"] for r in held_rows if r["method"] == chain_name)
    tps = {m: med(held_rows, m) for m in held_methods}
    g1 = g1_val >= G1_GATE
    g3 = tps[chain_name] > tps["dspark"]

    chain_held = [r for r in held_rows if r["method"] == chain_name]
    depth_acc = [sum(r["accepted_by_depth"][d] for r in chain_held) / max(sum(r["reached_depth"][d] for r in chain_held), 1)
                 for d in range(K_star)]
    summary = dict(target=target_repo, drafter=drafter_repo, kv_bits=8, cap=CAP, max_new=MAX_NEW, reps=REPS,
                   dev_median_tps=dev_tps, K_star=K_star, heldout_median_tps=tps,
                   heldout_tokens_per_forward={m: statistics.median(r["tok_per_fwd"] for r in held_rows if r["method"] == m)
                                               for m in held_methods},
                   chain_depth_acceptance=depth_acc, g1_heldout=g1_val, G1=g1, G3=g3,
                   rows=dev_rows + held_rows)
    RESULTS.write_text(json.dumps(summary, indent=2))
    print(f"HELD-OUT median tok/s: plain {tps['plain']:.1f} dspark {tps['dspark']:.1f} "
          + " ".join(f"rchain_{K} {tps[f'rchain_{K}']:.1f}" for K in KS))
    print(f"per-depth acceptance (K*={K_star}): {[round(a, 3) for a in depth_acc]}")
    print(f"G1 {g1} ({g1_val:.3f} >= {G1_GATE}) | G3 {g3}")
    print("GATE", "PASS" if (g1 and g3) else "FAIL")
    return 0 if (g1 and g3) else 1


if __name__ == "__main__":
    sys.exit(main())
