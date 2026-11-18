//! Runtime boundary for the independently installed HyperbolicMap/KV MCP plugin.

use crate::json::Json;
use crate::mcp_client::McpChild;
use crate::plugins::{PluginReceipt, PluginRegistry, PluginSpec};
use std::path::Path;

const REQUIRED_TOOLS: [&str; 5] = [
    "map.ingest",
    "map.query",
    "map.read_euclid",
    "map.kv_store",
    "map.kv_fetch",
];

pub struct MapPlugin {
    client: McpChild,
}

impl MapPlugin {
    /// Register only after the separate executable advertises both map and KV tools
    /// and returns a receipted health response.
    pub fn load(
        binary: &Path,
        root: &Path,
        registry: &mut PluginRegistry,
    ) -> Result<(Self, PluginReceipt), String> {
        if !binary.is_absolute() || !binary.is_file() || !root.is_absolute() {
            return Err(
                "plugin binary must be a file and binary/root paths must be absolute".into(),
            );
        }
        let mut client = McpChild::spawn(
            binary.to_str().ok_or("plugin binary path is not UTF-8")?,
            root.to_str().ok_or("plugin root path is not UTF-8")?,
        )?;
        for tool in REQUIRED_TOOLS {
            if !client.advertises(tool)? {
                return Err(format!("plugin missing required tool: {}", tool));
            }
        }
        let empty = Json::Obj(Default::default());
        let _ = client.call("map.health", &empty)?;
        let receipt = registry.load(PluginSpec {
            name: "hyperbolic-map-nested-kv".into(),
            version: "0.1.0".into(),
            depends_on: vec![],
            provides: REQUIRED_TOOLS.iter().map(|s| s.to_string()).collect(),
        })?;
        Ok((Self { client }, receipt))
    }

    pub fn client_mut(&mut self) -> &mut McpChild {
        &mut self.client
    }

    pub fn unload(mut self, registry: &mut PluginRegistry) -> Result<PluginReceipt, String> {
        let receipt = registry.unload("hyperbolic-map-nested-kv")?;
        self.client.close();
        Ok(receipt)
    }
}
