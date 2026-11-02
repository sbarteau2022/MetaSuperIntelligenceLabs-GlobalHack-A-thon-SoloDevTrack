//! Model Home cognitive-map tools for `crates/home-core`.
//!
//! Drop this file in as `crates/home-core/src/cogmap_tools.rs`, add
//! `pub mod cogmap_tools;` to `crates/home-core/src/lib.rs`, extend the
//! work-agent belt in `crates/home-core/src/standard.rs`, and add the five
//! cognitive-map path dependencies to `crates/home-core/Cargo.toml` (see the
//! patch files beside this module in the wiring package).
//!
//! Boundary: the real home-core Tool trait is async and serde_json-based. The
//! cogmap slices speak the integration slice's dependency-free JSON at their
//! boundary, so args are converted through their string form. Persistence is
//! intentionally OUT of this v1 module: state is in-memory only. The SQL
//! mapping exists in the integration slice, but no SQL is executed here.

use crate::tools::{spec, Tool};
use anyhow::Result;
use async_trait::async_trait;
use cogmap_integration::json::{parse as parse_cogmap_json, Json as CogmapJson};
use cogmap_slice2::attempt_loop::{GradeOutcome, LocalProcessRunner, Session, TaskTemplate};
use cogmap_slice3::brief::{cogmap_brief, cogmap_map, Curriculum, Goal};
use cogmap_slice4::optin::{Activation, ActivationPhase};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub const TOOL_BRIEF: &str = "cogmap_brief";
pub const TOOL_MAP: &str = "cogmap_map";
pub const TOOL_PREDICT: &str = "cogmap_predict";
pub const TOOL_SUBMIT: &str = "cogmap_submit";

/// Shared cognitive-map state for the four belt tools.
///
/// `session.state` is the Slice 1 reducer State; `activation` is the Slice 4
/// opt-in/kill-switch machine. Curriculum/goal/templates are app configuration:
/// brief/map work with an empty curriculum (unknown views), while deterministic
/// grading requires the embedding app to register task templates before using
/// `templateId` submits. State is in-memory v1; SQL persistence is not wired
/// into this module.
pub struct CogmapState {
    pub enrollment_id: String,
    pub session: Session,
    pub activation: Activation,
    pub curriculum: Curriculum,
    pub goal: Goal,
    pub templates: HashMap<String, TaskTemplate>,
    pub runner: LocalProcessRunner,
}

pub type SharedCogmapState = Arc<Mutex<CogmapState>>;

impl CogmapState {
    pub fn new(
        enrollment_id: impl Into<String>,
        curriculum: Curriculum,
        goal: Goal,
        templates: HashMap<String, TaskTemplate>,
    ) -> Self {
        let enrollment_id = enrollment_id.into();
        CogmapState {
            activation: Activation::new(&enrollment_id),
            enrollment_id,
            session: Session::new(),
            curriculum,
            goal,
            templates,
            runner: LocalProcessRunner::new(),
        }
    }

    /// Empty local default used when the belt is constructed before the app
    /// has injected curriculum/templates. All four tools remain callable;
    /// grading is unavailable until templates are registered and activation is
    /// Active.
    pub fn local_default() -> Self {
        Self::new(
            "local-stewart",
            Curriculum { version: "aie-v1".into(), concepts: Vec::new(), module_refs: Vec::new() },
            Goal {
                active_concept_ids: Vec::new(),
                current_difficulty: 1,
                paused_or_declined: false,
                missing_artifact_attempt: None,
                failed_remediation_cycles: 0,
            },
            HashMap::new(),
        )
    }

    pub fn shared(self) -> SharedCogmapState { Arc::new(Mutex::new(self)) }
    pub fn phase_name(&self) -> &'static str { phase_name(self.activation.phase) }
}

pub fn shared_state() -> SharedCogmapState { CogmapState::local_default().shared() }

pub fn phase_name(phase: ActivationPhase) -> &'static str {
    match phase {
        ActivationPhase::Shadow => "shadow",
        ActivationPhase::OptedIn => "opted_in",
        ActivationPhase::Active => "active",
        ActivationPhase::Disabled => "disabled",
    }
}

fn lock(state: &SharedCogmapState) -> Result<std::sync::MutexGuard<'_, CogmapState>> {
    state.lock().map_err(|_| anyhow::anyhow!("cogmap state mutex poisoned"))
}

fn args_json(args: &Value) -> Result<CogmapJson> {
    parse_cogmap_json(&args.to_string()).map_err(|e| anyhow::anyhow!(e))
}

fn ok(pairs: Vec<(&str, CogmapJson)>) -> String {
    let mut v = vec![("ok", CogmapJson::Bool(true))];
    v.extend(pairs);
    CogmapJson::obj(v).to_string()
}

