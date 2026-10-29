---
name: optiq-harness
description: Route work through the local Model Home agent harness, its Rust/Python tool loop, skills, Postgres RAG, and bounded Zed MCP bridge.
---

# OptiQ harness workflow

Target repository: `/Users/stewartbarteau/Desktop/Model Home`.

Read its `AGENTICHARNESS.md` before every change. Route substantive tasks with
`ask_model_home_agent`; use its Rust/Python tools, skills, checkpoints, and RAG
rather than creating parallel unrestricted equivalents.

Required constraints:

- Keep Cargo offline when it needs dependency resolution.
- Do not run `npm install`, `pip`, `brew`, `sudo`, remote clients, disk tools, or arbitrary network tooling inside the harness.
- Do not load a model or import MLX. OptiQ already serves on `127.0.0.1:8000`; preserve its sampler card and do not start another server.
- Keep model sampling and xgrammar enforcement owned by oMLX.
- Keep the session root jailed. New tools must be owned by a skill and be bounded like the Rust host.

Use the existing workflows before ad hoc steps:

```sh
# Model Home/README.md — discovery and smoke verification
export PATH="/opt/homebrew/bin:$PATH"
uv run home status
cargo test -p home-core
npm run build
scripts/start-zed-agent-stack.sh --smoke
```

For browser checks, keep Playwright outside the harness and point it only at a local test surface. Record that separation in the test report.
