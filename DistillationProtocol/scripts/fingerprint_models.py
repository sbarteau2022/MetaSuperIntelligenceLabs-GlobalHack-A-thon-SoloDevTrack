#!/usr/bin/env python3
"""Fingerprint model shards into receipts (prep-side only: reads weights, writes receipts).

Scope rule (Muse receipt precedent): shards only (*.safetensors), sorted
filename order, per-file sha256 + combined sha256 over the concatenation.
Writes DistillationProtocol/<name>_fingerprint_receipt.json. Never touches
weights, engine, or map. Run: python3 fingerprint_models.py [--only NAME]
"""

import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).parent
ROSTER = HERE.parent / "model_roster.json"

# Extra on-disk models not (yet) on the roster.
EXTRA = {
    "DeepSeek-V4-Pro": "/Volumes/WD Drive/models/DeepSeek-V4-Pro-4bit",
}

SKIP_SUFFIXES = ("mtp", "rapid")  # auxiliary shards, not teacher weights


def fingerprint_dir(name, path):
    p = Path(path)
    shards = sorted(
        f
        for f in p.glob("model*.safetensors")
        if not any(s in f.name for s in SKIP_SUFFIXES)
    )
    if not shards:
        return {
            "model": name,
            "path": str(p),
            "status": "NO_SHARDS",
            "note": "config-only or unsupported layout",
        }
    files, total, h = [], 0, hashlib.sha256()
    for f in shards:
        fh = hashlib.sha256()
        n = 0
        with open(f, "rb") as fp:
            for chunk in iter(lambda: fp.read(1 << 22), b""):
                fh.update(chunk)
                h.update(chunk)
                n += len(chunk)
        files.append({"file": f.name, "sha256": fh.hexdigest(), "bytes": n})
        total += n
    return {
        "model": name,
        "path": str(p),
        "scope": "shards only (*.safetensors), concatenated in sorted filename order",
        "files": files,
        "combined_sha256": "sha256:" + h.hexdigest(),
        "total_bytes": total,
        "method": "sha256 per shard + running combined (4MB chunks)",
    }


def main():
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    roster = json.loads(ROSTER.read_text())["models"]
    targets = [(m["name"], m["path"]) for m in roster]
    targets += [(k, v) for k, v in EXTRA.items()]
    for name, path in targets:
        if only and name != only:
            continue
        print(f"hashing {name} @ {path} ...", flush=True)
        try:
            rec = fingerprint_dir(name, path)
        except Exception as e:
            print(f"  ERROR: {e}")
            continue
        if rec.get("status") == "NO_SHARDS":
            print(f"  SKIP: {rec['note']}")
            continue
        out = (
            ROSTER.parent
            / f"{name.lower().replace('.', '').replace('-', '_')}_fingerprint_receipt.json"
        )
        out.write_text(json.dumps(rec, indent=1))
        print(
            f"  {len(rec['files'])} shards, {rec['total_bytes'] / 1e9:.1f} GB, "
            f"{rec['combined_sha256'][:23]}... -> {out.name}"
        )


if __name__ == "__main__":
    main()
