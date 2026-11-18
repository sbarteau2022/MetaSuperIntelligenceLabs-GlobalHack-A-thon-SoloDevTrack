//! Ares backend library: Phase 0 core governance + governed dispatch.
//!
//! Governance is baked in, not added on: no dispatch, no memory write, no
//! model call, no erasure, and no run ships without a compliance permit, an
//! audit-chain entry, and a Merkle anchor — from the first skeleton build
//! onward. See the core governance spec (2026-11-02).

pub mod compliance;
pub mod digest;
pub mod first_principles;
pub mod fixed;
pub mod governor;
pub mod harness;
pub mod http;
pub mod merkle;
pub mod mlir;
pub mod roles;
pub mod store;
pub mod surreal;
