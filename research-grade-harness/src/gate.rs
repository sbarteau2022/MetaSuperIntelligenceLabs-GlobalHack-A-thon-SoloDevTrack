//! Plan/research-before-execution gate: no tool call runs on an
//! unresearched packet. Planning and research are recorded in the session
//! log FIRST (ContextInject + research events); the engine refuses execution
//! when the log shows no research for the packet's information need.
//!
//! This is the "MUST REQUIRE PLANNING/RESEARCH BEFORE EXECUTION" rule as
//! a mechanical gate, not a prompt wish. The engine checks the log, not
//! the agent's intentions.

use crate::session::{SessionEventKind, SessionLog};

/// Information need declared per packet (mirrors task-decomp I*(L)).
#[derive(Clone, Debug)]
pub struct InformationNeed {
    pub packet_id: String,
    pub required_terms: Vec<String>,
    pub min_depth: u32,
}

/// Research record: what the research phase found for a packet.
#[derive(Clone, Debug)]
pub struct ResearchRecord {
    pub packet_id: String,
    pub covered_terms: Vec<String>,
    pub depth_reached: u32,
    pub rag_coverage: f64,
}

impl ResearchRecord {
    pub fn satisfies(&self, need: &InformationNeed, coverage_theta: f64) -> bool {
        self.packet_id == need.packet_id
            && self.depth_reached >= need.min_depth
            && self.rag_coverage >= coverage_theta
            && need
                .required_terms
                .iter()
                .all(|t| self.covered_terms.iter().any(|c| c == t))
    }
}

/// The gate: scan the session log for a research record satisfying the
/// need. No record = no execution. The refusal names the missing terms
/// (fail-explained at the harness level).
pub fn research_gate(
    log: &SessionLog,
    need: &InformationNeed,
    records: &[ResearchRecord],
    coverage_theta: f64,
) -> Result<(), Vec<String>> {
    // The log must show planning happened: at least one ContextInject.
    let planned = log
        .events
        .iter()
        .any(|e| e.kind == SessionEventKind::ContextInject);
    if !planned {
        return Err(vec!["no planning phase recorded in session log".into()]);
    }
    let rec = records.iter().find(|r| r.packet_id == need.packet_id);
    match rec {
        None => Err(vec![format!(
            "no research record for packet {}",
            need.packet_id
        )]),
        Some(r) if !r.satisfies(need, coverage_theta) => {
            let missing: Vec<String> = need
                .required_terms
                .iter()
                .filter(|t| !r.covered_terms.iter().any(|c| c == *t))
                .cloned()
                .collect();
            let mut reasons = vec![format!(
                "coverage {:.2} < theta {:.2}",
                r.rag_coverage, coverage_theta
            )];
            if r.depth_reached < need.min_depth {
                reasons.push(format!(
                    "depth {} < required {}",
                    r.depth_reached, need.min_depth
                ));
            }
            reasons.extend(missing.into_iter().map(|t| format!("missing term: {}", t)));
            Err(reasons)
        }
        Some(_) => Ok(()),
    }
}

/// Dynamic self-recursive optimization trigger: when the session bound
/// requires iteration (FEP spike) AND the current packet has no answer in
/// the map, the system MUST plan/research a new approach — not retry, not
/// escalate blindly. Returns the research directive.
pub fn dynamic_iteration_directive(
    packet_id: &str,
    bound_spike: bool,
    has_answer: bool,
) -> Option<String> {
    if bound_spike && !has_answer {
        Some(format!(
            "packet {}: bound spike with no cached answer — plan new approach, research first, then execute",
            packet_id
        ))
    } else {
        None
    }
}
