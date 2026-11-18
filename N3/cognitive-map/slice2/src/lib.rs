//! Model Home cognitive map — Slice 2 crate root.
//! Slice 1 (`cogmap`) stays the pure policy reducer; this crate adds the
//! Studio attempt loop around it without weakening any Slice 1 invariant.
pub mod attempt_loop;
pub mod hash;
