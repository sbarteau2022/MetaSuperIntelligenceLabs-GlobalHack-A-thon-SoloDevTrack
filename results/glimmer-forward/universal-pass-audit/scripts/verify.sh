#!/bin/sh
set -eu
audit_tmp=$(mktemp -d)
trap 'rm -r "$audit_tmp"' EXIT
mkdir "$audit_tmp/a" "$audit_tmp/b"
printf '%s' '{"model":"same"}' > "$audit_tmp/a/config.json"
printf '%s' '{"model":"same"}' > "$audit_tmp/b/config.json"
printf '%s' '{"tokenizer":"same"}' > "$audit_tmp/a/tokenizer.json"
printf '%s' '{"tokenizer":"same"}' > "$audit_tmp/b/tokenizer.json"
printf '%s' 'weight-a' > "$audit_tmp/a/model.safetensors"
printf '%s' 'weight-b' > "$audit_tmp/b/model.safetensors"
seat_a=$( { cat "$audit_tmp/a/config.json"; printf '\000'; cat "$audit_tmp/a/tokenizer.json"; } | shasum -a 256 | awk '{print $1}')
seat_b=$( { cat "$audit_tmp/b/config.json"; printf '\000'; cat "$audit_tmp/b/tokenizer.json"; } | shasum -a 256 | awk '{print $1}')
weight_a=$(shasum -a 256 "$audit_tmp/a/model.safetensors" | awk '{print $1}')
weight_b=$(shasum -a 256 "$audit_tmp/b/model.safetensors" | awk '{print $1}')
test "$seat_a" = "$seat_b"
test "$weight_a" != "$weight_b"
printf 'seat_identity_counterexample=PASS same_seat=%s distinct_weight_digests=true\n' "$seat_a"
