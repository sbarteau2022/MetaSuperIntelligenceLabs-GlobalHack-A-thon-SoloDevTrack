//! Map-owned atom ledger.
//!
//! The map's stored atoms live here and in `map.snapshot.bin`, nowhere else.
//! This replaces the old atomadb `mind.jsonl` write: every ingest appends
//! one JSON line here (O(1), immediate durability), and open/recover replay
//! it. The RAG is not involved in storing, loading, or recovering map atoms.

use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub fn ledger_path(root: &Path) -> PathBuf {
    root.join("map.atoms.jsonl")
}

pub fn append_atom(
    root: &Path,
    id: u64,
    text: &str,
    salience: f64,
    rank: u8,
    provenance: &str,
    taint: &str,
    domain: &str,
    pinned: bool,
) -> Result<(), String> {
    let line = json!({
        "op": "deposit",
        "kind": "atom",
        "id": id,
        "text": text,
        "salience": salience,
        "rank": rank,
        "provenance": provenance,
        "taint": taint,
        "domain": domain,
        "pinned": pinned,
    });
    let s = serde_json::to_string(&line).map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger_path(root))
        .map_err(|e| e.to_string())?;
    writeln!(f, "{s}").map_err(|e| e.to_string())?;
    Ok(())
}

/// All deposit/atom lines from the map ledger, in file order.
pub fn read_ledger(root: &Path) -> Vec<Value> {
    let raw = std::fs::read_to_string(ledger_path(root)).unwrap_or_default();
    let mut out = Vec::new();
    for line in raw.lines() {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if v.get("op").and_then(|x| x.as_str()) != Some("deposit") {
                continue;
            }
            if v.get("kind").and_then(|x| x.as_str()) != Some("atom") {
                continue;
            }
            out.push(v);
        }
    }
    out
}
