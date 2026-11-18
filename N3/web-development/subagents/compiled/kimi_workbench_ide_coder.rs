//! kimi-workbench-ide-coder — compiled subagent lifecycle.
//!
//! Agent A6 (Subagents-B), Wave 2, slice 5/6. Disjoint from Agent A5 slices 1-3.
//! Source: `subagent_definitions.json` entry `kimi-workbench-ide-coder`
//!   + `SUBAGENT_SWARM_LOGIC.md` section 5 (READ-ONLY refs, never modified).
//!
//! Role: Dockview Workbench, Monaco & CodeMirror Specialist.
//! Scope (owns, may write): `src/components/workbench/*`, `src/components/editors/*`.
//! Scope (never writes): A5 paths — `src/types/*`, `src/store/*` (architect contracts),
//!   `src/components/layout/*`, `src/components/ui/*`, `src/styles/*` (surface coder);
//!   and sibling B path `src/components/canvas|graph|dag/*` (graph coder) — read-only.
//! Tools (verbatim from JSON): EnableWriteTools=true, EnableMCPTools=false,
//!   EnableSubagentTools=false. Never holds MCP (critic is the sole MCP holder).
//! Judges: TsgoTsc -> AuditSh -> ViteBuild -> CriticGate. Coders NEVER self-approve.
//! Escalation: 2 consecutive failures at the same judge layer -> human queue.
//! Critic-introduced changes re-enter judge layer (b) (GLOBAL-LOGIC.md section 6,
//!   VERIFICATION RULE; proof lives in `kimi_visual_qa_critic.rs`).
//!
//! Date: 2026-10-31. Status: COMPILED (pure-Rust lifecycle; no TS-subset surface,
//!   so no TS shim was needed and ts_check/ts_to_rust gates are N/A — recorded, not skipped).

#![deny(warnings)]

/// Lifecycle: create -> inject -> run -> dispose. `Disposed` is terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Created,
    Injected,
    Running,
    Disposed,
}

/// Tool allowlist, verbatim from `subagent_definitions.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolAllowlist {
    pub write: bool,
    pub mcp: bool,
    pub subagent: bool,
}

/// Verbatim flags for this subagent. Critic is the sole MCP holder.
pub static TOOL_ALLOWLIST: ToolAllowlist = ToolAllowlist { write: true, mcp: false, subagent: false };

/// Owned (writable) path prefixes for this slice.
pub static OWNS: &[&str] = &["src/components/workbench/", "src/components/editors/"];

/// A5-owned + sibling-B paths this agent must never write (read-only at most).
pub static NEVER_WRITES: &[&str] = &[
    "src/types/",
    "src/store/",
    "src/components/layout/",
    "src/components/ui/",
    "src/styles/",
    "src/components/canvas/",
    "src/components/graph/",
    "src/components/dag/",
];

/// Ordered judge layers. Coders never self-approve; only these gates pass work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    TsgoTsc,
    AuditSh,
    ViteBuild,
    CriticGate,
}

pub static JUDGES: &[Judge] = &[Judge::TsgoTsc, Judge::AuditSh, Judge::ViteBuild, Judge::CriticGate];

/// Consecutive failures at the same layer before terminal escalation.
pub const MAX_STRIKES: u32 = 2;

/// A releasable resource. All must be released on `dispose` (LIFO); none may leak on unload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disposer {
    pub name: &'static str,
    pub released: bool,
}

/// Typed runtime context supplied at injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContext {
    pub workspace_root: String,
    pub contracts_ready: bool,
}

/// Typed coder input: which workbench family to emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchCoderInput {
    pub workbench: String,
    pub pane_count: u32,
}

/// Typed coder output: emitted files (all inside `OWNS`) plus layout mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchCoderOutput {
    pub files: Vec<String>,
    pub layout_serializable: bool,
    pub workers_configured: bool,
}

/// Lifecycle failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleError {
    pub agent: &'static str,
    pub state: Lifecycle,
    pub reason: &'static str,
}

/// Escalation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscalationRoute {
    HumanQueue,
    Archive,
}

