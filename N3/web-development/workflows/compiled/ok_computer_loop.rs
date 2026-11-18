//! Kimi K2.5 "OK Computer" 5-horizon frontend loop — compiled bounded state machine.
//!
//! Source loop (markdown, interpreted):
//!   skills/web-development/kimi-frontend-kit/workflows/KIMI_OK_COMPUTER_LOOP.md
//!   Phase 0 brief/dials → Phase 1 swarm decompose → Phase 2 parallel slices →
//!   Phase 3 compiler gate → Phase 4 vision QA.
//!
//! Compiled mapping (states, typed phase I/O):
//!   Plan   : Brief -> PlanOutput   (md Phase 0 + Phase 1: design read, 3 dials,
//!                                    OKLCH token lock, explorer summary, contracts)
//!   Run    : PlanOutput -> RunOutput (md Phase 2: vertical slices A/B/C)
//!   Judge  : RunOutput -> JudgeOutput (md Phase 3 + 4 as four ordered layers:
//!                                    TsgoTsc, AuditSh, ViteBuild, DevToolsChecks)
//!   Receipt: terminal — Passed | Escalated (human/archive).
//!
//! Escalation semantics (READ-ONLY ref, not modified):
//!   orchestrator/src/swarm/mod.rs — Governor supervision:
//!   - `RestartStrategy::OneForOne` → restart only the failing child. Mirrored here:
//!     on strike 2 the machine re-enters ONLY the failing judge layer once.
//!   - `RestartIntensity { max_restarts, within_ticks }` + `ProcessState::
//!     EscalatedIntensityExceeded` → intensity exhausted, escalate. Mirrored here:
//!     `MAX_STRIKES = 2` consecutive failures at the same layer (or `MAX_ROUNDS`
//!     full loops exhausted) → terminal `Escalated` (human/archive), never retry.
//!
//! Roles ref (SUBAGENT_SWARM_LOGIC.md): explorer / architect / 3 coders / QA
//! critic map to `SliceId` + `JudgeLayer::DevToolsChecks`.
//!
//! Bounds (every loop is a `for` over a range — no `loop {}`, no `while`):
//!   MAX_ROUNDS = 3 full Plan→Run→Judge loops.
//!   MAX_JUDGE_ATTEMPTS_PER_ROUND = 4 judge invocations per round (strike-1 patch loop).
//!   MAX_STRIKES = 2 consecutive failures at the SAME layer → re-enter once → escalate.
//!
//! Agent 6 (Loop-B), Wave 1. Disjoint slice — does not touch Agent 5's file
//! (cognitive-map/slice2/src/attempt_loop.rs).

#![deny(warnings)]

/// Maximum full Plan → Run → Judge loops before terminal escalation.
pub const MAX_ROUNDS: u32 = 3;
/// Consecutive failures at the SAME judge layer that trigger one re-entry, then escalation.
pub const MAX_STRIKES: u32 = 2;
/// Judge invocations allowed inside one round (strike-1 patch-and-rejudge loop).
pub const MAX_JUDGE_ATTEMPTS_PER_ROUND: u32 = 4;

// ---------------------------------------------------------------------------
// Typed phase I/O
// ---------------------------------------------------------------------------

/// Explicit machine states. `Receipt` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineState {
    Plan,
    Run,
    Judge,
    Receipt,
}

/// Ordered judge layers = md Phase 3 gate + Phase 4 vision QA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JudgeLayer {
    TsgoTsc,
    AuditSh,
    ViteBuild,
    DevToolsChecks,
}

/// Fixed layer order. `const` so the judge sequence is compiled, not configured.
pub const JUDGE_LAYERS: [JudgeLayer; 4] = [
    JudgeLayer::TsgoTsc,
    JudgeLayer::AuditSh,
    JudgeLayer::ViteBuild,
    JudgeLayer::DevToolsChecks,
];

/// Plan-phase input: the raw brief packet.
#[derive(Debug, Clone)]
pub struct Brief {
    pub text: String,
}

/// The three parametric dials from md Phase 0 (each 1–10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dials {
    pub variance: u8,
    pub motion: u8,
    pub density: u8,
}

