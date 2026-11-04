//! Mission model: byte-for-byte mirror of backend `harness::mission` subset.
//!
//! Same struct names, same field order, same `chain_hash` canonical form:
//! `prev|packet|attempt|outcome|judges|n3|energy|erased|timestamp`
//! with `{:?}` energy formatting and GENESIS origin. `ReceiptLog::append`
//! is the sole mint (stamps prev_hash, fills zero timestamps, recomputes
//! evidence_hash); `verify_chain` recomputes every link.

use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

// ===========================================================================
// Runbook
// ===========================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PacketStatus {
    Pending,
    Ready,
    Running,
    Passed,
    Failed,
    Skipped,
    Escalated,
}

impl PacketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Ready => "Ready",
            Self::Running => "Running",
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::Escalated => "Escalated",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool: String,
    pub args_json: String,
    pub declared: bool,
}

#[derive(Debug, Clone)]
pub struct JudgeRef {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_rounds: u32,
    pub material_change_required: bool,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub id: String,
    pub task: String,
    pub depends_on: Vec<String>,
    pub declared_context_tokens: usize,
    pub tool_calls: Vec<ToolCall>,
    pub judges: Vec<JudgeRef>,
    pub branch: Option<String>,
    pub retry_policy: RetryPolicy,
    pub status: PacketStatus,
    pub attempts: u32,
    /// Opt-in: judge the joined output once instead of per call.
    pub judge_joined: bool,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub id: String,
    pub objective: String,
    pub packets: Vec<Packet>,
}

impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for p in &self.packets {
            if !ids.insert(p.id.clone()) {
                return Err(format!("duplicate packet id: {}", p.id));
            }
            if p.judges.is_empty() {
                return Err(format!("packet {} has no judges (hard gate)", p.id));
            }
        }
        let idset: HashSet<String> = self.packets.iter().map(|p| p.id.clone()).collect();
        for p in &self.packets {
            for d in &p.depends_on {
                if !idset.contains(d) {
                    return Err(format!("packet {} depends on unknown {}", p.id, d));
                }
            }
        }
        let mut adj: HashMap<String, Vec<String>> = self
            .packets
            .iter()
            .map(|p| (p.id.clone(), vec![]))
            .collect();
        let mut indeg: HashMap<String, usize> = self
            .packets
            .iter()
            .map(|p| (p.id.clone(), p.depends_on.len()))
            .collect();
        for p in &self.packets {
            for d in &p.depends_on {
                adj.get_mut(d).unwrap().push(p.id.clone());
            }
        }
        let mut q: VecDeque<String> = indeg
            .iter()
            .filter(|&(_, &v)| v == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut seen = 0usize;
        while let Some(n) = q.pop_front() {
            seen += 1;
            for m in adj.get(&n).unwrap() {
                let d = indeg.get_mut(m).unwrap();
                *d = d.saturating_sub(1);
                if *d == 0 {
                    q.push_back(m.clone());
                }
            }
        }
        if seen != self.packets.len() {
            return Err("plan cycle detected".into());
        }
        Ok(())
    }

    pub fn ready_packets(&self) -> Vec<String> {
        self.packets
            .iter()
            .filter(|p| p.status == PacketStatus::Pending)
            .filter(|p| {
                p.depends_on.iter().all(|d| {
                    self.packet(d)
                        .map(|x| x.status == PacketStatus::Passed)
                        .unwrap_or(false)
                })
            })
            .map(|p| p.id.clone())
            .collect()
    }

    pub fn packet(&self, id: &str) -> Option<&Packet> {
        self.packets.iter().find(|p| p.id == id)
    }

    pub fn packet_mut(&mut self, id: &str) -> Option<&mut Packet> {
        self.packets.iter_mut().find(|p| p.id == id)
    }
}

// ===========================================================================
// Judges (deterministic; coders never self-approve)
// ===========================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum JudgeKind {
    ExitCodeZero,
    GoldenHash { expected: String },
    Contains { needle: String },
    NotContains { needle: String },
}

