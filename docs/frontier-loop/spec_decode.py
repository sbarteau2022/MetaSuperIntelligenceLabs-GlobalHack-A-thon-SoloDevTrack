"""Greedy decoding: plain baseline and MTP speculative decoding (1 draft token per round).

Round invariant (p = position of the last committed token, `pending` = x_{p+1} already chosen, not yet in the
main cache, `h` = final-normed main hidden state at p):

  1. draft d = argmax head(h, pending)        (head's own KV cache gains entry p; committed-valid either way)
  2. snapshot main cache; run main model on [pending, d] -> logits_0 (predicts x_{p+2}), logits_1 (x_{p+3})
  3. accept if d == argmax logits_0:
       emit [pending, d]; pending = argmax logits_1; h = hidden at p+2;
       head cache gains entry p+1 = head(h_{p+1}, d)
     reject:
       restore snapshot (trim KV, reset recurrent state); emit [pending];
       run main model on [pending] alone -> pending = argmax, h = hidden at p+1

Each round costs one main forward (two tokens) on accept, two (block + re-run) on reject.
Output is greedy-identical to plain decoding up to fp differences between 1-token and 2-token forwards.
"""

import time

import mlx.core as mx
from mlx_lm.models.cache import ArraysCache, KVCache, QuantizedKVCache


def _argmax_last(logits: mx.array) -> int:
    return int(mx.argmax(logits[0, -1]).item())


def _snapshot(cache):
    snap = []
    for c in cache:
        if isinstance(c, (KVCache, QuantizedKVCache)):
            snap.append(("kv", c.offset))
        elif isinstance(c, ArraysCache):
            # ArraysCache.state is (list, left_padding, lengths); the list is mutated in place, so copy it.
            snap.append(("arr", (list(c.cache), c.left_padding, c.lengths)))
        else:
            raise TypeError(f"no rollback support for {type(c).__name__}")
    return snap


def _restore(cache, snap):
    for c, (kind, val) in zip(cache, snap):
        if kind == "kv":
            c.trim(c.offset - val)
        else:
            c.cache, c.left_padding, c.lengths = list(val[0]), val[1], val[2]


def greedy_decode(model, ids: mx.array, n: int, make_cache=None):
    """Plain greedy decode. Returns (tokens, seconds). Includes prefill."""
    lm = model.language_model
    cache = make_cache() if make_cache else model.make_cache()
    t0 = time.perf_counter()
    h = lm.model(ids, cache=cache)
    tok = _argmax_last(lm.lm_head(h[:, -1:]))
    out = [tok]
    while len(out) < n:
        h = lm.model(mx.array([[tok]]), cache=cache)
        tok = _argmax_last(lm.lm_head(h))
        out.append(tok)
    return out, time.perf_counter() - t0


def mtp_spec_decode(model, head, ids: mx.array, n: int, make_cache=None):
    """Greedy speculative decode with the 1-layer MTP head. Returns (tokens, seconds, stats)."""
    lm = model.language_model
    cache = make_cache() if make_cache else model.make_cache()
    head_cache = head.make_cache()
    t0 = time.perf_counter()

    # Prefill main model and head. Head entries cover committed positions 0..p-1 after prefill.
    h_all = lm.model(ids, cache=cache)  # [1, T, H]
    T = ids.shape[1]
    head(h_all[:, :-1], ids[:, 1:], cache=head_cache)
    pending = _argmax_last(lm.lm_head(h_all[:, -1:]))
    h = h_all[:, -1:]  # hidden at p = T-1

    out = []
    stats = {"drafted": 0, "accepted": 0, "forwards": 1}  # prefill counts as one forward
    while len(out) < n:
        # 1. draft
        d_logits = head(h, mx.array([[pending]]), cache=head_cache)
        d = _argmax_last(d_logits)
        stats["drafted"] += 1

        # 2. verify on the main model with a snapshot to roll back from
        snap = _snapshot(cache)
        block = mx.array([[pending, d]])
        h_blk = lm.model(block, cache=cache)  # [1, 2, H]
        stats["forwards"] += 1
        logits_blk = lm.lm_head(h_blk)  # [1, 2, V]
        t0_tok = int(mx.argmax(logits_blk[0, 0]).item())

        if t0_tok == d:
            # 3a. accept
            stats["accepted"] += 1
            out.extend([pending, d])
            pending = int(mx.argmax(logits_blk[0, 1]).item())
            # head entry for position p+1 = (h_{p+1}, d)
            head(h_blk[:, :1], mx.array([[d]]), cache=head_cache)
            h = h_blk[:, 1:2]
        else:
            # 3b. reject: roll back and advance one token
            _restore(cache, snap)
            stats["forwards"] += 1
            out.append(pending)
            h = lm.model(mx.array([[pending]]), cache=cache)
            pending = _argmax_last(lm.lm_head(h))
    return out[:n], time.perf_counter() - t0, stats




