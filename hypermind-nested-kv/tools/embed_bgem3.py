#!/usr/bin/env python3
"""Frozen BGE-M3 canonical text embedder (step 5).

Contract (matches schema/0002 + docs/steps-5):
- model: BAAI/bge-m3 @ 5617a9f61b028005a4858fdac845db406aefb181 (local only, never refetch)
- output: 1024D DENSE CLS vector, L2-normalized (NOT sparse, NOT colbert)
- batch JSONL in, JSONL out: {"id": <ext>, "vector": [1024 floats], "digest": <sha256 of text>}
- determinism: eval mode, no grad, fixed max_length=512, CPU float32
- teacher identity never enters the vector: input is atom text only

Usage: embed_bgem3.py --in atoms.jsonl --out vectors.jsonl
  input line:  {"id": "1", "text": "..."}
"""
import argparse, hashlib, json, sys

MODEL_DIR = "/Users/stewartbarteau/models/bge-m3-5617a9f"
EXPECTED_REV = "5617a9f61b028005a4858fdac845db406aefb181"

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="inp", required=True)
    ap.add_argument("--out", dest="out", required=True)
    ap.add_argument("--batch", type=int, default=16)
    args = ap.parse_args()

    import torch
    from transformers import AutoModel, AutoTokenizer

    tok = AutoTokenizer.from_pretrained(MODEL_DIR, local_files_only=True, trust_remote_code=False)
    mdl = AutoModel.from_pretrained(MODEL_DIR, local_files_only=True, trust_remote_code=False)
    mdl.eval()
    for p in mdl.parameters():
        p.requires_grad_(False)

    rows = []
    with open(args.inp, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                rows.append(json.loads(line))

    n = 0
    with open(args.out, "w", encoding="utf-8") as out:
        with torch.no_grad():
            for i in range(0, len(rows), args.batch):
                chunk = rows[i:i + args.batch]
                texts = [r["text"] for r in chunk]
                enc = tok(texts, padding=True, truncation=True, max_length=512, return_tensors="pt")
                hidden = mdl(**enc).last_hidden_state  # [B, T, 1024]
                cls = hidden[:, 0, :]                  # CLS pooling (BGE-M3 dense head input)
                norm = cls.norm(p=2, dim=1, keepdim=True).clamp_min(1e-12)
                vecs = (cls / norm).tolist()
                for r, v in zip(chunk, vecs):
                    assert len(v) == 1024, f"expected 1024D, got {len(v)}"
                    digest = hashlib.sha256(r["text"].encode("utf-8")).hexdigest()
                    out.write(json.dumps({"id": r["id"], "vector": v, "digest": digest}) + "\n")
                    n += 1
    print(f"embedded={n} encoder=bge-m3 rev={EXPECTED_REV}", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
