//! Step 3 — single Meta Automaton at the MCP-to-harness seam.
//!
//! One automaton, one transition function, three roles: validator,
//! confidence re-ranker, prediction/prefill operator. It is OVERWATCH
//! only: it admits, rejects, or clarifies proposals, and it updates
//! confidence from verified outcomes. It never takes a worker turn,
//! never commands the brigade, never executes a step, and never owns
//! a second memory store.
//!
//! Placement: this module is the seam core. Proposals arrive in the
//! single typed envelope below — retrieval, prefill, MTP head, and
//! drafter are interchangeable proposer types, not separate controller
//! loops. A proposal is immutable until verified. The transition
//! records validation audit, confidence re-ranking, and the next
//! prediction/prefill decision in one receipt, and commits or rolls
//! back atomically: Admit means the harness may execute; Reject means
//! rollback with no execution; Clarify means the proposer must
//! constrain further before any execution.
//!
//! Transition/rollback semantics are reused from the harness engine
//! (`run_sequential_live`): a packet that fails its judge, its gate,
//! or its receipt digest escalates and its dependents are skipped —
//! no silent scope widening, no synthetic PASS. This module adds the
//! automaton decision in front of that engine, it does not replace it.
//!
//! Confidence is Q32.31 (`crate::fixed::Q32_31`), per strategy, with a
//! frozen policy version, a preregistered floor, a calibration count,
//! and an outcome-history digest. Confidence changes ONLY on verified
//! outcomes: `update_from_outcome` refuses unverified outcomes. A
//! model proposal can never serve as its own outcome judge, and a
//! confidence change without an observed receipt is a stop-gate
//! violation, not a feature.
//!
//! Persistence: confidence state and transition receipts are written
//! to the governed SurrealDB substrate (backend authority: base URL,
//! namespace, database as configured for `SurrealClient`, default
//! http://127.0.0.1:8000 / ares / ares). HyperDX is observation
//! only. The MCP target must resolve to exactly that authority —
//! `verify_mcp_authority` fails closed on any mismatch, so no MCP
//! path can create a second map. The `map_plugin` path in
//! research-grade-harness is NOT used here until its target is
//! verified; this module does not claim it.
//!
//! This module does not yet wire into `dispatch_mission_live`: that
//! wiring is the next Step 3 step, after an MCP client that speaks
//! this envelope exists. Claiming seam placement from this module
//! alone would overstate it — the transition core is done, the live
//! seam wiring is not.

use crate::digest::Sha256Digest;
use crate::fixed::Q32_31;
use crate::surreal::sql_escape;

/// Interchangeable proposer types. One envelope, not one loop each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposerFamily {
    Retrieval,
    Prefill,
    Mtp,
    Drafter,
}

impl ProposerFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            ProposerFamily::Retrieval => "retrieval",
            ProposerFamily::Prefill => "prefill",
            ProposerFamily::Mtp => "mtp",
            ProposerFamily::Drafter => "drafter",
        }
    }
}

/// One typed proposer envelope, for retrieval and prefill alike.
/// `payload` is the proposer's typed content (retrieval query, or
/// prefill prefix + TPPE/KV contract identity). It is not a prompt:
/// the envelope carries identity, policy, and bounds, and the
/// automaton decides on measured audit signals, never on prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposerEnvelope {
    pub family: ProposerFamily,
    pub strategy_id: String,
    pub policy_version: String,
    /// Model/prefix/TPPE/KV contract identity. Must match the
    /// registered identity for this strategy, or the proposal rejects.
    pub prefix_identity: String,
    /// Map/adapter identity the proposal claims to read or prefetch.
    pub map_identity: String,
    pub payload: String,
    /// Unix-ms expiry. Proposals past expiry reject, never execute.
    pub expiry_ms: u64,
}

impl ProposerEnvelope {
    /// Canonical digest over the envelope's fields, in fixed order.
    pub fn digest(&self) -> String {
        Sha256Digest::compute(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                self.family.as_str(),
                self.strategy_id,
                self.policy_version,
                self.prefix_identity,
                self.map_identity,
                self.payload,
                self.expiry_ms,
            )
            .as_bytes(),
        )
        .to_hex()
    }
}

