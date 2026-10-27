# C27 — Workflows & Task Graph Orchestration

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Workflow packets, judges, retry/escalation, preregistration, autorunner admission.

## Subcategories (broad topics)

- C27.1 Packet/judge model and preregistered plans
- C27.2 depends_on, retry, escalation
- C27.3 Autorunner (N workers, ready-packet admission)
- C27.4 Parallel orchestration (agent/parallel/pipeline/phase)
- C27.5 Preregistration gate (409 without plan, 400 without problem)
- C27.6 Telemetry and receipts for workflow events

## Formal Workflow Model (Petri Net)

### Workflow as Petri Net

A **workflow** $W = (P, T, F, M_0)$ where:

- $P$ = Places (packet states: pending, running, done, failed, escalated)
- $T$ = Transitions (admit, execute, judge, retry, escalate)
- $F \subseteq (P \times T) \cup (T \times P)$ = Flow relation
- $M_0$ = Initial marking (single token in `pending`)

**Properties**:

- **Bounded**: Max tokens = `max_inflight`
- **Live**: No deadlocks (autorunner ensures progress)
- **Reversible**: Rollback via compensation transitions

### Packet as Colored Petri Net Token

```rust
struct Packet {
    id: PacketId,
    color: PacketColor,  // Determines routing
    marking: Marking,    // Current place
    data: PacketData,
    depends_on: Vec<PacketId>,
    retry_policy: RetryPolicy,
    escalation_policy: EscalationPolicy,
    judge: JudgeId,
    preregistered_plan: Option<Plan>,
}

enum PacketColor {
    Agent { lane: LaneId },
    Parallel { branches: Vec<PacketId> },
    Pipeline { stages: Vec<PacketId> },
    Phase { phase_id: u32 },
}
```

### Autorunner as Scheduler

```rust
struct Autorunner {
    workers: u32,
    ready_queue: PriorityQueue<PacketId, Priority>,
    running: HashMap<PacketId, WorkerHandle>,
    max_inflight: u32,
}

impl Autorunner {
    fn schedule(&mut self) {
        // Admit ready packets (all depends_on satisfied)
        let ready = self.find_ready_packets();

        for packet in ready {
            if self.running.len() >= self.max_inflight as usize {
                break;
            }

            let worker = self.spawn_worker(packet);
            self.running.insert(packet.id, worker);
        }
    }

    fn find_ready_packets(&self) -> Vec<PacketId> {
        self.packets.values()
            .filter(|p| p.marking == Marking::Pending)
            .filter(|p| p.depends_on.iter().all(|d| self.is_done(d)))
            .map(|p| p.id)
            .collect()
    }
}
```

### Retry/Escalation as Exponential Backoff + Circuit Breaker

```rust
struct RetryPolicy {
    max_attempts: u32,
    base_delay_ms: u64,
    max_delay_ms: u64,
    backoff_multiplier: f64,
    jitter: bool,
}

struct EscalationPolicy {
    max_escalations: u32,
    escalation_delay_ms: u64,
    escalation_target: EscalationTarget,  // Human, Admin, AlternativeLane
}

fn execute_with_retry(packet: &Packet, executor: &dyn Executor) -> Result<PacketResult> {
    let mut attempt = 0;
    let mut delay = packet.retry_policy.base_delay_ms;

    loop {
        match executor.execute(packet) {
            Ok(result) => return Ok(result),
            Err(e) if attempt < packet.retry_policy.max_attempts => {
                attempt += 1;
                sleep(delay_with_jitter(delay, packet.retry_policy.jitter));
                delay = (delay as f64 * packet.retry_policy.backoff_multiplier)
                    .min(packet.retry_policy.max_delay_ms as f64) as u64;
            }
            Err(e) => {
                // Escalate
                escalate(packet, &packet.escalation_policy, e)?;
                return Err(e);
            }
        }
    }
}
```

### Preregistration Gate (409/400)

