//! Shadow gate (Blueprint §K gate 4): record proposals without changing placement.
//! The full brief+decision pipeline (Slice 3 `cogmap_brief`, Slice 1 policy) runs on a
//! fixture attempt stream applied to a SCRATCH clone of learner state. What would be
//! decided is appended to a shadow log. The real learner state is never mutated, no
//! placement is emitted, no promotion is affected, and shadow entries are non-evidence:
//! they carry marker SHADOW, `evidence: false`, and never enter evidence windows (I12).
use cogmap::{reduce, Decision, Event, State, POLICY_VERSION};
use cogmap_slice3::brief::{cogmap_brief, Curriculum, Goal, TeachingBrief};

/// One shadow-log row: a would-be decision, marked SHADOW, never evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowEntry {
    pub shadow_seq: i64,
    pub marker: String, // always "SHADOW"
    pub evidence: bool, // always false — shadow decisions are non-evidence
    pub decision: Decision,
    pub decision_priority: u8,
    pub top_candidate_concept_id: Option<String>,
    pub candidate_concept_ids: Vec<String>,
    /// Gate result: would the promotion gate have fired (decision == offer_advance)?
    pub would_promote: bool,
    pub policy_version: String,
    pub basis_revision: i64, // revision of the scratch state the decision was computed from
    pub events_applied: usize, // fixture stream length replayed into scratch
}

#[derive(Default)]
pub struct ShadowGate {
    pub log: Vec<ShadowEntry>,
}

impl ShadowGate {
    pub fn new() -> Self {
        ShadowGate { log: Vec::new() }
    }

    /// Run the pipeline on `events` replayed into a scratch clone of `base`.
    /// `base` is taken by reference and is byte-identical afterwards (tested).
    /// Returns the recorded entry; the only mutation is to this gate's own log.
    pub fn run_on_stream(
        &mut self,
        base: &State,
        events: &[Event],
        curriculum: &Curriculum,
        goal: &Goal,
    ) -> Result<ShadowEntry, cogmap::Reject> {
        let mut scratch = base.clone();
        for e in events {
            scratch = reduce(scratch, e)?;
        }
        let brief: TeachingBrief = cogmap_brief(&scratch, curriculum, goal);
        let entry = ShadowEntry {
            shadow_seq: self.log.len() as i64 + 1,
            marker: "SHADOW".to_string(),
            evidence: false,
            decision: brief.decision,
            decision_priority: brief.decision_priority,
            top_candidate_concept_id: brief.candidates.first().map(|c| c.concept_id.clone()),
            candidate_concept_ids: brief
                .candidates
                .iter()
                .map(|c| c.concept_id.clone())
                .collect(),
            would_promote: brief.decision == Decision::OfferAdvance,
            policy_version: POLICY_VERSION.to_string(),
            basis_revision: scratch.revision,
            events_applied: events.len(),
        };
        self.log.push(entry.clone());
        Ok(entry)
    }
}
