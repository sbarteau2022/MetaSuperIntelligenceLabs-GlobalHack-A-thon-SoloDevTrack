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
//! to the governed SurrealDB substrate (backend authority configured
//! for `SurrealClient`, usually ares/ares). HyperDX is observation only.
//! The mind target is a separate, explicit SurrealDB authority. The
//! governor checks the live `globalmind-mcp` child against that target
//! before dispatch. `verify_mcp_authority` compares those two mind
//! identities; it does not equate mind and governance databases.
//!
//! The live `/api/harness/mcp/execute` route calls this transition
//! through the governor. A well-formed MCP response is not an
//! independent task judge and cannot raise confidence on its own.

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
    /// Legacy state digest. It omits `confidence_floor`, so a floor change is
    /// invisible in it. Kept only so older receipts stay recomputable; every
    /// new transition binds `digest_v2`.
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

impl StrategyState {
    /// State digest that binds the preregistered floor. A floor can no longer
    /// change without the receipt chain showing it.
    pub fn digest_v2(&self) -> String {
        Sha256Digest::compute(
            format!(
                "strategy-state-v2\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
                self.strategy_id,
                self.policy_version,
                self.prefix_identity,
                self.map_identity,
                self.confidence.0,
                self.confidence_floor.0,
                self.calibration_count,
                self.outcome_history_digest,
            )
            .as_bytes(),
        )
        .to_hex()
    }
}