```rust
fn validate_preregistration(packet: &Packet) -> Result<()> {
    // 400: No problem statement
    if packet.preregistered_plan.is_none() {
        return Err(ApiError::BadRequest {
            code: 400,
            message: "Preregistered plan required. Define problem, hypothesis, and falsifier.",
        });
    }

    let plan = packet.preregistered_plan.as_ref().unwrap();

    // 409: Plan exists but incomplete
    if plan.problem_statement.is_empty() {
        return Err(ApiError::Conflict {
            code: 409,
            message: "Problem statement cannot be empty",
        });
    }
    if plan.falsification_criteria.is_empty() {
        return Err(ApiError::Conflict {
            code: 409,
            message: "Falsification criteria required",
        });
    }
    if plan.predicted_outcome.is_none() {
        return Err(ApiError::Conflict {
            code: 409,
            message: "Predicted outcome with confidence required",
        });
    }

    Ok(())
}
```

### Judge Model

```rust
trait Judge: Send + Sync {
    fn evaluate(&self, packet: &Packet, result: &PacketResult) -> JudgeVerdict;
    fn held_out(&self) -> bool;  // True for held-out judges
}

struct DeterministicJudge {
    criteria: Vec<Criterion>,
    threshold: f32,
}

struct LLMJudge {
    model: ModelId,
    prompt_template: String,
    rubric: Rubric,
}

struct HumanJudge {
    approver_role: Role,
    timeout: Duration,
}
```

### Telemetry + Receipts

```rust
struct WorkflowTelemetry {
    event: WorkflowEvent,
    packet_id: PacketId,
    lane_id: LaneId,
    run_id: RunId,
    timestamp: Timestamp,
    receipt_hash: Hash,
}

enum WorkflowEvent {
    PacketAdmitted { packet_id },
    PacketStarted { packet_id, worker },
    PacketJudged { packet_id, verdict, judge_id },
    PacketRetried { packet_id, attempt, error },
    PacketEscalated { packet_id, target, reason },
    PacketCompleted { packet_id, status },
}
```

## Open questions / gaps

(None — fully specified)

---

## Contributor Deepening — Muse (2026-10-26)

### C27.7 Status-code inversion (depth: the codes contradict the contract)

The Preregistration Gate section assigns 400 = missing plan and 409 = incomplete plan. But C08 (F8/error model) and C18 (error table) define 400 = invalid request / missing problem statement and 409 = no preregistered plan / conflict [VERIFIED — C08 line 36, C18 error table both read directly]. So:

- C08/C18: 409 = NO PLAN (absent), 400 = missing problem statement.
- C27: 400 = no plan (absent), 409 = plan present but incomplete.

The two schemes agree that a missing problem statement is 400, but DISAGREE on absent-vs-incomplete plans (absent = 409 in C08/C18, 400 in C27). Frozen resolution [PROPOSED — resolution is my proposal; C08/C18 agree with each other and outnumber C27]: adopt the C08/C18 scheme everywhere — 400 = malformed request (missing problem statement OR malformed plan JSON), 409 = well-formed request conflicting with gate state (no plan on file, or plan present but failing completeness). C27's validate_preregistration swaps its two error constructors. Recommend a C36 correction entry; recording here, not silently editing the code sketch above.

### C27.8 Liveness vs breaker deadlock (depth: the Petri net assumes what the breaker denies)

The Petri-net properties claim Liveness ("No deadlocks — autorunner ensures progress"). But C30/ADR-14 circuit breakers ISOLATE lanes (no packets admitted) and C22.2 closes lanes on recall failure — an isolated lane with pending packets depending on it is a deadlock the autorunner cannot resolve by scheduling [INFERRED — composes C27 liveness claim with C30 breaker semantics; ends at those files]. Frozen completion:

1. Liveness holds ONLY over non-isolated lanes: restate as "Live modulo isolation — every ready packet on a non-isolated lane is eventually admitted." Packets blocked on isolated lanes are ESCALATED (C22.4 path), not retried into the breaker.
2. The find_ready_packets filter gains an isolation predicate: ready = pending AND deps-done AND lane-not-isolated. Without it, the scheduler admits packets to a lane the breaker closed — the exact retry-into-breaker spin C13.2 warns about.
3. Escalation target for breaker-blocked packets defaults to AlternativeLane (not Human): breaker storms already page humans via C22.2; routing every blocked packet to human approval DDoSes the approver. Human escalation reserved for packets that fail on the alternative lane too.

What was NOT checked here: autorunner implementation (outside workspace); whether isolation state is visible to the scheduler today (verify at C30 pass).
