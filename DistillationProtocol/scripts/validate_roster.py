#!/usr/bin/env python3
"""Validate a model roster JSON before one-pass distillation."""

import json
import sys

REQUIRED = ["name", "path", "quant", "num_layers"]


def main():
    if len(sys.argv) != 2:
        print("usage: validate_roster.py <roster.json>", file=sys.stderr)
        sys.exit(2)
    with open(sys.argv[1]) as f:
        try:
            data = json.load(f)
        except json.JSONDecodeError as e:
            print("invalid JSON: {}".format(e), file=sys.stderr)
            sys.exit(1)
    models = data.get("models", data if isinstance(data, list) else [])
    errors = []
    if not models:
        errors.append(
            "roster must contain >= 1 model (PROTOCOL.md section 4 hold point)"
        )
    for i, m in enumerate(models):
        for k in REQUIRED:
            if k not in m:
                errors.append("model[{}] missing '{}'".format(i, k))
        if "scope" not in m:
            m["scope"] = "private"
        if not isinstance(m.get("num_layers"), int):
            errors.append("model[{}] num_layers must be int".format(i))
    if errors:
        for e in errors:
            print(e, file=sys.stderr)
        sys.exit(1)
    print(
        json.dumps(
            {
                "status": "ok",
                "models": len(models),
                "shared": sum(1 for m in models if m.get("scope") == "shared"),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