/// Measured audit signals for one proposal. These are inputs to the
/// transition, produced by measurement — never by the proposer's own
/// confidence claim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AuditSignals {
    /// Retrieval/prefill confidence as measured, not self-reported.
    pub measured_confidence: Q32_31,
    pub unresolved_dependencies: u32,
    pub entity_ambiguity: bool,
    pub expected_information_gain: Q32_31,
}

/// Per-strategy automaton state, persisted in SurrealDB.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyState {
    pub strategy_id: String,
    pub policy_version: String,
    pub prefix_identity: String,
    pub map_identity: String,
    pub confidence: Q32_31,
    /// Preregistered floor: below it, the automaton clarifies.
    pub confidence_floor: Q32_31,
    pub calibration_count: u64,
    pub outcome_history_digest: String,
}

impl StrategyState {
    pub fn digest(&self) -> String {
        Sha256Digest::compute(
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}",
                self.strategy_id,
                self.policy_version,
                self.prefix_identity,
                self.map_identity,
                self.confidence.0,
                self.calibration_count,
                self.outcome_history_digest,
            )
            .as_bytes(),
        )
        .to_hex()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Admit,
    Reject,
    Clarify,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Admit => "admit",
            Verdict::Reject => "reject",
            Verdict::Clarify => "clarify",
        }
    }
}

/// One automaton transition record. Binds prior state, proposal,
/// measured audit signals, verdict, and updated state, so a
/// confidence change without an observed receipt cannot hide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomatonTransition {
    pub prior_state_digest: String,
    pub proposal_digest: String,
    pub verdict: Verdict,
    pub updated_confidence: Q32_31,
    pub updated_history_digest: String,
    pub committed: bool,
    pub receipt_digest: String,
}

fn transition_receipt(
    prior: &str,
    proposal: &str,
    verdict: Verdict,
    confidence: Q32_31,
    history: &str,
) -> String {
    Sha256Digest::compute(
        format!(
            "{}\n{}\n{}\n{}\n{}",
            prior,
            proposal,
            verdict.as_str(),
            confidence.0,
            history,
        )
        .as_bytes(),
    )
    .to_hex()
}

/// The single automaton transition. Deterministic, no model in the
/// control path: `now_ms` is an input, so tests can freeze time.
pub fn transition(
    state: &StrategyState,
    proposal: &ProposerEnvelope,
    signals: &AuditSignals,
    now_ms: u64,
) -> AutomatonTransition {
    let prior = state.digest();
    let proposal_digest = proposal.digest();
    let verdict = if proposal.policy_version != state.policy_version {
        Verdict::Reject
    } else if proposal.prefix_identity != state.prefix_identity
        || proposal.map_identity != state.map_identity
    {
        Verdict::Reject
    } else if now_ms > proposal.expiry_ms {
        Verdict::Reject
    } else if signals.unresolved_dependencies > 0
        || signals.entity_ambiguity
        || signals.measured_confidence < state.confidence_floor
    {
        Verdict::Clarify
    } else {
        Verdict::Admit
    };
    // Admission does not change confidence. Only a verified outcome
    // does (see update_from_outcome). A transition that admitted
    // and also raised confidence would be self-certification.
    let receipt = transition_receipt(
        &prior,
        &proposal_digest,
        verdict,
        state.confidence,
        &state.outcome_history_digest,
    );
    AutomatonTransition {
        prior_state_digest: prior,
        proposal_digest,
        verdict,
        updated_confidence: state.confidence,
        updated_history_digest: state.outcome_history_digest.clone(),
        committed: verdict == Verdict::Admit,
        receipt_digest: receipt,
    }
}