#[derive(Debug, Clone)]
pub struct Judge {
    pub id: String,
    pub kind: JudgeKind,
}

#[derive(Debug, Clone)]
pub enum JudgeOutcome {
    Pass,
    Fail(String),
}

#[derive(Debug, Clone)]
pub struct JudgeResult {
    pub judge_id: String,
    pub outcome: JudgeOutcome,
    pub evidence: String,
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

impl Judge {
    pub fn run(&self, output: &str, exit_code: i32) -> JudgeResult {
        match &self.kind {
            JudgeKind::ExitCodeZero => {
                let ok = exit_code == 0;
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("exit_code={}", exit_code))
                    },
                    evidence: format!("exit_code={}", exit_code),
                }
            }
            JudgeKind::GoldenHash { expected } => {
                let mut h = Sha256::new();
                h.update(output.as_bytes());
                let got = hex_encode(&h.finalize());
                let ok = got == *expected;
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!(
                            "hash mismatch: got {} expected {}",
                            got, expected
                        ))
                    },
                    evidence: format!("got={}", got),
                }
            }
            JudgeKind::Contains { needle } => {
                let ok = output.contains(needle);
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("missing '{}'", needle))
                    },
                    evidence: format!("contains={}", ok),
                }
            }
            JudgeKind::NotContains { needle } => {
                let ok = !output.contains(needle);
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("unexpected '{}'", needle))
                    },
                    evidence: format!("not_contains={}", ok),
                }
            }
        }
    }
}

// ===========================================================================
// Receipts (chain-authenticated; the log is the sole mint)
// ===========================================================================

