//! Model Home cognitive map — Slice 4 activation safety (Blueprint §K launch gates 4-5).
//! Slice 1 = pure reducer; Slice 2 = attempt loop; Slice 3 = read-only brief/map.
//! This crate adds ONLY the activation layer around them: a shadow gate that records
//! what the pipeline WOULD decide without touching learner state (gate 4), an explicit
//! human opt-in state machine (gate 5), offer-only placement, and a kill switch.
//! Nothing here writes reducer state except through Slice 1/2 paths the caller drives;
//! shadow entries are marked SHADOW and are never evidence (I07/I12).
pub mod optin;
pub mod shadow;
