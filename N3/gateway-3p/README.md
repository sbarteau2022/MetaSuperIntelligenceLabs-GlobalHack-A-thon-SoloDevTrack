# Claude Desktop → Rapid MLX gateway

This loopback-only Rust service preserves Claude Desktop's third-party inference UI while translating Anthropic Messages requests to Rapid MLX's OpenAI-compatible API.

## Contract

- Claude endpoint: `http://127.0.0.1:4001/v1/messages`
- Rapid MLX endpoint: `http://127.0.0.1:8001/v1/chat/completions`
- Desktop models: local Qwen (`claude-opus-4`), `Beam-1.0-501B-A23B`, `muse-spark-1.3-contributor`, and `nvidia/nemotron-3-ultra-550b-a55b`
- Provider routes: Rapid MLX on loopback, Reflection AI, Meta AI, and NVIDIA Integrate
- Provider credential: `GATEWAY_RAPID_MLX_KEY` (never logged)
- Both services bind to loopback only.

## Provider keys

Keys live only in `~/.config/gateway-3p/keys.env` with mode `0600`. Set or rotate one without putting it in shell history:

```sh
/Users/stewartbarteau/Desktop/N3/gateway-3p/scripts/set-key.sh beam
/Users/stewartbarteau/Desktop/N3/gateway-3p/scripts/set-key.sh muse
/Users/stewartbarteau/Desktop/N3/gateway-3p/scripts/set-key.sh nemotron
```

The prompt disables terminal echo, atomically replaces that provider's entry, and restarts the gateway. Key values are never logged.

Streaming must preserve Anthropic event order: `content_block_start` precedes every tool-input delta, tool blocks close before `message_delta`, and `message_stop` is last. This ordering is required for Claude Code to execute a tool and continue into the next model turn.

Claude Code plugin/MCP tool names that exceed the OpenAI 64-character function-name limit are converted to deterministic provider aliases and mapped back before Claude executes them. Tool history uses the same mapping across subsequent turns.

Beam follow-up turns also receive the prior assistant reasoning echoed as `reasoning_content`, as required by Reflection's multi-turn contract.

## Verify

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
curl -fsS http://127.0.0.1:4001/health
```

The launch agents are `com.stewartbarteau.rapid-mlx` and `com.stewartbarteau.gateway-3p`. Rapid MLX must be healthy before a Claude Code session starts.

The Rapid service uses the measured local workhorse checkpoint in correctness-first mode: one sequence, speculative decoding disabled, native Qwen tool parsing enabled, Claude Code mid-conversation system reminders preserved, and GPU memory capped at 75%. `RAPID_MLX_CONSTRAIN_TOOLS=0` keeps Claude Code's large second-turn plugin schema on Rapid's documented free-form fallback instead of rejecting it at the 64 KiB grammar compiler limit. The previous oMLX agents remain installed but unloaded, so rollback is reversible.