#[derive(Debug, Clone)]
pub struct Receipt {
    pub packet_id: String,
    pub attempt: u32,
    pub outcome: String, // "PASS" / "FAIL" / "SKIP" / "ESCALATE"
    pub judge_ids: Vec<String>,
    pub evidence_hash: String,
    pub energy_joules: f64,
    pub erased_bits: u64,
    pub timestamp_ms: u64,
    pub prev_hash: String,
    /// Exact 64-hex executor receipt digests bound to this execution.
    pub n3_receipts: Vec<String>,
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[allow(clippy::too_many_arguments)]
pub fn chain_hash(
    prev_hash: &str,
    packet_id: &str,
    attempt: u32,
    outcome: &str,
    judge_ids: &[String],
    n3_receipts: &[String],
    energy_joules: f64,
    erased_bits: u64,
    timestamp_ms: u64,
) -> String {
    let canon = format!(
        "{}|{}|{}|{}|{}|{}|{:?}|{}|{}",
        prev_hash,
        packet_id,
        attempt,
        outcome,
        judge_ids.join(","),
        n3_receipts.join(","),
        energy_joules,
        erased_bits,
        timestamp_ms,
    );
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    hex_encode(&h.finalize())
}

#[derive(Debug, Clone, Default)]
pub struct ReceiptLog {
    pub entries: Vec<Receipt>,
}

impl ReceiptLog {
    /// Chain-authenticated append: stamps `prev_hash` from the tip, fills
    /// the timestamp when 0, recomputes `evidence_hash`. Callers must NOT
    /// pre-set `evidence_hash` — the log is the sole mint.
    pub fn append(&mut self, mut r: Receipt) {
        let prev = self
            .entries
            .last()
            .map(|e| e.evidence_hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        r.prev_hash = prev;
        if r.timestamp_ms == 0 {
            r.timestamp_ms = now_ms();
        }
        r.evidence_hash = chain_hash(
            &r.prev_hash,
            &r.packet_id,
            r.attempt,
            &r.outcome,
            &r.judge_ids,
            &r.n3_receipts,
            r.energy_joules,
            r.erased_bits,
            r.timestamp_ms,
        );
        self.entries.push(r);
    }

    /// Recompute every link. `Err(seq)` names the first broken entry.
    pub fn verify_chain(&self) -> Result<usize, usize> {
        let mut prev = "GENESIS".to_string();
        for (seq, e) in self.entries.iter().enumerate() {
            if e.prev_hash != prev {
                return Err(seq);
            }
            if e.timestamp_ms == 0 {
                return Err(seq);
            }
            let recomputed = chain_hash(
                &e.prev_hash,
                &e.packet_id,
                e.attempt,
                &e.outcome,
                &e.judge_ids,
                &e.n3_receipts,
                e.energy_joules,
                e.erased_bits,
                e.timestamp_ms,
            );
            if recomputed != e.evidence_hash {
                return Err(seq);
            }
            prev = e.evidence_hash.clone();
        }
        Ok(self.entries.len())
    }

    pub fn passed_packets(&self) -> std::collections::HashSet<String> {
        self.entries
            .iter()
            .filter(|r| r.outcome == "PASS")
            .map(|r| r.packet_id.clone())
            .collect()
    }

    pub fn total_energy(&self) -> f64 {
        self.entries.iter().map(|r| r.energy_joules).sum()
    }

    pub fn resume_skips(&self, packet_id: &str) -> bool {
        self.entries
            .iter()
            .any(|r| r.packet_id == packet_id && r.outcome == "PASS")
    }
}

// ===========================================================================
// Energy (roadmap receipt fields — never a permit)
// ===========================================================================

pub const LANDAUER_J_PER_BIT: f64 = 2.87e-21;

#[derive(Clone, Debug)]
pub struct EnergyBudget {
    pub task_budget_j: f64,
    pub probe_budget_j: f64,
    pub spent_j: f64,
    pub landauer_bits_erased: u64,
}

impl EnergyBudget {
    pub fn new(task: f64) -> Self {
        Self {
            task_budget_j: task,
            probe_budget_j: task * 0.3,
            spent_j: 0.0,
            landauer_bits_erased: 0,
        }
    }
    pub fn remaining(&self) -> f64 {
        self.task_budget_j - self.spent_j
    }
    pub fn can_afford(&self, cost: f64) -> bool {
        self.spent_j + cost <= self.task_budget_j
    }
    pub fn charge(&mut self, cost: f64, erased_bits: u64) {
        self.spent_j += cost;
        self.landauer_bits_erased += erased_bits;
    }
}

// ===========================================================================
// Session log (append-only, hash-chained, replayable)
// ===========================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum SessionEventKind {
    ContextInject,
    ToolCall,
    ToolResult,
    JudgeVerdict,
    Checkpoint,
}

impl SessionEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ContextInject => "ContextInject",
            Self::ToolCall => "ToolCall",
            Self::ToolResult => "ToolResult",
            Self::JudgeVerdict => "JudgeVerdict",
            Self::Checkpoint => "Checkpoint",
        }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct SessionLog {
    pub events: Vec<SessionEvent>,
}

impl SessionLog {
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
        hex_encode(&h.finalize())
    }

    pub fn append(
        &mut self,
        session_id: &str,
        workflow_id: &str,
        kind: SessionEventKind,
        payload_json: String,
    ) -> &SessionEvent {
        let seq = self.events.len() as u64;
        let prev_hash = self
            .events
            .last()
            .map(|e| e.event_hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        let event_hash = Self::hash_of(
            seq,
            session_id,
            workflow_id,
            kind.as_str(),
            &payload_json,
            &prev_hash,
        );
        self.events.push(SessionEvent {
            seq,
            session_id: session_id.to_string(),
            workflow_id: workflow_id.to_string(),
            timestamp_ms: now_ms(),
            kind,
            payload_json,
            prev_hash,
            event_hash,
        });
        self.events.last().unwrap()
    }

    pub fn verify_chain(&self) -> Result<usize, usize> {
        let mut prev = "GENESIS".to_string();
        for (i, e) in self.events.iter().enumerate() {
            if e.seq != i as u64 || e.prev_hash != prev {
                return Err(i);
            }
            let recomputed = Self::hash_of(
                e.seq,
                &e.session_id,
                &e.workflow_id,
                e.kind.as_str(),
                &e.payload_json,
                &e.prev_hash,
            );
            if recomputed != e.event_hash {
                return Err(i);
            }
            prev = e.event_hash.clone();
        }
        Ok(self.events.len())
    }
}