def mtp_chain_decode(model, head, ids: mx.array, n: int, K: int, make_cache=None):
    """Greedy speculative decode with K chained MTP drafts per round. Returns (tokens, seconds, stats).

    Round invariant: p = last committed position; `pending` = x_{p+1}, chosen but not yet in the main cache;
    `h` = final-normed main hidden state at p; the head cache holds entries for positions 0..p-1.
      1. chain-draft d_1..d_K as lazy MLX arrays (no host sync per depth). Step j feeds the head's previous
         state and the previous draft token. Head cache entries made while drafting are trimmed afterwards.
      2. snapshot the main cache; run the main model on [pending, d_1..d_K] (K+1 tokens); one host sync reads
         the K draft tokens and the K+1 verifier argmaxes.
      3. accept the longest prefix a with d_{i+1} == argmax(logits[i]) for all i < a. Emit [pending, d_1..d_a].
         The next pending is argmax(logits[a]). Causality makes that valid even though the block ran past a.
         If a < K: restore the snapshot and re-run [pending, d_1..d_a] to rebuild recurrent state and h at p+1+a.
      4. add head cache entries for committed positions p..p+a in one batched call:
         hidden [h_p, hidden(p+1)..hidden(p+a)] with tokens [pending, d_1..d_a].
    """
    lm = model.language_model
    cache = make_cache() if make_cache else model.make_cache()
    head_cache = head.make_cache()
    t0 = time.perf_counter()

    h_all = lm.model(ids, cache=cache)  # [1, T, H]
    head.hidden(h_all[:, :-1], ids[:, 1:], cache=head_cache)
    pending = _argmax_last(lm.lm_head(h_all[:, -1:]))
    h = h_all[:, -1:]  # hidden at p = T-1

    out = []
    stats = {"rounds": 0, "drafted": 0, "accepted": 0, "forwards": 1,
             "accepted_by_depth": [0] * K, "reached_depth": [0] * K}
    while len(out) < n:
        stats["rounds"] += 1
        h_p, pend_p = h, pending

        # 1. chain draft, lazily
        head_off = head_cache[0].offset
        state = h_p
        tok_arr = mx.array([[pend_p]], dtype=mx.int32)
        draft_arrs = []
        for _ in range(K):
            state = head.hidden(state, tok_arr, cache=head_cache)
            tok_arr = mx.argmax(head.lm_head(state)[:, -1], axis=-1, keepdims=True).astype(mx.int32)
            draft_arrs.append(tok_arr)
        head_cache[0].trim(head_cache[0].offset - head_off)

        # 2. verify with one main-model forward over K+1 tokens (single host sync)
        snap = _snapshot(cache)
        block = mx.concatenate([mx.array([[pend_p]], dtype=mx.int32)] + draft_arrs, axis=1)  # [1, K+1]
        h_blk = lm.model(block, cache=cache)
        preds_arr = mx.argmax(lm.lm_head(h_blk)[0], axis=-1)
        mx.eval(preds_arr, *draft_arrs)
        draft = [int(a_.item()) for a_ in draft_arrs]
        preds = preds_arr.tolist()
        stats["forwards"] += 1

        # 3. longest accepted prefix
        a = 0
        while a < K and draft[a] == preds[a]:
            a += 1
        stats["drafted"] += K
        stats["accepted"] += a
        for d in range(min(a + 1, K)):
            stats["reached_depth"][d] += 1
            if d < a:
                stats["accepted_by_depth"][d] += 1
        out.extend([pend_p] + draft[:a])

        if a < K:
            _restore(cache, snap)
            h_seq = lm.model(mx.array([[pend_p] + draft[:a]], dtype=mx.int32), cache=cache)  # [1, a+1, H]
            stats["forwards"] += 1
            new_h = h_seq[:, a:a + 1]  # hidden at position p+1+a
        else:
            new_h = h_blk[:, K:K + 1]  # hidden at position p+1+K
            h_seq = h_blk
        new_pending = preds[a]

        # 4. head entries for committed positions p..p+a
        if a > 0:
            committed_hidden = mx.concatenate([h_p, h_seq[:, :a]], axis=1)
        else:
            committed_hidden = h_p
        head.hidden(committed_hidden, mx.array([[pend_p] + draft[:a]], dtype=mx.int32), cache=head_cache)

        h, pending = new_h, new_pending

    return out[:n], time.perf_counter() - t0, stats


