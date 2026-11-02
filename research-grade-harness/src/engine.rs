use crate::capability::CapabilityGate;
use crate::energy::EnergyBudget;
use crate::json::{self, Json};
use crate::judge::Judge;
use crate::mcp_client::McpChild;
use crate::receipt::{Receipt, ReceiptLog};
use crate::runbook::{Packet, PacketStatus, Plan};
use crate::sqlite_sink::TelemetrySink;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct EngineReport {
    pub passed: Vec<String>,
    pub failed: Vec<String>,
    pub skipped: Vec<String>,
    pub escalated: Vec<String>,
}

/// Typed result of one live packet execution (P0-2/P0-5 closure).
///
/// `n3_receipts` carries the EXACT `receipt_sha256` digests bound by the
/// server — never `"present"` markers or descriptive prose. The mint
/// (`ReceiptLog::append`) chains these into `evidence_hash`.
#[derive(Debug, Clone, Default)]
pub struct PacketOutcome {
    pub output: String,
    pub all_pass: bool,
    pub judge_ids: Vec<String>,
    pub energy_j: f64,
    pub erased_bits: u64,
    pub n3_receipts: Vec<String>,
    /// `Some` when the child transport broke (respawn-and-retry signal).
    /// Judge failures leave this `None` (escalate, never identical-retry).
    pub transport_error: Option<String>,
    pub degraded: Option<String>,
}

pub struct Engine {
    pub judges: HashMap<String, Judge>,
    pub max_parallel: usize,
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