/// Escalation record: failing layer, strike count, where it goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Escalation {
    pub agent: &'static str,
    pub layer: Judge,
    pub strikes: u32,
    pub route: EscalationRoute,
}

/// Route an exhaustion: strikes >= MAX_STRIKES -> human queue, else archive-and-note.
pub fn escalate(layer: Judge, strikes: u32) -> Escalation {
    let route: EscalationRoute = if strikes >= MAX_STRIKES {
        EscalationRoute::HumanQueue
    } else {
        EscalationRoute::Archive
    };
    Escalation { agent: "kimi-workbench-ide-coder", layer, strikes, route }
}

/// The compiled subagent: state + disposers. Drop-safe only via explicit `dispose`.
#[derive(Debug)]
pub struct WorkbenchIdeCoder {
    pub state: Lifecycle,
    pub disposers: Vec<Disposer>,
    pub released_log: Vec<&'static str>,
}

impl WorkbenchIdeCoder {
    /// create: register all resources up front, all unreleased.
    pub fn create() -> Self {
        WorkbenchIdeCoder {
            state: Lifecycle::Created,
            disposers: vec![
                Disposer { name: "dockview-api", released: false },
                Disposer { name: "monaco-models", released: false },
                Disposer { name: "codemirror-state", released: false },
            ],
            released_log: Vec::new(),
        }
    }

    /// inject: bind runtime context. Requires contracts on disk (architect ran first).
    pub fn inject(&mut self, ctx: &RuntimeContext) -> Result<(), LifecycleError> {
        if self.state != Lifecycle::Created {
            return Err(LifecycleError {
                agent: "kimi-workbench-ide-coder",
                state: self.state,
                reason: "inject requires Created",
            });
        }
        if !ctx.contracts_ready {
            return Err(LifecycleError {
                agent: "kimi-workbench-ide-coder",
                state: self.state,
                reason: "contracts gate: src/types + src/store must exist first",
            });
        }
        self.state = Lifecycle::Injected;
        Ok(())
    }

    /// run: emit typed plan. Every emitted file must sit inside `OWNS` (self-enforced).
    pub fn run(
        &mut self,
        input: &WorkbenchCoderInput,
    ) -> Result<WorkbenchCoderOutput, LifecycleError> {
        if self.state != Lifecycle::Injected && self.state != Lifecycle::Running {
            return Err(LifecycleError {
                agent: "kimi-workbench-ide-coder",
                state: self.state,
                reason: "run requires Injected",
            });
        }
        self.state = Lifecycle::Running;
        let files: Vec<String> = vec![
            format!("src/components/workbench/{}_layout.tsx", input.workbench),
            format!("src/components/editors/{}_editor.tsx", input.workbench),
        ];
        for f in files.iter() {
            if !WorkbenchIdeCoder::may_write(f) {
                return Err(LifecycleError {
                    agent: "kimi-workbench-ide-coder",
                    state: self.state,
                    reason: "emitted file outside OWNS",
                });
            }
        }
        Ok(WorkbenchCoderOutput { files, layout_serializable: true, workers_configured: true })
    }

    /// dispose: release every disposer LIFO. Idempotent; terminal.
    pub fn dispose(&mut self) {
        while let Some(mut d) = self.disposers.pop() {
            d.released = true;
            self.released_log.push(d.name);
        }
        self.state = Lifecycle::Disposed;
    }

    /// True only when disposed with zero outstanding resources (no leak on unload).
    pub fn leak_check(&self) -> bool {
        self.state == Lifecycle::Disposed
            && self.disposers.is_empty()
            && self.released_log.len() == 3
    }

    /// Ownership gate: writable iff under OWNS and never under A5/sibling paths.
    pub fn may_write(path: &str) -> bool {
        let owned: bool = OWNS.iter().any(|p| path.starts_with(p));
        let forbidden: bool = NEVER_WRITES.iter().any(|p| path.starts_with(p));
        owned && !forbidden
    }
}

