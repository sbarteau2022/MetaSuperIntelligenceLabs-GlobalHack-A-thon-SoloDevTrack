# Model-neutral proposer image build, 2026-11-10

- Source commit: `c4d4974bed17f70fca74e139888fcf3880f1755b`.
- Build command: `git archive HEAD | colima ssh -- docker build --progress=plain -t ares-frontier-worker:c4d4974 -f frontier-worker/Dockerfile -`.
- Result: exit 0. Docker image ID: `sha256:3353b185ddea05db61fd9c3d0e9d4cdb6d1c16f3785e24712d26035f5a6ce763`; inspect size: `390662746` bytes.
- Source gates: frontier-worker `cargo check --tests`, `cargo test --all-targets`, strict Clippy, and format check passed. Backend `cargo check --tests`, `cargo test --all-targets`, strict Clippy, and format check passed.
- After build, Colima reported `MemAvailable: 424804 KiB`, below the supervisor's `786432 KiB` admission threshold. No idle Docker worker run was started. SurrealDB and ClickStack remained healthy; Qdrant remained running.
- This image contains the proposal protocol and canonical target fallback. It does not bundle a model proposer, generate a code candidate, or certify improvement.
