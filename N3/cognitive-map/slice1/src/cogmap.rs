//! Model Home cognitive map — Slice 1 pure reducer (Blueprint v1.1 §§A-I, policy atlas-evidence-v1.1).
//! Pure: no I/O, network, RNG, or wall-clock. Same (state, events, versions) => same state (I11).
//! Model proposes, never certifies (I08): assessment admission is a reducer event with grader provenance,
//! not a client write. All scoring is integer math (§E); display division never gates a decision.
use std::collections::{BTreeMap, HashMap, HashSet};

pub const POLICY_VERSION: &str = "atlas-evidence-v1.1";
pub const MAX_SCORE_UNITS: i64 = 24_000;

// ---------- §E exact scoring ----------
/// rawUnits = 96v + 28j + 15q, integer 0..240. q is 4x evidence quality (0..4).
pub fn raw_units(v: u8, j: u8, q: u8) -> i64 {
    debug_assert!(v <= 1 && j <= 3 && q <= 4);
    96 * v as i64 + 28 * j as i64 + 15 * q as i64
}
/// scoreUnits = rawUnits * h, integer 0..24000.
pub fn score_units(v: u8, j: u8, q: u8, hint_percent: i64) -> i64 {
    debug_assert!([100, 90, 75, 55, 35].contains(&hint_percent));
    raw_units(v, j, q) * hint_percent
}
/// Hint tier -> percent. ASSUMPTION (labelled): tier0=100,1=90,2=75,3=55,>=4=35 (set from §E).
pub fn hint_percent(tier: u8) -> i64 {
    match tier {
        0 => 100,
        1 => 90,
        2 => 75,
        3 => 55,
        _ => 35,
    }
}
/// mean >= .80 iff sum >= 19200*n (§E). Integer only.
pub fn mean_ge_80(sum_units: i64, n: i64) -> bool {
    n > 0 && sum_units >= 19_200 * n
}
/// mean < .50 iff sum < 12000*n (§E).
pub fn mean_lt_50(sum_units: i64, n: i64) -> bool {
    n > 0 && sum_units < 12_000 * n
}
/// FNR <= .20 iff defective>0 and 5*miss <= defective (§E).
pub fn fnr_ok(defective: i64, miss: i64) -> bool {
    defective > 0 && 5 * miss <= defective
}
/// FPR <= .25 iff clean_opp>0 and 4*false_alarm <= clean_opp (§E).
pub fn fpr_ok(clean_opp: i64, false_alarm: i64) -> bool {
    clean_opp > 0 && 4 * false_alarm <= clean_opp
}