def mtp_chain_decode_rollback(target, head, ids: mx.array, n: int, K: int):
    """Chained MTP drafts with DSpark's hybrid rollback instead of snapshot-and-re-run.

    `target` is mlx_dspark's Target. Its verify capture records each gated-DeltaNet layer's recurrence inputs
    during the block forward. On a partial accept, target.rollback(cache, n_rejected, accepted) rebuilds the
    recurrent state at the accept point with the recurrence kernel only (no second model forward).
    Same round structure as mtp_chain_decode; the only change is step 2/3's restore-and-re-run.
    """
    model = target.model
    lm = model.language_model
    cache = target.make_cache()
    head_cache = head.make_cache()
    t0 = time.perf_counter()

    h_all = lm.model(ids, cache=cache)
    head.hidden(h_all[:, :-1], ids[:, 1:], cache=head_cache)
    pending = _argmax_last(lm.lm_head(h_all[:, -1:]))
    h = h_all[:, -1:]

    out = []
    stats = {"rounds": 0, "drafted": 0, "accepted": 0, "forwards": 1, "rollbacks": 0,
             "accepted_by_depth": [0] * K, "reached_depth": [0] * K}
    while len(out) < n:
        stats["rounds"] += 1
        h_p, pend_p = h, pending

        # 1. chain draft (lazy)
        head_off = head_cache[0].offset
        state = h_p
        tok_arr = mx.array([[pend_p]], dtype=mx.int32)
        draft_arrs = []
        for _ in range(K):
            state = head.hidden(state, tok_arr, cache=head_cache)
            tok_arr = mx.argmax(head.lm_head(state)[:, -1], axis=-1, keepdims=True).astype(mx.int32)
            draft_arrs.append(tok_arr)
        head_cache[0].trim(head_cache[0].offset - head_off)

        # 2. verify: one forward over [pending, d_1..d_K] with DSpark's recurrence capture
        block = mx.concatenate([mx.array([[pend_p]], dtype=mx.int32)] + draft_arrs, axis=1)
        target._stash = None
        with target._capture_linear() as stash:
            h_blk = lm.model(block, cache=cache)
        target._stash = stash
        target._spec_width = K + 1
        preds_arr = mx.argmax(lm.lm_head(h_blk)[0], axis=-1)
        mx.eval(preds_arr, *draft_arrs)
        draft = [int(a_.item()) for a_ in draft_arrs]
        preds = preds_arr.tolist()
        stats["forwards"] += 1

        # 3. longest accepted prefix; roll back the rejected tail lazily
        a = 0
        while a < K and draft[a] == preds[a]:
            a += 1
        stats["drafted"] += K
        stats["accepted"] += a
        for d in range(min(a + 1, K)):
            stats["reached_depth"][d] += 1
            if d < a:
                stats["accepted_by_depth"][d] += 1
        out.extend([pend_p] + draft[:a])
        if K - a > 0:
            stats["rollbacks"] += 1
        target.rollback(cache, K - a, draft[:a])
        new_h = h_blk[:, a:a + 1]          # hidden at position p+1+a, straight from the verify forward
        new_pending = preds[a]

        # 4. head entries for committed positions p..p+a
        if a > 0:
            committed_hidden = mx.concatenate([h_p, h_blk[:, :a]], axis=1)
        else:
            committed_hidden = h_p
        head.hidden(committed_hidden, mx.array([[pend_p] + draft[:a]], dtype=mx.int32), cache=head_cache)

        h, pending = new_h, new_pending

    return out[:n], time.perf_counter() - t0, stats
