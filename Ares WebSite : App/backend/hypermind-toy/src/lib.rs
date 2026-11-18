//! hypermind-toy: solo rehearsal of the backend governed RAG+memory path.
//!
//! Mirrors (subset, same semantics — see MIRROR.md):
//! - `mission`: Plan/Packet validation, Judge (4 kinds), chain-authenticated
//!   ReceiptLog, EnergyBudget, Block/WorkingSetCache, research_gate.
//! - `runtime`: CapabilityGate, Executor boundary, Engine (judge_output,
//!   sequential + live-sequential execution, mission entry points).
//! - `mind`: toy Lorentz H3 atoms, shell projection, Lorentz inner product,
//!   VP-tree exact-prune kNN, data-dependent depth gate
//!   (cost = ln(k)+k − qmass·8), capability leaves embedded by problem text,
//!   KV store with producer edges + exact invalidation.
//!
//! What is NOT mirrored (out of scope, named so the port does not assume):
//! compliance permits, SurrealDB/Qdrant/HyperDX stores, SSE/HTTP, oMLX seats,
//! Merkle vault, carved views, statutory registry. The toy's `Executor` is a
//! scripted stand-in with 64-hex digests, never a model call.

pub mod mind;
pub mod mission;
pub mod runtime;
