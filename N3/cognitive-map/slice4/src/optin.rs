//! Opt-in state machine + kill switch (Blueprint §K gate 5, PLAN Slice 4).
//! Per enrollment: Shadow -> OptedIn -> Active. Opt-in and activation are HUMAN
//! commands only (actor must be Stewart); the model can never opt itself in (I08).
//! Before Active, applying any placement/promotion returns BlockedNotOptedIn.
//! After Active, placement is still OFFER-ONLY: an offer is returned, nothing moves;
//! the learner accepts and a recheck runs (Blueprint §F, I09). Kill switch (disable)
//! halts new assessments instantly — attempts may still be recorded, grading is
//! blocked — and every transition lands in an audit log.
use cogmap::State;

pub const HUMAN_ACTOR: &str = "Stewart";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationPhase {
    Shadow,
    OptedIn,
    Active,
    Disabled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditEntry {
    pub seq: i64,
    pub actor: String,
    pub action: String, // opt_in | activate | disable | placement_offered | placement_blocked
    pub phase_after: ActivationPhase,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActivationError {
    NotHuman,          // actor is not the named human; model/system cannot opt in
    BadPhase,          // transition not allowed from current phase
    BlockedNotOptedIn, // placement/promotion attempted before Active
    GradingHalted,     // kill switch on (or not Active): grading blocked, recording allowed
}

/// An offer, not a move. Carrying this value changes nothing; acceptance is a
/// separate learner command followed by a recheck in a later wiring step.
#[derive(Clone, Debug, PartialEq)]
pub struct PlacementOffer {
    pub concept_id: String,
    pub from_difficulty: u8,
    pub to_difficulty: u8,
    pub offer_only: bool, // always true in v1.1
}

#[derive(Clone, Debug)]
pub struct Activation {
    pub enrollment_id: String,
    pub phase: ActivationPhase,
    pub audit: Vec<AuditEntry>,
}

impl Activation {
    pub fn new(enrollment_id: &str) -> Self {
        Activation {
            enrollment_id: enrollment_id.to_string(),
            phase: ActivationPhase::Shadow,
            audit: Vec::new(),
        }
    }
    fn record(&mut self, actor: &str, action: &str) {
        let seq = self.audit.len() as i64 + 1;
        self.audit.push(AuditEntry {
            seq,
            actor: actor.to_string(),
            action: action.to_string(),
            phase_after: self.phase,
        });
    }
    fn require_human(actor: &str) -> Result<(), ActivationError> {
        if actor == HUMAN_ACTOR {
            Ok(())
        } else {
            Err(ActivationError::NotHuman)
        }
    }

    /// Shadow -> OptedIn. Human command only, audited.
    pub fn opt_in(&mut self, actor: &str) -> Result<(), ActivationError> {
        Self::require_human(actor)?;
        if self.phase != ActivationPhase::Shadow {
            return Err(ActivationError::BadPhase);
        }
        self.phase = ActivationPhase::OptedIn;
        self.record(actor, "opt_in");
        Ok(())
    }

    /// OptedIn -> Active. Human command only, audited.
    pub fn activate(&mut self, actor: &str) -> Result<(), ActivationError> {
        Self::require_human(actor)?;
        if self.phase != ActivationPhase::OptedIn {
            return Err(ActivationError::BadPhase);
        }
        self.phase = ActivationPhase::Active;
        self.record(actor, "activate");
        Ok(())
    }

    /// Kill switch: any phase -> Disabled. Human command only, audited.
    /// Attempts can still be recorded; grading is blocked (see `guard_grade`).
    pub fn disable(&mut self, actor: &str) -> Result<(), ActivationError> {
        Self::require_human(actor)?;
        self.phase = ActivationPhase::Disabled;
        self.record(actor, "disable");
        Ok(())
    }

    /// Recording attempts/events is always allowed, including while Disabled
    /// (kill switch blocks grading, not the ledger).
    pub fn guard_record(&self) -> Result<(), ActivationError> {
        Ok(())
    }

    /// Grading gate: only an Active enrollment may admit new assessments.
    /// Shadow/OptedIn have not completed opt-in; Disabled is the kill switch.
    pub fn guard_grade(&self) -> Result<(), ActivationError> {
        match self.phase {
            ActivationPhase::Active => Ok(()),
            _ => Err(ActivationError::GradingHalted),
        }
    }

    /// Apply a placement/promotion. Pre-Active: BlockedNotOptedIn (audited).
    /// Active: returns an OFFER only — learner state and difficulty do not move;
    /// acceptance + recheck are separate steps outside this call (I09).
    pub fn apply_placement(
        &mut self,
        _state: &State,
        concept_id: &str,
        from_difficulty: u8,
    ) -> Result<PlacementOffer, ActivationError> {
        if self.phase != ActivationPhase::Active {
            self.record(HUMAN_ACTOR, "placement_blocked");
            return Err(ActivationError::BlockedNotOptedIn);
        }
        self.record(HUMAN_ACTOR, "placement_offered");
        Ok(PlacementOffer {
            concept_id: concept_id.to_string(),
            from_difficulty,
            to_difficulty: from_difficulty + 1,
            offer_only: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow::ShadowGate;
    use cogmap::{reduce, Event};
    use cogmap_slice3::brief::{cogmap_map, ConceptInfo, ContentRef, Curriculum, Goal};

    fn curriculum() -> Curriculum {
        Curriculum {
            version: "aie-v1".into(),
            concepts: vec![
                ConceptInfo {
                    id: "c-goal".into(),
                    label: "Goal".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec!["c-pre".into()],
                    canonical_course_order: 2,
                    canonical_module_order: 1,
                },
                ConceptInfo {
                    id: "c-pre".into(),
                    label: "Pre".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec![],
                    canonical_course_order: 1,
                    canonical_module_order: 1,
                },
            ],
            module_refs: vec![ContentRef {
                concept_id: "c-pre".into(),
                module_id: "AIE-101-M01".into(),
                source_path: "materials/AIE-101/AIE-101-M01.md".into(),
                source_hash: "sha256:x".into(),
            }],
        }
    }
    fn goal() -> Goal {
        Goal {
            active_concept_ids: vec!["c-goal".into()],
            current_difficulty: 1,
            paused_or_declined: false,
            missing_artifact_attempt: None,
            failed_remediation_cycles: 0,
        }
    }
    fn stream(att: &str, concept: &str, variant: &str, seq: i64, grade: &str) -> Vec<Event> {
        vec![
            Event::AssignTask {
                event_id: format!("{att}-a"),
                attempt_id: att.into(),
                concept_id: concept.into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: variant.into(),
                independent: true,
            },
            Event::CommitPrediction {
                event_id: format!("{att}-p"),
                attempt_id: att.into(),
            },
            Event::Reveal {
                event_id: format!("{att}-r"),
                attempt_id: att.into(),
            },
            Event::Submit {
                event_id: format!("{att}-s"),
                attempt_id: att.into(),
            },
            Event::AssessmentAccepted {
                event_id: format!("{att}-g"),
                attempt_id: att.into(),
                assessment_id: grade.into(),
                v: 1,
                j: 3,
                q: 4,
                seq,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        ]
    }
    fn demonstrated_events() -> Vec<Event> {
        let mut v = Vec::new();
        v.extend(stream("a1", "c-pre", "v1", 1, "g1"));
        v.extend(stream("a2", "c-pre", "v2", 2, "g2"));
        v.extend(stream("a3", "c-pre", "v1", 3, "g3"));
        v
    }

    #[test]
    fn shadow_matches_live_pipeline_with_zero_mutation() {
        let base = State::default();
        let before = base.clone();
        let events = demonstrated_events();
        let mut live = base.clone();
        for e in &events {
            live = reduce(live, e).unwrap();
        }
        let live_brief = cogmap_slice3::brief::cogmap_brief(&live, &curriculum(), &goal());
        let mut gate = ShadowGate::new();
        let entry = gate
            .run_on_stream(&base, &events, &curriculum(), &goal())
            .unwrap();
        assert_eq!(entry.decision, live_brief.decision);
        assert_eq!(entry.decision_priority, live_brief.decision_priority);
        assert_eq!(
            entry.candidate_concept_ids,
            live_brief
                .candidates
                .iter()
                .map(|c| c.concept_id.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(entry.basis_revision, live.revision);
        assert_eq!(base, before);
        assert_eq!(base.revision, 0);
        assert!(base.observations.is_empty() && base.heads.is_empty());
        assert_eq!(gate.log.len(), 1);
        assert_eq!(entry.marker, "SHADOW");
        assert!(!entry.evidence);
    }

    #[test]
    fn shadow_entries_never_enter_evidence_windows() {
        let mut gate = ShadowGate::new();
        let base = State::default();
        gate.run_on_stream(&base, &demonstrated_events(), &curriculum(), &goal())
            .unwrap();
        let proj = cogmap_map(&base, "local-stewart", &curriculum(), None);
        for cell in &proj.knowledge_about_learner.cells {
            assert!(cell.evidence_refs.is_empty());
            assert!(cell
                .window_assessment_ids
                .iter()
                .all(|id| !id.contains("SHADOW")));
        }
        assert!(cogmap::build_window(&base.observations, "SPEC").is_empty());
    }

    #[test]
    fn placement_blocked_before_opt_in() {
        let s = State::default();
        let mut act = Activation::new("local-stewart");
        assert_eq!(
            act.apply_placement(&s, "c-pre", 1),
            Err(ActivationError::BlockedNotOptedIn)
        );
        act.opt_in("Stewart").unwrap();
        assert_eq!(
            act.apply_placement(&s, "c-pre", 1),
            Err(ActivationError::BlockedNotOptedIn)
        );
        assert!(
            act.audit
                .iter()
                .filter(|a| a.action == "placement_blocked")
                .count()
                == 2
        );
    }

    #[test]
    fn opt_in_is_human_only_and_placement_stays_offer_only() {
        let s = State::default();
        let mut act = Activation::new("local-stewart");
        assert_eq!(
            act.opt_in("qwen-local-model"),
            Err(ActivationError::NotHuman)
        );
        assert_eq!(act.phase, ActivationPhase::Shadow);
        act.opt_in("Stewart").unwrap();
        act.activate("Stewart").unwrap();
        let before = s.clone();
        let offer = act.apply_placement(&s, "c-pre", 1).unwrap();
        assert!(offer.offer_only);
        assert_eq!((offer.from_difficulty, offer.to_difficulty), (1, 2));
        assert_eq!(s, before);
        let actions: Vec<&str> = act.audit.iter().map(|a| a.action.as_str()).collect();
        assert_eq!(actions, vec!["opt_in", "activate", "placement_offered"]);
        assert!(act.audit.iter().all(|a| a.actor == "Stewart"));
    }

    #[test]
    fn kill_switch_blocks_grading_not_recording() {
        let mut act = Activation::new("local-stewart");
        act.opt_in("Stewart").unwrap();
        act.activate("Stewart").unwrap();
        assert!(act.guard_grade().is_ok());
        act.disable("Stewart").unwrap();
        assert_eq!(act.phase, ActivationPhase::Disabled);
        assert_eq!(act.guard_grade(), Err(ActivationError::GradingHalted));
        assert!(act.guard_record().is_ok());
        let mut s = State::default();
        s = reduce(
            s,
            &Event::AssignTask {
                event_id: "k1".into(),
                attempt_id: "ka".into(),
                concept_id: "c-pre".into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
        )
        .unwrap();
        assert_eq!(s.attempts["ka"].state, cogmap::AttemptState::Assigned);
        let mut act2 = Activation::new("local-stewart");
        assert_eq!(
            act2.disable("qwen-local-model"),
            Err(ActivationError::NotHuman)
        );
        assert_eq!(act.audit.last().unwrap().action, "disable");
    }

    #[test]
    fn grading_blocked_until_active() {
        let mut act = Activation::new("local-stewart");
        assert_eq!(act.guard_grade(), Err(ActivationError::GradingHalted));
        act.opt_in("Stewart").unwrap();
        assert_eq!(act.guard_grade(), Err(ActivationError::GradingHalted));
        act.activate("Stewart").unwrap();
        assert!(act.guard_grade().is_ok());
    }
}