// ===========================================================================
// Research gate
// ===========================================================================

#[derive(Clone, Debug)]
pub struct InformationNeed {
    pub packet_id: String,
    pub required_terms: Vec<String>,
    pub min_depth: u32,
}

#[derive(Clone, Debug)]
pub struct ResearchRecord {
    pub packet_id: String,
    pub covered_terms: Vec<String>,
    pub depth_reached: u32,
    pub rag_coverage: f64,
}

impl ResearchRecord {
    pub fn satisfies(&self, need: &InformationNeed, coverage_theta: f64) -> bool {
        self.packet_id == need.packet_id
            && self.depth_reached >= need.min_depth
            && self.rag_coverage >= coverage_theta
            && need
                .required_terms
                .iter()
                .all(|t| self.covered_terms.iter().any(|c| c == t))
    }
}

/// The gate: the session log must show planning (ContextInject) and a
/// research record satisfying the need. Refusals name the missing terms.
pub fn research_gate(
    log: &SessionLog,
    need: &InformationNeed,
    records: &[ResearchRecord],
    coverage_theta: f64,
) -> Result<(), Vec<String>> {
    let planned = log
        .events
        .iter()
        .any(|e| e.kind == SessionEventKind::ContextInject);
    if !planned {
        return Err(vec!["no planning phase recorded in session log".into()]);
    }
    let rec = records.iter().find(|r| r.packet_id == need.packet_id);
    match rec {
        None => Err(vec![format!(
            "no research record for packet {}",
            need.packet_id
        )]),
        Some(r) if !r.satisfies(need, coverage_theta) => {
            let missing: Vec<String> = need
                .required_terms
                .iter()
                .filter(|t| !r.covered_terms.iter().any(|c| c == *t))
                .cloned()
                .collect();
            let mut reasons = vec![format!(
                "coverage {:.2} < theta {:.2}",
                r.rag_coverage, coverage_theta
            )];
            if r.depth_reached < need.min_depth {
                reasons.push(format!(
                    "depth {} < required {}",
                    r.depth_reached, need.min_depth
                ));
            }
            reasons.extend(missing.into_iter().map(|t| format!("missing term: {}", t)));
            Err(reasons)
        }
        Some(_) => Ok(()),
    }
}

// ===========================================================================
// Working-set cache (prompt/context blocks under governance)
// ===========================================================================

pub const MASS_SCALE: i64 = 1_000_000;

#[derive(Clone, Debug)]
pub struct Block {
    pub key: String,
    pub parent: Option<String>,
    pub token: String,
    pub tokens: usize,
    pub mass: i64,
    pub anchored: bool,
    pub refcount: u32,
    pub locked: bool,
    pub plan_covered: bool,
    pub leak_rate_ppm: u32,
}

pub struct WorkingSetCache {
    pub model_id: String,
    pub tokenizer_id: String,
    pub quant_id: String,
    pub capacity_blocks: usize,
    pub anchor_budget: usize,
    pub anchored_count: usize,
    pub blocks: HashMap<String, Block>,
}

impl WorkingSetCache {
    pub fn new(model: &str, tok: &str, q: &str, cap: usize) -> Self {
        Self {
            model_id: model.into(),
            tokenizer_id: tok.into(),
            quant_id: q.into(),
            capacity_blocks: cap,
            anchor_budget: cap / 4,
            anchored_count: 0,
            blocks: HashMap::new(),
        }
    }

    pub fn key(&self, parent: Option<&str>, token: &str) -> String {
        let mut h = Sha256::new();
        if let Some(p) = parent {
            h.update(p.as_bytes());
        }
        h.update(token.as_bytes());
        h.update(self.model_id.as_bytes());
        h.update(self.tokenizer_id.as_bytes());
        h.update(self.quant_id.as_bytes());
        hex_encode(&h.finalize())
    }