/// Confidence update from a VERIFIED outcome only. Returns Err and
/// leaves state untouched on an unverified outcome — the guard
/// against circular self-evidence.
pub fn update_from_outcome(
    state: &mut StrategyState,
    verified: bool,
    success: bool,
) -> Result<AutomatonTransition, String> {
    if !verified {
        return Err("unverified outcome cannot update confidence".to_string());
    }
    let prior = state.digest();
    let step = Q32_31::from_f64(0.1);
    state.confidence = if success {
        state.confidence + (Q32_31::ONE - state.confidence) * step
    } else {
        state.confidence * (Q32_31::ONE - step)
    };
    state.calibration_count += 1;
    state.outcome_history_digest = Sha256Digest::compute(
        format!(
            "{}\n{}\n{}",
            state.outcome_history_digest,
            if success { "success" } else { "failure" },
            state.calibration_count,
        )
        .as_bytes(),
    )
    .to_hex();
    let receipt = transition_receipt(
        &prior,
        "outcome",
        Verdict::Admit,
        state.confidence,
        &state.outcome_history_digest,
    );
    Ok(AutomatonTransition {
        prior_state_digest: prior,
        proposal_digest: "outcome".to_string(),
        verdict: Verdict::Admit,
        updated_confidence: state.confidence,
        updated_history_digest: state.outcome_history_digest.clone(),
        committed: true,
        receipt_digest: receipt,
    })
}

/// Fail-closed MCP authority check. The MCP target must resolve to
/// exactly the harness GlobalMind authority (base, namespace,
/// database). Any mismatch is an error, never a second map.
pub fn verify_mcp_authority(
    mcp_base: &str,
    mcp_ns: &str,
    mcp_db: &str,
    harness_base: &str,
    harness_ns: &str,
    harness_db: &str,
) -> Result<(), String> {
    if mcp_base != harness_base || mcp_ns != harness_ns || mcp_db != harness_db {
        return Err(format!(
            "MCP and harness target different GlobalMind authorities: mcp=({},{},{}) harness=({},{},{})",
            mcp_base, mcp_ns, mcp_db, harness_base, harness_ns, harness_db,
        ));
    }
    Ok(())
}

/// SurrealDB upsert for one strategy's confidence state. SQL text
/// only — the caller executes it through the governed client.
pub fn confidence_upsert_sql(state: &StrategyState) -> String {
    format!(
        "UPDATE automaton_confidence SET strategy_id = '{}', policy_version = '{}', prefix_identity = '{}', map_identity = '{}', confidence_raw = {}, confidence_floor_raw = {}, calibration_count = {}, outcome_history_digest = '{}' WHERE strategy_id = '{}';",
        sql_escape(&state.strategy_id),
        sql_escape(&state.policy_version),
        sql_escape(&state.prefix_identity),
        sql_escape(&state.map_identity),
        state.confidence.0,
        state.confidence_floor.0,
        state.calibration_count,
        sql_escape(&state.outcome_history_digest),
        sql_escape(&state.strategy_id),
    )
}

/// SurrealDB create for one strategy's confidence state, used
/// before the first upsert (Surreal UPDATE on a missing row is a
/// no-op, so a first run must create the row). SQL text only.
pub fn confidence_create_sql(state: &StrategyState) -> String {
    format!(
        "CREATE automaton_confidence SET strategy_id = '{}', policy_version = '{}', prefix_identity = '{}', map_identity = '{}', confidence_raw = {}, confidence_floor_raw = {}, calibration_count = {}, outcome_history_digest = '{}';",
        sql_escape(&state.strategy_id),
        sql_escape(&state.policy_version),
        sql_escape(&state.prefix_identity),
        sql_escape(&state.map_identity),
        state.confidence.0,
        state.confidence_floor.0,
        state.calibration_count,
        sql_escape(&state.outcome_history_digest),
    )
}