fn fail(error: &str) -> String {
    CogmapJson::obj(vec![
        ("ok", CogmapJson::Bool(false)),
        ("error", CogmapJson::Str(error.to_string())),
    ])
    .to_string()
}

fn req_str(input: &CogmapJson, key: &str) -> std::result::Result<String, String> {
    input
        .get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or_else(|| format!("missing string field '{key}'"))
}

fn opt_str(input: &CogmapJson, key: &str) -> Option<String> {
    input.get(key).and_then(|v| v.as_str()).map(|v| v.to_string())
}

fn goal_for(state: &CogmapState, input: &CogmapJson) -> Goal {
    let mut goal = state.goal.clone();
    if let Some(ids) = input.get("goalConceptIds").and_then(|v| v.str_vec()) {
        goal.active_concept_ids = ids;
    }
    if let Some(paused) = input.get("pausedOrDeclined").and_then(|v| v.as_bool()) {
        goal.paused_or_declined = paused;
    }
    goal
}

fn handle_brief(state: &CogmapState, args: &CogmapJson) -> String {
    let goal = goal_for(state, args);
    let brief = cogmap_brief(&state.session.state, &state.curriculum, &goal);
    ok(vec![
        ("brief", CogmapJson::Str(brief.to_json())),
        ("phase", CogmapJson::Str(state.phase_name().to_string())),
    ])
}

fn handle_map(state: &CogmapState, _args: &CogmapJson) -> String {
    let projection = cogmap_map(&state.session.state, &state.enrollment_id, &state.curriculum, None);
    ok(vec![
        ("map", CogmapJson::Str(projection.to_json())),
        ("phase", CogmapJson::Str(state.phase_name().to_string())),
    ])
}

fn handle_predict(state: &mut CogmapState, args: &CogmapJson) -> String {
    if let Err(e) = state.activation.guard_record() {
        return fail(&format!("{e:?}"));
    }
    let attempt_id = match req_str(args, "attemptId") { Ok(v) => v, Err(e) => return fail(&e) };
    let prediction = match req_str(args, "prediction") { Ok(v) => v, Err(e) => return fail(&e) };
    let event_id = match req_str(args, "eventId") { Ok(v) => v, Err(e) => return fail(&e) };
    match state.session.commit_prediction(&attempt_id, prediction.as_bytes(), &event_id) {
        Ok(hash) => ok(vec![("predictionHash", CogmapJson::Str(hash))]),
        Err(e) => fail(&format!("{e:?}")),
    }
}

fn handle_submit(state: &mut CogmapState, args: &CogmapJson) -> String {
    if let Err(e) = state.activation.guard_record() {
        return fail(&format!("{e:?}"));
    }
    let attempt_id = match req_str(args, "attemptId") { Ok(v) => v, Err(e) => return fail(&e) };
    let submission_id = match req_str(args, "submissionId") { Ok(v) => v, Err(e) => return fail(&e) };
    let artifact = match req_str(args, "artifact") { Ok(v) => v, Err(e) => return fail(&e) };
    let event_id = match req_str(args, "eventId") { Ok(v) => v, Err(e) => return fail(&e) };

    let submission = match state.session.submit(&attempt_id, &submission_id, artifact.as_bytes(), &event_id) {
        Ok(s) => s,
        Err(e) => return fail(&format!("{e:?}")),
    };

    let Some(template_id) = opt_str(args, "templateId") else {
        return ok(vec![
            ("submissionId", CogmapJson::Str(submission.submission_id)),
            ("artifactHash", CogmapJson::Str(submission.artifact_hash)),
            ("grading", CogmapJson::Str("not_requested".into())),
        ]);
    };

    if let Err(e) = state.activation.guard_grade() {
        return ok(vec![
            ("submissionId", CogmapJson::Str(submission.submission_id)),
            ("artifactHash", CogmapJson::Str(submission.artifact_hash)),
            ("grading", CogmapJson::Str("blocked".into())),
            ("reason", CogmapJson::Str(format!("{e:?}"))),
            ("phase", CogmapJson::Str(state.phase_name().to_string())),
        ]);
    }

    let Some(template) = state.templates.get(&template_id).cloned() else {
        return fail("unknown templateId");
    };
    let assessment_id = opt_str(args, "assessmentId").unwrap_or_else(|| "g-auto".into());
    let accept_event_id = opt_str(args, "acceptEventId").unwrap_or_else(|| "e-accept".into());

    match state.session.grade_and_admit(
        &attempt_id,
        &template,
        &mut state.runner,
        "e-job",
        &accept_event_id,
        &assessment_id,
    ) {
        Ok(GradeOutcome::Assessed(grade, admitted_id)) => ok(vec![
            ("submissionId", CogmapJson::Str(submission.submission_id)),
            ("grading", CogmapJson::Str("assessed".into())),
            ("assessmentId", CogmapJson::Str(admitted_id)),
            ("v", CogmapJson::Num(grade.v as f64)),
            ("j", CogmapJson::Num(grade.j as f64)),
            ("q", CogmapJson::Num(grade.q as f64)),
            ("falseNegative", CogmapJson::Bool(grade.false_negative)),
            ("falsePositive", CogmapJson::Bool(grade.false_positive)),
            ("fullyClean", CogmapJson::Bool(grade.fully_clean)),
        ]),
        Ok(GradeOutcome::Unresolved(job_id)) => ok(vec![
            ("submissionId", CogmapJson::Str(submission.submission_id)),
            ("grading", CogmapJson::Str("unresolved".into())),
            ("jobId", CogmapJson::Str(job_id)),
        ]),
        Err(e) => fail(&format!("{e:?}")),
    }
}

