"""Step 2 gate: the 1-layer MTP head, fed by the main model's hidden states, predicts usable next tokens.

Pre-registered gate (set before the first measurement):
  - logits shape ends in vocab (248320)
  - top-1 agreement with the main model's own next-token top-1, two steps ahead, >= 0.30
    (chance is 1/248320 ~ 4e-6)

Run with the mlx-dspark python (it bundles mlx and mlx_lm):
  ~/.local/share/uv/tools/mlx-dspark/bin/python docs/frontier-loop/gates/step2_mtp_gate.py
"""

import pathlib
import sys

import mlx.core as mx
from mlx_lm import load

REPO = pathlib.Path("/Users/stewartbarteau/Desktop/Qwen3.6Cyber")
FRONTIER = REPO / "docs" / "frontier-loop"
sys.path.insert(0, str(FRONTIER))

from mtp_head import load_mtp_head, mtp_logits  # noqa: E402  (fails until implemented)

AGREEMENT_GATE = 0.30
SEQ_LEN = 512


def main() -> int:
    model, tok = load(str(REPO))
    lm = model.language_model

    text = (FRONTIER / "GOAL.md").read_text() + (FRONTIER / "LOG.md").read_text()
    ids = tok.encode(text)[:SEQ_LEN]
    x = mx.array([ids])
    T = x.shape[1]

    hidden = lm.model(x)  # [1, T, H], final-normed hidden states of the main model
    main_logits = lm.lm_head(hidden)  # [1, T, V]
    vocab = main_logits.shape[-1]

    head = load_mtp_head(REPO, model)
    # position t: input hidden h_t and token x_{t+1}; the head predicts x_{t+2}
    mtp = mtp_logits(head, hidden[:, :-1], x[:, 1:])  # [1, T-1, V]
    assert mtp.shape[-1] == vocab, f"MTP logits vocab {mtp.shape[-1]} != {vocab}"

    n = T - 2  # positions t where both x_{t+2} and main next-token prediction exist
    mtp_top = mx.argmax(mtp[:, :n], axis=-1)[0]
    main_top = mx.argmax(main_logits[:, 1 : n + 1], axis=-1)[0]  # main's prediction for x_{t+2}
    truth = x[0, 2 : n + 2]
    mx.eval(mtp_top, main_top, truth)

    agree = float((mtp_top == main_top).astype(mx.float32).mean())
    true_acc = float((mtp_top == truth).astype(mx.float32).mean())
    main_acc = float((main_top == truth).astype(mx.float32).mean())
    print(f"positions={n} vocab={vocab}")
    print(f"MTP top-1 agreement with main model (t+2): {agree:.3f}  gate >= {AGREEMENT_GATE}")
    print(f"MTP top-1 accuracy vs true x_(t+2):       {true_acc:.3f}")
    print(f"main top-1 accuracy vs true x_(t+2):      {main_acc:.3f}")
    print(f"chance: {1 / vocab:.2e}")

    passed = agree >= AGREEMENT_GATE
    print("GATE", "PASS" if passed else "FAIL")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
