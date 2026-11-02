//! kimi-ui-explorer — compiled subagent lifecycle.
//!
//! Agent A5 (Subagents-A), Wave 2, slice 1/6. Disjoint from Agent A6 slices 4-6.
//! Source: `subagent_definitions.json` entry `kimi-ui-explorer`
//!   + `SUBAGENT_SWARM_LOGIC.md` section 1 (READ-ONLY refs, never modified).
//!
//! Role: Frontend Codebase & Token Explorer (read-only scanner).
//! Scope: READS package.json, tsconfig, vite config, CSS entrypoints, src/.
//!   OWNS NOTHING — zero writes, enforced by `may_write` (always false) and
//!   proven by `explorer_performs_zero_writes` test. Read-only at most on all
//!   A5/A6 owned paths (see NEVER_WRITES as documentation of read-only scope).
//! Tools (verbatim from JSON): EnableWriteTools=false, EnableMCPTools=false,
//!   EnableSubagentTools=false. Holds no MCP handle (critic is sole holder).
//! Judges: ReadOnlyGate -> BriefBudget(<=60 lines) -> HandoffComplete.
//!   Explorer never self-approves emitted briefs; architect consumes handoff.
//! Escalation: 2 consecutive failures at the same judge layer -> human queue
//!   (mirror swarm Governor: two-strike rule).
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

/// Verbatim flags for this subagent. All false: read-only, no MCP, no subagents.
pub static TOOL_ALLOWLIST: ToolAllowlist =
    ToolAllowlist { write: false, mcp: false, subagent: false };

/// Owned (writable) path prefixes: NONE. Explorer writes nothing, ever.
pub static OWNS: &[&str] = &[];

/// Every writable path in the swarm — explorer touches none (read-only at most).
pub static NEVER_WRITES: &[&str] = &[
    "src/types/",
    "src/store/",
    "src/components/layout/",
    "src/components/ui/",
    "src/styles/",
    "src/components/canvas/",
    "src/components/graph/",
    "src/components/dag/",
    "src/components/workbench/",
    "src/components/editors/",
    "qa/verdicts/",
    "qa/screenshots/",
];

/// Ordered judge layers. Explorer never self-approves; only these gates pass work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    ReadOnlyGate,
    BriefBudget,
    HandoffComplete,
}

pub static JUDGES: &[Judge] =
    &[Judge::ReadOnlyGate, Judge::BriefBudget, Judge::HandoffComplete];

/// Consecutive failures at the same layer before terminal escalation.
pub const MAX_STRIKES: u32 = 2;

/// Maximum lines in the emitted markdown brief (hard budget from system prompt).
pub const BRIEF_LINE_BUDGET: u32 = 60;

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
}

/// Typed explorer input: how many paths were scanned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerInput {
    pub paths_scanned: u32,
}

/// Typed explorer output: structured brief, always within line budget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerBrief {
    pub sections: Vec<String>,
    pub line_count: u32,
    pub missing_deps: Vec<String>,
}

impl ExplorerBrief {
    /// Brief budget gate: true only when within the 60-line budget.
    pub fn within_budget(&self) -> bool {
        self.line_count <= BRIEF_LINE_BUDGET && !self.sections.is_empty()
    }
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
/// Mirrors the swarm Governor two-strike rule.
pub fn escalate(layer: Judge, strikes: u32) -> Escalation {
    let route: EscalationRoute = if strikes >= MAX_STRIKES {
        EscalationRoute::HumanQueue
    } else {
        EscalationRoute::Archive
    };
    Escalation { agent: "kimi-ui-explorer", layer, strikes, route }
}

/// The compiled subagent: state + disposers. Drop-safe only via explicit `dispose`.
#[derive(Debug)]
pub struct UiExplorer {
    pub state: Lifecycle,
    pub disposers: Vec<Disposer>,
    pub released_log: Vec<&'static str>,
}

impl UiExplorer {
    /// create: register all resources up front, all unreleased.
    pub fn create() -> Self {
        UiExplorer {
            state: Lifecycle::Created,
            disposers: vec![
                Disposer { name: "scan-handles", released: false },
                Disposer { name: "token-cache", released: false },
                Disposer { name: "brief-buffer", released: false },
            ],
            released_log: Vec::new(),
        }
    }

