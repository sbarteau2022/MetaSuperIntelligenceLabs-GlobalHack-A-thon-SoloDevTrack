//! Harness runtime: capability gate, executor boundary, and the engine.
//!
//! The engine runs the typed workflow (packet → lane execution → judge →
//! receipt → atom candidate). Telemetry is exported to HyperDX (OLTP
//! logs/traces) and memory to Qdrant; the engine itself never owns a
//! local telemetry database. Two execution paths, doctrinally separated:
//!
//! - `run_sequential` / `run_parallel`: **fail-closed local orchestrators**.
//!   No executor is attached, so no packet can ever PASS. They exist so
//!   validation callers receive explicit ESCALATE/FAIL receipts instead of
//!   synthetic success evidence.
//! - `run_sequential_live`: executes `map.*` tool calls through an
//!   `Executor` (the boundary the backend wires: governed model seats and
//!   the mind-map MCP child), judges the returned output, binds the
//!   executor's exact 64-hex receipt digests into the receipt chain, and
//!   charges the energy budget. Transport errors retry at most once;
//!   identical retry is forbidden by the failure doctrine.
//!
//! Governance: callers must hold a compliance permit before invoking a
//! live run (see `governor::dispatch_mission`). The engine itself checks
//! the capability gate for every `map.*` call — a capability with no
//! judged atom behind it is denied, never executed.

use crate::harness::mission::{
    EnergyBudget, Judge, JudgeOutcome, MissionReceipt, Packet, PacketStatus, Plan, Receipt,
    ReceiptLog,
};
use crate::harness::telemetry::HyperdxTelemetry;
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
/// capability node exists AND carries a judged atom AND a receipt — the
/// workflow document's rule: no judged atom, no receipt, no run in
/// telemetry → the capability does not appear, and it does not execute.
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

/// The execution boundary. Implementations: the native MTP automaton seat
/// (model-agnostic) and the mind-map MCP child. The engine never
/// talks to a provider directly.
pub trait Executor: Send {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String>;
}

/// Typed result of one live packet execution.
#[derive(Debug, Clone, Default)]
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

pub struct Engine {
    pub judges: HashMap<String, Judge>,
    pub max_parallel: usize,
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

    async fn execute_packet_live(
        &self,
        packet: &Packet,
        executor: &mut dyn Executor,
        telemetry: &HyperdxTelemetry,
        gate: &CapabilityGate,
        attempt: u32,
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
                let _ = telemetry
                    .record_call(
                        &format!("{}:gate", call_key),
                        &tc.tool,
                        &tc.args_json,
                        0,
                        false,
                        None,
                    )
                    .await;
                continue;
            }
            // Args must parse as a JSON object before any wire contact.
            // Malformed args are a deterministic packet defect.
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
                    // Receipt digest must be exact 64-hex, never a marker.
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
                    if telemetry
                        .record_call(
                            &call_key,
                            &tc.tool,
                            &tc.args_json,
                            out.dt_ns,
                            true,
                            Some(&out.receipt_sha256),
                        )
                        .await
                        .is_err()
                    {
                        all_pass = false;
                        degraded = Some(format!("telemetry-write-failed:{}", tc.tool));
                    }
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
                    let _ = telemetry
                        .record_call(
                            &format!("{}:transport", call_key),
                            &tc.tool,
                            &tc.args_json,
                            0,
                            false,
                            None,
                        )
                        .await;
                    break;
                }
            }
        }

        let mut out_text = outputs.join("\n");
        let had_transport = transport_error.is_some();
        if let Some(e) = transport_error.as_deref() {
            out_text.push_str(&format!("\ntransport error: {}", e));
        }
        // A packet with no admitted execution has no evidence. Fail closed:
        // manufacturing a PASS marker here would let a judge certify work
        // that never ran.
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
        if n3_receipts.is_empty() && outputs.iter().any(|o| !o.starts_with("capability-denied")) {
            // Executed but produced no bound digest: not PASS evidence.
            if !outputs.is_empty() && degraded.is_none() {
                all_pass = false;
                degraded = Some("missing-receipt".into());
            }
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
    /// path can never emit PASS. Validation callers receive explicit
    /// ESCALATE receipts instead of synthetic success.
    pub fn run_sequential(
        &self,
        plan: &mut Plan,
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
    /// with packet tool calls executed through `executor` under the
    /// capability gate and judged from real returned output.
    #[allow(clippy::too_many_arguments)]
    pub async fn run_sequential_live(
        &self,
        plan: &mut Plan,
        receipts: &mut ReceiptLog,
        budget: &mut EnergyBudget,
        executor: &mut dyn Executor,
        telemetry: &HyperdxTelemetry,
        gate: &CapabilityGate,
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

            let mut out = self
                .execute_packet_live(&snapshot, executor, telemetry, gate, attempt)
                .await;
            let transport = out.transport_error.is_some();
            if transport && attempt > 1 {
                out.all_pass = false;
            }
            if budget.can_afford(out.energy_j) {
                budget.charge(out.energy_j, out.erased_bits);
            } else {
                out.all_pass = false;
            }
            if telemetry
                .record_verdict(
                    &format!("{}:{}", snapshot.id, attempt),
                    &snapshot.id,
                    if out.all_pass { "PASS" } else { "FAIL" },
                    &out.judge_ids.join(","),
                )
                .await
                .is_err()
            {
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
                    energy_joules: out.energy_j,
                    erased_bits: out.erased_bits,
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
/// escalates, receipts are written, nothing passes. This is the harness's
/// honesty floor — a mission that did not execute is not a run.
pub fn run_mission_local(
    input: crate::harness::mission::MissionInput,
    budget_j: f64,
) -> MissionReceipt {
    run_mission_with_prior_local(input, budget_j, None)
}

/// Resume: packets already PASS in a **chain-verified** prior log are
/// skipped, never re-run. An unverified (forged) prior skips nothing.
pub fn run_mission_with_prior_local(
    mut input: crate::harness::mission::MissionInput,
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
