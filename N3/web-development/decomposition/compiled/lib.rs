//! B2 (Decomp+Prereg), Wave 3-FINAL — disjoint from B1's swarm DAG files.
//!
//! Task (a): [`decomposition`] — explorer→architect→coders→critic breakdown
//!   pattern as typed DAG nodes (input type, output type, judge, receipt per
//!   node). Nodes MUST typecheck before any node runs, in DAG order.
//! Task (b): [`preregistration`] — plan struct sealed into a hashed immutable
//!   signed struct. Any change = a new preregistration (new hash).
//!
//! Sources (read-only, Trash + N3 — never modified):
//!   Trash `skills-desktop-plugins-20261030/design/skills/preregister.md`,
//!   `test-suites.md`, `agentic-runbook/skills/runbook.md`,
//!   `design/skills/design-spec.md`; N3 `web-development/docs/` (Design.md,
//!   Pre-Registration.md, CLAIMS_REGISTER.md) as conventions ref; kit
//!   `SUBAGENT_SWARM_LOGIC.md` for the explorer→architect→coders→critic
//!   pattern being typed here.
//! Date: 2026-10-31. Status: COMPILED.

#![deny(warnings)]

pub mod decomposition;
pub mod preregistration;
