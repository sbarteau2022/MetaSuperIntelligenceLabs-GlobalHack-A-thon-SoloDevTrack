#!/usr/bin/env bash
# One-page UI (IDE | chat | telemetry) on the live carved swarm. Loopback only: http://127.0.0.1:4820
# Starts an isolated frozen-Glimmer server on 18111 (never touches 8000/8011/4810), preloads the demo document.
set -euo pipefail
R="$(cd "$(dirname "$0")/../../.." && pwd)"
PORT=18111
if lsof -nP -iTCP:$PORT -sTCP:LISTEN >/dev/null 2>&1; then echo "port $PORT busy"; exit 1; fi
"$R/core_lm/target/release/serve_glimmer" 127.0.0.1:$PORT --model-dir "${GLIMMER_MODEL_DIR:-$HOME/models/Muse-Glimmer-q4km-gs128-MLX}" >/tmp/swarm-ui-server.log 2>&1 &
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
cargo build --offline --release --example swarm_live --example swarm_ui 2>&1 | tail -1
export REPO_ROOT="$R" SWARM_LIVE_BIN="$CARGO_TARGET_DIR/release/examples/swarm_live"
"$CARGO_TARGET_DIR/release/examples/swarm_ui"
