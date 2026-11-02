//! Belt tools (integration): the four cogmap_* handlers behind Activation guards.
//! `CogmapTool` mirrors the home-core Tool shape (name / JSON schema / invoke with
//! JSON in, JSON out) WITHOUT depending on home-core — the trait here is a local
//! stand-in with the same surface, so the Mac wiring is a thin adapter
//! (unverified, see README). Belt names fixed by PLAN: cogmap_brief,
//! cogmap_predict, cogmap_submit, cogmap_map.
//!
//! Guard policy (Slice 4 Activation):
//! - cogmap_brief / cogmap_map: READ-ONLY — allowed in every phase incl. Shadow
//!   and Disabled (kill switch blocks grading, never inspection).
//! - cogmap_predict: records a hash-sealed prediction (guard_record, always
//!   allowed); Slice 2 enforces commit-only-pre-reveal.
//! - cogmap_submit: records an immutable finalized submission (guard_record,
//!   always allowed); the GRADE step behind it runs only when
//!   Activation::guard_grade() passes (Active). Pre-opt-in / Disabled submits
//!   succeed as recordings and report grading:"blocked" — no assessment is
//!   admitted, no negative evidence minted.
use cogmap_slice2::attempt_loop::{GradeOutcome, LocalProcessRunner, Session, TaskTemplate};
use cogmap_slice3::brief::{cogmap_brief, cogmap_map, Curriculum, Goal};
use cogmap_slice4::optin::{Activation, ActivationError};
use std::collections::HashMap;

use crate::json::{parse as parse_json, Json};

/// Local stand-in for the home-core Tool trait (same name/schema/invoke shape).
pub trait CogmapTool {
    fn name(&self) -> &'static str;
    fn schema_json(&self) -> String;
    fn invoke(&mut self, ctx: &mut ToolCtx, input_json: &str) -> String;
}

/// Everything a handler needs: one Session (single state holder), the
/// enrollment's Activation, curriculum + goal for brief/map, template registry
/// and the deterministic runner for grading.
pub struct ToolCtx {
    pub enrollment_id: String,
    pub session: Session,
    pub activation: Activation,
    pub curriculum: Curriculum,
    pub goal: Goal,
    pub templates: HashMap<String, TaskTemplate>,
    pub runner: LocalProcessRunner,
}

fn ok(pairs: Vec<(&str, Json)>) -> String {
    let mut v = vec![("ok", Json::Bool(true))];
    v.extend(pairs);
    Json::obj(v).to_string()
}
fn fail(error: &str) -> String {
    Json::obj(vec![
        ("ok", Json::Bool(false)),
        ("error", Json::Str(error.to_string())),
    ])
    .to_string()
}
fn parse_input(input: &str) -> Result<Json, String> {
    parse_json(input)
}
fn req_str(input: &Json, key: &str) -> Result<String, String> {
    input
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or_else(|| format!("missing string field '{key}'"))
}

/// Shared goal resolution: caller may override the goal concept list per call.
fn goal_for(ctx: &ToolCtx, input: &Json) -> Goal {
    let mut g = ctx.goal.clone();
    if let Some(ids) = input.get("goalConceptIds").and_then(|v| v.str_vec()) {
        g.active_concept_ids = ids;
    }
    if let Some(p) = input.get("pausedOrDeclined").and_then(|v| v.as_bool()) {
        g.paused_or_declined = p;
    }
    g
}

// ---------------- cogmap_brief (read-only, all phases) ----------------
pub struct BriefTool;
impl CogmapTool for BriefTool {
    fn name(&self) -> &'static str {
        "cogmap_brief"
    }
    fn schema_json(&self) -> String {
        r#"{"name":"cogmap_brief","description":"Structured teaching brief for the model (read-only; model speaks from this, cannot write the map).","parameters":{"type":"object","properties":{"goalConceptIds":{"type":"array","items":{"type":"string"}},"pausedOrDeclined":{"type":"boolean"}}}}"#.to_string()
    }
    fn invoke(&mut self, ctx: &mut ToolCtx, input_json: &str) -> String {
        let input = match parse_input(input_json) {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let goal = goal_for(ctx, &input);
        let brief = cogmap_brief(&ctx.session.state, &ctx.curriculum, &goal);
        ok(vec![
            ("brief", Json::Str(brief.to_json())),
            ("phase", Json::Str(phase_name(ctx).to_string())),
        ])
        // NOTE: brief JSON is embedded as a string to keep this crate dependency-free;
        // the home-core adapter may re-parse it into a JSON value (unverified wiring).
    }
}

