//! Persistence mapping: Slice 1 reducer domain <-> Slice 1 `cogmap` schema rows.
//! NO database driver and NO live DB here (hermetic, like every prior slice):
//! this module owns (a) the SQL statements as string constants, (b) parameter
//! vectors in column order, (c) row structs + conversions both directions, and
//! (d) State reconstruction. Round-trip identity is proven by tests; executing
//! the SQL against Postgres 5433 is Mac-side wiring (see README, unverified).
//!
//! Source of truth = cogmap.events (append-only). attempts / assessments /
//! dimension_observations are projections of the same event stream; replaying
//! events through the Slice 1 reducer rebuilds State exactly (I11).
use cogmap::{Assessment, Attempt, AttemptState, Event, Observation, State};
use cogmap_slice4::optin::{Activation, ActivationPhase, AuditEntry};

use crate::json::{parse as parse_json, Json};

// ---------- typed SQL parameters (column order = statement order) ----------
#[derive(Clone, Debug, PartialEq)]
pub enum SqlParam {
    Text(String),
    Int(i64),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PersistError {
    BadEventType(String),
    BadPhase(String),
    BadState(String),
    Json(String),
    Reducer(String),
}

// ---------- SQL statements (schema: slice1/schema.sql + integration/activation.sql) ----------
pub const INSERT_EVENT_SQL: &str =
    "INSERT INTO cogmap.events (enrollment_id, revision, command_id, event_type, payload) \
     VALUES ($1::uuid, $2, $3, $4, $5::jsonb) RETURNING seq";
pub const SELECT_EVENTS_SQL: &str =
    "SELECT seq, enrollment_id::text, revision, command_id, event_type, payload::text \
     FROM cogmap.events WHERE enrollment_id = $1::uuid ORDER BY seq ASC";
pub const INSERT_RECEIPT_SQL: &str =
    "INSERT INTO cogmap.receipts (enrollment_id, command_id, canonical_hash, first_seq, last_seq, revision, response) \
     VALUES ($1::uuid, $2, $3, $4, $5, $6, $7::jsonb) \
     ON CONFLICT (enrollment_id, command_id) DO NOTHING";
pub const INSERT_ATTEMPT_SQL: &str =
    "INSERT INTO cogmap.attempts (id, enrollment_id, task_template_id, task_variant, concept_id, dimension, \
     difficulty, state, prediction_hash, prediction_seq, reveal_seq, hint_tier, independent) \
     VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
     ON CONFLICT (enrollment_id, id) DO UPDATE SET state = EXCLUDED.state, \
     prediction_hash = EXCLUDED.prediction_hash, prediction_seq = EXCLUDED.prediction_seq, \
     reveal_seq = EXCLUDED.reveal_seq, hint_tier = EXCLUDED.hint_tier";
pub const INSERT_ASSESSMENT_SQL: &str =
    "INSERT INTO cogmap.assessments (id, enrollment_id, attempt_id, submission_id, supersedes_id, is_head, active, \
     grader_id, grader_version, reviewer_id, verdict, justification, evidence_quality_x4, hint_percent, \
     false_negative, false_positive, is_fully_clean, is_defective_task, artifact_hash) \
     VALUES ($1::uuid, $2::uuid, $3::uuid, $4::uuid, $5::uuid, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)";
pub const INSERT_OBSERVATION_SQL: &str =
    "INSERT INTO cogmap.dimension_observations (enrollment_id, attempt_id, dimension, difficulty, assessment_id, \
     submission_seq, score_units, independent, hint_tier, is_fully_clean, is_defective_task, false_negative, false_positive) \
     VALUES ($1::uuid, $2::uuid, $3, $4, $5::uuid, $6, $7, $8, $9, $10, $11, $12, $13) \
     ON CONFLICT (enrollment_id, attempt_id, dimension) DO UPDATE SET assessment_id = EXCLUDED.assessment_id, \
     submission_seq = EXCLUDED.submission_seq, score_units = EXCLUDED.score_units"; // I05: one row per attempt/dimension
pub const ACTIVATION_UPSERT_SQL: &str =
    "INSERT INTO cogmap.activation (enrollment_id, phase, updated_at) VALUES ($1, $2, now()) \
     ON CONFLICT (enrollment_id) DO UPDATE SET phase = EXCLUDED.phase, updated_at = now()";
pub const ACTIVATION_AUDIT_INSERT_SQL: &str =
    "INSERT INTO cogmap.activation_audit (enrollment_id, actor, action, phase_after) VALUES ($1, $2, $3, $4) RETURNING seq";
pub const ACTIVATION_SELECT_SQL: &str =
    "SELECT phase FROM cogmap.activation WHERE enrollment_id = $1";
pub const ACTIVATION_AUDIT_SELECT_SQL: &str =
    "SELECT actor, action, phase_after FROM cogmap.activation_audit WHERE enrollment_id = $1 ORDER BY seq ASC";
/// The activation migration DDL itself (single source: the shipped .sql file).
pub const ACTIVATION_DDL: &str = include_str!("../activation.sql");

// ---------- events -> row -> events (round-trip identity for every variant) ----------
pub fn event_type_name(e: &Event) -> &'static str {
    match e {
        Event::AssignTask { .. } => "assign_task",
        Event::CommitPrediction { .. } => "commit_prediction",
        Event::ReplacePrediction { .. } => "replace_prediction",
        Event::Reveal { .. } => "reveal",
        Event::DeliverHint { .. } => "deliver_hint",
        Event::Submit { .. } => "submit",
        Event::AssessmentAccepted { .. } => "assessment_accepted",
        Event::AssessmentUnavailable { .. } => "assessment_unavailable",
        Event::Dispute { .. } => "dispute",
        Event::ResolveDisputeAssessed { .. } => "resolve_dispute_assessed",
        Event::ResolveDisputeInvalid { .. } => "resolve_dispute_invalid",
        Event::Invalidate { .. } => "invalidate",
        Event::Cancel { .. } => "cancel",
        Event::Regrade { .. } => "regrade",
    }
}

