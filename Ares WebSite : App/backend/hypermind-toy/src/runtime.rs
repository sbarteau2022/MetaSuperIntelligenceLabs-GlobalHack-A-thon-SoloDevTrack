//! Engine: CapabilityGate, Executor boundary, sequential + live-sequential
//! execution, mission entry points. Mirrors backend `harness::runtime`
//! semantics: same gate rule (judged atom AND receipt), same 64-hex
//! receipt-digest check, same energy charge
//! (dt_ns·1e-9 + declared_tokens·0.0005), same retry/escalate/skip shape,
//! same fail-closed honesty floor (no executor → ESCALATE, never PASS).

use crate::mind::Mind;
use crate::mission::{
    chain_hash, EnergyBudget, Judge, JudgeOutcome, MissionInput, MissionReceipt, Packet,
    PacketStatus, Receipt, ReceiptLog,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

// ===========================================================================
// Capability gate (the DAG capability tree as an admission check)
// ===========================================================================

pub const KNOWN_FAMILIES: [&str; 7] =
    ["map", "memory", "run", "judge", "receipt", "atom", "packet"];

#[derive(Debug, Clone)]
pub struct CapabilityNode {
    pub name: String,
    pub family: String,
    pub judged_atom: bool,
    pub receipt_present: bool,
}

/// Admission gate over the capability tree. A tool is admitted only if its
/// capability node exists AND carries a judged atom AND a receipt.
#[derive(Debug, Clone, Default)]
pub struct CapabilityGate {
    pub nodes: HashMap<String, CapabilityNode>,
}

impl CapabilityGate {
    pub fn register(&mut self, node: CapabilityNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    pub fn admit(&self, tool: &str) -> Result<&CapabilityNode, String> {
        // `map.kv_fetch` → family `map`; bare `map` → family `map`.
        let family = tool.split('.').next().unwrap_or(tool);
        if !KNOWN_FAMILIES.contains(&family) {
            return Err(format!("unknown capability family for tool {}", tool));
        }
        match self.nodes.get(tool).or_else(|| self.nodes.get(family)) {
            Some(n) if n.judged_atom && n.receipt_present => Ok(n),
            Some(n) => Err(format!(
                "capability {} lacks proof (judged_atom={}, receipt={})",
                tool, n.judged_atom, n.receipt_present
            )),
            None => Err(format!("capability {} not in the tree", tool)),
        }
    }

    pub fn deny_unknown(&self, tool: &str) -> String {
        match self.admit(tool) {
            Ok(_) => format!("capability-admitted:{}", tool),
            Err(e) => format!("capability-denied:{} ({})", tool, e),
        }
    }
}

// ===========================================================================
// Executor boundary
// ===========================================================================

/// Outcome of one executor tool call. `receipt_sha256` must be the exact
/// 64-hex digest bound by the serving system — never a marker.
#[derive(Debug, Clone)]
pub struct CallOutcome {
    pub inner_text: String,
    pub receipt_sha256: String,
    pub dt_ns: u64,
}

/// The execution boundary. The toy's executor is a scripted stand-in:
/// pre-registered (tool → text) answers with real 64-hex digests, so the
/// rehearsal exercises PASS paths without a model.
pub trait Executor: Send {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String>;
}

/// Scripted executor: deterministic answers + sha256(tool|args) digests.
/// Transport errors can be armed per tool to rehearse the retry path.
pub struct ScriptExecutor {
    pub answers: HashMap<String, String>,
    pub fail_once: HashMap<String, bool>,
    pub calls: Vec<(String, String)>,
}

impl ScriptExecutor {
    pub fn new() -> Self {
        Self {
            answers: HashMap::new(),
            fail_once: HashMap::new(),
            calls: Vec::new(),
        }
    }

    pub fn answer(mut self, tool: &str, text: &str) -> Self {
        self.answers.insert(tool.into(), text.into());
        self
    }

    /// Arm a one-shot transport error for `tool` (first call fails).
    pub fn fail_once(mut self, tool: &str) -> Self {
        self.fail_once.insert(tool.into(), true);
        self
    }

    fn digest(tool: &str, args: &str) -> String {
        let mut h = Sha256::new();
        h.update(tool.as_bytes());
        h.update(b"|");
        h.update(args.as_bytes());
        let mut s = String::with_capacity(64);
        for b in h.finalize() {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for ScriptExecutor {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        self.calls.push((tool.into(), args_json.into()));
        if self.fail_once.remove(tool).unwrap_or(false) {
            return Err(format!("scripted transport error for {}", tool));
        }
        match self.answers.get(tool) {
            Some(text) => Ok(CallOutcome {
                inner_text: text.clone(),
                receipt_sha256: Self::digest(tool, args_json),
                dt_ns: 1_000_000,
            }),
            None => Err(format!("no scripted answer for {}", tool)),
        }
    }
}

// ===========================================================================
// Engine
// ===========================================================================

#[derive(Debug, Clone, Default)]
pub struct EngineReport {
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub skipped: Vec<String>,
    pub escalated: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PacketOutcome {
    pub output: String,
    pub all_pass: bool,
    pub judge_ids: Vec<String>,
    pub energy_j: f64,
    pub erased_bits: u64,
    pub n3_receipts: Vec<String>,
    pub transport_error: Option<String>,
    pub degraded: Option<String>,
}

impl Default for PacketOutcome {
    fn default() -> Self {
        Self {
            output: String::new(),
            all_pass: false,
            judge_ids: Vec::new(),
            energy_j: 0.0,
            erased_bits: 0,
            n3_receipts: Vec::new(),
            transport_error: None,
            degraded: None,
        }
    }
}

pub struct Engine {
    pub judges: HashMap<String, Judge>,
    pub max_parallel: usize,
    /// Toy rehearsal lane: when set, ready packets may run concurrently
    /// under a Tokio semaphore in `run_parallel_live`. The sequential paths
    /// never read it (same as backend: field present, sequential ignores).
    pub parallel_slots: Option<usize>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            judges: HashMap::new(),
            max_parallel: 4,
            parallel_slots: None,
        }
    }

    pub fn register_judge(&mut self, j: Judge) {
        self.judges.insert(j.id.clone(), j);
    }

    /// Judge one packet's output against its registered judges.
    /// Pure: no IO, no budget, no receipts.
    pub fn judge_output(
        &self,
        packet: &Packet,
        output: &str,
        exit_code: i32,
    ) -> (bool, Vec<String>) {
        let mut all_pass = true;
        let mut judge_ids = Vec::new();
        for jr in &packet.judges {
            judge_ids.push(jr.id.clone());
            if let Some(j) = self.judges.get(&jr.id) {
                let r = j.run(output, exit_code);
                if !matches!(r.outcome, JudgeOutcome::Pass) {
                    all_pass = false;
                }
            } else {
                all_pass = false;
            }
        }
        (all_pass, judge_ids)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_packet_live(
        &self,
        packet: &Packet,
        executor: &mut dyn Executor,
        gate: &CapabilityGate,
        attempt: u32,
        ledger: &mut Vec<String>,
    ) -> PacketOutcome {
        let mut outputs: Vec<String> = Vec::new();
        let mut all_pass = true;
        let mut judge_ids: Vec<String> = Vec::new();
        let mut energy = 0.0f64;
        let mut erased = 0u64;
        let mut n3_receipts: Vec<String> = Vec::new();
        let mut degraded: Option<String> = None;
        let mut transport_error: Option<String> = None;

        for (call_idx, tc) in packet.tool_calls.iter().enumerate() {
            if !tc.tool.starts_with("map.") {
                continue; // non-map tools keep the caller's path
            }
            let call_key = format!("{}:{}:{}#{}", packet.id, attempt, tc.tool, call_idx);
            if gate.admit(&tc.tool).is_err() {
                let denial = gate.deny_unknown(&tc.tool);
                outputs.push(denial);
                all_pass = false;
                degraded = Some(format!("capability-denied:{}", tc.tool));
                ledger.push(format!("{}:gate denied", call_key));
                continue;
            }
            let args_ok = serde_json::from_str::<serde_json::Value>(&tc.args_json)
                .map(|v| v.is_object())
                .unwrap_or(false);
            if !args_ok {
                outputs.push(format!(
                    "args malformed or not an object: packet={} tool={}",
                    packet.id, tc.tool
                ));
                all_pass = false;
                degraded = Some(format!("args-malformed:{}", tc.tool));
                continue;
            }
            match executor.call(&tc.tool, &tc.args_json) {
                Ok(out) => {
                    if out.receipt_sha256.len() != 64
                        || !out.receipt_sha256.bytes().all(|b| b.is_ascii_hexdigit())
                    {
                        all_pass = false;
                        degraded = Some(format!("bad-receipt-digest:{}", tc.tool));
                        outputs.push(format!(
                            "executor returned a non-digest receipt for {}",
                            tc.tool
                        ));
                        continue;
                    }
                    let cost =
                        out.dt_ns as f64 * 1e-9 + (packet.declared_context_tokens as f64) * 0.0005;
                    energy += cost;
                    erased += (packet.declared_context_tokens as u64) * 4;
                    n3_receipts.push(out.receipt_sha256.clone());
                    ledger.push(format!("{}:call ok", call_key));
                    outputs.push(out.inner_text.clone());
                    if !packet.judge_joined {
                        let (pass, mut ids) = self.judge_output(packet, &out.inner_text, 0);
                        judge_ids.append(&mut ids);
                        if !pass {
                            all_pass = false;
                            degraded = Some(format!("judge-fail:{}", tc.tool));
                        }
                    }
                }
                Err(e) => {
                    transport_error = Some(e.clone());
                    all_pass = false;
                    degraded = Some(format!("transport-error:{}", tc.tool));
                    ledger.push(format!("{}:transport {}", call_key, e));
                    break;
                }
            }
        }

        let mut out_text = outputs.join("\n");
        let had_transport = transport_error.is_some();
        if let Some(e) = transport_error.as_deref() {
            out_text.push_str(&format!("\ntransport error: {}", e));
        }
        if outputs.is_empty() {
            out_text = format!(
                "execution unavailable: packet={} attempt={} has no admitted map.* output",
                packet.id, attempt
            );
            all_pass = false;
            n3_receipts.clear();
            degraded = Some("not-invoked".into());
        }
        if packet.judge_joined && !had_transport && !outputs.is_empty() {
            let (pass, ids) = self.judge_output(packet, &out_text, 0);
            judge_ids = ids;
            if !pass {
                all_pass = false;
                degraded = Some("judge-fail:joined".into());
            }
        }
        if n3_receipts.is_empty()
            && !outputs.is_empty()
            && degraded.is_none()
            && outputs.iter().any(|o| !o.starts_with("capability-denied"))
        {
            all_pass = false;
            degraded = Some("missing-receipt".into());
        }

        PacketOutcome {
            output: out_text,
            all_pass,
            judge_ids,
            energy_j: energy,
            erased_bits: erased,
            n3_receipts,
            transport_error,
            degraded,
        }
    }

    /// Fail-closed local orchestrator. No executor is attached, so this
    /// path can never emit PASS.
    pub fn run_sequential(
        &self,
        plan: &mut crate::mission::Plan,
        receipts: &mut ReceiptLog,
        _budget: &mut EnergyBudget,
    ) -> EngineReport {
        let mut report = EngineReport::default();
        loop {
            let ready: Vec<String> = plan.ready_packets();
            if ready.is_empty() {
                break;
            }
            let next = ready[0].clone();
            let pkt_idx = plan.packets.iter().position(|p| p.id == next).unwrap();
            plan.packets[pkt_idx].status = PacketStatus::Running;
            plan.packets[pkt_idx].attempts += 1;
            let attempt = plan.packets[pkt_idx].attempts;
            let judge_ids = plan.packets[pkt_idx]
                .judges
                .iter()
                .map(|judge| judge.id.clone())
                .collect::<Vec<_>>();

            plan.packets[pkt_idx].status = PacketStatus::Escalated;
            receipts.append(Receipt {
                packet_id: next.clone(),
                attempt,
                outcome: "ESCALATE".into(),
                judge_ids,
                evidence_hash: String::new(),
                energy_joules: 0.0,
                erased_bits: 0,
                timestamp_ms: 0,
                prev_hash: String::new(),
                n3_receipts: vec![],
            });
            report.escalated.push(next.clone());

            let pkt_id = plan.packets[pkt_idx].id.clone();
            let deps: Vec<String> = plan
                .packets
                .iter()
                .filter(|p| p.depends_on.contains(&pkt_id))
                .map(|p| p.id.clone())
                .collect();
            for d in deps {
                if let Some(pk) = plan.packet_mut(&d) {
                    if pk.status == PacketStatus::Pending {
                        pk.status = PacketStatus::Skipped;
                    }
                }
                if !report.skipped.contains(&d) {
                    report.skipped.push(d);
                }
            }
            if report.passed.len()
                + report.failed.len()
                + report.escalated.len()
                + report.skipped.len()
                >= plan.packets.len()
            {
                for p in &mut plan.packets {
                    if p.status == PacketStatus::Pending {
                        p.status = PacketStatus::Skipped;
                        report.skipped.push(p.id.clone());
                    }
                }
                break;
            }
        }
        report
    }

    /// Live sequential executor: same topology/retry/escalation semantics,
    /// packet tool calls executed through `executor` under the capability
    /// gate and judged from real returned output.
    pub fn run_sequential_live(
        &self,
        plan: &mut crate::mission::Plan,
        receipts: &mut ReceiptLog,
        budget: &mut EnergyBudget,
        executor: &mut dyn Executor,
        gate: &CapabilityGate,
        ledger: &mut Vec<String>,
    ) -> EngineReport {
        let mut report = EngineReport::default();
        loop {
            let ready: Vec<String> = plan.ready_packets();
            if ready.is_empty() {
                break;
            }
            let next = ready[0].clone();
            let pkt_idx = plan.packets.iter().position(|p| p.id == next).unwrap();
            plan.packets[pkt_idx].status = PacketStatus::Running;
            plan.packets[pkt_idx].attempts += 1;
            let attempt = plan.packets[pkt_idx].attempts;
            let snapshot = plan.packets[pkt_idx].clone();

            let mut out = self.execute_packet_live(&snapshot, executor, gate, attempt, ledger);
            let transport = out.transport_error.is_some();
            if transport && attempt > 1 {
                out.all_pass = false;
            }
            if budget.can_afford(out.energy_j) {
                budget.charge(out.energy_j, out.erased_bits);
            } else {
                out.all_pass = false;
            }

            if out.all_pass {
                plan.packets[pkt_idx].status = PacketStatus::Passed;
                receipts.append(Receipt {
                    packet_id: next.clone(),
                    attempt,
                    outcome: "PASS".into(),
                    judge_ids: out.judge_ids,
                    evidence_hash: String::new(),
                    energy_joules: out.energy_j,
                    erased_bits: out.erased_bits,
                    timestamp_ms: 0,
                    prev_hash: String::new(),
                    n3_receipts: out.n3_receipts,
                });
                report.passed.push(next);
            } else if attempt < snapshot.retry_policy.max_rounds
                && snapshot.retry_policy.material_change_required
                && !transport
            {
                plan.packets[pkt_idx].status = PacketStatus::Pending;
            } else {
                plan.packets[pkt_idx].status = PacketStatus::Escalated;
                receipts.append(Receipt {
                    packet_id: next.clone(),
                    attempt,
                    outcome: "ESCALATE".into(),
                    judge_ids: out.judge_ids,
                    evidence_hash: String::new(),
                    energy_joules: 0.0,
                    erased_bits: 0,
                    timestamp_ms: 0,
                    prev_hash: String::new(),
                    n3_receipts: out.n3_receipts,
                });
                report.escalated.push(next.clone());
                let deps: Vec<String> = plan
                    .packets
                    .iter()
                    .filter(|p| p.depends_on.contains(&snapshot.id))
                    .map(|p| p.id.clone())
                    .collect();
                for d in deps {
                    if let Some(pk) = plan.packet_mut(&d) {
                        pk.status = PacketStatus::Skipped;
                    }
                    report.skipped.push(d);
                }
            }
            if report.passed.len()
                + report.failed.len()
                + report.escalated.len()
                + report.skipped.len()
                >= plan.packets.len()
            {
                for p in &mut plan.packets {
                    if p.status == PacketStatus::Pending {
                        p.status = PacketStatus::Skipped;
                        report.skipped.push(p.id.clone());
                    }
                }
                break;
            }
        }
        for p in &plan.packets {
            match p.status {
                PacketStatus::Failed => {
                    if !report.failed.contains(&p.id) {
                        report.failed.push(p.id.clone());
                    }
                }
                PacketStatus::Skipped if !report.skipped.contains(&p.id) => {
                    report.skipped.push(p.id.clone());
                }
                _ => {}
            }
        }
        report
    }
}

/// Run a mission fail-closed locally (no executor): every packet
/// escalates, receipts are written, nothing passes.
pub fn run_mission_local(input: MissionInput, budget_j: f64) -> MissionReceipt {
    run_mission_with_prior_local(input, budget_j, None)
}

/// Resume: packets already PASS in a **chain-verified** prior log are
/// skipped, never re-run. An unverified (forged) prior skips nothing.
pub fn run_mission_with_prior_local(
    mut input: MissionInput,
    budget_j: f64,
    prior: Option<ReceiptLog>,
) -> MissionReceipt {
    let mut receipts = ReceiptLog::default();
    let mut prior_passed: Vec<String> = Vec::new();
    if let Some(prior) = &prior {
        let verified = prior.verify_chain().is_ok() && !prior.entries.is_empty();
        if verified {
            for packet in input.plan.packets.iter_mut() {
                if prior.resume_skips(&packet.id) {
                    packet.status = PacketStatus::Passed;
                    prior_passed.push(packet.id.clone());
                }
            }
            for r in prior.entries.iter().cloned() {
                receipts.entries.push(r);
            }
        }
    }
    let mut engine = Engine::new();
    for judge in input.judges {
        engine.register_judge(judge);
    }
    let mut budget = EnergyBudget::new(budget_j);
    let mut report = engine.run_sequential(&mut input.plan, &mut receipts, &mut budget);
    for id in prior_passed {
        if !report.passed.contains(&id) {
            report.passed.push(id);
        }
    }
    MissionReceipt {
        mission_id: input.plan.id.clone(),
        passed: report.passed,
        failed: report.failed,
        skipped: report.skipped,
        escalated: report.escalated,
        receipts,
        energy_spent_j: budget.spent_j,
        energy_remaining_j: budget.remaining(),
    }
}

/// One externally produced receipt row (simulating backend store rows).
/// Tuple order: packet, prev, attempt, outcome, judges, n3, energy, erased,
/// ts, evidence.
#[allow(clippy::type_complexity)]
pub type ExternalRow = (
    String,
    String,
    u32,
    String,
    Vec<String>,
    Vec<String>,
    f64,
    u64,
    u64,
    String,
);

/// Verify an external receipt chain with the toy's own recomputation
/// (mirrors the backend's Python-cross-check pattern: independent
/// arithmetic, never the producer's word for it).
pub fn verify_external_chain(entries: &[ExternalRow]) -> Result<usize, usize> {
    // (packet_id, prev_hash, attempt, outcome, judge_ids, n3, energy, erased, ts, evidence)
    let mut prev = "GENESIS".to_string();
    for (seq, e) in entries.iter().enumerate() {
        if e.1 != prev {
            return Err(seq);
        }
        if e.8 == 0 {
            return Err(seq);
        }
        let recomputed = chain_hash(&e.1, &e.0, e.2, &e.3, &e.4, &e.5, e.6, e.7, e.8);
        if recomputed != e.9 {
            return Err(seq);
        }
        prev = e.9.clone();
    }
    Ok(entries.len())
}

/// Cross-check helper: recompute a sha256 hex digest (independent path for tests).
pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let mut out = String::with_capacity(64);
    for b in h.finalize() {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// Resident mind handle for rehearsals (so tests read what they wrote).
pub struct ToyMind {
    pub mind: Mind,
}

impl ToyMind {
    pub fn new() -> Self {
        Self { mind: Mind::new() }
    }
}

impl Default for ToyMind {
    fn default() -> Self {
        Self::new()
    }
}
