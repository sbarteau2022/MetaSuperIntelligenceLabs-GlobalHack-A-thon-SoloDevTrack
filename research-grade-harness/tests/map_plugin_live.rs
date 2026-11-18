#![cfg(feature = "live-map-plugin")]

use rgah::{MapPlugin, PluginRegistry, parse_json, write_json};
use std::path::Path;

#[test]
fn standalone_map_and_kv_plugin_roundtrip() {
    let binary = std::env::var("HYPERBOLIC_MAP_PLUGIN_BIN").expect("set HYPERBOLIC_MAP_PLUGIN_BIN");
    let root = std::env::temp_dir().join(format!("rgah-map-plugin-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut registry = PluginRegistry::new();
    let (mut plugin, loaded) = MapPlugin::load(Path::new(&binary), &root, &mut registry).unwrap();
    assert_eq!(loaded.op, "load");
    let ingest_args = parse_json(r#"{"text":"standalone plugin roundtrip evidence atom with enough distinct terms for the structural judge","provenance":"rgah-plugin-live"}"#).unwrap();
    let ingest = plugin
        .client_mut()
        .call("map.ingest", &ingest_args)
        .unwrap();
    let ingest_text = write_json(&ingest.inner);
    assert!(
        ingest.receipt_sha256.len() == 64 && ingest_text.contains("\"status\":\"ok\""),
        "{ingest_text}"
    );
    let store_args = parse_json(r#"{"key":"plugin/smoke","value":"real stored value"}"#).unwrap();
    let stored = plugin
        .client_mut()
        .call("map.kv_store", &store_args)
        .unwrap();
    let stored_text = write_json(&stored.inner);
    assert!(
        stored_text.contains("\"status\":\"stored\""),
        "{stored_text}"
    );
    let fetch_args = parse_json(r#"{"key":"plugin/smoke"}"#).unwrap();
    let fetched = plugin
        .client_mut()
        .call("map.kv_fetch", &fetch_args)
        .unwrap();
    let fetched_text = write_json(&fetched.inner);
    assert!(fetched_text.contains("real stored value"), "{fetched_text}");
    // error-contract probes: schema violation and unknown tool both Err.
    let empty = parse_json("{}").unwrap();
    assert!(plugin.client_mut().call("map.kv_store", &empty).is_err());
    assert!(
        plugin
            .client_mut()
            .call("map.no_such_tool", &empty)
            .is_err()
    );
    assert_eq!(plugin.unload(&mut registry).unwrap().op, "unload");
    assert!(registry.active.is_empty());
}