pub struct CogmapBrief { state: SharedCogmapState }
pub struct CogmapMap { state: SharedCogmapState }
pub struct CogmapPredict { state: SharedCogmapState }
pub struct CogmapSubmit { state: SharedCogmapState }

#[async_trait]
impl Tool for CogmapBrief {
    fn name(&self) -> &str { TOOL_BRIEF }

    fn spec(&self) -> Value {
        spec(
            TOOL_BRIEF,
            "Structured teaching brief for the model. Read-only: the model speaks from this brief and cannot write the cognitive map.",
            json!({
                "goalConceptIds": { "type": "array", "items": { "type": "string" } },
                "pausedOrDeclined": { "type": "boolean" }
            }),
            &[],
        )
    }

    async fn run(&self, args: Value) -> Result<String> {
        let state = lock(&self.state)?;
        let args = args_json(&args)?;
        Ok(handle_brief(&state, &args))
    }
}

#[async_trait]
impl Tool for CogmapMap {
    fn name(&self) -> &str { TOOL_MAP }

    fn spec(&self) -> Value {
        spec(
            TOOL_MAP,
            "Inspectable cognitive map projection. Read-only; every non-unknown claim cites evidence and the policy version.",
            json!({}),
            &[],
        )
    }

    async fn run(&self, args: Value) -> Result<String> {
        let state = lock(&self.state)?;
        let args = args_json(&args)?;
        Ok(handle_map(&state, &args))
    }
}

#[async_trait]
impl Tool for CogmapPredict {
    fn name(&self) -> &str { TOOL_PREDICT }

    fn spec(&self) -> Value {
        spec(
            TOOL_PREDICT,
            "Commit a hash-sealed prediction BEFORE reveal. Post-reveal commits are rejected.",
            json!({
                "attemptId": { "type": "string", "description": "Attempt id assigned by the Studio attempt loop." },
                "prediction": { "type": "string", "description": "Prediction text to hash-seal before reveal." },
                "eventId": { "type": "string", "description": "Idempotent command/event id for this prediction commit." }
            }),
            &["attemptId", "prediction", "eventId"],
        )
    }

    async fn run(&self, args: Value) -> Result<String> {
        let mut state = lock(&self.state)?;
        let args = args_json(&args)?;
        Ok(handle_predict(&mut state, &args))
    }
}

#[async_trait]
impl Tool for CogmapSubmit {
    fn name(&self) -> &str { TOOL_SUBMIT }

    fn spec(&self) -> Value {
        spec(
            TOOL_SUBMIT,
            "Submit an immutable finalized artifact. Deterministic grading runs only when activation is Active; otherwise grading is reported blocked.",
            json!({
                "attemptId": { "type": "string", "description": "Attempt id assigned by the Studio attempt loop." },
                "submissionId": { "type": "string", "description": "Immutable submission id; retries must reuse the same bytes." },
                "artifact": { "type": "string", "description": "Finalized artifact text/bytes submitted for deterministic grading." },
                "eventId": { "type": "string", "description": "Idempotent command/event id for this submission." },
                "templateId": { "type": "string", "description": "Optional registered task template id. Omit for recording only." },
                "assessmentId": { "type": "string", "description": "Optional assessment id to admit when grading is Active." },
                "acceptEventId": { "type": "string", "description": "Optional event id for assessment admission." }
            }),
            &["attemptId", "submissionId", "artifact", "eventId"],
        )
    }

    async fn run(&self, args: Value) -> Result<String> {
        let mut state = lock(&self.state)?;
        let args = args_json(&args)?;
        Ok(handle_submit(&mut state, &args))
    }
}

/// Build the four cognitive-map tools around one shared state handle.
pub fn cogmap_tools(state: SharedCogmapState) -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(CogmapBrief { state: state.clone() }),
        Arc::new(CogmapMap { state: state.clone() }),
        Arc::new(CogmapPredict { state: state.clone() }),
        Arc::new(CogmapSubmit { state }),
    ]
}
