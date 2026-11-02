//! Model Home cognitive map — Slice 3 crate root.
//! Slice 1 (`cogmap`) is the pure policy reducer; Slice 2 (`cogmap_slice2`)
//! is the Studio attempt loop. This crate adds only READ-ONLY views over
//! reducer state: `cogmap_brief` (structured teaching brief for the model)
//! and `cogmap_map` (inspectable map projection). Neither writes state:
//! the model receives JSON and can only speak (Blueprint §G, I07/I08).
pub mod brief;
