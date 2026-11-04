#!/usr/bin/env bash
# Launch the live carved-swarm demo: isolated frozen-Glimmer server on 18111 + REPL.
# Run from a normal terminal (needs the Metal GPU). Never touches ports 8000/8011/4810.
#   ./run_swarm_live.sh            # preloads the built-in novel-facts doc, then gives you the prompt
#   SWARM_PRELOAD=/path/doc.txt ./run_swarm_live.sh   # carve your own document instead
set -euo pipefail
R="$(cd "$(dirname "$0")/../../.." && pwd)"
PORT=18111
if lsof -nP -iTCP:$PORT -sTCP:LISTEN >/dev/null 2>&1; then echo "port $PORT busy"; exit 1; fi
"$R/core_lm/target/release/serve_glimmer" 127.0.0.1:$PORT --model-dir "${GLIMMER_MODEL_DIR:-$HOME/models/Muse-Glimmer-q4km-gs128-MLX}" >/tmp/swarm-live-server.log 2>&1 &
SERVER=$!
trap 'kill $SERVER 2>/dev/null || true' EXIT
echo "loading frozen Muse Glimmer (about a minute)..."
until lsof -nP -iTCP:$PORT -sTCP:LISTEN >/dev/null 2>&1; do sleep 2; done
export ARES_GLIMMER_NATIVE_URL="http://127.0.0.1:$PORT"
export ARES_MCP_MIND_BIN="$R/globalmind-runtime/target/debug/globalmind-mcp"
export ARES_MCP_MIND_ROOT="$R"
export GLOBALMIND_SCHEMA_DIR="$R/hypermind-nested-kv/schema"
export SWARM_PRELOAD="${SWARM_PRELOAD:-demo}"
export SWARM_MAX_TOKENS="${SWARM_MAX_TOKENS:-12}"
export CARGO_TARGET_DIR="$R/globalmind-runtime/target/ares-backend"
cd "$R/Ares WebSite : App/backend"
cargo build --offline --release --example swarm_live 2>&1 | tail -1
"$CARGO_TARGET_DIR/release/examples/swarm_live"