fn s(v: &str) -> Json {
    Json::Str(v.to_string())
}
fn n(v: i64) -> Json {
    Json::Num(v as f64)
}
fn b(v: bool) -> Json {
    Json::Bool(v)
}

/// Canonical payload for cogmap.events.payload (jsonb). event_id rides inside so
/// a filtered stream (gaps are normal, §B) still self-describes for replay/dedup.
pub fn event_payload_json(e: &Event) -> String {
    let pairs: Vec<(&str, Json)> = match e {
        Event::AssignTask {
            event_id,
            attempt_id,
            concept_id,
            dimension,
            difficulty,
            variant,
            independent,
        } => vec![
            ("eventId", s(event_id)),
            ("attemptId", s(attempt_id)),
            ("conceptId", s(concept_id)),
            ("dimension", s(dimension)),
            ("difficulty", n(*difficulty as i64)),
            ("variant", s(variant)),
            ("independent", b(*independent)),
        ],
        Event::CommitPrediction {
            event_id,
            attempt_id,
        }
        | Event::ReplacePrediction {
            event_id,
            attempt_id,
        }
        | Event::Reveal {
            event_id,
            attempt_id,
        }
        | Event::Submit {
            event_id,
            attempt_id,
        }
        | Event::AssessmentUnavailable {
            event_id,
            attempt_id,
        }
        | Event::Dispute {
            event_id,
            attempt_id,
        }
        | Event::ResolveDisputeInvalid {
            event_id,
            attempt_id,
        }
        | Event::Invalidate {
            event_id,
            attempt_id,
        }
        | Event::Cancel {
            event_id,
            attempt_id,
        } => vec![("eventId", s(event_id)), ("attemptId", s(attempt_id))],
        Event::DeliverHint {
            event_id,
            attempt_id,
            tier,
        } => vec![
            ("eventId", s(event_id)),
            ("attemptId", s(attempt_id)),
            ("tier", n(*tier as i64)),
        ],
        Event::AssessmentAccepted {
            event_id,
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
        }
        | Event::ResolveDisputeAssessed {
            event_id,
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
        }
        | Event::Regrade {
            event_id,
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
        } => vec![
            ("eventId", s(event_id)),
            ("attemptId", s(attempt_id)),
            ("assessmentId", s(assessment_id)),
            ("v", n(*v as i64)),
            ("j", n(*j as i64)),
            ("q", n(*q as i64)),
            ("seq", n(*seq)),
            ("falseNegative", b(*false_negative)),
            ("falsePositive", b(*false_positive)),
            ("fullyClean", b(*fully_clean)),
            ("defectiveTask", b(*defective_task)),
        ],
    };
    Json::obj(pairs).to_string()
}

/// In-memory image of one cogmap.events row (seq assigned by the DB on insert).
#[derive(Clone, Debug, PartialEq)]
pub struct EventRow {
    pub seq: i64,
    pub enrollment_id: String,
    pub revision: i64,
    pub command_id: Option<String>,
    pub event_type: String,
    pub payload_json: String,
}

pub fn event_to_row(
    e: &Event,
    enrollment_id: &str,
    revision: i64,
    command_id: Option<&str>,
) -> EventRow {
    EventRow {
        seq: 0, // DB-assigned (bigserial); 0 = not yet inserted
        enrollment_id: enrollment_id.to_string(),
        revision,
        command_id: command_id.map(|c| c.to_string()),
        event_type: event_type_name(e).to_string(),
        payload_json: event_payload_json(e),
    }
}

/// INSERT_EVENT_SQL parameters, in order: enrollment, revision, command, type, payload.
pub fn event_insert_params(row: &EventRow) -> Vec<SqlParam> {
    vec![
        SqlParam::Text(row.enrollment_id.clone()),
        SqlParam::Int(row.revision),
        row.command_id
            .clone()
            .map(SqlParam::Text)
            .unwrap_or(SqlParam::Null),
        SqlParam::Text(row.event_type.clone()),
        SqlParam::Text(row.payload_json.clone()),
    ]
}

