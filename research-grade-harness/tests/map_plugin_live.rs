#![cfg(feature = "live-map-plugin")]

use rgah::{MapPlugin, PluginRegistry};
use std::path::Path;

#[test]
fn standalone_map_and_kv_plugin_roundtrip() {
    let binary = std::env::var("HYPERBOLIC_MAP_PLUGIN_BIN").expect("set HYPERBOLIC_MAP_PLUGIN_BIN");
    let root = std::env::temp_dir().join(format!("rgah-map-plugin-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut registry = PluginRegistry::new();
    let (mut plugin, loaded) = MapPlugin::load(Path::new(&binary), &root, &mut registry).unwrap();
    assert_eq!(loaded.op, "load");
    let (ingest, has_receipt, _) = plugin.client_mut().call(
        "map.ingest",
        r#"{"text":"standalone plugin roundtrip evidence atom with enough distinct terms for the structural judge","provenance":"rgah-plugin-live"}"#,
    ).unwrap();
    assert!(
        has_receipt && ingest.contains("\"status\":\"ok\""),
        "{ingest}"
    );
    let (stored, _, _) = plugin
        .client_mut()
        .call(
            "map.kv_store",
            r#"{"key":"plugin/smoke","value":"real stored value"}"#,
        )
        .unwrap();
    assert!(stored.contains("\"status\":\"stored\""), "{stored}");
    let (fetched, _, _) = plugin
        .client_mut()
        .call("map.kv_fetch", r#"{"key":"plugin/smoke"}"#)
        .unwrap();
    assert!(fetched.contains("real stored value"), "{fetched}");
    assert!(plugin.client_mut().call("map.kv_store", "{}").is_err());
    assert!(plugin.client_mut().call("map.no_such_tool", "{}").is_err());
    assert_eq!(plugin.unload(&mut registry).unwrap().op, "unload");
    assert!(registry.active.is_empty());
}