fn main() {
    let mut agent: WorkbenchIdeCoder = WorkbenchIdeCoder::create();
    let ctx: RuntimeContext =
        RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
    if let Err(e) = agent.inject(&ctx) {
        eprintln!("inject failed: {:?}", e);
        std::process::exit(1);
    }
    let input: WorkbenchCoderInput =
        WorkbenchCoderInput { workbench: "ide".to_string(), pane_count: 4 };
    match agent.run(&input) {
        Ok(o) => println!("emit {:?} serializable={}", o.files, o.layout_serializable),
        Err(e) => {
            eprintln!("run failed: {:?}", e);
            std::process::exit(1);
        }
    }
    agent.dispose();
    if !agent.leak_check() {
        eprintln!("LEAK: disposers outstanding");
        std::process::exit(1);
    }
    println!("disposed clean: no leak");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn injected() -> WorkbenchIdeCoder {
        let mut a: WorkbenchIdeCoder = WorkbenchIdeCoder::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
        assert_eq!(a.inject(&ctx), Ok(()));
        a
    }

    #[test]
    fn lifecycle_create_inject_run_dispose_no_leak() {
        let mut a: WorkbenchIdeCoder = WorkbenchIdeCoder::create();
        assert_eq!(a.state, Lifecycle::Created);
        assert!(!a.leak_check());
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
        assert_eq!(a.inject(&ctx), Ok(()));
        let out: WorkbenchCoderOutput = a
            .run(&WorkbenchCoderInput { workbench: "ide".to_string(), pane_count: 4 })
            .expect("run ok");
        assert!(out.layout_serializable);
        assert!(out.workers_configured);
        assert!(out.files.iter().all(|f| WorkbenchIdeCoder::may_write(f)));
        a.dispose();
        assert!(a.leak_check());
        assert_eq!(
            a.released_log,
            vec!["codemirror-state", "monaco-models", "dockview-api"]
        );
    }

    #[test]
    fn run_before_inject_is_rejected() {
        let mut a: WorkbenchIdeCoder = WorkbenchIdeCoder::create();
        let err: LifecycleError = a
            .run(&WorkbenchCoderInput { workbench: "x".to_string(), pane_count: 1 })
            .expect_err("must reject");
        assert_eq!(err.reason, "run requires Injected");
    }

    #[test]
    fn inject_requires_contracts_gate() {
        let mut a: WorkbenchIdeCoder = WorkbenchIdeCoder::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: false };
        let err: LifecycleError = a.inject(&ctx).expect_err("must reject");
        assert_eq!(err.state, Lifecycle::Created);
    }

    #[test]
    fn dispose_is_idempotent_and_terminal() {
        let mut a: WorkbenchIdeCoder = injected();
        a.dispose();
        a.dispose();
        assert!(a.leak_check());
        let err: LifecycleError = a
            .run(&WorkbenchCoderInput { workbench: "x".to_string(), pane_count: 1 })
            .expect_err("disposed is terminal");
        assert_eq!(err.state, Lifecycle::Disposed);
    }

    #[test]
    fn ownership_allows_workbench_editors_only() {
        assert!(WorkbenchIdeCoder::may_write("src/components/workbench/dock.tsx"));
        assert!(WorkbenchIdeCoder::may_write("src/components/editors/monaco.tsx"));
        for p in NEVER_WRITES.iter() {
            assert!(!WorkbenchIdeCoder::may_write(&format!("{}file.ts", p)), "deny {}", p);
        }
        assert!(!WorkbenchIdeCoder::may_write("src/components/canvas/scene.tsx"));
    }

    #[test]
    fn tool_allowlist_preserved_never_holds_mcp() {
        assert_eq!(
            TOOL_ALLOWLIST,
            ToolAllowlist { write: true, mcp: false, subagent: false }
        );
    }

    #[test]
    fn escalation_routes_on_strike_exhaustion() {
        let e1: Escalation = escalate(Judge::ViteBuild, 1);
        assert_eq!(e1.route, EscalationRoute::Archive);
        let e2: Escalation = escalate(Judge::ViteBuild, 2);
        assert_eq!(e2.route, EscalationRoute::HumanQueue);
        assert_eq!(e2.agent, "kimi-workbench-ide-coder");
    }
}