fn gs(p: &Json, key: &str) -> Result<String, PersistError> {
    p.get(key)
        .and_then(|v| v.as_str())
        .map(|v| v.to_string())
        .ok_or_else(|| PersistError::Json(format!("missing string field {key}")))
}
fn gi(p: &Json, key: &str) -> Result<i64, PersistError> {
    p.get(key)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| PersistError::Json(format!("missing int field {key}")))
}
fn gb(p: &Json, key: &str) -> Result<bool, PersistError> {
    p.get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| PersistError::Json(format!("missing bool field {key}")))
}

pub fn event_from_row(row: &EventRow) -> Result<Event, PersistError> {
    let p = parse_json(&row.payload_json).map_err(PersistError::Json)?;
    let event_id = gs(&p, "eventId")?;
    let attempt_id = gs(&p, "attemptId")?;
    let graded =
        |p: &Json| -> Result<(String, u8, u8, u8, i64, bool, bool, bool, bool), PersistError> {
            Ok((
                gs(p, "assessmentId")?,
                gi(p, "v")? as u8,
                gi(p, "j")? as u8,
                gi(p, "q")? as u8,
                gi(p, "seq")?,
                gb(p, "falseNegative")?,
                gb(p, "falsePositive")?,
                gb(p, "fullyClean")?,
                gb(p, "defectiveTask")?,
            ))
        };
    match row.event_type.as_str() {
        "assign_task" => Ok(Event::AssignTask {
            event_id,
            attempt_id,
            concept_id: gs(&p, "conceptId")?,
            dimension: gs(&p, "dimension")?,
            difficulty: gi(&p, "difficulty")? as u8,
            variant: gs(&p, "variant")?,
            independent: gb(&p, "independent")?,
        }),
        "commit_prediction" => Ok(Event::CommitPrediction {
            event_id,
            attempt_id,
        }),
        "replace_prediction" => Ok(Event::ReplacePrediction {
            event_id,
            attempt_id,
        }),
        "reveal" => Ok(Event::Reveal {
            event_id,
            attempt_id,
        }),
        "deliver_hint" => Ok(Event::DeliverHint {
            event_id,
            attempt_id,
            tier: gi(&p, "tier")? as u8,
        }),
        "submit" => Ok(Event::Submit {
            event_id,
            attempt_id,
        }),
        "assessment_accepted" => {
            let (a, v, j, q, seq, fn_, fp, fc, dt) = graded(&p)?;
            Ok(Event::AssessmentAccepted {
                event_id,
                attempt_id,
                assessment_id: a,
                v,
                j,
                q,
                seq,
                false_negative: fn_,
                false_positive: fp,
                fully_clean: fc,
                defective_task: dt,
            })
        }
        "assessment_unavailable" => Ok(Event::AssessmentUnavailable {
            event_id,
            attempt_id,
        }),
        "dispute" => Ok(Event::Dispute {
            event_id,
            attempt_id,
        }),
        "resolve_dispute_assessed" => {
            let (a, v, j, q, seq, fn_, fp, fc, dt) = graded(&p)?;
            Ok(Event::ResolveDisputeAssessed {
                event_id,
                attempt_id,
                assessment_id: a,
                v,
                j,
                q,
                seq,
                false_negative: fn_,
                false_positive: fp,
                fully_clean: fc,
                defective_task: dt,
            })
        }
        "resolve_dispute_invalid" => Ok(Event::ResolveDisputeInvalid {
            event_id,
            attempt_id,
        }),
        "invalidate" => Ok(Event::Invalidate {
            event_id,
            attempt_id,
        }),
        "cancel" => Ok(Event::Cancel {
            event_id,
            attempt_id,
        }),
        "regrade" => {
            let (a, v, j, q, seq, fn_, fp, fc, dt) = graded(&p)?;
            Ok(Event::Regrade {
                event_id,
                attempt_id,
                assessment_id: a,
                v,
                j,
                q,
                seq,
                false_negative: fn_,
                false_positive: fp,
                fully_clean: fc,
                defective_task: dt,
            })
        }
        other => Err(PersistError::BadEventType(other.to_string())),
    }
}

/// Rebuild State by replaying persisted event rows (seq order) through the
/// reducer — the authoritative reconstruction (I11 bit-exact with live state).
pub fn state_from_event_rows(rows: &[EventRow]) -> Result<State, PersistError> {
    let mut ordered: Vec<&EventRow> = rows.iter().collect();
    ordered.sort_by_key(|r| r.seq);
    let mut state = State::default();
    for row in ordered {
        let ev = event_from_row(row)?;
        state = cogmap::reduce(state, &ev).map_err(|r| PersistError::Reducer(format!("{r:?}")))?;
    }
    Ok(state)
}

