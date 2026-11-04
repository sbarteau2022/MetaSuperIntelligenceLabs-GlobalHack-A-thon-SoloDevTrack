"""Offline frozen-checkpoint oracle. Never imported by model serving or MCP.

Usage:
  python offline_glimmer_mlx_oracle.py MODEL_DIR MLX_LM_MODELS_DIR OUTPUT.bin

Requires mlx and the mlx_lm.models.base/cache source modules. The package
initializers are deliberately bypassed to avoid network/provider dependencies.
The token tape is pinned to the real Glimmer tokenizer result recorded in the
independent oracle receipt.
"""

import importlib.util
import json
import struct
import sys
import types
from pathlib import Path

import mlx.core as mx
import mlx.nn as nn


def load_module(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def main() -> None:
    if len(sys.argv) != 4:
        raise SystemExit(__doc__)
    root = Path(sys.argv[1]).resolve(strict=True)
    models_dir = Path(sys.argv[2]).resolve(strict=True)
    output = Path(sys.argv[3])
    for name, path in [
        ("mlx_lm", models_dir.parent),
        ("mlx_lm.models", models_dir),
    ]:
        package = types.ModuleType(name)
        package.__path__ = [str(path)]
        sys.modules[name] = package
    for name in ("base", "cache"):
        load_module(f"mlx_lm.models.{name}", models_dir / f"{name}.py")
    arch = load_module("muse_glimmer_mlx", root / "muse_glimmer_mlx.py")
    config = json.loads((root / "config.json").read_text())
    weights = {}
    shards = sorted(root.glob("model-*.safetensors"))
    if not shards:
        raise RuntimeError("checkpoint has no model shards")
    for shard in shards:
        weights.update(mx.load(str(shard)))
    model = arch.Model(arch.ModelArgs.from_dict(config))
    weights = model.sanitize(weights)
    quant = config["quantization"]

    def predicate(path, module):
        override = quant.get(path)
        if isinstance(override, dict):
            return override
        if not hasattr(module, "to_quantized") or f"{path}.scales" not in weights:
            return False
        scales = weights[f"{path}.scales"]
        in_dims = module.weight.shape[-1]
        group = in_dims // scales.shape[-1]
        bits = weights[f"{path}.weight"].shape[-1] * 32 // in_dims
        if scales.dtype != mx.uint8:
            mode = "affine"
        else:
            mode = {(4, 16): "nvfp4", (4, 32): "mxfp4", (8, 32): "mxfp8"}[
                (bits, group)
            ]
        return {"group_size": group, "bits": bits, "mode": mode}

    nn.quantize(
        model,
        group_size=quant["group_size"],
        bits=quant["bits"],
        mode=quant.get("mode", "affine"),
        class_predicate=predicate,
    )
    model.load_weights(list(weights.items()), strict=True)
    tokens = [200000, 954, 7963, 323, 11698, 373]
    logits = model(mx.array([tokens]))
    last = logits[0, -1].astype(mx.float32)
    mx.eval(last)
    values = last.tolist()
    output.write_bytes(b"".join(struct.pack("<f", value) for value in values))
    print(
        f"real_checkpoint=1 shards={len(shards)} logits={len(values)} "
        f"argmax={int(mx.argmax(last).item())} output={output}"
    )


if __name__ == "__main__":
    main()
