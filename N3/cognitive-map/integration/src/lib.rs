//! Model Home cognitive map — INTEGRATION slice (PLAN wiring layer).
//! Slice 1 = pure reducer + schema; Slice 2 = Studio attempt loop; Slice 3 =
//! read-only brief/map; Slice 4 = shadow gate + opt-in + kill switch. This
//! crate wires them together: persist.rs maps reducer events / assessments /
//! observations to the `cogmap` schema and back (plus Slice 4 activation
//! persistence), and tools.rs exposes the four PLAN belt tools
//! (cogmap_brief / cogmap_predict / cogmap_submit / cogmap_map) behind the
//! Activation guards. NO live database and NO home-core dependency here:
//! executing SQL and adapting CogmapTool onto the real home-core Tool trait
//! are Mac-side steps (see README, labelled unverified).
pub mod json;
pub mod persist;
pub mod tools;