// ---------------- cogmap_map (read-only, all phases) ----------------
pub struct MapTool;
impl CogmapTool for MapTool {
    fn name(&self) -> &'static str {
        "cogmap_map"
    }
    fn schema_json(&self) -> String {
        r#"{"name":"cogmap_map","description":"Inspectable cognitive map projection; every non-unknown claim cites evidence + policy version (I07). Read-only.","parameters":{"type":"object","properties":{}}}"#.to_string()
    }
    fn invoke(&mut self, ctx: &mut ToolCtx, _input_json: &str) -> String {
        let proj = cogmap_map(
            &ctx.session.state,
            &ctx.enrollment_id,
            &ctx.curriculum,
            None,
        );
        ok(vec![
            ("map", Json::Str(proj.to_json())),
            ("phase", Json::Str(phase_name(ctx).to_string())),
        ])
    }
}

// ---------------- cogmap_predict (commit only, pre-reveal) ----------------
pub struct PredictTool;
impl CogmapTool for PredictTool {
    fn name(&self) -> &'static str {
        "cogmap_predict"
    }
    fn schema_json(&self) -> String {
        r#"{"name":"cogmap_predict","description":"Commit a hash-sealed prediction BEFORE reveal. Post-reveal commits are rejected.","parameters":{"type":"object","required":["attemptId","prediction","eventId"],"properties":{"attemptId":{"type":"string"},"prediction":{"type":"string"},"eventId":{"type":"string"}}}}"#.to_string()
    }
    fn invoke(&mut self, ctx: &mut ToolCtx, input_json: &str) -> String {
        if let Err(e) = ctx.activation.guard_record() {
            return fail(&format!("{e:?}"));
        }
        let input = match parse_input(input_json) {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let (attempt_id, prediction, event_id) = match (
            req_str(&input, "attemptId"),
            req_str(&input, "prediction"),
            req_str(&input, "eventId"),
        ) {
            (Ok(a), Ok(p), Ok(e)) => (a, p, e),
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => return fail(&e),
        };
        match ctx
            .session
            .commit_prediction(&attempt_id, prediction.as_bytes(), &event_id)
        {
            Ok(hash) => ok(vec![("predictionHash", Json::Str(hash))]),
            Err(e) => fail(&format!("{e:?}")),
        }
    }
}

