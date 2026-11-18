"""Step 3 matched gate: plain greedy, MTP speculative, and DSpark, all in ONE process with the same settings.

Why this exists: the first Step 3 run compared an in-process MTP harness (bf16 KV, no serving patches)
against the HTTP DSpark server (kv-bits 8, fp16 activations, serving patches). This script removes that
confound. It applies the same environment as serve.sh (QWEN_* variables), loads the same target and drafter
through mlx_dspark's own loaders, applies the same kernel patches as the CLI, and uses kv-bits 8 for every method.

Pre-registered gate (the criteria are unchanged from the first run; only the setup is matched):
  G1 correctness: teacher-forced, every MTP token is within 0.5 logits of the reference argmax (no-cache forward
     over prompt+output). Gate >= 0.98.
  G2 acceptance: p_mtp = accepted/drafted (MTP drafts 1 per round). p_dspark = (accept_len - 1)/cap, cap = 4.
     Gate: p_mtp >= p_dspark - 0.05. Also reported: tokens per main-model forward for both (the
     like-for-like number; DSpark's per-slot figure is an approximation).
  G3 speed: median tokens/s (prefill included) of MTP > DSpark, on the same prompts, same process.

Run:  ~/.local/share/uv/tools/mlx-dspark/bin/python docs/frontier-loop/gates/step3_matched_gate.py
Writes gates/step3_matched_results.json.
"""

import json
import os
import pathlib
import statistics
import sys

ENV = {
    "QWEN_PREFIX_CACHE_MAX_BYTES": "8589934592",
    "QWEN_SINGLE_CONVERSATION_SNAPSHOT": "1",
    "QWEN_INTERIOR_CHECKPOINT_MAX_TOKENS": "32768",
    "QWEN_LONG_CONTEXT": "1",
    "QWEN_FP16_ACTIVATIONS": "1",
    "QWEN_PREFILL_CHUNK_TOKENS": "512",
    "QWEN_ACTIVE_KV_MAX_BYTES": "8589934592",
}
for k, v in ENV.items():  # same environment as serve.sh
    os.environ.setdefault(k, v)

REPO = pathlib.Path("/Users/stewartbarteau/Desktop/Qwen3.6Cyber")
FRONTIER = REPO / "docs" / "frontier-loop"
GATES = FRONTIER / "gates"
sys.path.insert(0, str(GATES))
sys.path.insert(0, str(FRONTIER))
sys.path.insert(0, "/Users/stewartbarteau/.local/share/qwen38-performance")

import mlx.core as mx  # noqa: E402

from step3_prompts import MAX_NEW, PROMPTS  # noqa: E402
from mtp_head import load_mtp_head  # noqa: E402
from spec_decode import greedy_decode, mtp_spec_decode  # noqa: E402

MATCH_GATE = 0.98
TIE_TOL = 0.5
ACCEPT_SLACK = 0.05
CAP = 4
RESULTS = GATES / "step3_matched_results.json"
REPS = 2


def setup():
    """Mirror cli.py's dspark path, with the serving-layer patches from run_server.py that affect generation."""
    import run_server  # noqa: F401  (module import only; serving-layer hooks are not installed here)
    from long_context import install as install_long_context
    from mlx_dspark.calibrate import (apply_cpu_split, apply_multirow_attn, apply_sdpa_split,
                                      apply_small_m, apply_wide_gemm)
    from mlx_dspark.generate import speculative_generate, encode_prompt
    from mlx_dspark.load import (load_drafter, load_target, lookup_drafts_default,
                                 resolve_drafter_window, resolve_mode)

    install_long_context()
    mode, target_repo, drafter_repo = resolve_mode(str(REPO), mode="dspark", drafter="Koopah/Qwen3.6-35B-A3B-NVFP4-DSPARK",
                                                   family=None, target=None)
    target, tok = load_target(target_repo, require_tap=True, kv_bits=8)
    drafter, _ = load_drafter(drafter_repo, quantize=True, bits=4)
    drafter.ctx_window = resolve_drafter_window(None, target_repo)
    apply_small_m(target, drafter, target_repo=target_repo, drafter_repo=drafter_repo, enabled=None)
    apply_sdpa_split(target, target_repo=target_repo, enabled=None)
    apply_multirow_attn(target, drafter, enabled=None)
    apply_wide_gemm(target, drafter, target_repo=target_repo, min_rows=None)
    apply_cpu_split(target, drafter, target_repo=target_repo, frac=None)
    lookup = lookup_drafts_default(target_repo)
    print(f"target={target_repo}\ndrafter={drafter_repo}\nlookup_drafts={lookup}")
    return target, tok, drafter, speculative_generate, encode_prompt, lookup, target_repo, drafter_repo


def reference_ok(model, ids_list, out_tokens):
    """Teacher-forced check: fraction of output tokens within TIE_TOL of the no-cache argmax."""
    lm = model.language_model
    seq = mx.array([ids_list + out_tokens])
    logits = lm.lm_head(lm.model(seq))[0]
    T0 = len(ids_list)
    ok = 0
    for j, tok in enumerate(out_tokens):
        row = logits[T0 - 1 + j]
        if float(row[tok].item()) >= float(mx.max(row).item()) - TIE_TOL:
            ok += 1
    return ok / len(out_tokens)


