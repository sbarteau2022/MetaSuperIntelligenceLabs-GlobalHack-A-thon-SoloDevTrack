//! Map live mission shape tests (no server, no model).
//! Proves the mission that `POST /api/harness/mission/live` runs with
//! {"mission": "map"} is valid, uses only gate-seeded map.* tools, and
//! carries a registered judge for every packet.
use ares_backend::harness::mission::map_live_mission;
use ares_backend::harness::runtime::{CapabilityGate, CapabilityNode};

#[test]
fn map_live_mission_validates() {
    let input = map_live_mission();
    assert_eq!(input.plan.id, "mission-map-live-2026-11-03");
    input.plan.validate().expect("map live plan must validate");
    assert!(!input.judges.is_empty());
}

#[test]
fn map_live_mission_uses_only_seeded_gate_tools() {
    let mut gate = CapabilityGate::default();
    for tool in ["map.query", "map.kv_fetch", "map.ingest"] {
        gate.register(CapabilityNode {
            name: tool.to_string(),
            family: "map".to_string(),
            judged_atom: true,
            receipt_present: true,
        });
    }
    let input = map_live_mission();
    for packet in &input.plan.packets {
        assert!(
            !packet.judges.is_empty(),
            "packet {} has no judges",
            packet.id
        );
        for tc in &packet.tool_calls {
            assert!(tc.tool.starts_with("map."), "non-map tool {}", tc.tool);
            gate.admit(&tc.tool)
                .unwrap_or_else(|e| panic!("tool {} not admitted: {}", tc.tool, e));
            let v: serde_json::Value =
                serde_json::from_str(&tc.args_json).expect("args must be a JSON object");
            assert!(v.is_object(), "args must be a JSON object");
        }
        for jr in &packet.judges {
            assert!(
                input.judges.iter().any(|j| j.id == jr.id),
                "packet judge {} not registered",
                jr.id
            );
        }
    }
    assert_eq!(
        input.plan.packets[1].depends_on,
        vec!["packet-map-01".to_string()]
    );
}