// ---------- attempts projection rows ----------
pub fn attempt_state_str(s: AttemptState) -> &'static str {
    match s {
        AttemptState::Assigned => "assigned",
        AttemptState::PredictionCommitted => "prediction_committed",
        AttemptState::Revealed => "revealed",
        AttemptState::AssessmentPending => "assessment_pending",
        AttemptState::Assessed => "assessed",
        AttemptState::Disputed => "disputed",
        AttemptState::Invalid => "invalid",
        AttemptState::Cancelled => "cancelled",
    }
}
pub fn attempt_state_from_str(v: &str) -> Result<AttemptState, PersistError> {
    match v {
        "assigned" => Ok(AttemptState::Assigned),
        "prediction_committed" => Ok(AttemptState::PredictionCommitted),
        "revealed" => Ok(AttemptState::Revealed),
        "assessment_pending" => Ok(AttemptState::AssessmentPending),
        "assessed" => Ok(AttemptState::Assessed),
        "disputed" => Ok(AttemptState::Disputed),
        "invalid" => Ok(AttemptState::Invalid),
        "cancelled" => Ok(AttemptState::Cancelled),
        other => Err(PersistError::BadState(other.to_string())),
    }
}

/// Image of cogmap.attempts. prediction hash/seqs live ONLY in these columns —
/// the reducer Attempt does not carry them (Session returns the hash at commit).
#[derive(Clone, Debug, PartialEq)]
pub struct AttemptRow {
    pub attempt_id: String,
    pub enrollment_id: String,
    pub task_template_id: String,
    pub concept_id: String,
    pub dimension: String,
    pub difficulty: u8,
    pub state: String,
    pub prediction_hash: Option<String>,
    pub prediction_seq: Option<i64>,
    pub reveal_seq: Option<i64>,
    pub hint_tier: u8,
    pub independent: bool,
    pub variant: String,
}
impl AttemptRow {
    pub fn from_attempt(
        a: &Attempt,
        enrollment_id: &str,
        task_template_id: &str,
        prediction_hash: Option<String>,
        prediction_seq: Option<i64>,
        reveal_seq: Option<i64>,
    ) -> AttemptRow {
        AttemptRow {
            attempt_id: a.id.clone(),
            enrollment_id: enrollment_id.to_string(),
            task_template_id: task_template_id.to_string(),
            concept_id: a.concept_id.clone(),
            dimension: a.dimension.clone(),
            difficulty: a.difficulty,
            state: attempt_state_str(a.state).to_string(),
            prediction_hash,
            prediction_seq,
            reveal_seq,
            hint_tier: a.hint_tier,
            independent: a.independent,
            variant: a.variant.clone(),
        }
    }
    pub fn into_attempt(self) -> Result<Attempt, PersistError> {
        Ok(Attempt {
            id: self.attempt_id,
            concept_id: self.concept_id,
            dimension: self.dimension,
            difficulty: self.difficulty,
            variant: self.variant,
            state: attempt_state_from_str(&self.state)?,
            prediction_committed: self.prediction_hash.is_some() || self.prediction_seq.is_some(),
            prediction_before_reveal: self.reveal_seq.is_some(),
            hint_tier: self.hint_tier,
            independent: self.independent,
        })
    }
    /// INSERT_ATTEMPT_SQL parameters in column order (task_variant column = variant).
    pub fn insert_params(&self) -> Vec<SqlParam> {
        vec![
            SqlParam::Text(self.attempt_id.clone()),
            SqlParam::Text(self.enrollment_id.clone()),
            SqlParam::Text(self.task_template_id.clone()),
            SqlParam::Text(self.variant.clone()),
            SqlParam::Text(self.concept_id.clone()),
            SqlParam::Text(self.dimension.clone()),
            SqlParam::Int(self.difficulty as i64),
            SqlParam::Text(self.state.clone()),
            self.prediction_hash
                .clone()
                .map(SqlParam::Text)
                .unwrap_or(SqlParam::Null),
            self.prediction_seq
                .map(SqlParam::Int)
                .unwrap_or(SqlParam::Null),
            self.reveal_seq.map(SqlParam::Int).unwrap_or(SqlParam::Null),
            SqlParam::Int(self.hint_tier as i64),
            SqlParam::Bool(self.independent),
        ]
    }
}

// ---------- assessments projection rows (I04 head) ----------
#[derive(Clone, Debug, PartialEq)]
pub struct AssessmentRow {
    pub assessment_id: String,
    pub attempt_id: String,
    pub score_units: i64, // DB generated column; carried here for reconstruction
    pub false_negative: bool,
    pub false_positive: bool,
    pub fully_clean: bool,
    pub defective_task: bool,
    pub active: bool,
    pub seq: i64,
}
impl AssessmentRow {
    pub fn from_assessment(a: &Assessment) -> AssessmentRow {
        AssessmentRow {
            assessment_id: a.id.clone(),
            attempt_id: a.attempt_id.clone(),
            score_units: a.score_units,
            false_negative: a.false_negative,
            false_positive: a.false_positive,
            fully_clean: a.fully_clean,
            defective_task: a.defective_task,
            active: a.active,
            seq: a.seq,
        }
    }
    pub fn into_assessment(self) -> Assessment {
        Assessment {
            id: self.assessment_id,
            attempt_id: self.attempt_id,
            score_units: self.score_units,
            false_negative: self.false_negative,
            false_positive: self.false_positive,
            fully_clean: self.fully_clean,
            defective_task: self.defective_task,
            active: self.active,
            seq: self.seq,
        }
    }
}