// ---------- §C attempt state machine ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptState {
    Assigned,
    PredictionCommitted,
    Revealed,
    AssessmentPending,
    Assessed,
    Disputed,
    Invalid,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attempt {
    pub id: String,
    pub concept_id: String,
    pub dimension: String,
    pub difficulty: u8,
    pub variant: String,
    pub state: AttemptState,
    pub prediction_committed: bool,
    pub prediction_before_reveal: bool,
    pub hint_tier: u8,
    pub independent: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Assessment {
    pub id: String,
    pub attempt_id: String,
    pub score_units: i64,
    pub false_negative: bool,
    pub false_positive: bool,
    pub fully_clean: bool,
    pub defective_task: bool,
    pub active: bool,
    pub seq: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Observation {
    pub attempt_id: String,
    pub dimension: String,
    pub seq: i64,
    pub score_units: i64,
    pub independent: bool,
    pub hint_tier: u8,
    pub variant: String,
    pub fully_clean: bool,
    pub defective_task: bool,
    pub false_negative: bool,
    pub false_positive: bool,
    pub prediction_before_reveal: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    pub revision: i64,
    pub attempts: HashMap<String, Attempt>,
    pub heads: HashMap<String, Assessment>, // by attempt_id; one head per submission (I04, schema-enforced)
    pub observations: Vec<Observation>,
    applied_event_ids: HashSet<String>, // I03/I11 dedup inside reducer
}
#[derive(Clone, Debug)]
pub enum Event {
    AssignTask {
        event_id: String,
        attempt_id: String,
        concept_id: String,
        dimension: String,
        difficulty: u8,
        variant: String,
        independent: bool,
    },
    CommitPrediction {
        event_id: String,
        attempt_id: String,
    },
    ReplacePrediction {
        event_id: String,
        attempt_id: String,
    },
    Reveal {
        event_id: String,
        attempt_id: String,
    },
    DeliverHint {
        event_id: String,
        attempt_id: String,
        tier: u8,
    },
    Submit {
        event_id: String,
        attempt_id: String,
    },
    AssessmentAccepted {
        event_id: String,
        attempt_id: String,
        assessment_id: String,
        v: u8,
        j: u8,
        q: u8,
        seq: i64,
        false_negative: bool,
        false_positive: bool,
        fully_clean: bool,
        defective_task: bool,
    },
    AssessmentUnavailable {
        event_id: String,
        attempt_id: String,
    },
    Dispute {
        event_id: String,
        attempt_id: String,
    },
    ResolveDisputeAssessed {
        event_id: String,
        attempt_id: String,
        assessment_id: String,
        v: u8,
        j: u8,
        q: u8,
        seq: i64,
        false_negative: bool,
        false_positive: bool,
        fully_clean: bool,
        defective_task: bool,
    },
    ResolveDisputeInvalid {
        event_id: String,
        attempt_id: String,
    },
    Invalidate {
        event_id: String,
        attempt_id: String,
    },
    Cancel {
        event_id: String,
        attempt_id: String,
    },
    Regrade {
        event_id: String,
        attempt_id: String,
        assessment_id: String,
        v: u8,
        j: u8,
        q: u8,
        seq: i64,
        false_negative: bool,
        false_positive: bool,
        fully_clean: bool,
        defective_task: bool,
    },
}
impl Event {
    fn id(&self) -> &str {
        match self {
            Event::AssignTask { event_id, .. }
            | Event::CommitPrediction { event_id, .. }
            | Event::ReplacePrediction { event_id, .. }
            | Event::Reveal { event_id, .. }
            | Event::DeliverHint { event_id, .. }
            | Event::Submit { event_id, .. }
            | Event::AssessmentAccepted { event_id, .. }
            | Event::AssessmentUnavailable { event_id, .. }
            | Event::Dispute { event_id, .. }
            | Event::ResolveDisputeAssessed { event_id, .. }
            | Event::ResolveDisputeInvalid { event_id, .. }
            | Event::Invalidate { event_id, .. }
            | Event::Cancel { event_id, .. }
            | Event::Regrade { event_id, .. } => event_id,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Reject {
    UnknownAttempt,
    BadTransition,
    RevealWithoutPrediction,
    DuplicateHead,
}

fn get<'a>(s: &'a State, id: &str) -> Result<&'a Attempt, Reject> {
    s.attempts.get(id).ok_or(Reject::UnknownAttempt)
}
fn accept(s: &mut State, attempt_id: &str, a: Assessment) {
    // I05: one observation per attempt/dimension — replace on regrade, never append a second.
    let dim = get(s, attempt_id)
        .map(|a| a.dimension.clone())
        .unwrap_or_default();
    s.observations
        .retain(|o| !(o.attempt_id == attempt_id && o.dimension == dim));
    let att = &s.attempts[attempt_id];
    s.observations.push(Observation {
        attempt_id: attempt_id.into(),
        dimension: dim,
        seq: a.seq,
        score_units: a.score_units,
        independent: att.independent,
        hint_tier: att.hint_tier,
        variant: att.variant.clone(),
        fully_clean: a.fully_clean,
        defective_task: a.defective_task,
        false_negative: a.false_negative,
        false_positive: a.false_positive,
        prediction_before_reveal: att.prediction_before_reveal,
    });
    s.heads.insert(attempt_id.into(), a);
}

/// Pure reducer (§A). Duplicate event_id = no-op returning Ok (I03 at command layer + I11 determinism).
pub fn reduce(mut s: State, e: &Event) -> Result<State, Reject> {
    if s.applied_event_ids.contains(e.id()) {
        return Ok(s);
    }
    match e {
        Event::AssignTask {
            attempt_id,
            concept_id,
            dimension,
            difficulty,
            variant,
            independent,
            ..
        } => {
            s.attempts.insert(
                attempt_id.clone(),
                Attempt {
                    id: attempt_id.clone(),
                    concept_id: concept_id.clone(),
                    dimension: dimension.clone(),
                    difficulty: *difficulty,
                    variant: variant.clone(),
                    state: AttemptState::Assigned,
                    prediction_committed: false,
                    prediction_before_reveal: false,
                    hint_tier: 0,
                    independent: *independent,
                },
            );
        }
        Event::CommitPrediction { attempt_id, .. }
        | Event::ReplacePrediction { attempt_id, .. } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Assigned && a.state != AttemptState::PredictionCommitted {
                return Err(Reject::BadTransition);
            }
            let mut a = a;
            a.state = AttemptState::PredictionCommitted;
            a.prediction_committed = true;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::Reveal { attempt_id, .. } => {
            let a = get(&s, attempt_id)?.clone();
            if !a.prediction_committed {
                return Err(Reject::RevealWithoutPrediction);
            } // predict-before-reveal gate (§C)
            if a.state != AttemptState::PredictionCommitted {
                return Err(Reject::BadTransition);
            }
            let mut a = a;
            a.state = AttemptState::Revealed;
            a.prediction_before_reveal = true;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::DeliverHint {
            attempt_id, tier, ..
        } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Revealed {
                return Err(Reject::BadTransition);
            }
            let mut a = a;
            a.hint_tier = (*tier).max(a.hint_tier);
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::Submit { attempt_id, .. } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Revealed {
                return Err(Reject::BadTransition);
            }
            let mut a = a;
            a.state = AttemptState::AssessmentPending;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::AssessmentAccepted {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            seq,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::AssessmentPending {
                return Err(Reject::BadTransition);
            }
            if s.heads.contains_key(attempt_id) {
                return Err(Reject::DuplicateHead);
            } // I04
            let su = score_units(*v, *j, *q, hint_percent(a.hint_tier));
            accept(
                &mut s,
                attempt_id,
                Assessment {
                    id: assessment_id.clone(),
                    attempt_id: attempt_id.clone(),
                    score_units: su,
                    false_negative: *false_negative,
                    false_positive: *false_positive,
                    fully_clean: *fully_clean,
                    defective_task: *defective_task,
                    active: true,
                    seq: *seq,
                },
            );
            let mut a = a;
            a.state = AttemptState::Assessed;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::AssessmentUnavailable { attempt_id, .. } => {
            // stays pending, no negative evidence (§C)
            if get(&s, attempt_id)?.state != AttemptState::AssessmentPending {
                return Err(Reject::BadTransition);
            }
        }
        Event::Dispute { attempt_id, .. } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Assessed {
                return Err(Reject::BadTransition);
            }
            if let Some(h) = s.heads.get_mut(attempt_id) {
                h.active = false;
            } // removed from active evidence (§C)
            s.observations.retain(|o| o.attempt_id != *attempt_id);
            let mut a = a;
            a.state = AttemptState::Disputed;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::ResolveDisputeAssessed {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            seq,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => {
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Disputed {
                return Err(Reject::BadTransition);
            }
            let su = score_units(*v, *j, *q, hint_percent(a.hint_tier));
            accept(
                &mut s,
                attempt_id,
                Assessment {
                    id: assessment_id.clone(),
                    attempt_id: attempt_id.clone(),
                    score_units: su,
                    false_negative: *false_negative,
                    false_positive: *false_positive,
                    fully_clean: *fully_clean,
                    defective_task: *defective_task,
                    active: true,
                    seq: *seq,
                },
            );
            let mut a = a;
            a.state = AttemptState::Assessed;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::ResolveDisputeInvalid { attempt_id, .. } => {
            if get(&s, attempt_id)?.state != AttemptState::Disputed {
                return Err(Reject::BadTransition);
            }
            let mut a = get(&s, attempt_id)?.clone();
            a.state = AttemptState::Invalid;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::Invalidate { attempt_id, .. } => {
            // any active state (§C); evidence removed
            let _ = get(&s, attempt_id)?;
            if let Some(h) = s.heads.get_mut(attempt_id) {
                h.active = false;
            }
            s.observations.retain(|o| o.attempt_id != *attempt_id);
            let mut a = get(&s, attempt_id)?.clone();
            a.state = AttemptState::Invalid;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::Cancel { attempt_id, .. } => {
            let a = get(&s, attempt_id)?.clone();
            if matches!(
                a.state,
                AttemptState::Assessed | AttemptState::Invalid | AttemptState::Cancelled
            ) {
                return Err(Reject::BadTransition);
            }
            let mut a = a;
            a.state = AttemptState::Cancelled;
            s.attempts.insert(attempt_id.clone(), a);
        }
        Event::Regrade {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            seq,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => {
            // New head, same attempt, count stays one (§C, I04/I05). Old head superseded by replacement in map.
            let a = get(&s, attempt_id)?.clone();
            if a.state != AttemptState::Assessed {
                return Err(Reject::BadTransition);
            }
            let su = score_units(*v, *j, *q, hint_percent(a.hint_tier));
            accept(
                &mut s,
                attempt_id,
                Assessment {
                    id: assessment_id.clone(),
                    attempt_id: attempt_id.clone(),
                    score_units: su,
                    false_negative: *false_negative,
                    false_positive: *false_positive,
                    fully_clean: *fully_clean,
                    defective_task: *defective_task,
                    active: true,
                    seq: *seq,
                },
            );
        }
    }
    s.applied_event_ids.insert(e.id().to_string());
    s.revision += 1;
    Ok(s)
}

// ---------- §F window, status, promotion ----------
/// Window: dedup by attempt/dimension, sort by seq ascending, retain last 8 (§F).
pub fn build_window(obs: &[Observation], dimension: &str) -> Vec<Observation> {
    let mut by_attempt: BTreeMap<String, Observation> = BTreeMap::new();
    for o in obs.iter().filter(|o| o.dimension == dimension) {
        by_attempt
            .entry(o.attempt_id.clone())
            .or_insert_with(|| o.clone());
    }
    let mut w: Vec<Observation> = by_attempt.into_values().collect();
    w.sort_by_key(|o| o.seq);
    if w.len() > 8 {
        w.drain(..w.len() - 8);
    }
    w
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConceptStatus {
    Unknown,
    NeedsRecheck,
    Insufficient,
    Developing,
    DemonstratedAtD,
}

/// Concept status total order (§F). No dimension-score copying: caller passes concept-mapped evidence only.
pub fn concept_status(
    distinct_attempts: usize,
    independent: usize,
    variants: usize,
    mean_ok: bool,
    gate_ok: bool,
    contested: bool,
    retention_requested: bool,
    has_evidence: bool,
) -> ConceptStatus {
    if !has_evidence {
        return ConceptStatus::Unknown;
    }
    if contested || retention_requested {
        return ConceptStatus::NeedsRecheck;
    }
    if distinct_attempts < 3 || independent < 1 || variants < 2 {
        return ConceptStatus::Insufficient;
    }
    if !mean_ok || !gate_ok {
        return ConceptStatus::Developing;
    }
    ConceptStatus::DemonstratedAtD
}

/// Promotion: offer only, never automatic (§F). Caller rechecks on accept (I09).
pub fn promotion_eligible(
    current_d: u8,
    window: &[Observation],
    independent_count: usize,
    clean_independent: usize,
    defective_independent: usize,
    gates_pass: bool,
    pending_blocks: bool,
    disputes_open: bool,
) -> bool {
    if current_d >= 5 || pending_blocks || disputes_open || !gates_pass {
        return false;
    }
    if independent_count < 5 || clean_independent < 2 || defective_independent < 2 {
        return false;
    }
    let last2_hint_ok = window.iter().rev().take(2).all(|o| o.hint_tier <= 1);
    let last3_pred_ok = window
        .iter()
        .rev()
        .take(3)
        .all(|o| o.prediction_before_reveal);
    window.len() >= 5 && last2_hint_ok && last3_pred_ok
}

/// Support trigger (§F): last three eligible scores each < .50. Integer: score_units < 12000.
pub fn support_trigger(window: &[Observation]) -> bool {
    window.len() >= 3 && window.iter().rev().take(3).all(|o| o.score_units < 12_000)
}

// ---------- §G deterministic teaching decision (priority 1-8) ----------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Hold,
    Explain,
    AskEvidence,
    HumanReview,
    Diagnostic,
    OfferPrereq,
    OfferAdvance,
    NextPractice,
}
pub fn decide(
    paused_or_declined: bool,
    grading_blocked: bool,
    missing_artifact: bool,
    failed_remediation_cycles: u32,
    support: bool,
    prereq_missing: bool,
    promotion: bool,
) -> Decision {
    if paused_or_declined {
        Decision::Hold
    }
    // 1
    else if grading_blocked {
        Decision::Explain
    }
    // 2
    else if missing_artifact {
        Decision::AskEvidence
    }
    // 3
    else if failed_remediation_cycles >= 2 {
        Decision::HumanReview
    }
    // 4
    else if support {
        Decision::Diagnostic
    }
    // 5
    else if prereq_missing {
        Decision::OfferPrereq
    }
    // 6
    else if promotion {
        Decision::OfferAdvance
    }
    // 7
    else {
        Decision::NextPractice
    } // 8
}

#[cfg(test)]
mod tests {
    use super::*;
    fn happy_path_to_pending() -> State {
        let mut s = State::default();
        s = reduce(
            s,
            &Event::AssignTask {
                event_id: "e1".into(),
                attempt_id: "a1".into(),
                concept_id: "c1".into(),
                dimension: "d1".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::CommitPrediction {
                event_id: "e2".into(),
                attempt_id: "a1".into(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Reveal {
                event_id: "e3".into(),
                attempt_id: "a1".into(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Submit {
                event_id: "e4".into(),
                attempt_id: "a1".into(),
            },
        )
        .unwrap();
        s
    }
    #[test]
    fn policy_vector_score_max_min_and_hint_scaling() {
        assert_eq!(raw_units(1, 3, 4), 240);
        assert_eq!(score_units(1, 3, 4, 100), 24_000);
        assert_eq!(score_units(0, 0, 0, 100), 0);
        assert_eq!(score_units(1, 3, 4, 35), 8_400);
        assert_eq!(score_units(1, 0, 0, 90), 8_640); // (96)*90
    }
    #[test]
    fn policy_vector_gates_boundaries() {
        assert!(mean_ge_80(19_200, 1));
        assert!(!mean_ge_80(19_199, 1));
        assert!(mean_lt_50(11_999, 1));
        assert!(!mean_lt_50(12_000, 1));
        assert!(fnr_ok(5, 1));
        assert!(!fnr_ok(5, 2));
        assert!(!fnr_ok(0, 0));
        assert!(fpr_ok(4, 1));
        assert!(!fpr_ok(4, 2));
        assert!(!fpr_ok(0, 0));
    }
    #[test]
    fn hint_tiers_map_to_blueprint_set() {
        assert_eq!([0, 1, 2, 3, 4].map(hint_percent), [100, 90, 75, 55, 35]);
    }
    #[test]
    fn state_machine_happy_path_and_reveal_gate() {
        let s = happy_path_to_pending();
        assert_eq!(s.attempts["a1"].state, AttemptState::AssessmentPending);
        // Reveal without prediction must reject
        let mut s2 = State::default();
        s2 = reduce(
            s2,
            &Event::AssignTask {
                event_id: "x1".into(),
                attempt_id: "a2".into(),
                concept_id: "c1".into(),
                dimension: "d1".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
        )
        .unwrap();
        assert_eq!(
            reduce(
                s2,
                &Event::Reveal {
                    event_id: "x2".into(),
                    attempt_id: "a2".into()
                }
            ),
            Err(Reject::RevealWithoutPrediction)
        );
    }
    #[test]
    fn assessment_dedup_regrade_keeps_one_observation_i05() {
        let mut s = happy_path_to_pending();
        s = reduce(
            s,
            &Event::AssessmentAccepted {
                event_id: "e5".into(),
                attempt_id: "a1".into(),
                assessment_id: "g1".into(),
                v: 1,
                j: 3,
                q: 4,
                seq: 1,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        )
        .unwrap();
        assert_eq!(s.observations.len(), 1);
        s = reduce(
            s,
            &Event::Regrade {
                event_id: "e6".into(),
                attempt_id: "a1".into(),
                assessment_id: "g2".into(),
                v: 0,
                j: 0,
                q: 0,
                seq: 1,
                false_negative: true,
                false_positive: false,
                fully_clean: false,
                defective_task: true,
            },
        )
        .unwrap();
        assert_eq!(s.observations.len(), 1);
        assert_eq!(s.heads["a1"].id, "g2"); // I04 new head, I05 count stays one
    }
    #[test]
    fn duplicate_event_is_noop_i03() {
        let s = happy_path_to_pending();
        let rev = s.revision;
        let s2 = reduce(
            s.clone(),
            &Event::Submit {
                event_id: "e4".into(),
                attempt_id: "a1".into(),
            },
        )
        .unwrap();
        assert_eq!(s2.revision, rev);
        assert_eq!(s2.observations.len(), s.observations.len());
    }
    #[test]
    fn window_last_eight_and_dedup() {
        let mk = |i: i64| Observation {
            attempt_id: format!("a{i}"),
            dimension: "d1".into(),
            seq: i,
            score_units: 20_000,
            independent: true,
            hint_tier: 0,
            variant: "v".into(),
            fully_clean: false,
            defective_task: false,
            false_negative: false,
            false_positive: false,
            prediction_before_reveal: true,
        };
        let obs: Vec<_> = (1..=10).map(mk).collect();
        let w = build_window(&obs, "d1");
        assert_eq!(w.len(), 8);
        assert_eq!(w[0].seq, 3);
        assert_eq!(w[7].seq, 10);
        let mut dup = obs.clone();
        dup.push(mk(1));
        assert_eq!(build_window(&dup, "d1").len(), 8);
    }
    #[test]
    fn concept_status_order() {
        assert_eq!(
            concept_status(0, 0, 0, false, false, false, false, false),
            ConceptStatus::Unknown
        );
        assert_eq!(
            concept_status(3, 1, 2, true, true, true, false, true),
            ConceptStatus::NeedsRecheck
        );
        assert_eq!(
            concept_status(2, 1, 2, true, true, false, false, true),
            ConceptStatus::Insufficient
        );
        assert_eq!(
            concept_status(3, 1, 2, false, true, false, false, true),
            ConceptStatus::Developing
        );
        assert_eq!(
            concept_status(3, 1, 2, true, true, false, false, true),
            ConceptStatus::DemonstratedAtD
        );
    }
    #[test]
    fn promotion_offer_only_gates() {
        let mk = |i: i64| Observation {
            attempt_id: format!("a{i}"),
            dimension: "d1".into(),
            seq: i,
            score_units: 20_000,
            independent: true,
            hint_tier: 0,
            variant: "v".into(),
            fully_clean: true,
            defective_task: true,
            false_negative: false,
            false_positive: false,
            prediction_before_reveal: true,
        };
        let w: Vec<_> = (1..=5).map(mk).collect();
        assert!(promotion_eligible(1, &w, 5, 2, 2, true, false, false));
        assert!(!promotion_eligible(5, &w, 5, 2, 2, true, false, false)); // D<5 required
        assert!(!promotion_eligible(1, &w, 5, 2, 2, true, true, false)); // pending blocks
    }
    #[test]
    fn decision_priority_order() {
        assert_eq!(
            decide(true, true, true, 9, true, true, true),
            Decision::Hold
        );
        assert_eq!(
            decide(false, true, true, 9, true, true, true),
            Decision::Explain
        );
        assert_eq!(
            decide(false, false, false, 2, true, true, true),
            Decision::HumanReview
        );
        assert_eq!(
            decide(false, false, false, 0, false, false, true),
            Decision::OfferAdvance
        );
        assert_eq!(
            decide(false, false, false, 0, false, false, false),
            Decision::NextPractice
        );
    }
    #[test]
    fn dispute_removes_active_evidence() {
        let mut s = happy_path_to_pending();
        s = reduce(
            s,
            &Event::AssessmentAccepted {
                event_id: "e5".into(),
                attempt_id: "a1".into(),
                assessment_id: "g1".into(),
                v: 1,
                j: 3,
                q: 4,
                seq: 1,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Dispute {
                event_id: "e7".into(),
                attempt_id: "a1".into(),
            },
        )
        .unwrap();
        assert!(s.observations.is_empty());
        assert_eq!(s.attempts["a1"].state, AttemptState::Disputed);
    }
}