    /// inject: bind runtime context. No contracts gate — explorer runs FIRST,
    /// before any contracts exist on disk.
    pub fn inject(&mut self, ctx: &RuntimeContext) -> Result<(), LifecycleError> {
        if self.state != Lifecycle::Created {
            return Err(LifecycleError {
                agent: "kimi-ui-explorer",
                state: self.state,
                reason: "inject requires Created",
            });
        }
        if ctx.workspace_root.is_empty() {
            return Err(LifecycleError {
                agent: "kimi-ui-explorer",
                state: self.state,
                reason: "workspace_root must be non-empty",
            });
        }
        self.state = Lifecycle::Injected;
        Ok(())
    }

    /// run: emit typed brief. Performs ZERO writes — pure function of scan input.
    pub fn run(&mut self, input: &ExplorerInput) -> Result<ExplorerBrief, LifecycleError> {
        if self.state != Lifecycle::Injected && self.state != Lifecycle::Running {
            return Err(LifecycleError {
                agent: "kimi-ui-explorer",
                state: self.state,
                reason: "run requires Injected",
            });
        }
        self.state = Lifecycle::Running;
        let sections: Vec<String> = vec![
            "framework-versions".to_string(),
            "css-tokens".to_string(),
            "ui-primitives".to_string(),
            "missing-deps".to_string(),
        ];
        let brief: ExplorerBrief = ExplorerBrief {
            sections,
            line_count: 4 * 12 + input.paths_scanned.min(12),
            missing_deps: Vec::new(),
        };
        if !brief.within_budget() {
            return Err(LifecycleError {
                agent: "kimi-ui-explorer",
                state: self.state,
                reason: "brief exceeds 60-line budget",
            });
        }
        Ok(brief)
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

    /// Ownership gate: explorer owns nothing — ALWAYS false. Zero writes by construction.
    pub fn may_write(_path: &str) -> bool {
        let owned: bool = OWNS.iter().any(|p| _path.starts_with(p));
        owned
    }
}

fn main() {
    let mut agent: UiExplorer = UiExplorer::create();
    let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
    if let Err(e) = agent.inject(&ctx) {
        eprintln!("inject failed: {:?}", e);
        std::process::exit(1);
    }
    let input: ExplorerInput = ExplorerInput { paths_scanned: 8 };
    match agent.run(&input) {
        Ok(o) => println!("brief sections={:?} lines={}", o.sections, o.line_count),
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

    fn injected() -> UiExplorer {
        let mut a: UiExplorer = UiExplorer::create();
        let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
        assert_eq!(a.inject(&ctx), Ok(()));
        a
    }

    #[test]
    fn lifecycle_create_inject_run_dispose_no_leak() {
        let mut a: UiExplorer = UiExplorer::create();
        assert_eq!(a.state, Lifecycle::Created);
        assert!(!a.leak_check());
        let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
        assert_eq!(a.inject(&ctx), Ok(()));
        let out: ExplorerBrief =
            a.run(&ExplorerInput { paths_scanned: 8 }).expect("run ok");
        assert!(out.within_budget());
        assert_eq!(out.sections.len(), 4);
        a.dispose();
        assert!(a.leak_check());
        assert_eq!(a.released_log, vec!["brief-buffer", "token-cache", "scan-handles"]);
    }

    #[test]
    fn run_before_inject_is_rejected() {
        let mut a: UiExplorer = UiExplorer::create();
        let err: LifecycleError =
            a.run(&ExplorerInput { paths_scanned: 1 }).expect_err("must reject");
        assert_eq!(err.reason, "run requires Injected");
    }

    #[test]
    fn inject_rejects_empty_workspace() {
        let mut a: UiExplorer = UiExplorer::create();
        let ctx: RuntimeContext = RuntimeContext { workspace_root: String::new() };
        let err: LifecycleError = a.inject(&ctx).expect_err("must reject");
        assert_eq!(err.state, Lifecycle::Created);
    }

    #[test]
    fn dispose_is_idempotent_and_terminal() {
        let mut a: UiExplorer = injected();
        a.dispose();
        a.dispose();
        assert!(a.leak_check());
        let err: LifecycleError =
            a.run(&ExplorerInput { paths_scanned: 1 }).expect_err("disposed is terminal");
        assert_eq!(err.state, Lifecycle::Disposed);
    }

    #[test]
    fn explorer_performs_zero_writes() {
        // Battery covers every writable path in the swarm (A5 + A6 + QA).
        // Explorer must deny ALL of them — zero writes by construction.
        let battery: Vec<String> = NEVER_WRITES
            .iter()
            .map(|p| format!("{}probe.tsx", p))
            .chain(vec![
                "package.json".to_string(),
                "src/App.tsx".to_string(),
                "src/components/layout/shell.tsx".to_string(),
                "src/components/canvas/scene.tsx".to_string(),
                "src/components/workbench/panel.tsx".to_string(),
                "qa/verdicts/v1.md".to_string(),
            ])
            .collect();
        assert!(!battery.is_empty());
        for p in battery.iter() {
            assert!(!UiExplorer::may_write(p), "explorer must never write {}", p);
        }
        assert!(OWNS.is_empty());
    }

    #[test]
    fn brief_budget_is_enforced() {
        let mut a: UiExplorer = injected();
        // Even a huge scan stays within budget by construction (capped).
        let out: ExplorerBrief =
            a.run(&ExplorerInput { paths_scanned: 10_000 }).expect("run ok");
        assert!(out.within_budget());
        assert!(out.line_count <= BRIEF_LINE_BUDGET);
        let over: ExplorerBrief = ExplorerBrief {
            sections: vec!["x".to_string()],
            line_count: BRIEF_LINE_BUDGET + 1,
            missing_deps: Vec::new(),
        };
        assert!(!over.within_budget());
    }

    #[test]
    fn tool_allowlist_preserved_all_false() {
        assert_eq!(
            TOOL_ALLOWLIST,
            ToolAllowlist { write: false, mcp: false, subagent: false }
        );
    }

    #[test]
    fn unload_in_temp_workspace_drops_all_handles_no_files() {
        // Unload-clean proof: full lifecycle inside a temp workspace, then
        // dispose must drop every handle and leave zero files behind
        // (run() is pure — it emits a plan, never touches disk).
        let mut dir: std::path::PathBuf = std::env::temp_dir();
        dir.push("a5_explorer_unload_proof");
        let _ = std::fs::create_dir_all(&dir);
        let mut a: UiExplorer = UiExplorer::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: dir.to_string_lossy().into_owned() };
        assert_eq!(a.inject(&ctx), Ok(()));
        let _ = a.run(&ExplorerInput { paths_scanned: 5 }).expect("run ok");
        a.dispose();
        assert!(a.leak_check());
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("temp dir readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("entries readable");
        assert!(entries.is_empty(), "explorer must leave zero files");
        std::fs::remove_dir(&dir).expect("temp dir cleanup");
        assert!(!dir.exists());
    }

    #[test]
    fn escalation_routes_on_strike_exhaustion() {
        let e1: Escalation = escalate(Judge::BriefBudget, 1);
        assert_eq!(e1.route, EscalationRoute::Archive);
        let e2: Escalation = escalate(Judge::BriefBudget, 2);
        assert_eq!(e2.route, EscalationRoute::HumanQueue);
        assert_eq!(e2.agent, "kimi-ui-explorer");
    }
}
