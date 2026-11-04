//! RG-AH — the research-grade harness, as native backend modules.
//!
//! Ported design (implemented backend-native, never linked against the
//! `rgah` crate): runbook, judge, receipt, mission, energy, policy_kernel,
//! cache, session, gate, HyperDX telemetry exporter, and the Qdrant memory
//! store. The harness is the run
//! runtime; governance (`crate::compliance`) is the law over it:
//!
//! - Every mission run passes under a compliance permit (spawn gate).
//! - Every packet with a `map.*` tool call is capability-gated (GAP rule).
//! - No executor → ESCALATE receipts, never PASS. No synthetic runs, ever.
//! - Receipts are chain-authenticated (GENESIS origin, recomputed links);
//!   a forged prior can never mark a packet passed on resume.
//!
//! Runtime framework: the Rust & Tokio systems-engineering doctrine —
//! bounded concurrency (semaphores), `spawn_blocking` for blocking I/O,
//! explicit timeouts and byte caps, telemetry exported append-only into
//! HyperDX as OTLP traces/logs/metrics, memory persisted append-only into
//! Qdrant (research RAG, session summaries, and searchable memories),
//! secrets resolved at call time from 0600 `secrets.env`, never serialized.

pub mod automaton;
pub mod baseline;
pub mod glimmer_tokenizer;
pub mod mcp;
pub mod mcp_server;
pub mod mind_recall_executor;
pub mod mission;
pub mod model_seat;
pub mod native_mtp;
pub mod output_receipt;
pub mod qdrant_memory;
pub mod qwen_vl_native;
pub mod runtime;
pub mod scope;
pub mod telemetry;
pub mod tppe;
pub mod tppe_memory;