/// Deterministic re-rank of a held set: confidence descending on the raw
/// Q32.31 value, ties broken by `strategy_id` ascending. Strategy ids must be
/// distinct, because that is what makes the order total; a duplicate id fails
/// closed instead of ranking. No model, prose, or self-reported score is read.
pub fn rerank_strategies(states: &mut [StrategyState]) -> Result<(), String> {
    for (i, a) in states.iter().enumerate() {
        if states[..i].iter().any(|b| b.strategy_id == a.strategy_id) {
            return Err(format!(
                "duplicate strategy_id in held set: {}",
                a.strategy_id
            ));
        }
    }
    states.sort_by(|a, b| {
        b.confidence
            .0
            .cmp(&a.confidence.0)
            .then_with(|| a.strategy_id.cmp(&b.strategy_id))
    });
    Ok(())
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
    let prior = state.digest_v2();
    let proposal_digest = proposal.digest();
    let verdict = if proposal.policy_version != state.policy_version
        || proposal.prefix_identity != state.prefix_identity
        || proposal.map_identity != state.map_identity
        || now_ms > proposal.expiry_ms
    {
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
    let prior = state.digest_v2();
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

/// Confidence update from a VERIFIED prediction error in [0, 1]: the share of a
/// proposal the model's own forward rejected. Confidence moves a tenth of the
/// way toward `1 - error`, in Q32.31 only. An unverified outcome is refused, and
/// the proposer can never supply the error it is judged by.
pub fn update_from_prediction_error(
    state: &mut StrategyState,
    verified: bool,
    error: Q32_31,
) -> Result<AutomatonTransition, String> {
    if !verified {
        return Err("unverified outcome cannot update confidence".to_string());
    }
    if error < Q32_31::ZERO || error > Q32_31::ONE {
        return Err("prediction error outside [0, 1]".to_string());
    }
    let prior = state.digest_v2();
    let step = Q32_31::from_f64(0.1);
    let target = Q32_31::ONE - error;
    state.confidence = state.confidence + (target - state.confidence) * step;
    state.calibration_count += 1;
    state.outcome_history_digest = Sha256Digest::compute(
        format!(
            "{}\nerror:{}\n{}",
            state.outcome_history_digest, error.0, state.calibration_count,
        )
        .as_bytes(),
    )
    .to_hex();
    let receipt = transition_receipt(
        &prior,
        "prediction-error",
        Verdict::Admit,
        state.confidence,
        &state.outcome_history_digest,
    );
    Ok(AutomatonTransition {
        prior_state_digest: prior,
        proposal_digest: "prediction-error".to_string(),
        verdict: Verdict::Admit,
        updated_confidence: state.confidence,
        updated_history_digest: state.outcome_history_digest.clone(),
        committed: true,
        receipt_digest: receipt,
    })
}

/// Probe admission: the only way past the confidence floor.
///
/// Confidence moves only on verified outcomes, and a strategy below its floor
/// is never run, so it would never be judged again and could never recover.
/// This function lets the automaton itself, and nothing else, admit one
/// bounded probe after `probe_every` consecutive floor-gated proposals. It
/// returns `Some` only when the ordinary verdict is Clarify and nothing but
/// the floor caused it: open dependencies and entity ambiguity are never
/// probed past, and a Reject is never probed at all. Admission leaves
/// confidence unchanged; the probe's verified outcome does the moving.
pub fn probe_admit(
    state: &StrategyState,
    proposal: &ProposerEnvelope,
    signals: &AuditSignals,
    now_ms: u64,
    gated_rounds: u32,
    probe_every: u32,
) -> Option<AutomatonTransition> {
    if probe_every == 0 || gated_rounds < probe_every {
        return None;
    }
    if signals.unresolved_dependencies > 0 || signals.entity_ambiguity {
        return None;
    }
    let base = transition(state, proposal, signals, now_ms);
    if base.verdict != Verdict::Clarify {
        return None;
    }
    let tagged = format!("probe:{}", base.proposal_digest);
    let receipt = transition_receipt(
        &base.prior_state_digest,
        &tagged,
        Verdict::Admit,
        state.confidence,
        &state.outcome_history_digest,
    );
    Some(AutomatonTransition {
        prior_state_digest: base.prior_state_digest,
        proposal_digest: tagged,
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

    #[test]
    fn one_success_from_point_six_is_the_papers_raw_value() {
        // Meta-Superposition v2, Proof 5 Step 6: raw 1374389534 is 0.64 in
        // Q32.31, and it follows from a starting confidence of exactly 0.6.
        let mut s = state();
        update_from_outcome(&mut s, true, true).expect("verified outcome");
        assert_eq!(s.confidence.0, 1_374_389_534);
    }

    #[test]
    fn digest_v2_binds_the_floor_and_legacy_digest_does_not() {
        let a = state();
        let mut b = state();
        b.confidence_floor = Q32_31::from_f64(0.9);
        assert_eq!(a.digest(), b.digest(), "legacy digest cannot see the floor");
        assert_ne!(a.digest_v2(), b.digest_v2(), "v2 must bind the floor");
        let ta = transition(&a, &proposal(), &signals(0.8), 100);
        let tb = transition(&b, &proposal(), &signals(0.8), 100);
        assert_ne!(ta.prior_state_digest, tb.prior_state_digest);
    }

    fn ranked(id: &str, conf: f64) -> StrategyState {
        let mut s = state();
        s.strategy_id = id.to_string();
        s.confidence = Q32_31::from_f64(conf);
        s
    }

    #[test]
    fn rerank_is_confidence_descending_then_id_ascending_and_input_order_free() {
        let mut v = vec![
            ranked("c", 0.5),
            ranked("a", 0.5),
            ranked("z", 0.9),
            ranked("b", 0.1),
        ];
        rerank_strategies(&mut v).unwrap();
        let ids: Vec<&str> = v.iter().map(|s| s.strategy_id.as_str()).collect();
        assert_eq!(ids, ["z", "a", "c", "b"]);
        // Every permutation of the input yields the same order.
        let base = ["c", "a", "z", "b"];
        let conf = |id: &str| match id {
            "z" => 0.9,
            "b" => 0.1,
            _ => 0.5,
        };
        let mut perm = base.to_vec();
        for rot in 0..4 {
            perm.rotate_left(rot);
            let mut w: Vec<StrategyState> = perm.iter().map(|i| ranked(i, conf(i))).collect();
            w.reverse();
            rerank_strategies(&mut w).unwrap();
            let got: Vec<&str> = w.iter().map(|s| s.strategy_id.as_str()).collect();
            assert_eq!(got, ["z", "a", "c", "b"]);
        }
    }

    #[test]
    fn rerank_refuses_duplicate_strategy_ids() {
        let mut v = vec![ranked("a", 0.5), ranked("a", 0.7)];
        assert!(rerank_strategies(&mut v).is_err());
    }

    #[test]
    fn probe_admits_only_past_the_floor_and_only_when_due() {
        let low = {
            let mut s = state();
            s.confidence = Q32_31::from_f64(0.1);
            s
        };
        let sig = signals(0.1);
        // Gated, but not yet due.
        assert!(probe_admit(&low, &proposal(), &sig, 100, 1, 2).is_none());
        // Disabled.
        assert!(probe_admit(&low, &proposal(), &sig, 100, 99, 0).is_none());
        // Due: admitted, tagged, and confidence untouched.
        let p = probe_admit(&low, &proposal(), &sig, 100, 2, 2).expect("due probe");
        assert_eq!(p.verdict, Verdict::Admit);
        assert!(p.committed);
        assert!(p.proposal_digest.starts_with("probe:"));
        assert_eq!(p.updated_confidence, low.confidence);
        let plain = transition(&low, &proposal(), &sig, 100);
        assert_ne!(p.receipt_digest, plain.receipt_digest);
        // Open dependencies and ambiguity are never probed past.
        let mut dep = signals(0.1);
        dep.unresolved_dependencies = 1;
        assert!(probe_admit(&low, &proposal(), &dep, 100, 9, 2).is_none());
        let mut amb = signals(0.1);
        amb.entity_ambiguity = true;
        assert!(probe_admit(&low, &proposal(), &amb, 100, 9, 2).is_none());
        // A Reject is never probed: expired, wrong identity.
        assert!(probe_admit(&low, &proposal(), &sig, 2_000_000, 9, 2).is_none());
        let mut foreign = proposal();
        foreign.map_identity = "other-map".to_string();
        assert!(probe_admit(&low, &foreign, &sig, 100, 9, 2).is_none());
        // An already-admissible proposal needs no probe.
        assert!(probe_admit(&state(), &proposal(), &signals(0.8), 100, 9, 2).is_none());
    }
}
