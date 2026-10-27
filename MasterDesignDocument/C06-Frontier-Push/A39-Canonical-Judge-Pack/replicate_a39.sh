#!/bin/bash
# A39 replication / cold-start run. Staged: requires the patched runner
# (runner_receipt_patch.md applied, cargo check clean) and a FROZEN corpus.
# Run on a second machine by copying this script + the frozen corpus +
# the preregistration amendment + the runner binary, and setting VAULT_SRC
# to that machine's vault path. Do NOT run against a live vault.
set -euo pipefail
: "${CORPUS:?set CORPUS=path/to/corpus_a39_frozen_v1.json}"
: "${RUNNER:?set RUNNER=path/to/ablation_runner (patched binary)}"
: "${VAULT_SRC:?set VAULT_SRC=path/to/source.atoma_map}"
: "${MODEL_SHARD_DIR:?set MODEL_SHARD_DIR=path/to/model shards}"
OUTDIR="${OUTDIR:-./a39_replication}"
mkdir -p "$OUTDIR"
THROWAWAY="$OUTDIR/throwaway_a39.atoma_map"
echo "corpus sha256:"; shasum -a 256 "$CORPUS"
echo "runner sha256:"; shasum -a 256 "$RUNNER"
echo "vault source sha256:"; shasum -a 256 "$VAULT_SRC"
cp "$VAULT_SRC" "$THROWAWAY"
echo "throwaway copy sha256:"; shasum -a 256 "$THROWAWAY"
# Cold start: fresh copy, no warm-up, no persist, no prior session history.
"$RUNNER" --arm control   --corpus "$CORPUS" --output "$OUTDIR/control.jsonl"   --seed 0xC0DECAFE --vault-copy "$THROWAWAY" --no-warmup --no-persist --model-shard-dir "$MODEL_SHARD_DIR"
"$RUNNER" --arm treatment --corpus "$CORPUS" --output "$OUTDIR/treatment.jsonl" --seed 0xC0DECAFE --vault-copy "$THROWAWAY" --no-warmup --no-persist --model-shard-dir "$MODEL_SHARD_DIR"
python3 analyze_a39.py "$OUTDIR/control.jsonl" "$OUTDIR/treatment.jsonl" --corpus "$CORPUS" --out "$OUTDIR/a39_report.json"
echo "throwaway hash after run:"; shasum -a 256 "$THROWAWAY"
# Post-proof deletion (alignment option B): delete the throwaway copy.
# Comment out this rm only if you are intentionally keeping it for audit,
# and record that decision in the report.
rm "$THROWAWAY" "$THROWAWAY.wal" 2>/dev/null || rm -f "$THROWAWAY"
echo "throwaway deleted. Report: $OUTDIR/a39_report.json"