def main() -> int:
    target, tok, drafter, speculative_generate, encode_prompt, lookup, target_repo, drafter_repo = setup()
    model = target.model
    make_cache = target.make_cache
    head = load_mtp_head(REPO, model, dtype=mx.float16)  # match the main model's fp16 activations

    prompts = [encode_prompt(tok, msgs[0]["content"], use_chat=True) for msgs in PROMPTS]

    def run_plain(ids):
        out, secs = greedy_decode(model, mx.array([ids]), MAX_NEW, make_cache=make_cache)
        return dict(tokens=out, seconds=secs, forwards=len(out))

    def run_mtp(ids):
        out, secs, st = mtp_spec_decode(model, head, mx.array([ids]), MAX_NEW, make_cache=make_cache)
        return dict(tokens=out, seconds=secs, forwards=st["forwards"], drafted=st["drafted"],
                    accepted=st["accepted"])

    def run_dspark(content):
        res = speculative_generate(target, tok, drafter, content, max_new_tokens=MAX_NEW,
                                   max_draft_tokens=CAP, lookup_drafts=lookup, lookup_long_draft=32,
                                   confidence_threshold=0.0, temperature=0, top_p=1, top_k=1, seed=0,
                                   apply_chat_template=True, on_text=None)
        return dict(tokens=res.token_ids, seconds=res.seconds, forwards=res.target_forwards,
                    accept_len=res.mean_accept_len, num_tokens=res.num_tokens)

    # Warm-up: kernels compile on first use; do not time it.
    run_plain(prompts[0][:64] if len(prompts[0]) > 64 else prompts[0])
    run_mtp(prompts[0])
    run_dspark(PROMPTS[0][0]["content"])

    rows = []
    for rep in range(REPS):
        for i, (ids, msgs) in enumerate(zip(prompts, PROMPTS)):
            content = msgs[0]["content"]
            # Interleave methods per prompt so drift in the machine state affects all three alike.
            for name, fn in (("plain", lambda: run_plain(ids)),
                             ("mtp", lambda: run_mtp(ids)),
                             ("dspark", lambda: run_dspark(content))):
                r = fn()
                n = len(r["tokens"]) if name != "dspark" else r["num_tokens"]
                row = dict(rep=rep, prompt=i, method=name, tokens=n, seconds=r["seconds"],
                           tps=n / r["seconds"], forwards=r["forwards"], tok_per_fwd=n / r["forwards"])
                if name == "mtp":
                    row.update(p_mtp=r["accepted"] / r["drafted"], drafted=r["drafted"], accepted=r["accepted"])
                    row["g1_teacher_forced"] = reference_ok(model, ids, r["tokens"][:n])
                if name == "dspark":
                    row.update(accept_len=r["accept_len"], p_dspark=(r["accept_len"] - 1) / CAP)
                rows.append(row)
                print(json.dumps({k: (round(v, 4) if isinstance(v, float) else v) for k, v in row.items()}))

    def med(name, key):
        return statistics.median(r[key] for r in rows if r["method"] == name)

    tps = {m: med(m, "tps") for m in ("plain", "mtp", "dspark")}
    tpf = {m: med(m, "tok_per_fwd") for m in ("plain", "mtp", "dspark")}
    mtp_rows = [r for r in rows if r["method"] == "mtp"]
    dsp_rows = [r for r in rows if r["method"] == "dspark"]
    p_mtp = sum(r["accepted"] for r in mtp_rows) / sum(r["drafted"] for r in mtp_rows)
    p_dspark = sum(r["p_dspark"] for r in dsp_rows) / len(dsp_rows)
    g1_val = sum(r["g1_teacher_forced"] for r in mtp_rows) / len(mtp_rows)

    g1 = g1_val >= MATCH_GATE
    g2 = p_mtp >= p_dspark - ACCEPT_SLACK
    g3 = tps["mtp"] > tps["dspark"]

    summary = dict(target=target_repo, drafter=drafter_repo, kv_bits=8, cap=CAP, max_new=MAX_NEW, reps=REPS,
                   median_tps=tps, median_tokens_per_forward=tpf, p_mtp=p_mtp, p_dspark=p_dspark,
                   g1_teacher_forced=g1_val, G1=g1, G2=g2, G3=g3, rows=rows)
    RESULTS.write_text(json.dumps(summary, indent=2))
    print("\nmedian tok/s:", {k: round(v, 1) for k, v in tps.items()})
    print("median tokens per main forward:", {k: round(v, 2) for k, v in tpf.items()})
    print(f"p_mtp {p_mtp:.3f} vs p_dspark {p_dspark:.3f} (gate >= {p_dspark - ACCEPT_SLACK:.3f})")
    print(f"G1 {g1} ({g1_val:.3f}) | G2 {g2} | G3 {g3}")
    print("GATE", "PASS" if (g1 and g2 and g3) else "FAIL")
    return 0 if (g1 and g2 and g3) else 1


if __name__ == "__main__":
    sys.exit(main())