impl Dials {
    /// Clamp each dial into the legal 1–10 range.
    pub fn clamped(variance: u8, motion: u8, density: u8) -> Self {
        Self {
            variance: variance.clamp(1, 10),
            motion: motion.clamp(1, 10),
            density: density.clamp(1, 10),
        }
    }
}

/// Plan-phase output: design read + dials + shared contracts (md Phase 0 + 1).
#[derive(Debug, Clone)]
pub struct PlanOutput {
    pub design_read: String,
    pub dials: Dials,
    pub contracts: Vec<String>,
}

/// Vertical slices from md Phase 2 (plus explorer/architect outputs folded into Plan).
/// Partition rule (GLOBAL-LOGIC.md §6.3): slices are vertical, disjoint by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceId {
    SurfaceMotion,
    SpatialGraph,
    WorkbenchIde,
}

/// One vertical slice's output. Ownership is exclusive per `SliceId`.
#[derive(Debug, Clone)]
pub struct SliceOutput {
    pub slice: SliceId,
    pub files: Vec<String>,
    pub notes: String,
}

/// Run-phase output: explorer summary + the three vertical slices.
#[derive(Debug, Clone)]
pub struct RunOutput {
    pub explorer_summary: String,
    pub slices: Vec<SliceOutput>,
}

/// Judge-phase input: the run under test plus round/attempt counters (proof of bound).
/// Counters are audit payload: prod judges consume them to enforce per-round budgets.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JudgeInput {
    pub round: u32,
    pub attempt: u32,
    pub run: RunOutput,
}

/// One layer's verdict.
#[derive(Debug, Clone)]
pub struct LayerVerdict {
    pub layer: JudgeLayer,
    pub pass: bool,
    pub detail: String,
}

/// Judge-phase output: one verdict per layer, in `JUDGE_LAYERS` order.
#[derive(Debug, Clone)]
pub struct JudgeOutput {
    pub verdicts: Vec<LayerVerdict>,
}

/// Terminal receipt. Both variants terminate the machine.
#[derive(Debug, Clone)]
pub enum Terminal {
    Passed {
        rounds_used: u32,
    },
    Escalated {
        layer: Option<JudgeLayer>,
        rounds_used: u32,
        strikes: u32,
        reason: String,
    },
}

/// Final receipt: terminal state + full audit log.
#[derive(Debug, Clone)]
pub struct Receipt {
    pub terminal: Terminal,
    pub log: Vec<String>,
}

// ---------------------------------------------------------------------------
// Judge trait (injectable: real gates in prod, scripted dummy in test)
// ---------------------------------------------------------------------------

/// A judge answers a full 4-layer verdict and can re-enter a single layer once.
pub trait Judge {
    fn judge(&mut self, input: &JudgeInput) -> JudgeOutput;
    fn retry_layer(&mut self, layer: JudgeLayer, input: &JudgeInput) -> LayerVerdict;
}

// ---------------------------------------------------------------------------
// Phases (pure functions, typed I/O)
// ---------------------------------------------------------------------------

/// Plan: brief → design read + dials + contracts (md Phase 0 + Phase 1).
pub fn plan_phase(brief: &Brief) -> PlanOutput {
    PlanOutput {
        design_read: format!(
            "Reading this as: frontend surface for packet '{}'.",
            brief.text
        ),
        dials: Dials::clamped(7, 6, 5),
        contracts: vec![
            "src/types/domain.ts".to_string(),
            "src/store/useAppStore.ts".to_string(),
        ],
    }
}

/// Run: contracts → explorer summary + three disjoint vertical slices (md Phase 2).
pub fn run_phase(plan: &PlanOutput) -> RunOutput {
    let owned = |slice: SliceId, files: &[&str], notes: &str| SliceOutput {
        slice,
        files: files.iter().map(|s| s.to_string()).collect(),
        notes: format!("{} | contracts: {:?}", notes, plan.contracts),
    };
    RunOutput {
        explorer_summary: "explorer: repo scanned, primitives inventoried (dummy)".to_string(),
        slices: vec![
            owned(
                SliceId::SurfaceMotion,
                &["src/components/layout", "src/components/ui", "src/styles"],
                "Shell, Double-Bezel surfaces, motion choreography",
            ),
            owned(
                SliceId::SpatialGraph,
                &[
                    "src/components/canvas",
                    "src/components/graph",
                    "src/components/dag",
                ],
                "Spatial canvas / WebGL / knowledge graph / DAG engine",
            ),
            owned(
                SliceId::WorkbenchIde,
                &["src/components/workbench", "src/components/editors"],
                "Dockview workbench + Monaco / CodeMirror editors",
            ),
        ],
    }
}

