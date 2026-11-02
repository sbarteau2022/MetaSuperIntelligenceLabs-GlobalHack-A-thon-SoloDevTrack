//! kimi-surface-motion-coder — compiled subagent lifecycle.
//!
//! Agent A5 (Subagents-A), Wave 2, slice 3/6. Disjoint from Agent A6 slices 4-6.
//! Source: `subagent_definitions.json` entry `kimi-surface-motion-coder`
//!   + `SUBAGENT_SWARM_LOGIC.md` section 3 (READ-ONLY refs, never modified).
//!
//! Role: Principal UI Surface & Motion Choreographer.
//! Scope (owns, may write): `src/components/layout/*`, `src/components/ui/*`,
//!   `src/styles/*` ONLY.
//! Scope (never writes): architect contracts (`src/types/*`, `src/store/*`) —
//!   read-only after the contracts gate — plus all A6 canvas/graph/dag,
//!   workbench/editors, and QA verdict/screenshot paths.
//! Tools (verbatim from JSON): EnableWriteTools=true, EnableMCPTools=false,
//!   EnableSubagentTools=false. Never holds MCP (critic is the sole MCP holder).
//! Judges: TsgoTsc -> AuditSh -> ViteBuild -> CriticGate. Coders NEVER self-approve.
//! Escalation: 2 consecutive failures at the same judge layer -> human queue
//!   (mirror swarm Governor: two-strike rule).
//! Contracts gate: `inject` requires `contracts_ready == true` (architect ran first).
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

/// Owned (writable) path prefixes for this slice: surface only.
pub static OWNS: &[&str] =
    &["src/components/layout/", "src/components/ui/", "src/styles/"];

/// A5-architect + A6 + QA paths this agent must never write (read-only at most).
pub static NEVER_WRITES: &[&str] = &[
    "src/types/",
    "src/store/",
    "src/components/canvas/",
    "src/components/graph/",
    "src/components/dag/",
    "src/components/workbench/",
    "src/components/editors/",
    "qa/verdicts/",
    "qa/screenshots/",
];

/// Ordered judge layers. Coders never self-approve; only these gates pass work.
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

/// Typed runtime context supplied at injection. Contracts must be ready
/// (architect ran first) or `inject` is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeContext {
    pub workspace_root: String,
    pub contracts_ready: bool,
}

/// Typed surface-coder input: which surface family to emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceCoderInput {
    pub surface: String,
}

/// Typed surface-coder output: emitted files (all inside `OWNS`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceCoderOutput {
    pub files: Vec<String>,
    pub bezel: &'static str,
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
    Escalation { agent: "kimi-surface-motion-coder", layer, strikes, route }
}

/// The compiled subagent: state + disposers. Drop-safe only via explicit `dispose`.
#[derive(Debug)]
pub struct SurfaceMotionCoder {
    pub state: Lifecycle,
    pub disposers: Vec<Disposer>,
    pub released_log: Vec<&'static str>,
}

impl SurfaceMotionCoder {
    /// create: register all resources up front, all unreleased.
    pub fn create() -> Self {
        SurfaceMotionCoder {
            state: Lifecycle::Created,
            disposers: vec![
                Disposer { name: "theme-sheet", released: false },
                Disposer { name: "bezel-components", released: false },
                Disposer { name: "motion-choreography", released: false },
            ],
            released_log: Vec::new(),
        }
    }

    /// inject: bind runtime context. Requires contracts on disk (architect ran first).
    pub fn inject(&mut self, ctx: &RuntimeContext) -> Result<(), LifecycleError> {
        if self.state != Lifecycle::Created {
            return Err(LifecycleError {
                agent: "kimi-surface-motion-coder",
                state: self.state,
                reason: "inject requires Created",
            });
        }
        if !ctx.contracts_ready {
            return Err(LifecycleError {
                agent: "kimi-surface-motion-coder",
                state: self.state,
                reason: "contracts gate: src/types + src/store must exist first",
            });
        }
        self.state = Lifecycle::Injected;
        Ok(())
    }

