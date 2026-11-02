//! Model Home cognitive map — Slice 3 read-only views (Blueprint §§F,G,J + Projection shape).
//! `cogmap_brief`: structured teaching brief. `cogmap_map`: inspectable map projection.
//! Both take `&State` and perform ZERO reducer writes — the model receives JSON and can
//! only speak; it cannot issue a grade, change a level, or write the map (I07/I08, §G).
//! Separation (§A): `knowledge_of_content` (versioned curriculum graph + packet refs)
//! is stored/returned apart from `knowledge_about_learner` (evidence cells). Relatedness
//! and content proximity never become learner evidence (I12).
use cogmap::{
    build_window, concept_status, decide, mean_ge_80, promotion_eligible, support_trigger,
    AttemptState, ConceptStatus, Decision, Observation, State, POLICY_VERSION,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

// ---------- curriculum manifest (published, versioned; §J manifest contract) ----------
#[derive(Clone, Debug, PartialEq)]
pub struct ConceptInfo {
    pub id: String,
    pub label: String,
    pub dimension_ids: Vec<String>,
    pub prerequisite_ids: Vec<String>,
    pub canonical_course_order: i64,
    pub canonical_module_order: i64,
}
/// A content reference: where curriculum content lives. This is knowledge OF
/// content — it says nothing about the learner and must never be read as evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct ContentRef {
    pub concept_id: String,
    pub module_id: String,
    pub source_path: String,
    pub source_hash: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Curriculum {
    pub version: String,
    pub concepts: Vec<ConceptInfo>,
    pub module_refs: Vec<ContentRef>,
}
impl Curriculum {
    pub fn concept(&self, id: &str) -> Option<&ConceptInfo> {
        self.concepts.iter().find(|c| c.id == id)
    }
}

// ---------- goal (learner-declared scope for a brief; §G active goal) ----------
#[derive(Clone, Debug)]
pub struct Goal {
    pub active_concept_ids: Vec<String>,
    pub current_difficulty: u8, // current D for the goal's dimensions (placement lives outside reducer in v1.1)
    pub paused_or_declined: bool,
    pub missing_artifact_attempt: Option<String>,
    pub failed_remediation_cycles: u32,
}

// ---------- shared evidence claim shape (I07) ----------
/// Every non-unknown claim cites its assessment evidence + policy version.
#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceRef {
    pub assessment_id: String,
    pub attempt_id: String,
    pub concept_id: String,
    pub dimension: String,
    pub policy_version: String,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rate {
    pub numerator: i64,
    pub denominator: i64,
}

// ---------- map projection cells (§1 cell fields, Projection shape) ----------
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub concept_id: String,
    pub dimension: String,
    pub difficulty: u8,
    pub status: ConceptStatus,
    pub window_assessment_ids: Vec<String>,
    /// Null (None) when unobserved — never zero (§J projection: return null, not zero).
    pub mean_score_units: Option<i64>,
    pub n_clean: i64,
    pub n_defective: i64,
    pub n_independent: i64,
    pub fnr: Option<Rate>,
    pub fpr: Option<Rate>,
    pub last_assessed_seq: Option<i64>,
    pub last_independent_seq: Option<i64>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub policy_version: String,
    pub projection_revision: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Misconception {
    pub claim: String,
    pub concept_id: String,
    pub status: String, // tentative|supported|contested|resolved (reviewed claims only, §1/§F)
    pub supporting_assessment_ids: Vec<String>,
    pub counterexample_assessment_ids: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ContentGraph {
    pub curriculum_version: String,
    pub concepts: Vec<ConceptInfo>,
    pub module_refs: Vec<ContentRef>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LearnerOverlay {
    pub cells: Vec<Cell>,
    pub pending_assessments: Vec<String>, // attempt ids; pending is absent evidence, not zero (§F)
}
#[derive(Clone, Debug, PartialEq)]
pub struct MapProjection {
    pub enrollment_id: String,
    pub revision: i64,
    pub through_seq: i64,
    pub curriculum_version: String,
    pub policy_version: String,
    pub dimensions: Vec<String>,
    pub knowledge_of_content: ContentGraph,
    pub knowledge_about_learner: LearnerOverlay,
    pub misconceptions: Vec<Misconception>,
    pub pending_assessments: Vec<String>,
    pub active_decision_id: Option<String>,
    pub updated_at: Option<String>,
}

// ---------- teaching brief (§G) ----------
#[derive(Clone, Debug, PartialEq)]
pub struct WindowSummary {
    pub dimension: String,
    pub n: i64,
    pub sum_units: i64,
    pub mean_ge_80: bool,
    pub defective_count: i64,
    pub miss_count: i64,
    pub clean_opp_count: i64,
    pub false_alarm_count: i64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub concept_id: String,
    pub outside_active_goal: bool,
    pub prerequisite_distance: i64,
    pub independent_coverage_capped_at_5: i64,
    pub last_independent_seq_or_minus_1: i64,
    pub canonical_course_order: i64,
    pub canonical_module_order: i64,
    pub evidence_refs: Vec<EvidenceRef>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TeachingBrief {
    pub decision: Decision,
    pub decision_priority: u8, // 1..8, §G table order
    pub goal_concept_ids: Vec<String>,
    pub candidates: Vec<Candidate>,
    pub window_summaries: Vec<WindowSummary>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub policy_version: String,
    pub basis_revision: i64,
    pub through_seq: i64,
}

// ---------- helpers (pure reads only) ----------
pub fn status_str(s: ConceptStatus) -> &'static str {
    match s {
        ConceptStatus::Unknown => "unknown",
        ConceptStatus::NeedsRecheck => "needs_recheck",
        ConceptStatus::Insufficient => "insufficient",
        ConceptStatus::Developing => "developing",
        ConceptStatus::DemonstratedAtD => "demonstrated",
    }
}
pub fn decision_str(d: Decision) -> &'static str {
    match d {
        Decision::Hold => "hold",
        Decision::Explain => "explain",
        Decision::AskEvidence => "ask_evidence",
        Decision::HumanReview => "human_review",
        Decision::Diagnostic => "diagnostic",
        Decision::OfferPrereq => "offer_prereq",
        Decision::OfferAdvance => "offer_advance",
        Decision::NextPractice => "next_practice",
    }
}
pub fn decision_priority(d: Decision) -> u8 {
    match d {
        Decision::Hold => 1,
        Decision::Explain => 2,
        Decision::AskEvidence => 3,
        Decision::HumanReview => 4,
        Decision::Diagnostic => 5,
        Decision::OfferPrereq => 6,
        Decision::OfferAdvance => 7,
        Decision::NextPractice => 8,
    }
}

fn through_seq(state: &State) -> i64 {
    state.observations.iter().map(|o| o.seq).max().unwrap_or(0)
}

/// Concept-mapped observations only: an observation counts toward a concept solely
/// because its attempt was assigned to that concept. Dimension scores are never
/// copied across concepts (§F, forbidden).
fn concept_window(
    state: &State,
    concept_id: &str,
    dimension: &str,
    difficulty: u8,
) -> Vec<Observation> {
    let mut by_attempt: BTreeMap<String, Observation> = BTreeMap::new();
    for o in state
        .observations
        .iter()
        .filter(|o| o.dimension == dimension)
    {
        if let Some(att) = state.attempts.get(&o.attempt_id) {
            if att.concept_id == concept_id && att.difficulty == difficulty {
                by_attempt
                    .entry(o.attempt_id.clone())
                    .or_insert_with(|| o.clone());
            }
        }
    }
    let mut w: Vec<Observation> = by_attempt.into_values().collect();
    w.sort_by_key(|o| o.seq);
    if w.len() > 8 {
        w.drain(..w.len() - 8);
    }
    w
}

fn evidence_refs_for(
    state: &State,
    window: &[Observation],
    concept_id: &str,
    dimension: &str,
) -> Vec<EvidenceRef> {
    window
        .iter()
        .filter_map(|o| {
            state.heads.get(&o.attempt_id).map(|h| EvidenceRef {
                assessment_id: h.id.clone(),
                attempt_id: o.attempt_id.clone(),
                concept_id: concept_id.to_string(),
                dimension: dimension.to_string(),
                policy_version: POLICY_VERSION.to_string(),
            })
        })
        .collect()
}

fn cell_for(state: &State, concept_id: &str, dimension: &str, difficulty: u8) -> Cell {
    let w = concept_window(state, concept_id, dimension, difficulty);
    let has_evidence = !w.is_empty();
    let contested = state.attempts.values().any(|a| {
        a.concept_id == concept_id
            && a.dimension == dimension
            && a.difficulty == difficulty
            && a.state == AttemptState::Disputed
    });
    let sum: i64 = w.iter().map(|o| o.score_units).sum();
    let n = w.len() as i64;
    let distinct_variants: BTreeSet<&str> = w.iter().map(|o| o.variant.as_str()).collect();
    let independent = w.iter().filter(|o| o.independent).count();
    let defective = w.iter().filter(|o| o.defective_task).count() as i64;
    let miss = w.iter().filter(|o| o.false_negative).count() as i64;
    let clean_opp = w.iter().filter(|o| !o.defective_task).count() as i64;
    let false_alarm = w.iter().filter(|o| o.false_positive).count() as i64;
    let mean_ok = has_evidence && mean_ge_80(sum, n);
    let gate_ok = (!has_evidence)
        || ((defective == 0 || cogmap::fnr_ok(defective, miss))
            && (clean_opp == 0 || cogmap::fpr_ok(clean_opp, false_alarm)));
    let status = concept_status(
        w.len(),
        independent,
        distinct_variants.len(),
        mean_ok,
        gate_ok,
        contested,
        false,
        has_evidence,
    );
    Cell {
        concept_id: concept_id.to_string(),
        dimension: dimension.to_string(),
        difficulty,
        status,
        window_assessment_ids: evidence_refs_for(state, &w, concept_id, dimension)
            .iter()
            .map(|r| r.assessment_id.clone())
            .collect(),
        mean_score_units: if has_evidence { Some(sum / n) } else { None },
        n_clean: w.iter().filter(|o| !o.defective_task).count() as i64,
        n_defective: defective,
        n_independent: independent as i64,
        fnr: if defective > 0 {
            Some(Rate {
                numerator: miss,
                denominator: defective,
            })
        } else {
            None
        },
        fpr: if clean_opp > 0 {
            Some(Rate {
                numerator: false_alarm,
                denominator: clean_opp,
            })
        } else {
            None
        },
        last_assessed_seq: w.iter().map(|o| o.seq).max(),
        last_independent_seq: w.iter().filter(|o| o.independent).map(|o| o.seq).max(),
        evidence_refs: evidence_refs_for(state, &w, concept_id, dimension),
        policy_version: POLICY_VERSION.to_string(),
        projection_revision: state.revision,
    }
}

fn concept_demonstrated(state: &State, curriculum: &Curriculum, concept_id: &str) -> bool {
    match curriculum.concept(concept_id) {
        Some(c) => c.dimension_ids.iter().any(|d| {
            (0..=5u8).any(|diff| {
                cell_for(state, concept_id, d, diff).status == ConceptStatus::DemonstratedAtD
            })
        }),
        None => false,
    }
}

/// Transitive prerequisite closure of the active goal, with BFS distances (§G).
fn reachable_with_distance(curriculum: &Curriculum, goal: &Goal) -> HashMap<String, i64> {
    let mut dist: HashMap<String, i64> = HashMap::new();
    let mut queue: VecDeque<(String, i64)> = VecDeque::new();
    for id in &goal.active_concept_ids {
        dist.insert(id.clone(), 0);
        queue.push_back((id.clone(), 0));
    }
    while let Some((id, d)) = queue.pop_front() {
        if let Some(c) = curriculum.concept(&id) {
            for pre in &c.prerequisite_ids {
                if !dist.contains_key(pre) {
                    dist.insert(pre.clone(), d + 1);
                    queue.push_back((pre.clone(), d + 1));
                }
            }
        }
    }
    dist
}

// ---------- cogmap_map: inspectable projection (READ-ONLY) ----------
pub fn cogmap_map(
    state: &State,
    enrollment_id: &str,
    curriculum: &Curriculum,
    updated_at: Option<&str>,
) -> MapProjection {
    let mut dims: BTreeSet<String> = BTreeSet::new();
    for c in &curriculum.concepts {
        for d in &c.dimension_ids {
            dims.insert(d.clone());
        }
    }
    for o in &state.observations {
        dims.insert(o.dimension.clone());
    }

    let mut cells = Vec::new();
    for c in &curriculum.concepts {
        for d in &c.dimension_ids {
            for diff in 0..=5u8 {
                cells.push(cell_for(state, &c.id, d, diff));
            }
        }
    }
    let pending: Vec<String> = state
        .attempts
        .values()
        .filter(|a| a.state == AttemptState::AssessmentPending)
        .map(|a| a.id.clone())
        .collect();
    MapProjection {
        enrollment_id: enrollment_id.to_string(),
        revision: state.revision,
        through_seq: through_seq(state),
        curriculum_version: curriculum.version.clone(),
        policy_version: POLICY_VERSION.to_string(),
        dimensions: dims.into_iter().collect(),
        knowledge_of_content: ContentGraph {
            curriculum_version: curriculum.version.clone(),
            concepts: curriculum.concepts.clone(),
            module_refs: curriculum.module_refs.clone(),
        },
        knowledge_about_learner: LearnerOverlay {
            cells,
            pending_assessments: pending.clone(),
        },
        misconceptions: Vec::new(), // reviewed claims only; reducer holds none in v1.1 slices
        pending_assessments: pending,
        active_decision_id: None,
        updated_at: updated_at.map(|s| s.to_string()),
    }
}

// ---------- cogmap_brief: structured teaching brief (READ-ONLY) ----------
pub fn cogmap_brief(state: &State, curriculum: &Curriculum, goal: &Goal) -> TeachingBrief {
    let dist = reachable_with_distance(curriculum, goal);
    let scope: BTreeSet<String> = dist.keys().cloned().collect();

    // Priority 2: relevant grading disputed/invalid/pending, scoped to goal (§G note:
    // a dispute in an unrelated concept must not freeze the course).
    let grading_blocked = state.attempts.values().any(|a| {
        scope.contains(&a.concept_id)
            && matches!(
                a.state,
                AttemptState::Disputed | AttemptState::Invalid | AttemptState::AssessmentPending
            )
    });
    // Priority 3: current attempt missing required artifact (revealed, not submitted).
    let missing_artifact = goal.missing_artifact_attempt.is_some()
        || state
            .attempts
            .values()
            .any(|a| scope.contains(&a.concept_id) && a.state == AttemptState::Revealed);
    // Priority 5: support trigger on any in-scope dimension window (§F integer scores).
    let mut scope_dims: BTreeSet<String> = BTreeSet::new();
    for a in state.attempts.values() {
        if scope.contains(&a.concept_id) {
            scope_dims.insert(a.dimension.clone());
        }
    }
    let support = scope_dims.iter().any(|d| {
        let obs: Vec<Observation> = state
            .observations
            .iter()
            .filter(|o| {
                state
                    .attempts
                    .get(&o.attempt_id)
                    .map(|a| scope.contains(&a.concept_id))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        support_trigger(&build_window(&obs, d))
    });
    // Priority 6: a direct prerequisite of an active concept is not demonstrated.
    let prereq_missing = goal.active_concept_ids.iter().any(|id| {
        curriculum
            .concept(id)
            .map(|c| {
                c.prerequisite_ids
                    .iter()
                    .any(|p| !concept_demonstrated(state, curriculum, p))
            })
            .unwrap_or(false)
    });
    // Priority 7: promotion gates on any in-scope dimension window.
    let promotion = scope_dims.iter().any(|d| {
        let w = build_window(&state.observations, d);
        let independent_count = w.iter().filter(|o| o.independent).count();
        let clean_ind = w.iter().filter(|o| o.independent && o.fully_clean).count();
        let def_ind = w
            .iter()
            .filter(|o| o.independent && o.defective_task)
            .count();
        let sum: i64 = w.iter().map(|o| o.score_units).sum();
        let defective = w.iter().filter(|o| o.defective_task).count() as i64;
        let miss = w.iter().filter(|o| o.false_negative).count() as i64;
        let clean_opp = w.iter().filter(|o| !o.defective_task).count() as i64;
        let fa = w.iter().filter(|o| o.false_positive).count() as i64;
        let gates = mean_ge_80(sum, w.len() as i64)
            && (defective == 0 || cogmap::fnr_ok(defective, miss))
            && (clean_opp == 0 || cogmap::fpr_ok(clean_opp, fa));
        promotion_eligible(
            goal.current_difficulty,
            &w,
            independent_count,
            clean_ind,
            def_ind,
            gates,
            false,
            false,
        )
    });

    let decision = decide(
        goal.paused_or_declined,
        grading_blocked,
        missing_artifact,
        goal.failed_remediation_cycles,
        support,
        prereq_missing,
        promotion,
    );

    // Candidates: reachable concepts only (unreachable excluded, §G), ordered by the
    // lexicographic tuple ascending.
    let mut candidates: Vec<Candidate> = Vec::new();
    for (concept_id, distance) in &dist {
        let info = curriculum.concept(concept_id);
        let obs: Vec<&Observation> = state
            .observations
            .iter()
            .filter(|o| {
                state
                    .attempts
                    .get(&o.attempt_id)
                    .map(|a| &a.concept_id == concept_id)
                    .unwrap_or(false)
            })
            .collect();
        let independent = obs.iter().filter(|o| o.independent).count() as i64;
        let last_ind = obs
            .iter()
            .filter(|o| o.independent)
            .map(|o| o.seq)
            .max()
            .unwrap_or(-1);
        let mut refs: Vec<EvidenceRef> = Vec::new();
        for o in &obs {
            if let Some(h) = state.heads.get(&o.attempt_id) {
                refs.push(EvidenceRef {
                    assessment_id: h.id.clone(),
                    attempt_id: o.attempt_id.clone(),
                    concept_id: concept_id.clone(),
                    dimension: o.dimension.clone(),
                    policy_version: POLICY_VERSION.to_string(),
                });
            }
        }
        candidates.push(Candidate {
            concept_id: concept_id.clone(),
            outside_active_goal: !goal.active_concept_ids.contains(concept_id),
            prerequisite_distance: *distance,
            independent_coverage_capped_at_5: independent.min(5),
            last_independent_seq_or_minus_1: last_ind,
            canonical_course_order: info.map(|c| c.canonical_course_order).unwrap_or(i64::MAX),
            canonical_module_order: info.map(|c| c.canonical_module_order).unwrap_or(i64::MAX),
            evidence_refs: refs,
        });
    }
    candidates.sort_by(|a, b| {
        (
            a.outside_active_goal,
            a.prerequisite_distance,
            a.independent_coverage_capped_at_5,
            a.last_independent_seq_or_minus_1,
            a.canonical_course_order,
            a.canonical_module_order,
            a.concept_id.clone(),
        )
            .cmp(&(
                b.outside_active_goal,
                b.prerequisite_distance,
                b.independent_coverage_capped_at_5,
                b.last_independent_seq_or_minus_1,
                b.canonical_course_order,
                b.canonical_module_order,
                b.concept_id.clone(),
            ))
    });

    // Window summaries (integers only) for in-scope dimensions.
    let mut window_summaries = Vec::new();
    for d in &scope_dims {
        let w = build_window(&state.observations, d);
        let sum: i64 = w.iter().map(|o| o.score_units).sum();
        window_summaries.push(WindowSummary {
            dimension: d.clone(),
            n: w.len() as i64,
            sum_units: sum,
            mean_ge_80: mean_ge_80(sum, w.len() as i64),
            defective_count: w.iter().filter(|o| o.defective_task).count() as i64,
            miss_count: w.iter().filter(|o| o.false_negative).count() as i64,
            clean_opp_count: w.iter().filter(|o| !o.defective_task).count() as i64,
            false_alarm_count: w.iter().filter(|o| o.false_positive).count() as i64,
        });
    }
    let mut all_refs: Vec<EvidenceRef> = candidates
        .iter()
        .flat_map(|c| c.evidence_refs.clone())
        .collect();
    all_refs.sort_by(|a, b| a.assessment_id.cmp(&b.assessment_id));
    all_refs.dedup();

    TeachingBrief {
        decision,
        decision_priority: decision_priority(decision),
        goal_concept_ids: goal.active_concept_ids.clone(),
        candidates,
        window_summaries,
        evidence_refs: all_refs,
        policy_version: POLICY_VERSION.to_string(),
        basis_revision: state.revision,
        through_seq: through_seq(state),
    }
}

// ---------- minimal JSON (zero-dependency, hermetic like Slices 1-2) ----------
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
fn opt_i64(v: Option<i64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "null".into())
}
fn opt_str(v: &Option<String>) -> String {
    v.as_ref()
        .map(|s| format!("\"{}\"", esc(s)))
        .unwrap_or_else(|| "null".into())
}
fn refs_json(refs: &[EvidenceRef]) -> String {
    let items: Vec<String> = refs.iter().map(|r| format!(
        "{{\"assessmentId\":\"{}\",\"attemptId\":\"{}\",\"conceptId\":\"{}\",\"dimension\":\"{}\",\"policyVersion\":\"{}\"}}",
        esc(&r.assessment_id), esc(&r.attempt_id), esc(&r.concept_id), esc(&r.dimension), esc(&r.policy_version)
    )).collect();
    format!("[{}]", items.join(","))
}
fn rate_json(r: Option<Rate>) -> String {
    r.map(|x| {
        format!(
            "{{\"numerator\":{},\"denominator\":{}}}",
            x.numerator, x.denominator
        )
    })
    .unwrap_or_else(|| "null".into())
}
impl Cell {
    pub fn to_json(&self) -> String {
        let ids: Vec<String> = self
            .window_assessment_ids
            .iter()
            .map(|i| format!("\"{}\"", esc(i)))
            .collect();
        format!(
            "{{\"conceptId\":\"{}\",\"dimension\":\"{}\",\"difficulty\":{},\"status\":\"{}\",\"windowAssessmentIds\":[{}],\"meanScoreUnits\":{},\"nClean\":{},\"nDefective\":{},\"nIndependent\":{},\"fnr\":{},\"fpr\":{},\"lastAssessedSeq\":{},\"lastIndependentSeq\":{},\"evidenceRefs\":{},\"policyVersion\":\"{}\",\"projectionRevision\":{}}}",
            esc(&self.concept_id), esc(&self.dimension), self.difficulty, status_str(self.status),
            ids.join(","), opt_i64(self.mean_score_units), self.n_clean, self.n_defective, self.n_independent,
            rate_json(self.fnr), rate_json(self.fpr), opt_i64(self.last_assessed_seq), opt_i64(self.last_independent_seq),
            refs_json(&self.evidence_refs), esc(&self.policy_version), self.projection_revision
        )
    }
}
impl MapProjection {
    pub fn to_json(&self) -> String {
        let dims: Vec<String> = self
            .dimensions
            .iter()
            .map(|d| format!("\"{}\"", esc(d)))
            .collect();
        let concepts: Vec<String> = self.knowledge_of_content.concepts.iter().map(|c| {
            let dims: Vec<String> = c.dimension_ids.iter().map(|d| format!("\"{}\"", esc(d))).collect();
            let pres: Vec<String> = c.prerequisite_ids.iter().map(|d| format!("\"{}\"", esc(d))).collect();
            format!("{{\"id\":\"{}\",\"label\":\"{}\",\"dimensionIds\":[{}],\"prerequisiteIds\":[{}],\"canonicalCourseOrder\":{},\"canonicalModuleOrder\":{}}}",
                esc(&c.id), esc(&c.label), dims.join(","), pres.join(","), c.canonical_course_order, c.canonical_module_order)
        }).collect();
        let refs: Vec<String> = self.knowledge_of_content.module_refs.iter().map(|r| format!(
            "{{\"conceptId\":\"{}\",\"moduleId\":\"{}\",\"sourcePath\":\"{}\",\"sourceHash\":\"{}\"}}",
            esc(&r.concept_id), esc(&r.module_id), esc(&r.source_path), esc(&r.source_hash)
        )).collect();
        let cells: Vec<String> = self
            .knowledge_about_learner
            .cells
            .iter()
            .map(|c| c.to_json())
            .collect();
        let pending: Vec<String> = self
            .pending_assessments
            .iter()
            .map(|p| format!("\"{}\"", esc(p)))
            .collect();
        format!(
            "{{\"enrollmentId\":\"{}\",\"revision\":{},\"throughSeq\":{},\"curriculumVersion\":\"{}\",\"policyVersion\":\"{}\",\"dimensions\":[{}],\"knowledgeOfContent\":{{\"curriculumVersion\":\"{}\",\"concepts\":[{}],\"moduleRefs\":[{}]}},\"knowledgeAboutLearner\":{{\"cells\":[{}]}},\"misconceptions\":[],\"pendingAssessments\":[{}],\"activeDecisionId\":{},\"updatedAt\":{}}}",
            esc(&self.enrollment_id), self.revision, self.through_seq, esc(&self.curriculum_version), esc(&self.policy_version),
            dims.join(","), esc(&self.knowledge_of_content.curriculum_version), concepts.join(","), refs.join(","),
            cells.join(","), pending.join(","), opt_str(&self.active_decision_id), opt_str(&self.updated_at)
        )
    }
}
impl TeachingBrief {
    pub fn to_json(&self) -> String {
        let cands: Vec<String> = self.candidates.iter().map(|c| format!(
            "{{\"conceptId\":\"{}\",\"outsideActiveGoal\":{},\"prerequisiteDistance\":{},\"independentCoverageCappedAt5\":{},\"lastIndependentSeqOrMinus1\":{},\"canonicalCourseOrder\":{},\"canonicalModuleOrder\":{},\"evidenceRefs\":{}}}",
            esc(&c.concept_id), c.outside_active_goal, c.prerequisite_distance, c.independent_coverage_capped_at_5,
            c.last_independent_seq_or_minus_1, c.canonical_course_order, c.canonical_module_order, refs_json(&c.evidence_refs)
        )).collect();
        let wins: Vec<String> = self.window_summaries.iter().map(|w| format!(
            "{{\"dimension\":\"{}\",\"n\":{},\"sumUnits\":{},\"meanGe80\":{},\"defectiveCount\":{},\"missCount\":{},\"cleanOppCount\":{},\"falseAlarmCount\":{}}}",
            esc(&w.dimension), w.n, w.sum_units, w.mean_ge_80, w.defective_count, w.miss_count, w.clean_opp_count, w.false_alarm_count
        )).collect();
        let goals: Vec<String> = self
            .goal_concept_ids
            .iter()
            .map(|g| format!("\"{}\"", esc(g)))
            .collect();
        format!(
            "{{\"decision\":\"{}\",\"decisionPriority\":{},\"goalConceptIds\":[{}],\"candidates\":[{}],\"windowSummaries\":[{}],\"evidenceRefs\":{},\"policyVersion\":\"{}\",\"basisRevision\":{},\"throughSeq\":{}}}",
            decision_str(self.decision), self.decision_priority, goals.join(","), cands.join(","), wins.join(","),
            refs_json(&self.evidence_refs), esc(&self.policy_version), self.basis_revision, self.through_seq
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cogmap::{reduce, Event};

    fn curriculum() -> Curriculum {
        Curriculum {
            version: "aie-v1".into(),
            concepts: vec![
                ConceptInfo {
                    id: "c-goal".into(),
                    label: "Goal concept".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec!["c-pre".into()],
                    canonical_course_order: 2,
                    canonical_module_order: 1,
                },
                ConceptInfo {
                    id: "c-pre".into(),
                    label: "Prerequisite".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec![],
                    canonical_course_order: 1,
                    canonical_module_order: 1,
                },
                ConceptInfo {
                    id: "c-far".into(),
                    label: "Unrelated".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec![],
                    canonical_course_order: 9,
                    canonical_module_order: 9,
                },
                ConceptInfo {
                    id: "c-empty".into(),
                    label: "No evidence yet".into(),
                    dimension_ids: vec!["SPEC".into()],
                    prerequisite_ids: vec![],
                    canonical_course_order: 3,
                    canonical_module_order: 1,
                },
            ],
            module_refs: vec![ContentRef {
                concept_id: "c-pre".into(),
                module_id: "AIE-101-M01".into(),
                source_path: "materials/AIE-101/AIE-101-M01.md".into(),
                source_hash: "sha256:packet-hash-not-evidence".into(),
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
    fn assessed(
        s: State,
        att: &str,
        concept: &str,
        variant: &str,
        seq: i64,
        grade_id: &str,
        v: u8,
    ) -> State {
        let mut s = s;
        let p = att.to_string();
        s = reduce(
            s,
            &Event::AssignTask {
                event_id: format!("{p}-assign"),
                attempt_id: p.clone(),
                concept_id: concept.into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: variant.into(),
                independent: true,
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::CommitPrediction {
                event_id: format!("{p}-pred"),
                attempt_id: p.clone(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Reveal {
                event_id: format!("{p}-reveal"),
                attempt_id: p.clone(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Submit {
                event_id: format!("{p}-submit"),
                attempt_id: p.clone(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::AssessmentAccepted {
                event_id: format!("{p}-accept"),
                attempt_id: p.clone(),
                assessment_id: grade_id.into(),
                v,
                j: 3,
                q: 4,
                seq,
                false_negative: false,
                false_positive: false,
                fully_clean: v == 1,
                defective_task: false,
            },
        )
        .unwrap();
        s
    }
    fn demonstrated_state() -> State {
        // c-pre demonstrated at D1: 3 distinct attempts, >=1 independent, 2 variants, high scores.
        let mut s = State::default();
        s = assessed(s, "a1", "c-pre", "v1", 1, "g1", 1);
        s = assessed(s, "a2", "c-pre", "v2", 2, "g2", 1);
        s = assessed(s, "a3", "c-pre", "v1", 3, "g3", 1);
        s
    }

    #[test]
    fn brief_and_map_are_read_only_zero_mutation() {
        let s = demonstrated_state();
        let before = s.clone();
        let brief = cogmap_brief(&s, &curriculum(), &goal());
        let proj = cogmap_map(
            &s,
            "local-stewart",
            &curriculum(),
            Some("2026-10-06T00:00:00Z"),
        );
        assert_eq!(s, before); // full state equality: no reducer writes happened
        assert_eq!(s.revision, before.revision);
        assert_eq!(brief.basis_revision, before.revision);
        assert_eq!(proj.revision, before.revision);
    }

    #[test]
    fn decision_priority_order_on_fixtures() {
        let cur = curriculum();
        // 1: paused beats everything
        let mut g = goal();
        g.paused_or_declined = true;
        let b = cogmap_brief(&demonstrated_state(), &cur, &g);
        assert_eq!((b.decision, b.decision_priority), (Decision::Hold, 1));
        // 2: pending assessment in scope blocks with explain
        let mut s = demonstrated_state();
        s = reduce(
            s,
            &Event::AssignTask {
                event_id: "p-assign".into(),
                attempt_id: "pend".into(),
                concept_id: "c-goal".into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::CommitPrediction {
                event_id: "p-pred".into(),
                attempt_id: "pend".into(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Reveal {
                event_id: "p-reveal".into(),
                attempt_id: "pend".into(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Submit {
                event_id: "p-submit".into(),
                attempt_id: "pend".into(),
            },
        )
        .unwrap();
        let b = cogmap_brief(&s, &cur, &goal());
        assert_eq!((b.decision, b.decision_priority), (Decision::Explain, 2));
        // 3: revealed-but-unsubmitted in scope -> ask_evidence
        let mut s = demonstrated_state();
        s = reduce(
            s,
            &Event::AssignTask {
                event_id: "r-assign".into(),
                attempt_id: "rev".into(),
                concept_id: "c-goal".into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::CommitPrediction {
                event_id: "r-pred".into(),
                attempt_id: "rev".into(),
            },
        )
        .unwrap();
        s = reduce(
            s,
            &Event::Reveal {
                event_id: "r-reveal".into(),
                attempt_id: "rev".into(),
            },
        )
        .unwrap();
        let b = cogmap_brief(&s, &cur, &goal());
        assert_eq!(
            (b.decision, b.decision_priority),
            (Decision::AskEvidence, 3)
        );
        // 4: two failed remediation cycles
        let mut g = goal();
        g.failed_remediation_cycles = 2;
        let b = cogmap_brief(&demonstrated_state(), &cur, &g);
        assert_eq!(
            (b.decision, b.decision_priority),
            (Decision::HumanReview, 4)
        );
        // 6: prerequisite not demonstrated (empty state: c-pre unknown)
        let b = cogmap_brief(&State::default(), &cur, &goal());
        assert_eq!(
            (b.decision, b.decision_priority),
            (Decision::OfferPrereq, 6)
        );
        // 8: prereq demonstrated, nothing else fires -> next practice
        let b = cogmap_brief(&demonstrated_state(), &cur, &goal());
        assert_eq!(
            (b.decision, b.decision_priority),
            (Decision::NextPractice, 8)
        );
    }

    #[test]
    fn every_map_claim_has_evidence_or_is_unknown() {
        let proj = cogmap_map(&demonstrated_state(), "local-stewart", &curriculum(), None);
        for cell in &proj.knowledge_about_learner.cells {
            if cell.status == ConceptStatus::Unknown {
                assert!(cell.evidence_refs.is_empty());
                assert_eq!(cell.mean_score_units, None); // unknown is null, not zero
                assert_eq!(cell.fnr, None);
                assert_eq!(cell.fpr, None);
                assert!(cell.window_assessment_ids.is_empty());
            } else {
                assert!(
                    !cell.evidence_refs.is_empty(),
                    "non-unknown claim without evidence: {:?}",
                    cell
                );
                assert!(cell
                    .evidence_refs
                    .iter()
                    .all(|r| r.policy_version == POLICY_VERSION)); // I07
                assert!(cell.mean_score_units.is_some());
            }
        }
        // The demonstrated cell cites the actual assessment ids.
        let cell = proj
            .knowledge_about_learner
            .cells
            .iter()
            .find(|c| c.concept_id == "c-pre" && c.difficulty == 1)
            .unwrap();
        assert_eq!(cell.status, ConceptStatus::DemonstratedAtD);
        assert_eq!(
            cell.window_assessment_ids,
            vec!["g1".to_string(), "g2".into(), "g3".into()]
        );
        // Brief JSON also carries policy version (model can only speak from this).
        let brief = cogmap_brief(&demonstrated_state(), &curriculum(), &goal());
        assert!(brief
            .to_json()
            .contains("\"policyVersion\":\"atlas-evidence-v1.1\""));
        assert!(proj.to_json().contains("\"meanScoreUnits\":null"));
    }

    #[test]
    fn candidate_ordering_is_lexicographic_and_excludes_unreachable() {
        let brief = cogmap_brief(&demonstrated_state(), &curriculum(), &goal());
        let ids: Vec<&str> = brief
            .candidates
            .iter()
            .map(|c| c.concept_id.as_str())
            .collect();
        assert!(!ids.contains(&"c-far")); // unreachable from goal: excluded (§G)
        assert_eq!(ids[0], "c-goal"); // active goal first: outsideActiveGoal=false, distance 0
        assert_eq!(ids[1], "c-pre"); // its prerequisite next, distance 1
        let pre = brief
            .candidates
            .iter()
            .find(|c| c.concept_id == "c-pre")
            .unwrap();
        assert_eq!(pre.independent_coverage_capped_at_5, 3);
        assert_eq!(pre.last_independent_seq_or_minus_1, 3);
    }

    #[test]
    fn content_is_not_learner_evidence_separation() {
        let proj = cogmap_map(&demonstrated_state(), "local-stewart", &curriculum(), None);
        // Content side carries packet refs with source hashes.
        assert_eq!(proj.knowledge_of_content.module_refs.len(), 1);
        assert_eq!(
            proj.knowledge_of_content.module_refs[0].module_id,
            "AIE-101-M01"
        );
        // Learner side: no cell evidence ref may point at content (module ids / packet hashes).
        for cell in &proj.knowledge_about_learner.cells {
            for r in &cell.evidence_refs {
                assert!(!r.assessment_id.starts_with("AIE-"));
                assert_ne!(
                    r.assessment_id,
                    proj.knowledge_of_content.module_refs[0].source_hash
                );
            }
        }
        // A concept with content but no attempts is still unknown (content != competence).
        let empty = proj
            .knowledge_about_learner
            .cells
            .iter()
            .find(|c| c.concept_id == "c-empty" && c.difficulty == 1)
            .unwrap();
        assert_eq!(empty.status, ConceptStatus::Unknown);
    }
}