/// SurrealDB insert for one transition receipt. SQL text only.
pub fn transition_insert_sql(t: &AutomatonTransition) -> String {
    format!(
        "CREATE automaton_receipt SET prior_state_digest = '{}', proposal_digest = '{}', verdict = '{}', updated_confidence_raw = {}, updated_history_digest = '{}', committed = {}, receipt_digest = '{}';",
        sql_escape(&t.prior_state_digest),
        sql_escape(&t.proposal_digest),
        t.verdict.as_str(),
        t.updated_confidence.0,
        sql_escape(&t.updated_history_digest),
        t.committed,
        sql_escape(&t.receipt_digest),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> StrategyState {
        StrategyState {
            strategy_id: "retrieval-a".to_string(),
            policy_version: "policy-v1".to_string(),
            prefix_identity: "prefix-1".to_string(),
            map_identity: "globalmind-ares".to_string(),
            confidence: Q32_31::from_f64(0.6),
            confidence_floor: Q32_31::from_f64(0.5),
            calibration_count: 3,
            outcome_history_digest: "h0".to_string(),
        }
    }

    fn proposal() -> ProposerEnvelope {
        ProposerEnvelope {
            family: ProposerFamily::Retrieval,
            strategy_id: "retrieval-a".to_string(),
            policy_version: "policy-v1".to_string(),
            prefix_identity: "prefix-1".to_string(),
            map_identity: "globalmind-ares".to_string(),
            payload: "query: carved swarm".to_string(),
            expiry_ms: 1_000_000,
        }
    }

    fn signals(conf: f64) -> AuditSignals {
        AuditSignals {
            measured_confidence: Q32_31::from_f64(conf),
            unresolved_dependencies: 0,
            entity_ambiguity: false,
            expected_information_gain: Q32_31::from_f64(0.7),
        }
    }

    #[test]
    fn admit_above_floor_with_no_open_dependencies() {
        let t = transition(&state(), &proposal(), &signals(0.8), 100);
        assert_eq!(t.verdict, Verdict::Admit);
        assert!(t.committed);
        // Admission alone does not move confidence.
        assert_eq!(t.updated_confidence, state().confidence);
    }

    #[test]
    fn clarify_below_floor_or_with_open_dependencies() {
        let t = transition(&state(), &proposal(), &signals(0.4), 100);
        assert_eq!(t.verdict, Verdict::Clarify);
        assert!(!t.committed);
        let mut sig = signals(0.8);
        sig.unresolved_dependencies = 2;
        let t = transition(&state(), &proposal(), &sig, 100);
        assert_eq!(t.verdict, Verdict::Clarify);
    }

    #[test]
    fn reject_on_policy_or_identity_mismatch_or_expiry() {
        let mut p = proposal();
        p.policy_version = "policy-v2".to_string();
        assert_eq!(
            transition(&state(), &p, &signals(0.8), 100).verdict,
            Verdict::Reject
        );
        let mut p = proposal();
        p.map_identity = "other-map".to_string();
        assert_eq!(
            transition(&state(), &p, &signals(0.8), 100).verdict,
            Verdict::Reject
        );
        assert_eq!(
            transition(&state(), &proposal(), &signals(0.8), 2_000_000).verdict,
            Verdict::Reject
        );
    }

    #[test]
    fn unverified_outcome_cannot_move_confidence() {
        let mut s = state();
        let before = s.clone();
        assert!(update_from_outcome(&mut s, false, true).is_err());
        assert_eq!(s, before);
        let t = update_from_outcome(&mut s, true, true).expect("verified outcome");
        assert!(t.committed);
        assert!(s.confidence > before.confidence);
        assert_eq!(s.calibration_count, before.calibration_count + 1);
    }

    #[test]
    fn envelope_digest_is_stable_and_strategy_bound() {
        assert_eq!(proposal().digest(), proposal().digest());
        let mut p = proposal();
        p.strategy_id = "other".to_string();
        assert_ne!(p.digest(), proposal().digest());
    }

    #[test]
    fn mcp_authority_mismatch_fails_closed() {
        assert!(verify_mcp_authority(
            "http://127.0.0.1:8000",
            "ares",
            "ares",
            "http://127.0.0.1:8000",
            "ares",
            "ares"
        )
        .is_ok());
        assert!(verify_mcp_authority(
            "http://127.0.0.1:8000",
            "hypermind",
            "core",
            "http://127.0.0.1:8000",
            "ares",
            "ares"
        )
        .is_err());
    }

    #[test]
    fn surreal_sql_carries_strategy_and_raw_confidence() {
        let sql = confidence_upsert_sql(&state());
        assert!(sql.contains("retrieval-a"));
        assert!(sql.contains(&state().confidence.0.to_string()));
        let sql = transition_insert_sql(&transition(&state(), &proposal(), &signals(0.8), 100));
        assert!(sql.contains("admit"));
    }
}