/// INSERT_ASSESSMENT_SQL params for a graded event (v/j/q/hint live on the event,
/// not the reducer Assessment). Returns None for non-grading events.
pub fn assessment_insert_params(
    enrollment_id: &str,
    submission_id: &str,
    grader_id: &str,
    grader_version: &str,
    artifact_hash: &str,
    hint_tier: u8,
    e: &Event,
) -> Option<Vec<SqlParam>> {
    let (attempt_id, assessment_id, v, j, q, fn_, fp, fc, dt, supersedes) = match e {
        Event::AssessmentAccepted {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => (
            attempt_id,
            assessment_id,
            *v,
            *j,
            *q,
            *false_negative,
            *false_positive,
            *fully_clean,
            *defective_task,
            None,
        ),
        Event::ResolveDisputeAssessed {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => (
            attempt_id,
            assessment_id,
            *v,
            *j,
            *q,
            *false_negative,
            *false_positive,
            *fully_clean,
            *defective_task,
            None,
        ),
        Event::Regrade {
            attempt_id,
            assessment_id,
            v,
            j,
            q,
            false_negative,
            false_positive,
            fully_clean,
            defective_task,
            ..
        } => (
            attempt_id,
            assessment_id,
            *v,
            *j,
            *q,
            *false_negative,
            *false_positive,
            *fully_clean,
            *defective_task,
            Some("superseded-head-resolved-by-caller"),
        ),
        _ => return None,
    };
    let _ = supersedes; // caller binds $5 to the prior head id on regrade (documented in README)
    Some(vec![
        SqlParam::Text(assessment_id.clone()),
        SqlParam::Text(enrollment_id.to_string()),
        SqlParam::Text(attempt_id.clone()),
        SqlParam::Text(submission_id.to_string()),
        SqlParam::Null, // $5 supersedes_id: null for first head; prior head id on regrade
        SqlParam::Bool(true),
        SqlParam::Bool(true),
        SqlParam::Text(grader_id.to_string()),
        SqlParam::Text(grader_version.to_string()),
        SqlParam::Null, // reviewer_id: required for semantic criteria; deterministic runner leaves null (§D.5)
        SqlParam::Int(v as i64),
        SqlParam::Int(j as i64),
        SqlParam::Int(q as i64),
        SqlParam::Int(cogmap::hint_percent(hint_tier)),
        SqlParam::Bool(fn_),
        SqlParam::Bool(fp),
        SqlParam::Bool(fc),
        SqlParam::Bool(dt),
        SqlParam::Text(artifact_hash.to_string()),
    ])
}

// ---------- dimension_observations projection rows (I05) ----------
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationRow {
    pub attempt_id: String,
    pub dimension: String,
    pub difficulty: u8,
    pub assessment_id: String,
    pub submission_seq: i64,
    pub score_units: i64,
    pub independent: bool,
    pub hint_tier: u8,
    pub is_fully_clean: bool,
    pub is_defective_task: bool,
    pub false_negative: bool,
    pub false_positive: bool,
}
impl ObservationRow {
    pub fn from_observation(
        o: &Observation,
        difficulty: u8,
        assessment_id: &str,
    ) -> ObservationRow {
        ObservationRow {
            attempt_id: o.attempt_id.clone(),
            dimension: o.dimension.clone(),
            difficulty,
            assessment_id: assessment_id.to_string(),
            submission_seq: o.seq,
            score_units: o.score_units,
            independent: o.independent,
            hint_tier: o.hint_tier,
            is_fully_clean: o.fully_clean,
            is_defective_task: o.defective_task,
            false_negative: o.false_negative,
            false_positive: o.false_positive,
        }
    }
    /// NOTE (lossy column set, by schema design): cogmap.dimension_observations has
    /// no variant / prediction_before_reveal columns, so those two fields are
    /// restored from the attempt row at reconstruction time.
    pub fn into_observation(self, variant: String, prediction_before_reveal: bool) -> Observation {
        Observation {
            attempt_id: self.attempt_id,
            dimension: self.dimension,
            seq: self.submission_seq,
            score_units: self.score_units,
            independent: self.independent,
            hint_tier: self.hint_tier,
            variant,
            fully_clean: self.is_fully_clean,
            defective_task: self.is_defective_task,
            false_negative: self.false_negative,
            false_positive: self.false_positive,
            prediction_before_reveal,
        }
    }
    pub fn insert_params(&self, enrollment_id: &str) -> Vec<SqlParam> {
        vec![
            SqlParam::Text(enrollment_id.to_string()),
            SqlParam::Text(self.attempt_id.clone()),
            SqlParam::Text(self.dimension.clone()),
            SqlParam::Int(self.difficulty as i64),
            SqlParam::Text(self.assessment_id.clone()),
            SqlParam::Int(self.submission_seq),
            SqlParam::Int(self.score_units),
            SqlParam::Bool(self.independent),
            SqlParam::Int(self.hint_tier as i64),
            SqlParam::Bool(self.is_fully_clean),
            SqlParam::Bool(self.is_defective_task),
            SqlParam::Bool(self.false_negative),
            SqlParam::Bool(self.false_positive),
        ]
    }
}

/// Rebuild State from projection rows (attempts + heads + observations).
/// Event replay (state_from_event_rows) is authoritative; this path exists for
/// reporting queries and must agree with it (tested on fixtures).
pub fn state_from_projection(
    revision: i64,
    attempt_rows: Vec<AttemptRow>,
    head_rows: Vec<AssessmentRow>,
    observation_rows: Vec<ObservationRow>,
) -> Result<State, PersistError> {
    let mut state = State::default();
    state.revision = revision; // applied_event_ids is reducer-private; replay path populates it
    let mut attempts_by_id = std::collections::HashMap::new();
    for row in attempt_rows {
        let a = row.into_attempt()?;
        attempts_by_id.insert(a.id.clone(), a.clone());
        state.attempts.insert(a.id.clone(), a);
    }
    for row in head_rows {
        let a = row.into_assessment();
        state.heads.insert(a.attempt_id.clone(), a);
    }
    for row in observation_rows {
        let (variant, pbr) = attempts_by_id
            .get(&row.attempt_id)
            .map(|a| (a.variant.clone(), a.prediction_before_reveal))
            .unwrap_or_default();
        state.observations.push(row.into_observation(variant, pbr));
    }
    Ok(state)
}

// ---------- activation persistence (Slice 4 Activation <-> cogmap.activation*) ----------
pub fn phase_str(p: ActivationPhase) -> &'static str {
    match p {
        ActivationPhase::Shadow => "shadow",
        ActivationPhase::OptedIn => "opted_in",
        ActivationPhase::Active => "active",
        ActivationPhase::Disabled => "disabled",
    }
}
pub fn phase_from_str(v: &str) -> Result<ActivationPhase, PersistError> {
    match v {
        "shadow" => Ok(ActivationPhase::Shadow),
        "opted_in" => Ok(ActivationPhase::OptedIn),
        "active" => Ok(ActivationPhase::Active),
        "disabled" => Ok(ActivationPhase::Disabled),
        other => Err(PersistError::BadPhase(other.to_string())),
    }
}
pub fn activation_upsert_params(a: &Activation) -> Vec<SqlParam> {
    vec![
        SqlParam::Text(a.enrollment_id.clone()),
        SqlParam::Text(phase_str(a.phase).to_string()),
    ]
}
pub fn activation_audit_params(enrollment_id: &str, entry: &AuditEntry) -> Vec<SqlParam> {
    vec![
        SqlParam::Text(enrollment_id.to_string()),
        SqlParam::Text(entry.actor.clone()),
        SqlParam::Text(entry.action.clone()),
        SqlParam::Text(phase_str(entry.phase_after).to_string()),
    ]
}
/// Reconstruct an Activation from its persisted phase + ordered audit rows.
pub fn activation_from_rows(
    enrollment_id: &str,
    phase: &str,
    audit: Vec<(String, String, String)>,
) -> Result<Activation, PersistError> {
    let mut a = Activation::new(enrollment_id);
    a.phase = phase_from_str(phase)?;
    for (i, (actor, action, phase_after)) in audit.into_iter().enumerate() {
        a.audit.push(AuditEntry {
            seq: i as i64 + 1,
            actor,
            action,
            phase_after: phase_from_str(&phase_after)?,
        });
    }
    Ok(a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cogmap::{reduce, Event};

    fn all_event_variants() -> Vec<Event> {
        vec![
            Event::AssignTask {
                event_id: "e01".into(),
                attempt_id: "a1".into(),
                concept_id: "c1".into(),
                dimension: "SPEC".into(),
                difficulty: 2,
                variant: "v1".into(),
                independent: true,
            },
            Event::CommitPrediction {
                event_id: "e02".into(),
                attempt_id: "a1".into(),
            },
            Event::ReplacePrediction {
                event_id: "e03".into(),
                attempt_id: "a1".into(),
            },
            Event::Reveal {
                event_id: "e04".into(),
                attempt_id: "a1".into(),
            },
            Event::DeliverHint {
                event_id: "e05".into(),
                attempt_id: "a1".into(),
                tier: 3,
            },
            Event::Submit {
                event_id: "e06".into(),
                attempt_id: "a1".into(),
            },
            Event::AssessmentAccepted {
                event_id: "e07".into(),
                attempt_id: "a1".into(),
                assessment_id: "g1".into(),
                v: 1,
                j: 2,
                q: 3,
                seq: 42,
                false_negative: false,
                false_positive: true,
                fully_clean: false,
                defective_task: true,
            },
            Event::AssessmentUnavailable {
                event_id: "e08".into(),
                attempt_id: "a1".into(),
            },
            Event::Dispute {
                event_id: "e09".into(),
                attempt_id: "a1".into(),
            },
            Event::ResolveDisputeAssessed {
                event_id: "e10".into(),
                attempt_id: "a1".into(),
                assessment_id: "g2".into(),
                v: 0,
                j: 1,
                q: 2,
                seq: 43,
                false_negative: true,
                false_positive: false,
                fully_clean: false,
                defective_task: false,
            },
            Event::ResolveDisputeInvalid {
                event_id: "e11".into(),
                attempt_id: "a1".into(),
            },
            Event::Invalidate {
                event_id: "e12".into(),
                attempt_id: "a1".into(),
            },
            Event::Cancel {
                event_id: "e13".into(),
                attempt_id: "a1".into(),
            },
            Event::Regrade {
                event_id: "e14".into(),
                attempt_id: "a1".into(),
                assessment_id: "g3".into(),
                v: 1,
                j: 3,
                q: 4,
                seq: 44,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        ]
    }

    #[test]
    fn every_event_variant_round_trips_through_row() {
        for ev in all_event_variants() {
            let row = event_to_row(&ev, "enr-1", 7, Some("cmd-1"));
            // params bind in statement order; payload survives verbatim
            let params = event_insert_params(&row);
            assert_eq!(params.len(), 5);
            assert_eq!(params[3], SqlParam::Text(row.event_type.clone()));
            assert_eq!(params[4], SqlParam::Text(row.payload_json.clone()));
            let back = event_from_row(&row).unwrap();
            assert_eq!(
                format!("{ev:?}"),
                format!("{back:?}"),
                "round trip: {}",
                row.event_type
            );
        }
    }

    #[test]
    fn event_replay_reconstruction_equals_live_state() {
        // A legal stream across two attempts incl. dispute + regrade.
        let stream = vec![
            Event::AssignTask {
                event_id: "s1".into(),
                attempt_id: "a1".into(),
                concept_id: "c1".into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
            Event::CommitPrediction {
                event_id: "s2".into(),
                attempt_id: "a1".into(),
            },
            Event::Reveal {
                event_id: "s3".into(),
                attempt_id: "a1".into(),
            },
            Event::Submit {
                event_id: "s4".into(),
                attempt_id: "a1".into(),
            },
            Event::AssessmentAccepted {
                event_id: "s5".into(),
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
            Event::Dispute {
                event_id: "s6".into(),
                attempt_id: "a1".into(),
            },
            Event::ResolveDisputeAssessed {
                event_id: "s7".into(),
                attempt_id: "a1".into(),
                assessment_id: "g2".into(),
                v: 1,
                j: 2,
                q: 4,
                seq: 1,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
            Event::Regrade {
                event_id: "s8".into(),
                attempt_id: "a1".into(),
                assessment_id: "g3".into(),
                v: 1,
                j: 3,
                q: 4,
                seq: 1,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        ];
        let mut live = State::default();
        let mut rows = Vec::new();
        for (i, ev) in stream.iter().enumerate() {
            live = reduce(live, ev).unwrap();
            let mut row = event_to_row(ev, "enr-1", live.revision, None);
            row.seq = i as i64 + 1;
            rows.push(row);
        }
        let rebuilt = state_from_event_rows(&rows).unwrap();
        assert_eq!(live, rebuilt); // I11: replay is bit-exact, revision + dedup set included
    }

    #[test]
    fn projection_rows_round_trip_and_agree_with_replay() {
        let mut s = State::default();
        let stream = vec![
            Event::AssignTask {
                event_id: "p1".into(),
                attempt_id: "a1".into(),
                concept_id: "c1".into(),
                dimension: "SPEC".into(),
                difficulty: 1,
                variant: "v1".into(),
                independent: true,
            },
            Event::CommitPrediction {
                event_id: "p2".into(),
                attempt_id: "a1".into(),
            },
            Event::Reveal {
                event_id: "p3".into(),
                attempt_id: "a1".into(),
            },
            Event::DeliverHint {
                event_id: "p4".into(),
                attempt_id: "a1".into(),
                tier: 1,
            },
            Event::Submit {
                event_id: "p5".into(),
                attempt_id: "a1".into(),
            },
            Event::AssessmentAccepted {
                event_id: "p6".into(),
                attempt_id: "a1".into(),
                assessment_id: "g1".into(),
                v: 1,
                j: 3,
                q: 4,
                seq: 9,
                false_negative: false,
                false_positive: false,
                fully_clean: true,
                defective_task: false,
            },
        ];
        for ev in &stream {
            s = reduce(s, ev).unwrap();
        }
        let att = s.attempts["a1"].clone();
        let arow = AttemptRow::from_attempt(
            &att,
            "enr-1",
            "T-1",
            Some("sha256:sealed".into()),
            Some(2),
            Some(3),
        );
        assert_eq!(arow.insert_params().len(), 13);
        assert_eq!(arow.clone().into_attempt().unwrap(), att); // attempt row round trip
        let head = s.heads["a1"].clone();
        assert_eq!(
            AssessmentRow::from_assessment(&head).into_assessment(),
            head
        ); // head row round trip
        let obs = s.observations[0].clone();
        let orow = ObservationRow::from_observation(&obs, att.difficulty, "g1");
        assert_eq!(orow.insert_params("enr-1").len(), 13);
        assert_eq!(
            orow.clone()
                .into_observation(att.variant.clone(), att.prediction_before_reveal),
            obs
        ); // observation row round trip
           // projection reconstruction agrees with the live (replay-built) state
        let rebuilt = state_from_projection(
            s.revision,
            vec![arow],
            vec![AssessmentRow::from_assessment(&head)],
            vec![orow],
        )
        .unwrap();
        assert_eq!(rebuilt.attempts["a1"], att);
        assert_eq!(rebuilt.heads["a1"], head);
        assert_eq!(rebuilt.observations, s.observations);
    }

    #[test]
    fn sql_statements_match_schema_shape() {
        assert!(
            INSERT_EVENT_SQL.contains("cogmap.events")
                && INSERT_EVENT_SQL.contains("RETURNING seq")
        );
        assert_eq!(INSERT_EVENT_SQL.matches('$').count(), 5);
        assert!(SELECT_EVENTS_SQL.contains("ORDER BY seq ASC"));
        assert!(
            INSERT_ATTEMPT_SQL.contains("cogmap.attempts")
                && INSERT_ATTEMPT_SQL.contains("ON CONFLICT (enrollment_id, id)")
        );
        assert_eq!(INSERT_ATTEMPT_SQL.matches('$').count(), 13);
        assert!(INSERT_ASSESSMENT_SQL.contains("cogmap.assessments"));
        assert_eq!(INSERT_ASSESSMENT_SQL.matches('$').count(), 19);
        assert!(!INSERT_ASSESSMENT_SQL.contains("score_units")); // generated column: never inserted (§E)
        assert!(INSERT_OBSERVATION_SQL.contains("cogmap.dimension_observations"));
        assert!(
            INSERT_OBSERVATION_SQL.contains("ON CONFLICT (enrollment_id, attempt_id, dimension)")
        ); // I05 upsert
        assert_eq!(INSERT_OBSERVATION_SQL.matches('$').count(), 13);
        assert!(INSERT_RECEIPT_SQL.contains("ON CONFLICT (enrollment_id, command_id) DO NOTHING")); // I03
                                                                                                    // graded-event assessment params: hint percent from tier (slice1 §E set)
        let ev = Event::AssessmentAccepted {
            event_id: "x".into(),
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
        };
        let p = assessment_insert_params("enr-1", "sub-1", "runner", "v1", "sha256:art", 2, &ev)
            .unwrap();
        assert_eq!(p.len(), 19);
        assert_eq!(p[13], SqlParam::Int(75)); // hint_tier 2 -> 75
        assert!(assessment_insert_params(
            "e",
            "s",
            "g",
            "v",
            "h",
            0,
            &Event::Submit {
                event_id: "x".into(),
                attempt_id: "a".into()
            }
        )
        .is_none());
    }

    #[test]
    fn activation_persistence_round_trip_including_kill_switch() {
        assert!(ACTIVATION_DDL.contains("CREATE TABLE IF NOT EXISTS cogmap.activation"));
        assert!(ACTIVATION_DDL.contains("'disabled'"));
        assert_eq!(ACTIVATION_UPSERT_SQL.matches('$').count(), 2);
        // Drive a real Activation to Disabled, persist its image, reconstruct.
        let mut act = Activation::new("local-stewart");
        act.opt_in("Stewart").unwrap();
        act.activate("Stewart").unwrap();
        act.disable("Stewart").unwrap(); // kill switch
        let params = activation_upsert_params(&act);
        assert_eq!(
            params,
            vec![
                SqlParam::Text("local-stewart".into()),
                SqlParam::Text("disabled".into())
            ]
        );
        let audit_rows: Vec<(String, String, String)> = act
            .audit
            .iter()
            .map(|e| {
                (
                    e.actor.clone(),
                    e.action.clone(),
                    phase_str(e.phase_after).to_string(),
                )
            })
            .collect();
        for e in &act.audit {
            assert_eq!(activation_audit_params("local-stewart", e).len(), 4);
        }
        let back = activation_from_rows("local-stewart", "disabled", audit_rows).unwrap();
        assert_eq!(back.phase, ActivationPhase::Disabled);
        assert_eq!(back.audit, act.audit);
        assert_eq!(
            back.guard_grade(),
            Err(cogmap_slice4::optin::ActivationError::GradingHalted)
        ); // persisted kill switch still blocks
        assert!(phase_from_str("bogus").is_err());
    }
}
