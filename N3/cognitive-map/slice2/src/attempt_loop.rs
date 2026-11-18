//! Model Home cognitive map — Slice 2 Studio attempt loop (Blueprint v1.1 §§C,D,H).
//! Pipeline: assign -> commit prediction (hash-sealed, must precede reveal) -> reveal
//! (reducer event committed BEFORE content is returned) -> submit immutable artifact
//! (SHA-256, finalized) -> VM runner job -> deterministic grading (runner criteria ONLY)
//! -> admission into the Slice 1 reducer as an assessment head (I04/I05).
//! Semantic (model-judged) criteria are NEVER admitted here: `admit_semantic` refuses
//! without a named human reviewer (Stewart) and, even then, stays provisional — it is
//! not written into the reducer by this slice (Blueprint §D.5, PLAN Slice 2 fence).
use cogmap::{reduce, Event, Reject, State};
use std::collections::HashMap;
use std::fmt;

use crate::hash::sha256_hex;

// ---------------- templates & ground truth (server-side only) ----------------
/// Ground truth NEVER leaves the server: reveal returns the task body (snippet),
/// never the verdict, defect list, decoys, or this hash's preimage (§5: never send
/// clean flags / answer keys to the learner client).
#[derive(Clone, Debug)]
pub struct TaskTemplate {
    pub template_id: String,
    pub concept_id: String,
    pub dimension: String,
    pub difficulty: u8, // D0..D5
    pub subtlety: u8,   // S1..S5
    pub variant: String,
    pub independent: bool, // distinct task, core answer not supplied (§D)
    pub defective_task: bool,
    pub required_defects: Vec<String>, // e.g. ["D1"]
    pub decoys: Vec<String>,           // e.g. ["X1"]
    pub ground_truth: String,          // canonical ground-truth text (private)
    pub task_body: String,             // snippet shown at reveal
    pub test_bundle_name: String,
    pub test_bundle: String, // python source of the test bundle
    pub criteria: Vec<CriterionSpec>,
}
impl TaskTemplate {
    pub fn ground_truth_hash(&self) -> String {
        sha256_hex(self.ground_truth.as_bytes())
    }
    pub fn test_bundle_hash(&self) -> String {
        sha256_hex(self.test_bundle.as_bytes())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum CriterionKind {
    Verdict,
    Localization(String),
    Decoy(String),
    Evidence,
}
#[derive(Clone, Debug)]
pub struct CriterionSpec {
    pub test_id: String,
    pub kind: CriterionKind,
}

// ---------------- submissions: immutable, finalized (Blueprint §§C,D,H) ----------------
#[derive(Clone, Debug, PartialEq)]
pub struct Submission {
    pub submission_id: String,
    pub attempt_id: String,
    pub artifact_hash: String, // sha256 of the exact artifact bytes
    pub finalized: bool,       // only finalized artifacts may enter a command (§H)
}
#[derive(Default)]
pub struct ArtifactStore {
    artifacts: HashMap<String, Vec<u8>>, // artifact_hash -> bytes (immutable revisions, never overwritten)
    submissions: HashMap<String, Submission>, // submission_id -> submission
    by_attempt: HashMap<String, String>, // attempt_id -> submission_id (one scored submission per attempt, §J)
}
#[derive(Debug, PartialEq)]
pub enum LoopError {
    Reducer(Reject),
    UnknownAttempt,
    AlreadyRevealed,
    PostRevealPrediction,
    SubmissionConflict, // same submission_id, different bytes; or second scored submission
    ArtifactNotFinalized,
    ReviewerRequired, // semantic grade without a named human reviewer (§D.5)
    Runner(String),
}
impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl From<Reject> for LoopError {
    fn from(r: Reject) -> Self {
        LoopError::Reducer(r)
    }
}

// ---------------- VM runner abstraction (Blueprint §D.2, §H) ----------------
/// Everything the runner consumed, pinned by hash so a grade is reproducible (§D.2:
/// environment image digest, input hash, test bundle hash, exit status, output hash).
#[derive(Clone, Debug, PartialEq)]
pub struct RunnerJob {
    pub job_id: String,
    pub attempt_id: String,
    pub submission_hash: String, // == artifact_hash of the finalized submission
    pub test_bundle_hash: String,
    pub artifact: Vec<u8>,
    pub test_bundle: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RunnerRecord {
    pub job_id: String,
    pub env_digest: String, // digest of the execution environment (VM image / runtime descriptor)
    pub input_hash: String, // hash of what the runner was fed (job inputs)
    pub test_bundle_hash: String,
    pub exit_status: i32, // runner diagnostic only; grading uses per-test results
    pub output_hash: String,
    pub results: Vec<(String, bool)>, // test_id -> passed
}
pub trait Runner {
    fn run(&mut self, job: &RunnerJob) -> Result<RunnerRecord, LoopError>;
}

/// Portable runner executing the bundle with a local python3 process. The Mac
/// `vm_exec` (Lima microVM) runner implements the same trait; that integration is
/// UNVERIFIED on the Mac (see README) and is never assumed by the tests here.
pub struct LocalProcessRunner {
    pub python: String,
}
impl LocalProcessRunner {
    pub fn new() -> Self {
        LocalProcessRunner {
            python: "python3".into(),
        }
    }
    fn exec_once(&mut self, job: &RunnerJob) -> Result<RunnerRecord, LoopError> {
        let dir =
            std::env::temp_dir().join(format!("cogmap-run-{}-{}", std::process::id(), job.job_id));
        std::fs::create_dir_all(&dir).map_err(|e| LoopError::Runner(e.to_string()))?;
        let artifact_path = dir.join("submission.txt");
        let bundle_path = dir.join("test_bundle.py");
        std::fs::write(&artifact_path, &job.artifact)
            .map_err(|e| LoopError::Runner(e.to_string()))?;
        std::fs::write(&bundle_path, &job.test_bundle)
            .map_err(|e| LoopError::Runner(e.to_string()))?;
        let out = std::process::Command::new(&self.python)
            .arg(&bundle_path)
            .arg(&artifact_path)
            .current_dir(&dir)
            .output()
            .map_err(|e| LoopError::Runner(e.to_string()))?;
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let mut results = Vec::new();
        for line in stdout.lines() {
            let mut it = line.split_whitespace();
            if it.next() == Some("TEST") {
                if let (Some(id), Some(res)) = (it.next(), it.next()) {
                    results.push((id.to_string(), res == "PASS"));
                }
            }
        }
        let env_digest = sha256_hex(format!("local-process:{}:py3", self.python).as_bytes());
        let record = RunnerRecord {
            job_id: job.job_id.clone(),
            env_digest,
            input_hash: sha256_hex(job.artifact.as_slice()),
            test_bundle_hash: job.test_bundle_hash.clone(),
            exit_status: out.status.code().unwrap_or(-1),
            output_hash: sha256_hex(stdout.as_bytes()),
            results,
        };
        let _ = std::fs::remove_dir_all(&dir);
        Ok(record)
    }
}
impl Runner for LocalProcessRunner {
    fn run(&mut self, job: &RunnerJob) -> Result<RunnerRecord, LoopError> {
        self.exec_once(job)
    }
}

/// Test-only wrapper: forces the second physical run of one job to diverge,
/// simulating a flaky/nondeterministic check (§D.2: flaky => unresolved criterion).
pub struct FlakySecondRun<R: Runner> {
    pub inner: R,
    runs: usize,
}
impl<R: Runner> FlakySecondRun<R> {
    pub fn new(inner: R) -> Self {
        FlakySecondRun { inner, runs: 0 }
    }
}
impl<R: Runner> Runner for FlakySecondRun<R> {
    fn run(&mut self, job: &RunnerJob) -> Result<RunnerRecord, LoopError> {
        let mut rec = self.inner.run(job)?;
        self.runs += 1;
        if self.runs % 2 == 0 {
            rec.exit_status = if rec.exit_status == 0 { 1 } else { 0 };
            rec.results = rec.results.into_iter().map(|(id, p)| (id, !p)).collect();
            rec.output_hash = sha256_hex(b"flaky-diverged-run");
        }
        Ok(rec)
    }
}

// ---------------- deterministic grading: runner criteria only (§§D,E) ----------------
/// Fixture-authored rubric mapping (declared per template criteria list, not
/// Blueprint-normative): v from the verdict test; j scaled from justification
/// tests; q scaled from evidence tests. All integer, no floats anywhere.
#[derive(Clone, Debug, PartialEq)]
pub struct Grade {
    pub v: u8,
    pub j: u8,
    pub q: u8,
    pub false_negative: bool,
    pub false_positive: bool,
    pub fully_clean: bool,
}
pub fn grade(template: &TaskTemplate, results: &[(String, bool)]) -> Grade {
    let get = |id: &str| {
        results
            .iter()
            .find(|(t, _)| t == id)
            .map(|(_, p)| *p)
            .unwrap_or(false)
    };
    let mut v = 0u8;
    let (mut just_pass, mut just_total) = (0i64, 0i64);
    let (mut ev_pass, mut ev_total) = (0i64, 0i64);
    let mut false_positive = false;
    let mut missed_required = false;
    for c in &template.criteria {
        let passed = get(&c.test_id);
        match &c.kind {
            CriterionKind::Verdict => {
                if passed {
                    v = 1;
                }
            }
            CriterionKind::Localization(defect) => {
                just_total += 1;
                if passed {
                    just_pass += 1;
                }
                if !passed && template.required_defects.iter().any(|d| d == defect) {
                    missed_required = true;
                }
            }
            CriterionKind::Decoy(_) => {
                just_total += 1;
                if passed {
                    just_pass += 1;
                } else {
                    false_positive = true;
                }
            }
            CriterionKind::Evidence => {
                ev_total += 1;
                if passed {
                    ev_pass += 1;
                }
            }
        }
    }
    if template.defective_task
        && !template
            .criteria
            .iter()
            .any(|c| matches!(c.kind, CriterionKind::Localization(_)))
    {
        // A defective task with no localization criterion cannot certify a catch.
        missed_required = true;
    }
    if !template.defective_task {
        // Clean task: justification = clean justification (§3) — correct verdict with no false claim.
        just_total = just_total.max(1);
        just_pass = if v == 1 && !false_positive {
            just_total
        } else {
            just_pass
        };
    }
    let j = if just_total == 0 {
        0
    } else {
        ((3 * just_pass) / just_total) as u8
    };
    let q = if ev_total == 0 {
        0
    } else {
        ((4 * ev_pass + ev_total / 2) / ev_total) as u8
    };
    Grade {
        v,
        j,
        q,
        false_negative: template.defective_task && missed_required,
        false_positive,
        fully_clean: !template.defective_task
            && v == 1
            && !false_positive
            && just_pass == just_total,
    }
}

/// Provisional semantic (model-judged) grade. NEVER admitted by this slice (§D.5).
#[derive(Clone, Debug)]
pub struct SemanticProposal {
    pub v: u8,
    pub j: u8,
    pub q: u8,
    pub rationale: String,
}

// ---------------- the session orchestrator ----------------
pub enum GradeOutcome {
    /// Deterministic grade admitted into the reducer as the assessment head.
    Assessed(Grade, String), // grade, assessment_id
    /// Runner flaky/unavailable: stays assessment_pending, NO observation (§C,§D.2).
    Unresolved(String),
}
pub struct Session {
    pub state: State,
    pub store: ArtifactStore,
    seq: i64, // submission sequence source (server-held in production; monotone here)
}
impl Session {
    pub fn new() -> Self {
        Session {
            state: State::default(),
            store: ArtifactStore::default(),
            seq: 0,
        }
    }
    fn next_seq(&mut self) -> i64 {
        self.seq += 1;
        self.seq
    }
    fn template_ok(&self, t: &TaskTemplate) -> Result<(), LoopError> {
        // Sanity only; hashes pin the exact fixture/rubric bytes used (§D manifest rule).
        if t.ground_truth.is_empty() || t.test_bundle.is_empty() {
            return Err(LoopError::Runner("empty template".into()));
        }
        Ok(())
    }

    /// Assign: creates the attempt via the reducer. Ground truth stays server-side.
    pub fn assign(
        &mut self,
        t: &TaskTemplate,
        event_id: &str,
        attempt_id: &str,
    ) -> Result<(), LoopError> {
        self.template_ok(t)?;
        self.state = reduce(
            self.state.clone(),
            &Event::AssignTask {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
                concept_id: t.concept_id.clone(),
                dimension: t.dimension.clone(),
                difficulty: t.difficulty,
                variant: t.variant.clone(),
                independent: t.independent,
            },
        )?;
        Ok(())
    }

    /// Commit prediction: hash-sealed BEFORE reveal. The sealed bytes are stored, the
    /// prediction_hash recorded. Post-reveal commits are rejected (§C) — see tests.
    pub fn commit_prediction(
        &mut self,
        attempt_id: &str,
        prediction: &[u8],
        event_id: &str,
    ) -> Result<String, LoopError> {
        let cur = self
            .state
            .attempts
            .get(attempt_id)
            .cloned()
            .ok_or(LoopError::UnknownAttempt)?;
        if cur.state == cogmap::AttemptState::Revealed
            || cur.state == cogmap::AttemptState::AssessmentPending
            || cur.state == cogmap::AttemptState::Assessed
        {
            return Err(LoopError::PostRevealPrediction);
        }
        let hash = sha256_hex(prediction);
        self.store
            .artifacts
            .insert(format!("prediction:{attempt_id}"), prediction.to_vec());
        self.state = reduce(
            self.state.clone(),
            &Event::CommitPrediction {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
            },
        )?;
        Ok(hash)
    }

    /// Reveal: the reducer event is committed BEFORE the task body is returned to the
    /// caller (§C: "persist release before returning content"). Without a committed
    /// prediction this returns Err and no content.
    pub fn reveal(
        &mut self,
        attempt_id: &str,
        event_id: &str,
        t: &TaskTemplate,
    ) -> Result<String, LoopError> {
        self.state = reduce(
            self.state.clone(),
            &Event::Reveal {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
            },
        )?;
        Ok(t.task_body.clone())
    }

    /// Hint (tier recorded pre-submission, feeds score multiplier §E).
    pub fn hint(&mut self, attempt_id: &str, tier: u8, event_id: &str) -> Result<(), LoopError> {
        self.state = reduce(
            self.state.clone(),
            &Event::DeliverHint {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
                tier,
            },
        )?;
        Ok(())
    }

    /// Submit: immutable finalized artifact (SHA-256). Duplicate command
    /// (same submission_id + same bytes) is idempotent: same receipt, no state change
    /// (I03). Different bytes under one submission_id, or a second scored submission
    /// for the same attempt, are rejected (§J).
    pub fn submit(
        &mut self,
        attempt_id: &str,
        submission_id: &str,
        artifact: &[u8],
        event_id: &str,
    ) -> Result<Submission, LoopError> {
        let artifact_hash = sha256_hex(artifact);
        if let Some(prev) = self.store.submissions.get(submission_id) {
            if prev.artifact_hash == artifact_hash && prev.attempt_id == attempt_id {
                return Ok(prev.clone()); // I03: retry changes no domain state
            }
            return Err(LoopError::SubmissionConflict);
        }
        if self.store.by_attempt.contains_key(attempt_id) {
            return Err(LoopError::SubmissionConflict);
        }
        self.store
            .artifacts
            .insert(artifact_hash.clone(), artifact.to_vec()); // immutable revision stored (§D.1)
        let sub = Submission {
            submission_id: submission_id.into(),
            attempt_id: attempt_id.into(),
            artifact_hash,
            finalized: true,
        };
        self.store
            .submissions
            .insert(submission_id.into(), sub.clone());
        self.store
            .by_attempt
            .insert(attempt_id.into(), submission_id.into());
        self.state = reduce(
            self.state.clone(),
            &Event::Submit {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
            },
        )?;
        Ok(sub)
    }

    /// Run + grade + admit. The job runs TWICE: disagreement between runs is a
    /// flaky/nondeterministic check and yields an unresolved criterion, never a
    /// learner failure (§D.2, §C AssessmentUnavailable). Admission writes exactly
    /// one reducer assessment head and one dimension observation (I04/I05).
    pub fn grade_and_admit<R: Runner>(
        &mut self,
        attempt_id: &str,
        t: &TaskTemplate,
        runner: &mut R,
        run_event_id: &str,
        accept_event_id: &str,
        assessment_id: &str,
    ) -> Result<GradeOutcome, LoopError> {
        let sub_id = self
            .store
            .by_attempt
            .get(attempt_id)
            .cloned()
            .ok_or(LoopError::Reducer(Reject::BadTransition))?;
        let sub = self
            .store
            .submissions
            .get(&sub_id)
            .cloned()
            .ok_or(LoopError::Reducer(Reject::BadTransition))?;
        if !sub.finalized {
            return Err(LoopError::ArtifactNotFinalized);
        }
        let bytes = self
            .store
            .artifacts
            .get(&sub.artifact_hash)
            .cloned()
            .unwrap_or_default();
        let job = RunnerJob {
            job_id: sha256_hex(
                format!("job:{}:{}", sub.submission_id, sub.artifact_hash).as_bytes(),
            ),
            attempt_id: attempt_id.into(),
            submission_hash: sub.artifact_hash.clone(),
            test_bundle_hash: t.test_bundle_hash(),
            artifact: bytes,
            test_bundle: t.test_bundle.clone(),
        };
        let _ = run_event_id; // outbox job identity recorded on the job itself (§H); no reducer event for enqueue
        let r1 = runner.run(&job)?;
        let r2 = runner.run(&job)?;
        if r1.results != r2.results || r1.exit_status != r2.exit_status {
            self.state = reduce(
                self.state.clone(),
                &Event::AssessmentUnavailable {
                    event_id: accept_event_id.into(),
                    attempt_id: attempt_id.into(),
                },
            )?;
            return Ok(GradeOutcome::Unresolved(job.job_id));
        }
        let g = grade(t, &r1.results);
        let seq = self.next_seq();
        self.state = reduce(
            self.state.clone(),
            &Event::AssessmentAccepted {
                event_id: accept_event_id.into(),
                attempt_id: attempt_id.into(),
                assessment_id: assessment_id.into(),
                v: g.v,
                j: g.j,
                q: g.q,
                seq,
                false_negative: g.false_negative,
                false_positive: g.false_positive,
                fully_clean: g.fully_clean,
                defective_task: t.defective_task,
            },
        )?;
        Ok(GradeOutcome::Assessed(g, assessment_id.into()))
    }

    /// Semantic grades are provisional only. Without a named human reviewer there is
    /// no admission path at all; with one, this slice still does NOT write it — the
    /// proposal is returned for reviewer acceptance in a later slice (§D.5).
    pub fn admit_semantic(
        &mut self,
        _attempt_id: &str,
        p: SemanticProposal,
        reviewer: Option<&str>,
    ) -> Result<SemanticProposal, LoopError> {
        match reviewer {
            None => Err(LoopError::ReviewerRequired),
            Some(_) => Ok(p),
        }
    }

    pub fn dispute(&mut self, attempt_id: &str, event_id: &str) -> Result<(), LoopError> {
        self.state = reduce(
            self.state.clone(),
            &Event::Dispute {
                event_id: event_id.into(),
                attempt_id: attempt_id.into(),
            },
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cogmap::AttemptState;
    const DIR: &str = env!("CARGO_MANIFEST_DIR");
    fn load(p: &str) -> String {
        std::fs::read_to_string(format!("{DIR}/{p}")).unwrap()
    }
    fn clean_template() -> TaskTemplate {
        TaskTemplate {
            template_id: "T-CLEAN-1".into(),
            concept_id: "search-return".into(),
            dimension: "SPEC".into(),
            difficulty: 1,
            subtlety: 1,
            variant: "clean-a".into(),
            independent: true,
            defective_task: false,
            required_defects: vec![],
            decoys: vec!["X1".into()],
            ground_truth: "verdict=clean;decoy=X1(return -1 line)".into(),
            task_body: load("fixtures/clean/snippet.py"),
            test_bundle_name: "clean/test_bundle.py".into(),
            test_bundle: load("fixtures/clean/test_bundle.py"),
            criteria: vec![
                CriterionSpec {
                    test_id: "t_verdict".into(),
                    kind: CriterionKind::Verdict,
                },
                CriterionSpec {
                    test_id: "t_decoy_X1".into(),
                    kind: CriterionKind::Decoy("X1".into()),
                },
                CriterionSpec {
                    test_id: "t_evidence".into(),
                    kind: CriterionKind::Evidence,
                },
            ],
        }
    }
    fn defective_template() -> TaskTemplate {
        TaskTemplate {
            template_id: "T-DEFECT-1".into(),
            concept_id: "search-return".into(),
            dimension: "SPEC".into(),
            difficulty: 1,
            subtlety: 2,
            variant: "defect-a".into(),
            independent: true,
            defective_task: true,
            required_defects: vec!["D1".into()],
            decoys: vec![],
            ground_truth: "verdict=defect;defect=D1(range one past end)".into(),
            task_body: load("fixtures/defective/snippet.py"),
            test_bundle_name: "defective/test_bundle.py".into(),
            test_bundle: load("fixtures/defective/test_bundle.py"),
            criteria: vec![
                CriterionSpec {
                    test_id: "t_verdict".into(),
                    kind: CriterionKind::Verdict,
                },
                CriterionSpec {
                    test_id: "t_loc_D1".into(),
                    kind: CriterionKind::Localization("D1".into()),
                },
                CriterionSpec {
                    test_id: "t_evidence".into(),
                    kind: CriterionKind::Evidence,
                },
            ],
        }
    }
    fn flow_to_pending(s: &mut Session, t: &TaskTemplate, att: &str) {
        s.assign(t, &format!("{att}-assign"), att).unwrap();
        s.commit_prediction(att, b"VERDICT prediction sealed", &format!("{att}-pred"))
            .unwrap();
        let body = s.reveal(att, &format!("{att}-reveal"), t).unwrap();
        assert_eq!(body, t.task_body);
    }

    #[test]
    fn reveal_without_prediction_rejected_and_no_content() {
        let t = clean_template();
        let mut s = Session::new();
        s.assign(&t, "e-assign", "att-1").unwrap();
        let r = s.reveal("att-1", "e-reveal", &t);
        assert!(matches!(
            r,
            Err(LoopError::Reducer(Reject::RevealWithoutPrediction))
        )); // §C gate, content withheld
        assert_eq!(s.state.attempts["att-1"].state, AttemptState::Assigned);
    }
    #[test]
    fn post_reveal_prediction_rejected() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        let r = s.commit_prediction("att-1", b"late guess", "e-late");
        assert_eq!(r, Err(LoopError::PostRevealPrediction));
    }
    #[test]
    fn duplicate_submit_idempotent_no_state_change() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        let good = load("fixtures/clean/submission_good.txt");
        let a = s
            .submit("att-1", "sub-1", good.as_bytes(), "e-submit")
            .unwrap();
        let rev = s.state.revision;
        let b = s
            .submit("att-1", "sub-1", good.as_bytes(), "e-submit-retry")
            .unwrap(); // I03 retry
        assert_eq!(a, b);
        assert_eq!(s.state.revision, rev); // zero state change on retry
                                           // same id, different bytes -> conflict; second submission other id -> conflict (§J)
        assert_eq!(
            s.submit("att-1", "sub-1", b"tampered", "e-x"),
            Err(LoopError::SubmissionConflict)
        );
        assert_eq!(
            s.submit("att-1", "sub-2", good.as_bytes(), "e-y"),
            Err(LoopError::SubmissionConflict)
        );
    }
    #[test]
    fn clean_task_good_submission_assessed_clean() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        s.submit(
            "att-1",
            "sub-1",
            load("fixtures/clean/submission_good.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut runner = LocalProcessRunner::new();
        match s
            .grade_and_admit("att-1", &t, &mut runner, "e-job", "e-accept", "g1")
            .unwrap()
        {
            GradeOutcome::Assessed(g, _) => {
                assert_eq!((g.v, g.j), (1, 3));
                assert!(!g.false_negative && !g.false_positive && g.fully_clean);
            }
            other => panic!(
                "expected Assessed, got {:?}",
                matches!(other, GradeOutcome::Unresolved(_))
            ),
        }
        assert_eq!(s.state.attempts["att-1"].state, AttemptState::Assessed);
        assert_eq!(s.state.observations.len(), 1); // I05
        assert_eq!(s.state.heads["att-1"].id, "g1"); // I04 head
    }
    #[test]
    fn clean_task_false_claim_is_false_positive_not_fully_clean() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        s.submit(
            "att-1",
            "sub-1",
            load("fixtures/clean/submission_wrong.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut runner = LocalProcessRunner::new();
        match s
            .grade_and_admit("att-1", &t, &mut runner, "e-job", "e-accept", "g1")
            .unwrap()
        {
            GradeOutcome::Assessed(g, _) => {
                assert_eq!(g.v, 0);
                assert!(g.false_positive); // decoy claimed (§E: naming a decoy is FP)
                assert!(!g.fully_clean);
            }
            _ => panic!("expected Assessed"),
        }
    }
    #[test]
    fn defective_task_caught_and_missed_paths() {
        let t = defective_template();
        // caught
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        s.submit(
            "att-1",
            "sub-1",
            load("fixtures/defective/submission_good.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut runner = LocalProcessRunner::new();
        match s
            .grade_and_admit("att-1", &t, &mut runner, "e-job", "e-accept", "g1")
            .unwrap()
        {
            GradeOutcome::Assessed(g, _) => {
                assert_eq!((g.v, g.j), (1, 3));
                assert!(!g.false_negative);
            }
            _ => panic!("expected Assessed"),
        }
        // missed -> FN true (§E: missed required defect sets FN)
        let mut s2 = Session::new();
        flow_to_pending(&mut s2, &t, "att-1");
        s2.submit(
            "att-1",
            "sub-1",
            load("fixtures/defective/submission_missed.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut runner2 = LocalProcessRunner::new();
        match s2
            .grade_and_admit("att-1", &t, &mut runner2, "e-job", "e-accept", "g1")
            .unwrap()
        {
            GradeOutcome::Assessed(g, _) => {
                assert_eq!(g.v, 0);
                assert!(g.false_negative);
            }
            _ => panic!("expected Assessed"),
        }
    }
    #[test]
    fn flaky_runner_yields_unresolved_no_negative_evidence() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        s.submit(
            "att-1",
            "sub-1",
            load("fixtures/clean/submission_good.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut flaky = FlakySecondRun::new(LocalProcessRunner::new());
        match s
            .grade_and_admit("att-1", &t, &mut flaky, "e-job", "e-accept", "g1")
            .unwrap()
        {
            GradeOutcome::Unresolved(_) => {}
            _ => panic!("expected Unresolved"),
        }
        assert_eq!(
            s.state.attempts["att-1"].state,
            AttemptState::AssessmentPending
        ); // stays pending (§C)
        assert!(s.state.observations.is_empty()); // no negative evidence minted (§D.2)
        assert!(s.state.heads.is_empty());
    }
    #[test]
    fn dispute_removes_evidence_from_active_map() {
        let t = clean_template();
        let mut s = Session::new();
        flow_to_pending(&mut s, &t, "att-1");
        s.submit(
            "att-1",
            "sub-1",
            load("fixtures/clean/submission_good.txt").as_bytes(),
            "e-submit",
        )
        .unwrap();
        let mut runner = LocalProcessRunner::new();
        let _ = s
            .grade_and_admit("att-1", &t, &mut runner, "e-job", "e-accept", "g1")
            .unwrap();
        assert_eq!(s.state.observations.len(), 1);
        s.dispute("att-1", "e-dispute").unwrap();
        assert_eq!(s.state.attempts["att-1"].state, AttemptState::Disputed);
        assert!(s.state.observations.is_empty()); // affected result out of active evidence (§C)
    }
    #[test]
    fn semantic_grade_never_auto_accepted() {
        let mut s = Session::new();
        let p = SemanticProposal {
            v: 1,
            j: 3,
            q: 4,
            rationale: "model guess".into(),
        };
        assert_eq!(
            s.admit_semantic("att-1", p.clone(), None).unwrap_err(),
            LoopError::ReviewerRequired
        );
        let ok = s.admit_semantic("att-1", p, Some("Stewart")); // reviewer named: still provisional only
        assert!(ok.is_ok());
        assert!(s.state.heads.is_empty() && s.state.observations.is_empty()); // nothing written (§D.5 fence)
    }
    #[test]
    fn ground_truth_never_in_revealed_content() {
        let t = clean_template();
        assert_ne!(t.task_body, t.ground_truth);
        assert!(t.ground_truth_hash().len() == 64 && t.test_bundle_hash().len() == 64);
    }
}