    /// run: emit typed plan. Every emitted file must sit inside `OWNS` (self-enforced).
    pub fn run(&mut self, input: &SurfaceCoderInput) -> Result<SurfaceCoderOutput, LifecycleError> {
        if self.state != Lifecycle::Injected && self.state != Lifecycle::Running {
            return Err(LifecycleError {
                agent: "kimi-surface-motion-coder",
                state: self.state,
                reason: "run requires Injected",
            });
        }
        self.state = Lifecycle::Running;
        let files: Vec<String> = vec![
            format!("src/components/layout/{}_shell.tsx", input.surface),
            format!("src/components/ui/{}_panel.tsx", input.surface),
            format!("src/styles/{}_tokens.css", input.surface),
        ];
        for f in files.iter() {
            if !SurfaceMotionCoder::may_write(f) {
                return Err(LifecycleError {
                    agent: "kimi-surface-motion-coder",
                    state: self.state,
                    reason: "emitted file outside OWNS",
                });
            }
        }
        Ok(SurfaceCoderOutput { files, bezel: "double-bezel-doppelrand" })
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

    /// Ownership gate: writable iff under OWNS and never under contracts/A6/QA paths.
    pub fn may_write(path: &str) -> bool {
        let owned: bool = OWNS.iter().any(|p| path.starts_with(p));
        let forbidden: bool = NEVER_WRITES.iter().any(|p| path.starts_with(p));
        owned && !forbidden
    }
}

fn main() {
    let mut agent: SurfaceMotionCoder = SurfaceMotionCoder::create();
    let ctx: RuntimeContext =
        RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
    if let Err(e) = agent.inject(&ctx) {
        eprintln!("inject failed: {:?}", e);
        std::process::exit(1);
    }
    let input: SurfaceCoderInput = SurfaceCoderInput { surface: "bento".to_string() };
    match agent.run(&input) {
        Ok(o) => println!("emit {:?} bezel={}", o.files, o.bezel),
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

    fn injected() -> SurfaceMotionCoder {
        let mut a: SurfaceMotionCoder = SurfaceMotionCoder::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
        assert_eq!(a.inject(&ctx), Ok(()));
        a
    }

    #[test]
    fn lifecycle_create_inject_run_dispose_no_leak() {
        let mut a: SurfaceMotionCoder = SurfaceMotionCoder::create();
        assert_eq!(a.state, Lifecycle::Created);
        assert!(!a.leak_check());
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: true };
        assert_eq!(a.inject(&ctx), Ok(()));
        let out: SurfaceCoderOutput = a
            .run(&SurfaceCoderInput { surface: "bento".to_string() })
            .expect("run ok");
        assert_eq!(out.bezel, "double-bezel-doppelrand");
        assert!(out.files.iter().all(|f| SurfaceMotionCoder::may_write(f)));
        a.dispose();
        assert!(a.leak_check());
        assert_eq!(
            a.released_log,
            vec!["motion-choreography", "bezel-components", "theme-sheet"]
        );
    }

    #[test]
    fn run_before_inject_is_rejected() {
        let mut a: SurfaceMotionCoder = SurfaceMotionCoder::create();
        let err: LifecycleError = a
            .run(&SurfaceCoderInput { surface: "x".to_string() })
            .expect_err("must reject");
        assert_eq!(err.reason, "run requires Injected");
    }

    #[test]
    fn inject_requires_contracts_gate() {
        let mut a: SurfaceMotionCoder = SurfaceMotionCoder::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), contracts_ready: false };
        let err: LifecycleError = a.inject(&ctx).expect_err("must reject");
        assert_eq!(err.state, Lifecycle::Created);
    }

    #[test]
    fn dispose_is_idempotent_and_terminal() {
        let mut a: SurfaceMotionCoder = injected();
        a.dispose();
        a.dispose();
        assert!(a.leak_check());
        let err: LifecycleError = a
            .run(&SurfaceCoderInput { surface: "x".to_string() })
            .expect_err("disposed is terminal");
        assert_eq!(err.state, Lifecycle::Disposed);
    }

    #[test]
    fn ownership_allows_surface_paths_only() {
        assert!(SurfaceMotionCoder::may_write("src/components/layout/shell.tsx"));
        assert!(SurfaceMotionCoder::may_write("src/components/ui/panel.tsx"));
        assert!(SurfaceMotionCoder::may_write("src/styles/tokens.css"));
        for p in NEVER_WRITES.iter() {
            assert!(!SurfaceMotionCoder::may_write(&format!("{}file.ts", p)), "deny {}", p);
        }
        // Cross-slice denials: architect + A6 families.
        assert!(!SurfaceMotionCoder::may_write("src/types/app.ts"));
        assert!(!SurfaceMotionCoder::may_write("src/store/use_app_store.ts"));
        assert!(!SurfaceMotionCoder::may_write("src/components/canvas/scene.tsx"));
        assert!(!SurfaceMotionCoder::may_write("src/components/graph/knowledge.tsx"));
        assert!(!SurfaceMotionCoder::may_write("src/components/dag/flow.tsx"));
        assert!(!SurfaceMotionCoder::may_write("src/components/workbench/panel.tsx"));
        assert!(!SurfaceMotionCoder::may_write("src/components/editors/code.tsx"));
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
        dir.push("a5_surface_unload_proof");
        let _ = std::fs::create_dir_all(&dir);
        let mut a: SurfaceMotionCoder = SurfaceMotionCoder::create();
        let ctx: RuntimeContext = RuntimeContext {
            workspace_root: dir.to_string_lossy().into_owned(),
            contracts_ready: true,
        };
        assert_eq!(a.inject(&ctx), Ok(()));
        let _ = a.run(&SurfaceCoderInput { surface: "bento".to_string() }).expect("run ok");
        a.dispose();
        assert!(a.leak_check());
        let entries: Vec<_> = std::fs::read_dir(&dir)
            .expect("temp dir readable")
            .collect::<Result<Vec<_>, _>>()
            .expect("entries readable");
        assert!(entries.is_empty(), "surface coder must leave zero files");
        std::fs::remove_dir(&dir).expect("temp dir cleanup");
        assert!(!dir.exists());
    }

    #[test]
    fn escalation_routes_on_strike_exhaustion() {
        let e1: Escalation = escalate(Judge::ViteBuild, 1);
        assert_eq!(e1.route, EscalationRoute::Archive);
        let e2: Escalation = escalate(Judge::ViteBuild, 2);
        assert_eq!(e2.route, EscalationRoute::HumanQueue);
        assert_eq!(e2.agent, "kimi-surface-motion-coder");
    }
}
