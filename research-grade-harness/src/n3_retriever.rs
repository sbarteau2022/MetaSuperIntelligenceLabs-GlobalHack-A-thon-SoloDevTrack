//! Evidence source backed by N3 through the hypermind-mcp child over stdio.
//! `lookup` calls `map.query`; `ingest` calls `map.ingest`. Both return receipted results.

use crate::agent_step::Retriever;
use crate::json::Json;
use crate::mcp_client::McpChild;
use std::collections::BTreeMap;

const SWARM_RUN_ID: &str = "rgah-loop-retriever";
const TOP_K: i64 = 3;

pub struct N3Retriever {
    child: McpChild,
}

impl N3Retriever {
    /// Spawns hypermind-mcp in isolated mode (`HYPERMIND_SURREAL=off`), never the persistent authority.
    pub fn spawn(mcp_path: &str, root: &str) -> Result<Self, String> {
        let child = McpChild::spawn_with_env(mcp_path, root, &[("HYPERMIND_SURREAL", "off")])?;
        Ok(Self { child })
    }

    pub fn ingest(&mut self, text: &str) -> Result<(), String> {
        let mut args = BTreeMap::new();
        args.insert("text".to_string(), Json::Str(text.to_string()));
        args.insert("swarm_run_id".to_string(), Json::Str(SWARM_RUN_ID.to_string()));
        let out = self.child.call("map.ingest", &Json::Obj(args))?;
        if out.receipt_sha256.len() != 64 {
            return Err("map.ingest returned no receipt".into());
        }
        Ok(())
    }
}

impl Retriever for N3Retriever {
    fn lookup(&mut self, query: &str) -> Result<String, String> {
        let mut args = BTreeMap::new();
        args.insert("qtext".to_string(), Json::Str(query.to_string()));
        args.insert("k".to_string(), Json::Int(TOP_K));
        let out = self.child.call("map.query", &Json::Obj(args))?;
        if out.receipt_sha256.len() != 64 {
            return Err("map.query returned no receipt".into());
        }
        hit_texts(&out.inner)
    }
}

/// Plain-text evidence from a `map.query` result: each hit's `text`, one per line.
/// Scores, coordinates, and receipts stay in the typed object and never reach the prompt.
fn hit_texts(inner: &Json) -> Result<String, String> {
    let Some(Json::Arr(hits)) = inner.get("hits") else {
        return Err("map.query result has no hits array".into());
    };
    let lines: Vec<&str> = hits.iter().filter_map(|h| h.get("text").and_then(Json::as_str)).collect();
    Ok(lines.join("\n"))
}