// ---------------- cogmap_submit (immutable; grading gated on Active) ----------------
pub struct SubmitTool;
impl CogmapTool for SubmitTool {
    fn name(&self) -> &'static str {
        "cogmap_submit"
    }
    fn schema_json(&self) -> String {
        r#"{"name":"cogmap_submit","description":"Submit an immutable finalized artifact (SHA-256, one scored submission per attempt). Deterministic grading runs only when activation is Active; otherwise the submission is recorded and grading is reported blocked.","parameters":{"type":"object","required":["attemptId","submissionId","artifact","eventId"],"properties":{"attemptId":{"type":"string"},"submissionId":{"type":"string"},"artifact":{"type":"string"},"eventId":{"type":"string"},"templateId":{"type":"string"},"assessmentId":{"type":"string"},"acceptEventId":{"type":"string"}}}}"#.to_string()
    }
    fn invoke(&mut self, ctx: &mut ToolCtx, input_json: &str) -> String {
        if let Err(e) = ctx.activation.guard_record() {
            return fail(&format!("{e:?}"));
        }
        let input = match parse_input(input_json) {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let attempt_id = match req_str(&input, "attemptId") {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let submission_id = match req_str(&input, "submissionId") {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let artifact = match req_str(&input, "artifact") {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let event_id = match req_str(&input, "eventId") {
            Ok(v) => v,
            Err(e) => return fail(&e),
        };
        let sub =
            match ctx
                .session
                .submit(&attempt_id, &submission_id, artifact.as_bytes(), &event_id)
            {
                Ok(s) => s,
                Err(e) => return fail(&format!("{e:?}")),
            };
        // Grading gate: only Active may admit assessments (Slice 4 guard_grade).
        // No template named = recording only, by caller choice.
        let template_id = input
            .get("templateId")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        if template_id.is_none() {
            return ok(vec![
                ("submissionId", Json::Str(sub.submission_id)),
                ("artifactHash", Json::Str(sub.artifact_hash)),
                ("grading", Json::Str("not_requested".into())),
            ]);
        }
        if let Err(e) = ctx.activation.guard_grade() {
            return ok(vec![
                ("submissionId", Json::Str(sub.submission_id)),
                ("artifactHash", Json::Str(sub.artifact_hash)),
                ("grading", Json::Str("blocked".into())),
                ("reason", Json::Str(format!("{e:?}"))),
                ("phase", Json::Str(phase_name(ctx).to_string())),
            ]);
        }
        let template = match ctx.templates.get(&template_id.clone().unwrap()) {
            Some(t) => t.clone(),
            None => return fail("unknown templateId"),
        };
        let assessment_id = input
            .get("assessmentId")
            .and_then(|v| v.as_str())
            .unwrap_or("g-auto")
            .to_string();
        let accept_event = input
            .get("acceptEventId")
            .and_then(|v| v.as_str())
            .unwrap_or("e-accept")
            .to_string();
        match ctx.session.grade_and_admit(
            &attempt_id,
            &template,
            &mut ctx.runner,
            "e-job",
            &accept_event,
            &assessment_id,
        ) {
            Ok(GradeOutcome::Assessed(g, id)) => ok(vec![
                ("submissionId", Json::Str(sub.submission_id)),
                ("grading", Json::Str("assessed".into())),
                ("assessmentId", Json::Str(id)),
                ("v", Json::Num(g.v as f64)),
                ("j", Json::Num(g.j as f64)),
                ("q", Json::Num(g.q as f64)),
                ("falseNegative", Json::Bool(g.false_negative)),
                ("falsePositive", Json::Bool(g.false_positive)),
                ("fullyClean", Json::Bool(g.fully_clean)),
            ]),
            Ok(GradeOutcome::Unresolved(job)) => ok(vec![
                ("submissionId", Json::Str(sub.submission_id)),
                ("grading", Json::Str("unresolved".into())), // flaky: stays pending, no evidence
                ("jobId", Json::Str(job)),
            ]),
            Err(e) => fail(&format!("{e:?}")),
        }
    }
}

pub fn phase_name(ctx: &ToolCtx) -> &'static str {
    crate::persist::phase_str(ctx.activation.phase)
}

/// The belt: exactly the four PLAN tool names, in registration order.
pub fn register_belt() -> Vec<Box<dyn CogmapTool>> {
    vec![
        Box::new(BriefTool),
        Box::new(MapTool),
        Box::new(PredictTool),
        Box::new(SubmitTool),
    ]
}

/// Guard matrix cell, exposed for tests/diagnostics: which tools may run, and
/// whether their grading side effect is live, under a given activation phase.
pub fn guard_matrix(
    phase: cogmap_slice4::optin::ActivationPhase,
) -> Vec<(&'static str, bool, bool)> {
    // (tool, allowed, grading_live)
    let grading = phase == cogmap_slice4::optin::ActivationPhase::Active;
    vec![
        ("cogmap_brief", true, false),
        ("cogmap_map", true, false),
        ("cogmap_predict", true, false),
        ("cogmap_submit", true, grading),
    ]
}

pub fn grading_blocked_error() -> ActivationError {
    ActivationError::GradingHalted
}

#[cfg(test)]
mod tests {
    use super::*;
    use cogmap::AttemptState;
    use cogmap_slice2::attempt_loop::{CriterionKind, CriterionSpec};
    use cogmap_slice3::brief::{ConceptInfo, ContentRef};
    use cogmap_slice4::optin::ActivationPhase;

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
    fn template() -> TaskTemplate {
        TaskTemplate {
            template_id: "T1".into(),
            concept_id: "c-pre".into(),
            dimension: "SPEC".into(),
            difficulty: 1,
            subtlety: 1,
            variant: "v1".into(),
            independent: true,
            defective_task: false,
            required_defects: vec![],
            decoys: vec![],
            ground_truth: "verdict=clean".into(),
            task_body: "snippet".into(),
            test_bundle_name: "b.py".into(),
            test_bundle:
                "import sys\nprint('TEST t_verdict PASS')\nprint('TEST t_evidence PASS')\n".into(),
            criteria: vec![
                CriterionSpec {
                    test_id: "t_verdict".into(),
                    kind: CriterionKind::Verdict,
                },
                CriterionSpec {
                    test_id: "t_evidence".into(),
                    kind: CriterionKind::Evidence,
                },
            ],
        }
    }
    fn activation_in(phase: ActivationPhase) -> Activation {
        let mut a = Activation::new("local-stewart");
        match phase {
            ActivationPhase::Shadow => {}
            ActivationPhase::OptedIn => {
                a.opt_in("Stewart").unwrap();
            }
            ActivationPhase::Active => {
                a.opt_in("Stewart").unwrap();
                a.activate("Stewart").unwrap();
            }
            ActivationPhase::Disabled => {
                a.opt_in("Stewart").unwrap();
                a.activate("Stewart").unwrap();
                a.disable("Stewart").unwrap();
            }
        }
        a
    }
    /// Fresh ctx with attempt att-1 assigned + revealed (predict done via tool in-test where needed).
    fn ctx_in(phase: ActivationPhase, with_prediction: bool) -> ToolCtx {
        let t = template();
        let mut session = Session::new();
        session.assign(&t, "a-assign", "att-1").unwrap();
        if with_prediction {
            session
                .commit_prediction("att-1", b"sealed guess", "a-pred")
                .unwrap();
            session.reveal("att-1", "a-reveal", &t).unwrap();
        }
        let mut templates = HashMap::new();
        templates.insert("T1".to_string(), t);
        ToolCtx {
            enrollment_id: "local-stewart".into(),
            session,
            activation: activation_in(phase),
            curriculum: curriculum(),
            goal: goal(),
            templates,
            runner: LocalProcessRunner::new(),
        }
    }
    fn is_ok(out: &str) -> bool {
        out.contains("\"ok\":true")
    }

    #[test]
    fn belt_registers_exactly_the_four_plan_tools_with_schemas() {
        let belt = register_belt();
        let names: Vec<&str> = belt.iter().map(|t| t.name()).collect();
        assert_eq!(
            names,
            vec![
                "cogmap_brief",
                "cogmap_map",
                "cogmap_predict",
                "cogmap_submit"
            ]
        );
        for t in &belt {
            let s = t.schema_json();
            assert!(s.contains(&format!("\"name\":\"{}\"", t.name())));
            assert!(s.contains("\"parameters\""));
        }
    }

    #[test]
    fn guard_matrix_shape_is_phase_by_tool() {
        for phase in [
            ActivationPhase::Shadow,
            ActivationPhase::OptedIn,
            ActivationPhase::Active,
            ActivationPhase::Disabled,
        ] {
            let m = guard_matrix(phase);
            assert_eq!(m.len(), 4);
            assert!(m.iter().all(|(_, allowed, _)| *allowed)); // all four tools callable in every phase
            let submit = m.iter().find(|(n, _, _)| *n == "cogmap_submit").unwrap();
            assert_eq!(submit.2, phase == ActivationPhase::Active); // grading live only in Active
        }
    }

    #[test]
    fn shadow_brief_and_map_allowed_but_submit_grading_blocked_pre_opt_in() {
        for phase in [ActivationPhase::Shadow, ActivationPhase::OptedIn] {
            let mut ctx = ctx_in(phase, true);
            let mut brief = BriefTool;
            let out = brief.invoke(&mut ctx, "{}");
            assert!(is_ok(&out), "{phase:?} brief: {out}");
            assert!(out.contains("atlas-evidence-v1.1")); // brief carries policy version (I07)
            let mut map = MapTool;
            assert!(is_ok(&map.invoke(&mut ctx, "{}")));
            // Reading mutated nothing.
            assert_eq!(
                ctx.session.state.attempts["att-1"].state,
                AttemptState::Revealed
            );
            // Submit records fine, but grading is blocked and nothing is assessed.
            let mut submit = SubmitTool;
            let out = submit.invoke(&mut ctx,
                r#"{"attemptId":"att-1","submissionId":"sub-1","artifact":"my answer","eventId":"a-submit","templateId":"T1","assessmentId":"g1","acceptEventId":"a-accept"}"#);
            assert!(is_ok(&out), "{phase:?} submit: {out}");
            assert!(out.contains("\"grading\":\"blocked\""), "{phase:?}: {out}");
            assert_eq!(
                ctx.session.state.attempts["att-1"].state,
                AttemptState::AssessmentPending
            );
            assert!(
                ctx.session.state.heads.is_empty() && ctx.session.state.observations.is_empty()
            ); // no evidence admitted
        }
    }

    #[test]
    fn active_phase_grades_and_admits_exactly_one_head() {
        let mut ctx = ctx_in(ActivationPhase::Active, true);
        let mut submit = SubmitTool;
        let out = submit.invoke(&mut ctx,
            r#"{"attemptId":"att-1","submissionId":"sub-1","artifact":"my answer","eventId":"a-submit","templateId":"T1","assessmentId":"g1","acceptEventId":"a-accept"}"#);
        assert!(is_ok(&out), "active submit: {out}");
        assert!(out.contains("\"grading\":\"assessed\""));
        assert_eq!(
            ctx.session.state.attempts["att-1"].state,
            AttemptState::Assessed
        );
        assert_eq!(ctx.session.state.heads["att-1"].id, "g1");
        assert_eq!(ctx.session.state.observations.len(), 1); // I04/I05 through the tool boundary
    }

    #[test]
    fn predict_tool_commits_only_pre_reveal() {
        let mut ctx = ctx_in(ActivationPhase::Shadow, false);
        let mut predict = PredictTool;
        let out = predict.invoke(
            &mut ctx,
            r#"{"attemptId":"att-1","prediction":"I think clean","eventId":"a-pred"}"#,
        );
        assert!(is_ok(&out), "predict: {out}");
        assert!(out.contains("\"predictionHash\""));
        // Reveal, then a late prediction must be rejected by the tool.
        let t = template();
        ctx.session.reveal("att-1", "a-reveal", &t).unwrap();
        let late = predict.invoke(
            &mut ctx,
            r#"{"attemptId":"att-1","prediction":"late","eventId":"a-late"}"#,
        );
        assert!(
            !is_ok(&late) && late.contains("PostRevealPrediction"),
            "late: {late}"
        );
    }

    #[test]
    fn persisted_kill_switch_phase_blocks_tool_grading() {
        // Activation reconstructed from persisted rows (persist.rs), phase=disabled.
        let audit = vec![
            (
                "Stewart".to_string(),
                "opt_in".to_string(),
                "opted_in".to_string(),
            ),
            (
                "Stewart".to_string(),
                "activate".to_string(),
                "active".to_string(),
            ),
            (
                "Stewart".to_string(),
                "disable".to_string(),
                "disabled".to_string(),
            ),
        ];
        let act = crate::persist::activation_from_rows("local-stewart", "disabled", audit).unwrap();
        let mut ctx = ctx_in(ActivationPhase::Shadow, true);
        ctx.activation = act;
        assert_eq!(ctx.activation.phase, ActivationPhase::Disabled);
        // Inspection still works under the kill switch.
        let mut brief = BriefTool;
        assert!(is_ok(&brief.invoke(&mut ctx, "{}")));
        // Submission records, grading blocked by the persisted Disabled phase.
        let mut submit = SubmitTool;
        let out = submit.invoke(&mut ctx,
            r#"{"attemptId":"att-1","submissionId":"sub-1","artifact":"ans","eventId":"a-submit","templateId":"T1","assessmentId":"g1","acceptEventId":"a-accept"}"#);
        assert!(
            is_ok(&out) && out.contains("\"grading\":\"blocked\""),
            "disabled submit: {out}"
        );
        assert!(ctx.session.state.heads.is_empty());
    }
}