/// First failing layer in compiled order (plus its detail), or `None` when all pass.
pub fn first_failure(output: &JudgeOutput) -> Option<(JudgeLayer, String)> {
    for layer in JUDGE_LAYERS {
        let hit = output.verdicts.iter().find(|v| v.layer == layer && !v.pass);
        if let Some(v) = hit {
            return Some((layer, v.detail.clone()));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Machine: plan → run → judge → receipt. Bounded. Two-strike escalation.
// ---------------------------------------------------------------------------

fn push_state(log: &mut Vec<String>, state: MachineState, round: u32, detail: &str) {
    log.push(format!(
        "[round {}/{}] state={:?} {}",
        round, MAX_ROUNDS, state, detail
    ));
}

/// Execute the compiled loop. Always terminates with a `Receipt`.
///
/// Escalation rule (real code branch, not a comment): 2 consecutive failing
/// judge verdicts at the SAME layer → re-enter that layer once; if it still
/// fails → terminal `Escalated` (human/archive). Round budget exhaustion also
/// terminates as `Escalated`. No unbounded `loop {}` anywhere in this function.
pub fn execute<J: Judge>(brief: &Brief, judge: &mut J) -> Receipt {
    let mut log: Vec<String> = Vec::new();
    let mut strikes: u32 = 0;
    let mut failing: Option<JudgeLayer> = None;

    for round in 1..=MAX_ROUNDS {
        push_state(&mut log, MachineState::Plan, round, "brief+dials+contracts");
        let plan = plan_phase(brief);
        log.push(format!(
            "  plan: read='{}' dials={:?} contracts={:?}",
            plan.design_read, plan.dials, plan.contracts
        ));

        push_state(&mut log, MachineState::Run, round, "vertical slices A/B/C");
        let run = run_phase(&plan);
        let owned_files: usize = run.slices.iter().map(|s| s.files.len()).sum();
        let owned_ids: Vec<SliceId> = run.slices.iter().map(|s| s.slice).collect();
        log.push(format!(
            "  run: explorer='{}' slices={:?} ({} files)",
            run.explorer_summary, owned_ids, owned_files
        ));
        for s in &run.slices {
            log.push(format!("    slice {:?}: {}", s.slice, s.notes));
        }

        let mut passed_this_round = false;
        let mut attempt: u32 = 0;
        for _ in 0..MAX_JUDGE_ATTEMPTS_PER_ROUND {
            attempt += 1;
            let input = JudgeInput {
                round,
                attempt,
                run: run.clone(),
            };
            push_state(
                &mut log,
                MachineState::Judge,
                round,
                &format!("attempt {}/{}", attempt, MAX_JUDGE_ATTEMPTS_PER_ROUND),
            );
            let output = judge.judge(&input);

            match first_failure(&output) {
                None => {
                    strikes = 0;
                    failing = None;
                    log.push("  judge: ALL LAYERS PASS".to_string());
                    passed_this_round = true;
                    break;
                }
                Some((layer, detail)) => {
                    if failing == Some(layer) {
                        strikes += 1;
                    } else {
                        failing = Some(layer);
                        strikes = 1;
                    }
                    log.push(format!(
                        "  judge: FAIL at {:?} detail='{}' — strike {}/{} (consecutive, same layer)",
                        layer, detail, strikes, MAX_STRIKES
                    ));
                    if strikes >= MAX_STRIKES {
                        log.push(format!(
                            "  two-strike rule: re-entering {:?} ONCE (OneForOne)",
                            layer
                        ));
                        let retry = judge.retry_layer(layer, &input);
                        if retry.pass {
                            log.push(format!(
                                "  re-entry: {:?} now passes — strikes reset",
                                layer
                            ));
                            strikes = 0;
                            failing = None;
                            passed_this_round = true;
                            break;
                        }
                        log.push(format!(
                            "  re-entry: {:?} STILL FAILS detail='{}' — ESCALATE to human/archive",
                            layer, retry.detail
                        ));
                        return Receipt {
                            terminal: Terminal::Escalated {
                                layer: Some(layer),
                                rounds_used: round,
                                strikes,
                                reason: "two-strike re-entry failed at same layer".to_string(),
                            },
                            log,
                        };
                    }
                    // Strike 1: patch and re-judge; the `for` bound caps attempts.
                }
            }
        }

        if passed_this_round {
            push_state(&mut log, MachineState::Receipt, round, "PASSED");
            return Receipt {
                terminal: Terminal::Passed { rounds_used: round },
                log,
            };
        }
        log.push(format!(
            "  round {} judge attempts exhausted without pass — next round",
            round
        ));
    }

    push_state(
        &mut log,
        MachineState::Receipt,
        MAX_ROUNDS,
        "ROUND BUDGET EXHAUSTED",
    );
    Receipt {
        terminal: Terminal::Escalated {
            layer: failing,
            rounds_used: MAX_ROUNDS,
            strikes,
            reason: "MAX_ROUNDS budget exhausted".to_string(),
        },
        log,
    }
}

// ---------------------------------------------------------------------------
// Dummy packet harness (termination demo, not a prod gate)
// ---------------------------------------------------------------------------

/// Scripted dummy judge for the termination test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DummyMode {
    /// Fail TsgoTsc exactly once (call #1), pass everything after; re-entry passes.
    RecoverOnce,
    /// Fail DevToolsChecks on every call and every re-entry (forces escalation).
    FailDevtoolsForever,
    /// Alternate failing layer every call (never 2 consecutive same → budget path).
    AlternateForever,
}

/// Dummy judge with public counters (round/strike/attempt proof comes from output).
pub struct DummyJudge {
    pub mode: DummyMode,
    pub calls: u32,
    pub retries: u32,
}

impl DummyJudge {
    pub fn new(mode: DummyMode) -> Self {
        Self {
            mode,
            calls: 0,
            retries: 0,
        }
    }

    fn verdict(&self, layer: JudgeLayer, pass: bool, detail: &str) -> LayerVerdict {
        LayerVerdict {
            layer,
            pass,
            detail: detail.to_string(),
        }
    }

    fn full_pass(&self) -> JudgeOutput {
        JudgeOutput {
            verdicts: JUDGE_LAYERS
                .iter()
                .map(|l| self.verdict(*l, true, "dummy: pass"))
                .collect(),
        }
    }

    fn fail_only(&self, layer: JudgeLayer, detail: &str) -> JudgeOutput {
        JudgeOutput {
            verdicts: JUDGE_LAYERS
                .iter()
                .map(|l| {
                    if *l == layer {
                        self.verdict(*l, false, detail)
                    } else {
                        self.verdict(*l, true, "dummy: pass")
                    }
                })
                .collect(),
        }
    }
}

impl Judge for DummyJudge {
    fn judge(&mut self, _input: &JudgeInput) -> JudgeOutput {
        self.calls += 1;
        match self.mode {
            DummyMode::RecoverOnce => {
                if self.calls == 1 {
                    self.fail_only(
                        JudgeLayer::TsgoTsc,
                        "injected: fake tsgo diagnostic TS-1005",
                    )
                } else {
                    self.full_pass()
                }
            }
            DummyMode::FailDevtoolsForever => self.fail_only(
                JudgeLayer::DevToolsChecks,
                "injected: 3 zero-height canvases at 1440x900",
            ),
            DummyMode::AlternateForever => {
                if self.calls % 2 == 1 {
                    self.fail_only(JudgeLayer::TsgoTsc, "injected: alternating fail A")
                } else {
                    self.fail_only(JudgeLayer::AuditSh, "injected: alternating fail B")
                }
            }
        }
    }

    fn retry_layer(&mut self, layer: JudgeLayer, _input: &JudgeInput) -> LayerVerdict {
        self.retries += 1;
        match self.mode {
            DummyMode::RecoverOnce => self.verdict(layer, true, "dummy re-entry: fixed"),
            _ => self.verdict(layer, false, "dummy re-entry: still failing"),
        }
    }
}

/// Run one dummy packet end-to-end and print the audit log. Always returns.
pub fn run_dummy_packet(name: &str, mode: DummyMode) -> Receipt {
    println!("=== dummy packet '{}' mode={:?} ===", name, mode);
    println!(
        "bound: MAX_ROUNDS={} MAX_STRIKES={} MAX_JUDGE_ATTEMPTS_PER_ROUND={}",
        MAX_ROUNDS, MAX_STRIKES, MAX_JUDGE_ATTEMPTS_PER_ROUND
    );
    let brief = Brief {
        text: "fake brief: studio-grade bento dashboard (dummy)".to_string(),
    };
    let mut judge = DummyJudge::new(mode);
    let receipt = execute(&brief, &mut judge);
    for line in &receipt.log {
        println!("{}", line);
    }
    println!(
        "counters: judge_calls={} layer_retries={}",
        judge.calls, judge.retries
    );
    println!("final receipt: {:?}", receipt.terminal);
    match &receipt.terminal {
        Terminal::Passed { rounds_used } => {
            println!("TERMINATED: PASS after {} round(s)", rounds_used);
        }
        Terminal::Escalated {
            layer,
            rounds_used,
            strikes,
            reason,
        } => {
            println!(
                "TERMINATED: ESCALATED layer={:?} rounds={} strikes={} reason='{}'",
                layer, rounds_used, strikes, reason
            );
        }
    }
    receipt
}

fn main() {
    // Packet A: one injected judge failure → strike 1 → re-judge passes.
    run_dummy_packet("A-recover-once", DummyMode::RecoverOnce);
    println!();
    // Packet B: persistent DevTools failure → strike 1 → strike 2 → one re-entry
    // → still failing → escalate. Proves the two-strike branch fires and ends.
    run_dummy_packet("B-escalate", DummyMode::FailDevtoolsForever);
}

// ---------------------------------------------------------------------------
// Tests: termination, bound, escalation — all must hold for migration.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_brief() -> Brief {
        Brief {
            text: "test brief".to_string(),
        }
    }

    #[test]
    fn recover_once_passes_round_one() {
        let mut judge = DummyJudge::new(DummyMode::RecoverOnce);
        let receipt = execute(&dummy_brief(), &mut judge);
        match receipt.terminal {
            Terminal::Passed { rounds_used } => assert_eq!(rounds_used, 1),
            Terminal::Escalated { .. } => panic!("expected Passed, got escalation"),
        }
        assert_eq!(judge.calls, 2);
        assert_eq!(judge.retries, 0);
    }

    #[test]
    fn two_consecutive_same_layer_failures_escalate() {
        let mut judge = DummyJudge::new(DummyMode::FailDevtoolsForever);
        let receipt = execute(&dummy_brief(), &mut judge);
        match receipt.terminal {
            Terminal::Escalated {
                layer,
                rounds_used,
                strikes,
                ..
            } => {
                assert_eq!(layer, Some(JudgeLayer::DevToolsChecks));
                assert_eq!(rounds_used, 1);
                assert_eq!(strikes, MAX_STRIKES);
            }
            Terminal::Passed { .. } => panic!("expected Escalated, got pass"),
        }
        // 2 judge calls (strike 1, strike 2) + exactly 1 layer re-entry.
        assert_eq!(judge.calls, 2);
        assert_eq!(judge.retries, 1);
    }

    #[test]
    fn alternating_failures_exhaust_round_budget_and_terminate() {
        let mut judge = DummyJudge::new(DummyMode::AlternateForever);
        let receipt = execute(&dummy_brief(), &mut judge);
        match receipt.terminal {
            Terminal::Escalated { rounds_used, .. } => {
                assert_eq!(rounds_used, MAX_ROUNDS);
            }
            Terminal::Passed { .. } => panic!("expected budget-exhaustion escalation"),
        }
        // Hard bound proof: calls can never exceed rounds × attempts per round.
        assert!(judge.calls <= MAX_ROUNDS * MAX_JUDGE_ATTEMPTS_PER_ROUND);
        assert_eq!(judge.retries, 0); // strikes never reach 2 on alternating layers
    }

    #[test]
    fn dials_clamp_to_legal_range() {
        let d = Dials::clamped(0, 99, 5);
        assert_eq!(
            d,
            Dials {
                variance: 1,
                motion: 10,
                density: 5
            }
        );
    }
}