    /// Judge one packet's output text against its registered judges.
    /// Pure: no IO, no budget, no receipts. Shared by fail-closed local
    /// orchestration and the live N3 path so both judge the same way.
    /// Returns (all_pass, judge_ids_seen).
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
                if !matches!(r.outcome, crate::judge::JudgeOutcome::Pass) {
                    all_pass = false;
                }
            } else {
                all_pass = false;
            }
        }
        (all_pass, judge_ids)
    }

    /// Execute one packet through the live N3 child.
    /// For each `map.*` tool_call: capability-gate admit, then mcp.call,
    /// then judge the RETURNED inner text. Returns [`PacketOutcome`] (typed:
    /// output, judges, energy, exact bound N3 digests, transport flag,
    /// degraded note). Transport errors fail the attempt; the caller decides
    /// retry/escalate. Identical retry is forbidden by the failure doctrine:
    /// the tool set is fixed, so a transport error retries at most once
    /// (with a respawned child — fresh execution conditions), then escalates.
    #[allow(clippy::too_many_arguments)]
    pub fn execute_packet_n3(
        &self,
        packet: &Packet,
        mcp: &mut McpChild,
        sink: &mut TelemetrySink,
        gate: &CapabilityGate,
        swarm_run_id: &str,
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
                continue; // non-map tools keep the legacy path (caller appends it)
            }
            // GAP-1 (C-RUN-04 follow-up): idempotency keys carry the per-packet
            // call index (packet:attempt:tool#index) so multi-same-tool packets
            // record every call as its own row. Previously the seed's two
            // ingests collapsed to one row on the (packet,attempt,tool) key.
            let call_key = format!("{}:{}:{}#{}", packet.id, attempt, tc.tool, call_idx);
            if gate.admit(&tc.tool).is_err() {
                let denial = gate.dag.deny_unknown(&tc.tool);
                outputs.push(denial.clone());
                all_pass = false;
                degraded = Some(format!("capability-denied:{}", tc.tool));
                let _ = sink.record_call(
                    &format!("{}:gate", call_key),
                    &tc.tool,
                    &tc.args_json,
                    0,
                    false,
                    None,
                );
                continue;
            }
            // Strict boundary: packet args parse to typed JSON before any
            // wire contact. Malformed args are a deterministic packet defect
            // (judge-fail, no transport), never verbatim-spliced text.
            let args: Json = match json::parse(&tc.args_json) {
                Ok(Json::Obj(_)) => json::parse(&tc.args_json).unwrap_or(Json::Null),
                Ok(_) => {
                    outputs.push(format!(
                        "args not an object: packet={} tool={}",
                        packet.id, tc.tool
                    ));
                    all_pass = false;
                    degraded = Some(format!("args-not-object:{}", tc.tool));
                    continue;
                }
                Err(e) => {
                    outputs.push(format!(
                        "args malformed: packet={} tool={} {}",
                        packet.id, tc.tool, e
                    ));
                    all_pass = false;
                    degraded = Some(format!("args-malformed:{}", tc.tool));
                    continue;
                }
            };
            match mcp.call(&tc.tool, &args) {
                Ok(out) => {
                    let inner = json::write(&out.inner);
                    // Measured energy: wall time as J surrogate + declared tokens.
                    let cost =
                        out.dt_ns as f64 * 1e-9 + (packet.declared_context_tokens as f64) * 0.0005;
                    energy += cost;
                    erased += (packet.declared_context_tokens as u64) * 4;
                    n3_receipts.push(out.receipt_sha256.clone());
                    let recorded = sink.record_call(
                        &call_key,
                        &tc.tool,
                        &tc.args_json,
                        out.dt_ns,
                        true,
                        n3_receipts.last().map(|s| s.as_str()),
                    );
                    if recorded.is_err() {
                        all_pass = false;
                        degraded = Some(format!("telemetry-write-failed:{}", tc.tool));
                    }
                    outputs.push(inner.clone());
                    // GAP-2: per-call conjunctive judging is the DEFAULT
                    // semantic (judge_joined=false). judge_joined=true packets
                    // skip per-call judging; the joined output is judged once
                    // after the loop (each call still records its sink row).
                    if !packet.judge_joined {
                        let (pass, mut ids) = self.judge_output(packet, &inner, 0);
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
                    let _ = sink.record_call(
                        &format!("{}:transport", call_key),
                        &tc.tool,
                        &tc.args_json,
                        0,
                        false,
                        None,
                    );
                    break;
                }
            }
        }
        let mut out = outputs.join("\n");
        // transport_error is consumed by reference here; the GAP-2 block below
        // re-checks it via as_deref (no move — the tuple return needs no ownership).
        let had_transport = transport_error.is_some();
        if let Some(e) = transport_error.as_deref() {
            out.push_str(&format!("\ntransport error: {}", e));
        }
        // A packet with no admitted map.* execution has no evidence. It must
        // fail closed; manufacturing a PASS marker here would let a judge
        // certify work that never ran.
        if outputs.is_empty() {
            out = format!(
                "execution unavailable: packet={} attempt={} has no admitted map.* output",
                packet.id, attempt
            );
            all_pass = false;
            n3_receipts.clear();
            degraded = Some("n3-not-invoked".into());
        }
        // GAP-2: joined-output judging (opt-in). The packet's judges run once
        // over the aggregate output — the semantic for multi-contract packets
        // (gate+collect in one) that per-call judging cannot express without
        // gutting a judge. transport_error short-circuits: a broken pipe is
        // not a judge verdict (caller escalates on transport separately).
        if packet.judge_joined && !had_transport && !outputs.is_empty() {
            let (pass, ids) = self.judge_output(packet, &out, 0);
            judge_ids = ids;
            if !pass {
                all_pass = false;
                degraded = Some("judge-fail:joined".into());
            }
        }
        if n3_receipts.is_empty() {
            all_pass = false;
            degraded = Some("missing-receipt".into());
        }
        let _ = swarm_run_id;
        PacketOutcome {
            output: out,
            all_pass,
            judge_ids,
            energy_j: energy,
            erased_bits: erased,
            n3_receipts,
            transport_error,
            degraded,
        }
    }

    /// Fail-closed local orchestrator. This path intentionally has no packet
    /// executor and can never emit PASS. Use `run_sequential_n3` for live
    /// execution; keeping this API allows validation callers to receive an
    /// explicit escalation receipt instead of synthetic success evidence.
    pub fn run_sequential(
        &self,
        plan: &mut Plan,
        receipts: &mut ReceiptLog,
        _budget: &mut EnergyBudget,
    ) -> EngineReport {
        let mut report = EngineReport::default();
        // topological execution
        loop {
            let ready: Vec<String> = plan.ready_packets();
            if ready.is_empty() {
                break;
            }
            let next = ready[0].clone();
            // execute packet
            let pkt_idx = plan.packets.iter().position(|p| p.id == next).unwrap();
            plan.packets[pkt_idx].status = PacketStatus::Running;
            plan.packets[pkt_idx].attempts += 1;

            // material change flag toggles on retry
            let attempt = plan.packets[pkt_idx].attempts;
            let judge_ids = plan.packets[pkt_idx]
                .judges
                .iter()
                .map(|judge| judge.id.clone())
                .collect::<Vec<_>>();
            let cost = 0.0;

            plan.packets[pkt_idx].status = PacketStatus::Running;
            receipts.append(Receipt {
                packet_id: next.clone(),
                attempt,
                outcome: "ESCALATE".into(),
                judge_ids,
                evidence_hash: String::new(),
                energy_joules: cost,
                erased_bits: 0,
                timestamp_ms: 0,
                prev_hash: String::new(),
                n3_receipts: vec![],
            });
            report.escalated.push(next);
            // skip dependents
            let deps: Vec<String> = plan
                .packets
                .iter()
                .filter(|p| p.depends_on.contains(&plan.packets[pkt_idx].id))
                .map(|p| p.id.clone())
                .collect();
            for d in deps {
                if let Some(pk) = plan.packet_mut(&d) {
                    pk.status = PacketStatus::Skipped;
                }
                report.skipped.push(d);
            }
            // safety: prevent infinite loops if no progress
            if report.passed.len()
                + report.failed.len()
                + report.escalated.len()
                + report.skipped.len()
                >= plan.packets.len()
            {
                // mark remaining pending as skipped
                for p in &mut plan.packets {
                    if p.status == PacketStatus::Pending {
                        p.status = PacketStatus::Skipped;
                        report.skipped.push(p.id.clone());
                    }
                }
                break;
            }
        }
        // any remaining
        for p in &plan.packets {
            match p.status {
                PacketStatus::Failed => report.failed.push(p.id.clone()),
                PacketStatus::Skipped if !report.skipped.contains(&p.id) => {
                    report.skipped.push(p.id.clone());
                }
                _ => {}
            }
        }
        report
    }

    /// Fail-closed parallel local orchestrator. No packet is certified without
    /// a live executor; use `run_parallel_n3` for execution-backed receipts.
    pub fn run_parallel(
        &self,
        plan: &mut Plan,
        receipts: &mut ReceiptLog,
        _budget: &mut EnergyBudget,
    ) -> EngineReport {
        let mut report = EngineReport::default();
        // group by topological level
        let mut remaining: HashSet<String> = plan.packets.iter().map(|p| p.id.clone()).collect();
        while !remaining.is_empty() {
            let ready: Vec<String> = plan
                .ready_packets()
                .into_iter()
                .filter(|id| remaining.contains(id))
                .collect();
            if ready.is_empty() {
                break;
            }
            // process batch (up to max_parallel)
            for chunk in ready.chunks(self.max_parallel) {
                for id in chunk {
                    // reuse sequential single-packet logic by recursive? inline duplicate
                    let pkt_idx = plan.packets.iter().position(|p| p.id == *id).unwrap();
                    plan.packets[pkt_idx].status = PacketStatus::Running;
                    plan.packets[pkt_idx].attempts += 1;
                    let attempt = plan.packets[pkt_idx].attempts;
                    let judge_ids = plan.packets[pkt_idx]
                        .judges
                        .iter()
                        .map(|judge| judge.id.clone())
                        .collect();
                    plan.packets[pkt_idx].status = PacketStatus::Failed;
                    receipts.append(Receipt {
                        packet_id: id.clone(),
                        attempt,
                        outcome: "FAIL".into(),
                        judge_ids,
                        evidence_hash: String::new(),
                        energy_joules: 0.0,
                        erased_bits: 0,
                        timestamp_ms: 0,
                        prev_hash: String::new(),
                        n3_receipts: vec![],
                    });
                    report.failed.push(id.clone());
                    remaining.remove(id);
                }
            }
        }
        report
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Sequential executor through the live N3 child (same topology / retry /
    /// escalation as run_sequential, but packet tool_calls run on N3).
    /// Transport errors retry at most once (attempts < max_rounds AND first
    /// failure) then escalate: the tool set is fixed, so a second identical
    /// attempt is a failure-doctrine violation.
    #[allow(clippy::too_many_arguments)]
    pub fn run_sequential_n3(
        &self,
        plan: &mut Plan,
        receipts: &mut ReceiptLog,
        budget: &mut EnergyBudget,
        mcp: &mut McpChild,
        sink: &mut TelemetrySink,
        gate: &CapabilityGate,
        swarm_run_id: &str,
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

            let mut out = self.execute_packet_n3(&snapshot, mcp, sink, gate, swarm_run_id, attempt);
            let transport = out.transport_error.is_some();
            // Identical-retry guard: transport errors get ONE retry max.
            if transport && attempt > 1 {
                out.all_pass = false;
            }
            if budget.can_afford(out.energy_j) {
                budget.charge(out.energy_j, out.erased_bits);
            } else {
                out.all_pass = false;
            }
            let verdict_recorded = sink.record_verdict(
                &format!("{}:{}", snapshot.id, attempt),
                &snapshot.id,
                if out.all_pass { "PASS" } else { "FAIL" },
                &out.judge_ids.join(","),
            );
            if verdict_recorded.is_err() {
                out.all_pass = false;
            }

            if out.all_pass {
                plan.packets[pkt_idx].status = PacketStatus::Passed;
                // Chain mint: exact N3 digests bound into the receipt;
                // append() stamps prev/timestamp and recomputes the hash.
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
                report.escalated.push(next);
                let deps: Vec<String> = plan
                    .packets
                    .iter()
                    .filter(|p| p.depends_on.contains(&plan.packets[pkt_idx].id))
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
                PacketStatus::Failed => report.failed.push(p.id.clone()),
                PacketStatus::Skipped if !report.skipped.contains(&p.id) => {
                    report.skipped.push(p.id.clone());
                }
                _ => {}
            }
        }
        report
    }

    /// Parallel executor through the live N3 child: topological levels,
    /// same per-packet N3 semantics as run_sequential_n3.
    #[allow(clippy::too_many_arguments)]
    pub fn run_parallel_n3(
        &self,
        plan: &mut Plan,
        receipts: &mut ReceiptLog,
        budget: &mut EnergyBudget,
        mcp: &mut McpChild,
        sink: &mut TelemetrySink,
        gate: &CapabilityGate,
        swarm_run_id: &str,
    ) -> EngineReport {
        let mut report = EngineReport::default();
        let mut remaining: HashSet<String> = plan.packets.iter().map(|p| p.id.clone()).collect();
        while !remaining.is_empty() {
            let ready: Vec<String> = plan
                .ready_packets()
                .into_iter()
                .filter(|id| remaining.contains(id))
                .collect();
            if ready.is_empty() {
                break;
            }
            for chunk in ready.chunks(self.max_parallel) {
                for id in chunk {
                    let pkt_idx = plan.packets.iter().position(|p| p.id == *id).unwrap();
                    plan.packets[pkt_idx].status = PacketStatus::Running;
                    plan.packets[pkt_idx].attempts += 1;
                    let attempt = plan.packets[pkt_idx].attempts;
                    let snapshot = plan.packets[pkt_idx].clone();
                    let mut out =
                        self.execute_packet_n3(&snapshot, mcp, sink, gate, swarm_run_id, attempt);
                    let transport = out.transport_error.is_some();
                    if transport && attempt > 1 {
                        out.all_pass = false;
                    }
                    if budget.can_afford(out.energy_j) {
                        budget.charge(out.energy_j, out.erased_bits);
                    } else {
                        out.all_pass = false;
                    }
                    let verdict_recorded = sink.record_verdict(
                        &format!("{}:{}", snapshot.id, attempt),
                        &snapshot.id,
                        if out.all_pass { "PASS" } else { "FAIL" },
                        &out.judge_ids.join(","),
                    );
                    if verdict_recorded.is_err() {
                        out.all_pass = false;
                    }
                    if out.all_pass {
                        plan.packets[pkt_idx].status = PacketStatus::Passed;
                        receipts.append(Receipt {
                            packet_id: id.clone(),
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
                        report.passed.push(id.clone());
                    } else {
                        plan.packets[pkt_idx].status = PacketStatus::Failed;
                        report.failed.push(id.clone());
                    }
                    remaining.remove(id);
                }
            }
        }
        report
    }
}