    pub fn insert(
        &mut self,
        parent: Option<String>,
        token: String,
        tokens: usize,
        plan_covered: bool,
    ) -> String {
        let key = self.key(parent.as_deref(), &token);
        if self.blocks.contains_key(&key) {
            if let Some(b) = self.blocks.get_mut(&key) {
                b.refcount += 1;
            }
            return key;
        }
        while self.blocks.len() >= self.capacity_blocks {
            if !self.evict_one() {
                break;
            }
        }
        let leak = if plan_covered { 0 } else { 50_000 };
        let b = Block {
            key: key.clone(),
            parent,
            token,
            tokens,
            mass: MASS_SCALE,
            anchored: false,
            refcount: 1,
            locked: false,
            plan_covered,
            leak_rate_ppm: leak,
        };
        self.blocks.insert(key.clone(), b);
        key
    }

    pub fn decay_epoch(&mut self) {
        for b in self.blocks.values_mut() {
            if b.anchored {
                continue;
            }
            let leak = b.leak_rate_ppm as i64;
            let new = b.mass - (b.mass * leak / 1_000_000);
            b.mass = new.max(0);
        }
    }

    pub fn evict_one(&mut self) -> bool {
        let mut candidate: Option<(String, i64)> = None;
        for (k, b) in &self.blocks {
            if b.refcount == 0 && !b.anchored && !b.locked {
                let has_child = self.blocks.values().any(|x| x.parent.as_deref() == Some(k));
                if has_child {
                    continue;
                }
                match candidate {
                    None => candidate = Some((k.clone(), b.mass)),
                    Some((_, m)) if b.mass < m => candidate = Some((k.clone(), b.mass)),
                    _ => {}
                }
            }
        }
        if let Some((k, _)) = candidate {
            self.blocks.remove(&k);
            return true;
        }
        false
    }

    pub fn resident_mass_total(&self) -> i64 {
        self.blocks.values().map(|b| b.mass).sum()
    }
}

// ===========================================================================
// Mission entry points
// ===========================================================================

#[derive(Debug, Clone)]
pub struct MissionInput {
    pub plan: Plan,
    pub judges: Vec<Judge>,
}

#[derive(Debug, Clone)]
pub struct MissionReceipt {
    pub mission_id: String,
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub skipped: Vec<String>,
    pub escalated: Vec<String>,
    pub receipts: ReceiptLog,
    pub energy_spent_j: f64,
    pub energy_remaining_j: f64,
}

impl MissionReceipt {
    pub fn passed_all(&self) -> bool {
        self.failed.is_empty() && self.escalated.is_empty() && !self.passed.is_empty()
    }
}

/// Validate only. No run, no side effect. The go / no-go check.
pub fn dry_run(plan: &Plan) -> Result<(), String> {
    plan.validate()
}

/// Built-in geometry readiness mission: two packets, in order, with the
/// declared judge as the acceptance contract. Without a live executor this
/// escalates rather than manufacturing a PASS (proven in tests).
pub fn geometry_mission() -> MissionInput {
    let packet = |id: &str, task: &str, depends_on: Vec<String>| Packet {
        id: id.into(),
        task: task.into(),
        depends_on,
        declared_context_tokens: 100,
        tool_calls: vec![ToolCall {
            tool: "judge".into(),
            args_json: String::new(),
            declared: true,
        }],
        judges: vec![JudgeRef {
            id: "judge-pass".into(),
            kind: "contains".into(),
        }],
        branch: None,
        retry_policy: RetryPolicy {
            max_rounds: 2,
            material_change_required: true,
        },
        status: PacketStatus::Pending,
        attempts: 0,
        judge_joined: false,
    };
    MissionInput {
        plan: Plan {
            id: "mission-geometry-2026-10-29".into(),
            objective: "geometry training readiness: validate packet, judge, receipt chain".into(),
            packets: vec![
                packet(
                    "packet-geometry-01",
                    "validate geometry packet structure",
                    vec![],
                ),
                packet(
                    "packet-geometry-02",
                    "validate geometry packet ordering after packet-geometry-01",
                    vec!["packet-geometry-01".to_string()],
                ),
            ],
        },
        judges: vec![Judge {
            id: "judge-pass".into(),
            kind: JudgeKind::Contains {
                needle: "PASS".into(),
            },
        }],
    }
}
