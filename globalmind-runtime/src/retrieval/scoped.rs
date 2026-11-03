//! Read-only cognitive-well view for a carved Glimmer worker.
//! The caller must validate the governor permit and the MCP single-mind target first.
use super::{RecallBundle, RetrievalConfig, control_sql, digest_fields, record_id_of, retrieve};
use crate::{Digest, RecordId};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct ScopedEdge {
    pub id: RecordId,
    pub from: RecordId,
    pub to: RecordId,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub struct ScopedRecall {
    pub well_ids: Vec<RecordId>,
    pub bundle: RecallBundle,
    pub edges: Vec<ScopedEdge>,
    pub scope_digest: Digest,
}

fn rows<'a>(result: &'a [Value], name: &str, budget: u32) -> Result<&'a [Value], String> {
    let rows = result
        .first()
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name} query returned no row array"))?;
    if rows.len() as u32 >= budget {
        return Err(format!("{name} candidate budget overrun"));
    }
    Ok(rows)
}

/// Returns only persisted well members and edges internal to the visible atom set.
/// This is a trusted-caller API: governor permit verification is required before call.
/// Candidate scanning uses the existing workspace-wide read path and rejects saturation.
pub fn retrieve_scoped(
    config: &RetrievalConfig,
    well_ids: &[RecordId],
    query_h3: [f64; 4],
    query_t2: [f64; 2],
    query_provenance: &str,
    k: u32,
) -> Result<ScopedRecall, String> {
    config.validate()?;
    if well_ids.is_empty() || well_ids.len() as u32 >= config.candidate_budget {
        return Err("scope must contain 1..candidate_budget cognitive wells".into());
    }
    let wells: BTreeSet<_> = well_ids.iter().map(|id| id.as_str().to_owned()).collect();
    if wells.len() != well_ids.len() {
        return Err("duplicate cognitive well in scope".into());
    }
    let workspace = format!("workspace:`{}`", config.workspace.as_str());
    let well_refs = well_ids
        .iter()
        .map(|id| format!("cognitive_well:`{}`", id.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    let memberships = control_sql(
        config,
        &format!(
            "SELECT address, well FROM address_well_membership WHERE workspace = {workspace} AND well INSIDE [{well_refs}] LIMIT {};",
            config.candidate_budget
        ),
    )?;
    let membership_rows = rows(&memberships, "membership", config.candidate_budget)?;
    if membership_rows.is_empty() {
        return Err("scope has no persisted memberships".into());
    }
    let mut visible_addresses = BTreeSet::new();
    let mut seen_wells = BTreeSet::new();
    for row in membership_rows {
        let well = record_id_of(row, "well")?;
        if !wells.contains(well.as_str()) {
            return Err("out-of-scope membership row".into());
        }
        seen_wells.insert(well.as_str().to_owned());
        visible_addresses.insert(record_id_of(row, "address")?);
    }
    if seen_wells != wells {
        return Err("one or more scoped wells have no memberships".into());
    }
    let mut bundle = retrieve(
        config,
        query_h3,
        query_t2,
        query_provenance,
        config.candidate_budget,
    )?;
    bundle
        .members
        .retain(|member| visible_addresses.contains(&member.address));
    if bundle.members.is_empty() {
        return Err("scope has no retrievable atoms".into());
    }
    bundle.members.truncate(k as usize);
    for (rank, member) in bundle.members.iter_mut().enumerate() {
        member.rank = rank as u32;
    }
    let visible_atoms: BTreeSet<_> = bundle
        .members
        .iter()
        .map(|member| member.atom.clone())
        .collect();
    let edge_rows = control_sql(
        config,
        &format!(
            "SELECT id, in, out, kind FROM atom_edge WHERE workspace = {workspace} AND active = true LIMIT {};",
            config.candidate_budget
        ),
    )?;
    let mut edges = Vec::new();
    for row in rows(&edge_rows, "edge", config.candidate_budget)? {
        let from = record_id_of(row, "in")?;
        let to = record_id_of(row, "out")?;
        if visible_atoms.contains(&from) && visible_atoms.contains(&to) {
            edges.push(ScopedEdge {
                id: record_id_of(row, "id")?,
                from,
                to,
                kind: row
                    .get("kind")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "edge kind missing".to_string())?
                    .to_owned(),
            });
        }
    }
    edges.sort_by(|a, b| a.id.cmp(&b.id));
    let scope_digest = digest_fields(
        b"globalmind-scoped-recall-v1",
        &[
            config.workspace.as_str(),
            &wells.iter().cloned().collect::<Vec<_>>().join(","),
            bundle.query_digest.as_str(),
            &bundle
                .members
                .iter()
                .map(|m| m.atom.as_str())
                .collect::<Vec<_>>()
                .join(","),
            &edges
                .iter()
                .map(|e| e.id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        ],
    );
    bundle.ranking_digest = scope_digest.clone();
    Ok(ScopedRecall {
        well_ids: well_ids.to_vec(),
        bundle,
        edges,
        scope_digest,
    })
}
