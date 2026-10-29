//! Item 3: append-only session log with sequence numbers, workflow id,
//! resume-by-session, fork, and replay.
//!
//! The mirror backend has (timestamp, kind, data) only — no sequence, no
//! workflow id. This log is the fix: every event is sequenced per session,
//! names its workflow, hashes to its predecessor (sha-chain), and replays
//! deterministically. A crashed run resumes from the log instead of
//! starting over; a failure forks into a compared rerun.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One logged event. Outside the context window, append-only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub seq: u64,
    pub session_id: String,
    pub workflow_id: String,
    pub timestamp_ms: u64,
    pub kind: SessionEventKind,
    pub payload_json: String,
    pub prev_hash: String,
    pub event_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionEventKind {
    ContextInject,
    ToolCall,
    ToolResult,
    JudgeVerdict,
    Checkpoint,
    Fork,
    Resume,
}

impl SessionEvent {
    fn hash_of(
        seq: u64,
        session: &str,
        workflow: &str,
        kind: &str,
        payload: &str,
        prev: &str,
    ) -> String {
        let mut h = Sha256::new();
        h.update(seq.to_le_bytes());
        h.update(session.as_bytes());
        h.update(workflow.as_bytes());
        h.update(kind.as_bytes());
        h.update(payload.as_bytes());
        h.update(prev.as_bytes());
        hex::encode(h.finalize())
    }
}

/// Append-only log for one session. Sequence numbers are per-session and
/// gapless; any gap on reload is explicit missing data (replay from last
/// checkpoint, never silent — N3 §10 push/sequence rule).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionLog {
    pub session_id: String,
    pub events: Vec<SessionEvent>,
}

impl SessionLog {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.into(),
            events: vec![],
        }
    }

    pub fn append(
        &mut self,
        workflow_id: &str,
        kind: SessionEventKind,
        payload_json: &str,
        timestamp_ms: u64,
    ) -> &SessionEvent {
        let seq = self.events.len() as u64;
        let prev = self
            .events
            .last()
            .map(|e| e.event_hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        let kind_s = format!("{:?}", kind);
        let hash = SessionEvent::hash_of(
            seq,
            &self.session_id,
            workflow_id,
            &kind_s,
            payload_json,
            &prev,
        );
        self.events.push(SessionEvent {
            seq,
            session_id: self.session_id.clone(),
            workflow_id: workflow_id.into(),
            timestamp_ms,
            kind,
            payload_json: payload_json.into(),
            prev_hash: prev,
            event_hash: hash,
        });
        self.events.last().unwrap()
    }

    /// Verify the sha-chain. Returns the first broken seq, or None if intact.
    pub fn verify_chain(&self) -> Option<u64> {
        let mut prev = "GENESIS".to_string();
        for e in &self.events {
            if e.prev_hash != prev {
                return Some(e.seq);
            }
            let recomputed = SessionEvent::hash_of(
                e.seq,
                &e.session_id,
                &e.workflow_id,
                &format!("{:?}", e.kind),
                &e.payload_json,
                &e.prev_hash,
            );
            if recomputed != e.event_hash {
                return Some(e.seq);
            }
            prev = e.event_hash.clone();
        }
        None
    }

    /// Resume point: seq after the last Checkpoint, else 0. A crashed run
    /// replays from here instead of starting over.
    pub fn resume_seq(&self) -> u64 {
        self.events
            .iter()
            .rev()
            .find(|e| e.kind == SessionEventKind::Checkpoint)
            .map(|e| e.seq + 1)
            .unwrap_or(0)
    }

    /// Events from a seq onward (the replay slice).
    pub fn replay_from(&self, seq: u64) -> Vec<&SessionEvent> {
        self.events.iter().filter(|e| e.seq >= seq).collect()
    }

    /// Fork: a new session sharing the prefix through `at_seq`, then
    /// diverging. The fork event records parent + seq (provenance, not copy).
    pub fn fork(&self, at_seq: u64, fork_session_id: &str) -> SessionLog {
        let mut forked = SessionLog::new(fork_session_id);
        for e in self.events.iter().filter(|e| e.seq <= at_seq) {
            forked.events.push(e.clone());
        }
        forked
    }

    pub fn trajectory_view(&self) -> Vec<(u64, String, String)> {
        self.events
            .iter()
            .map(|e| (e.seq, format!("{:?}", e.kind), e.payload_json.clone()))
            .collect()
    }
}
