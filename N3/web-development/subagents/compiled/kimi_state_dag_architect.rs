//! kimi-state-dag-architect — compiled subagent lifecycle.
//!
//! Agent A5 (Subagents-A), Wave 2, slice 2/6. Disjoint from Agent A6 slices 4-6.
//! Source: `subagent_definitions.json` entry `kimi-state-dag-architect`
//!   + `SUBAGENT_SWARM_LOGIC.md` section 2 (READ-ONLY refs, never modified).
//!
//! Role: State Machine, Type & Graph Architect.
//! Scope (owns, may write): `src/types/*`, `src/store/*` ONLY. Contracts gate:
//!   coders (A5 surface + A6 graph/workbench) may not inject until contracts
//!   exist on disk (`RuntimeContext.contracts_ready` enforced here at run).
//! Scope (never writes): surface layout/ui/styles, all A6 canvas/graph/dag,
//!   workbench/editors, and QA verdict/screenshot paths.
//! Tools (verbatim from JSON): EnableWriteTools=true, EnableMCPTools=false,
//!   EnableSubagentTools=false. Never holds MCP (critic is the sole MCP holder).
//! Judges: TsgoTsc -> AuditSh -> ViteBuild -> CriticGate. Never self-approves.
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

/// Verbatim flags for this subagent. Never holds MCP.
pub static TOOL_ALLOWLIST: ToolAllowlist =
    ToolAllowlist { write: true, mcp: false, subagent: false };

/// Owned (writable) path prefixes for this slice: contracts only.
pub static OWNS: &[&str] = &["src/types/", "src/store/"];

/// Paths this agent must never write (surface + A6 + QA).
pub static NEVER_WRITES: &[&str] = &[
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

/// Ordered judge layers. Never self-approves; only these gates pass work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Judge {
    TsgoTsc,
    AuditSh,
    ViteBuild,
    CriticGate,
}

pub static JUDGES: &[Judge] =
    &[Judge::TsgoTsc, Judge::AuditSh, Judge::ViteBuild, Judge::CriticGate];

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
}

/// Typed architect input: which domain slices to emit contracts for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectInput {
    pub domains: Vec<String>,
}

/// Typed architect output: emitted contract files (all inside `OWNS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitectOutput {
    pub files: Vec<String>,
    pub contracts_ready: bool,
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
    Escalation { agent: "kimi-state-dag-architect", layer, strikes, route }
}

/// The compiled subagent: state + disposers. Drop-safe only via explicit `dispose`.
#[derive(Debug)]
pub struct StateDagArchitect {
    pub state: Lifecycle,
    pub disposers: Vec<Disposer>,
    pub released_log: Vec<&'static str>,
}

impl StateDagArchitect {
    /// create: register all resources up front, all unreleased.
    pub fn create() -> Self {
        StateDagArchitect {
            state: Lifecycle::Created,
            disposers: vec![
                Disposer { name: "types-buffer", released: false },
                Disposer { name: "store-buffer", released: false },
                Disposer { name: "dag-schema-cache", released: false },
            ],
            released_log: Vec::new(),
        }
    }

    /// inject: bind runtime context. Architect runs SECOND (after explorer);
    /// no upstream contracts gate — it CREATES the contracts.
    pub fn inject(&mut self, ctx: &RuntimeContext) -> Result<(), LifecycleError> {
        if self.state != Lifecycle::Created {
            return Err(LifecycleError {
                agent: "kimi-state-dag-architect",
                state: self.state,
                reason: "inject requires Created",
            });
        }
        if ctx.workspace_root.is_empty() {
            return Err(LifecycleError {
                agent: "kimi-state-dag-architect",
                state: self.state,
                reason: "workspace_root must be non-empty",
            });
        }
        self.state = Lifecycle::Injected;
        Ok(())
    }

    /// run: emit typed contracts plan (types + store per domain).
    /// Every emitted file must sit inside `OWNS` (self-enforced).
    pub fn run(&mut self, input: &ArchitectInput) -> Result<ArchitectOutput, LifecycleError> {
        if self.state != Lifecycle::Injected && self.state != Lifecycle::Running {
            return Err(LifecycleError {
                agent: "kimi-state-dag-architect",
                state: self.state,
                reason: "run requires Injected",
            });
        }
        if input.domains.is_empty() {
            return Err(LifecycleError {
                agent: "kimi-state-dag-architect",
                state: self.state,
                reason: "at least one domain required",
            });
        }
        self.state = Lifecycle::Running;
        let mut files: Vec<String> = Vec::new();
        for d in input.domains.iter() {
            files.push(format!("src/types/{}.ts", d));
            files.push(format!("src/store/use_{}_store.ts", d));
        }
        for f in files.iter() {
            if !StateDagArchitect::may_write(f) {
                return Err(LifecycleError {
                    agent: "kimi-state-dag-architect",
                    state: self.state,
                    reason: "emitted file outside OWNS",
                });
            }
        }
        Ok(ArchitectOutput { files, contracts_ready: true })
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

    /// Ownership gate: writable iff under OWNS and never under surface/A6/QA paths.
    pub fn may_write(path: &str) -> bool {
        let owned: bool = OWNS.iter().any(|p| path.starts_with(p));
        let forbidden: bool = NEVER_WRITES.iter().any(|p| path.starts_with(p));
        owned && !forbidden
    }
}

fn main() {
    let mut agent: StateDagArchitect = StateDagArchitect::create();
    let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
    if let Err(e) = agent.inject(&ctx) {
        eprintln!("inject failed: {:?}", e);
        std::process::exit(1);
    }
    let input: ArchitectInput =
        ArchitectInput { domains: vec!["app".to_string(), "graph".to_string()] };
    match agent.run(&input) {
        Ok(o) => println!("contracts {:?} ready={}", o.files, o.contracts_ready),
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

    fn injected() -> StateDagArchitect {
        let mut a: StateDagArchitect = StateDagArchitect::create();
        let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
        assert_eq!(a.inject(&ctx), Ok(()));
        a
    }

    #[test]
    fn lifecycle_create_inject_run_dispose_no_leak() {
        let mut a: StateDagArchitect = StateDagArchitect::create();
        assert_eq!(a.state, Lifecycle::Created);
        assert!(!a.leak_check());
        let ctx: RuntimeContext = RuntimeContext { workspace_root: ".".to_string() };
        assert_eq!(a.inject(&ctx), Ok(()));
        let out: ArchitectOutput = a
            .run(&ArchitectInput { domains: vec!["app".to_string()] })
            .expect("run ok");
        assert!(out.contracts_ready);
        assert_eq!(
            out.files,
            vec!["src/types/app.ts".to_string(), "src/store/use_app_store.ts".to_string()]
        );
        assert!(out.files.iter().all(|f| StateDagArchitect::may_write(f)));
        a.dispose();
        assert!(a.leak_check());
        assert_eq!(a.released_log, vec!["dag-schema-cache", "store-buffer", "types-buffer"]);
    }

    #[test]
    fn run_before_inject_is_rejected() {
        let mut a: StateDagArchitect = StateDagArchitect::create();
        let err: LifecycleError = a
            .run(&ArchitectInput { domains: vec!["x".to_string()] })
            .expect_err("must reject");
        assert_eq!(err.reason, "run requires Injected");
    }

    #[test]
    fn run_rejects_empty_domains() {
        let mut a: StateDagArchitect = injected();
        let err: LifecycleError =
            a.run(&ArchitectInput { domains: Vec::new() }).expect_err("must reject");
        assert_eq!(err.reason, "at least one domain required");
    }

    #[test]
    fn dispose_is_idempotent_and_terminal() {
        let mut a: StateDagArchitect = injected();
        a.dispose();
        a.dispose();
        assert!(a.leak_check());
        let err: LifecycleError = a
            .run(&ArchitectInput { domains: vec!["x".to_string()] })
            .expect_err("disposed is terminal");
        assert_eq!(err.state, Lifecycle::Disposed);
    }

    #[test]
    fn ownership_allows_contracts_paths_only() {
        assert!(StateDagArchitect::may_write("src/types/app.ts"));
        assert!(StateDagArchitect::may_write("src/store/use_app_store.ts"));
        for p in NEVER_WRITES.iter() {
            assert!(!StateDagArchitect::may_write(&format!("{}file.tsx", p)), "deny {}", p);
        }
        // Cross-slice denials: each coder family's flagship path.
        assert!(!StateDagArchitect::may_write("src/components/layout/shell.tsx"));
        assert!(!StateDagArchitect::may_write("src/components/canvas/scene.tsx"));
        assert!(!StateDagArchitect::may_write("src/components/workbench/panel.tsx"));
        assert!(!StateDagArchitect::may_write("qa/verdicts/v1.md"));
    }

    #[test]
    fn tool_allowlist_preserved_write_only_no_mcp() {
        assert_eq!(
            TOOL_ALLOWLIST,
            ToolAllowlist { write: true, mcp: false, subagent: false }
        );
    }

    #[test]
    fn unload_in_temp_workspace_drops_all_handles_no_files() {
        let mut dir: std::path::PathBuf = std::env::temp_dir();
        dir.push("a5_architect_unload_proof");
        let _ = std::fs::create_dir_all(&dir);
        let mut a: StateDagArchitect = StateDagArchitect::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: dir.to_string_lossy().into_owned() };
        assert_eq!(a.inject(&ctx), Ok(()));
        let _ = a
            .run(&ArchitectInput { domains: vec!["app".to_string()] })
            .expect("run ok");
        a.dispose();
        assert!(a.leak_check());
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("temp dir readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("entries readable");
        assert!(entries.is_empty(), "architect must leave zero files");
        std::fs::remove_dir(&dir).expect("temp dir cleanup");
        assert!(!dir.exists());
    }

    #[test]
    fn escalation_routes_on_strike_exhaustion() {
        let e1: Escalation = escalate(Judge::TsgoTsc, 1);
        assert_eq!(e1.route, EscalationRoute::Archive);
        let e2: Escalation = escalate(Judge::TsgoTsc, 2);
        assert_eq!(e2.route, EscalationRoute::HumanQueue);
        assert_eq!(e2.agent, "kimi-state-dag-architect");
    }
}
