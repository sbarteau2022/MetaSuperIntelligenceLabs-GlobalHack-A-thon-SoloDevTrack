//! kimi-visual-qa-critic — compiled subagent lifecycle (sole verifier).
//!
//! Agent A6 (Subagents-B), Wave 2, slice 6/6. Disjoint from Agent A5 slices 1-3.
//! Source: `subagent_definitions.json` entry `kimi-visual-qa-critic`
//!   + `SUBAGENT_SWARM_LOGIC.md` section 6 (READ-ONLY refs, never modified).
//!
//! Role: Vision-in-the-Loop Browser QA Critic — the SOLE verifier with live tools.
//! Scope (verdicts over): rendered layout geometry, console/network cleanliness,
//!   responsive collapse, interactive states. May patch CSS/TSX defects it finds, but
//!   every critic-introduced change MUST re-enter judge layer (b)
//!   (GLOBAL-LOGIC.md section 6, VERIFICATION RULE: "The critic may patch, but a
//!   critic-introduced change re-enters layer (b)"; LOOP EXIT: patch and re-enter
//!   at the FAILING layer, not from the top). Proof: `critic_patch_reenters_gate`
//!   test below — a critic fix forces `needs_layer_b_reentry == true` and the
//!   verdict only becomes `Pass` after both the patched layer AND layer (b)
//!   report clean.
//! Write scope: patch files ONLY (CSS/TSX defects under coder-owned paths). Verdicts
//!   live here; coder paths are read except when patching a found defect.
//! Tools (verbatim from JSON): EnableWriteTools=true, EnableMCPTools=true
//!   (Requires `chrome-devtools-mcp`), EnableSubagentTools=false.
//!   This is the SOLE MCP holder: no other B agent (graph/workbench) and no A5
//!   agent holds MCP. Coders NEVER self-approve; only deterministic gates and
//!   this critic may pass work.
//! Judges (the critic IS the last layer): TsgoTsc -> AuditSh -> ViteBuild ->
//!   CriticGate(self). Escalation: 2 consecutive failures at the same layer ->
//!   human queue.
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
/// SOLE MCP holder: `mcp: true` here, `false` everywhere else (B + A5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolAllowlist {
    pub write: bool,
    pub mcp: bool,
    pub subagent: bool,
}

/// Verbatim flags for this subagent. SOLE MCP holder.
pub static TOOL_ALLOWLIST: ToolAllowlist = ToolAllowlist { write: true, mcp: true, subagent: false };

/// MCP tool this critic requires. No other compiled subagent may name it.
pub static MCP_TOOL: &str = "chrome-devtools-mcp";

/// Verdict paths this critic owns (writes verdicts here; coder paths read-only
/// except defect patches, which must re-enter layer (b)).
pub static OWNS: &[&str] = &["qa/verdicts/", "qa/screenshots/"];

/// Judge layers this critic enforces. `CriticGate` is this agent itself.
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

/// Desktop and mobile viewports the critic must capture (from SystemPrompt).
pub static VIEWPORT_DESKTOP: (u32, u32) = (1440, 900);
pub static VIEWPORT_MOBILE: (u32, u32) = (390, 844);

/// Zoom threshold for semantic-zoom LOD label checks (graph scenes).
pub const SEMANTIC_ZOOM_K: f32 = 1.35;

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
    pub preview_url: String,
}

/// Typed critic input: one verification pass over a live preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticInput {
    pub console_errors: u32,
    pub network_404s: u32,
    pub horizontal_overflow: bool,
    pub zero_dimension_canvases: u32,
    pub patched: bool,
}

/// Typed critic output: verdict plus whether a layer-(b) re-entry is still owed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticVerdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CriticOutput {
    pub verdict: CriticVerdict,
    pub needs_layer_b_reentry: bool,
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
    Escalation { agent: "kimi-visual-qa-critic", layer, strikes, route }
}

/// The compiled subagent: state + disposers. Drop-safe only via explicit `dispose`.
#[derive(Debug)]
pub struct VisualQaCritic {
    pub state: Lifecycle,
    pub disposers: Vec<Disposer>,
    pub released_log: Vec<&'static str>,
    pub layer_b_clean: bool,
}

impl VisualQaCritic {
    /// create: register all resources up front, all unreleased.
    pub fn create() -> Self {
        VisualQaCritic {
            state: Lifecycle::Created,
            disposers: vec![
                Disposer { name: "devtools-session", released: false },
                Disposer { name: "screenshot-buffers", released: false },
                Disposer { name: "patch-workspace", released: false },
            ],
            released_log: Vec::new(),
            layer_b_clean: false,
        }
    }

    /// inject: bind runtime context. Requires a live preview URL to verify.
    pub fn inject(&mut self, ctx: &RuntimeContext) -> Result<(), LifecycleError> {
        if self.state != Lifecycle::Created {
            return Err(LifecycleError {
                agent: "kimi-visual-qa-critic",
                state: self.state,
                reason: "inject requires Created",
            });
        }
        if ctx.preview_url.is_empty() {
            return Err(LifecycleError {
                agent: "kimi-visual-qa-critic",
                state: self.state,
                reason: "critic gate: live preview URL required",
            });
        }
        self.state = Lifecycle::Injected;
        Ok(())
    }

    /// run: one verification pass. Clean evidence + (patched => layer-b clean) = Pass.
    /// A critic patch ALWAYS forces `needs_layer_b_reentry` until `mark_layer_b_clean`.
    pub fn run(&mut self, input: &CriticInput) -> Result<CriticOutput, LifecycleError> {
        if self.state != Lifecycle::Injected && self.state != Lifecycle::Running {
            return Err(LifecycleError {
                agent: "kimi-visual-qa-critic",
                state: self.state,
                reason: "run requires Injected",
            });
        }
        self.state = Lifecycle::Running;
        let evidence_clean: bool = input.console_errors == 0
            && input.network_404s == 0
            && !input.horizontal_overflow
            && input.zero_dimension_canvases == 0;
        if !evidence_clean {
            return Ok(CriticOutput { verdict: CriticVerdict::Fail, needs_layer_b_reentry: true });
        }
        if input.patched && !self.layer_b_clean {
            return Ok(CriticOutput { verdict: CriticVerdict::Fail, needs_layer_b_reentry: true });
        }
        Ok(CriticOutput { verdict: CriticVerdict::Pass, needs_layer_b_reentry: false })
    }

    /// Record that the critic-patched tree re-entered layer (b) and came back clean.
    pub fn mark_layer_b_clean(&mut self) {
        self.layer_b_clean = true;
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
}

fn main() {
    let mut agent: VisualQaCritic = VisualQaCritic::create();
    let ctx: RuntimeContext = RuntimeContext {
        workspace_root: ".".to_string(),
        preview_url: "http://localhost:5173".to_string(),
    };
    if let Err(e) = agent.inject(&ctx) {
        eprintln!("inject failed: {:?}", e);
        std::process::exit(1);
    }
    let clean: CriticInput = CriticInput {
        console_errors: 0,
        network_404s: 0,
        horizontal_overflow: false,
        zero_dimension_canvases: 0,
        patched: false,
    };
    match agent.run(&clean) {
        Ok(o) => println!("verdict {:?} reentry={}", o.verdict, o.needs_layer_b_reentry),
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

    fn clean_input() -> CriticInput {
        CriticInput {
            console_errors: 0,
            network_404s: 0,
            horizontal_overflow: false,
            zero_dimension_canvases: 0,
            patched: false,
        }
    }

    fn injected() -> VisualQaCritic {
        let mut a: VisualQaCritic = VisualQaCritic::create();
        let ctx: RuntimeContext = RuntimeContext {
            workspace_root: ".".to_string(),
            preview_url: "http://localhost:5173".to_string(),
        };
        assert_eq!(a.inject(&ctx), Ok(()));
        a
    }

    #[test]
    fn lifecycle_create_inject_run_dispose_no_leak() {
        let a: VisualQaCritic = VisualQaCritic::create();
        assert_eq!(a.state, Lifecycle::Created);
        assert!(!a.leak_check());
        let mut b: VisualQaCritic = injected();
        let out: CriticOutput = b.run(&clean_input()).expect("run ok");
        assert_eq!(out.verdict, CriticVerdict::Pass);
        assert!(!out.needs_layer_b_reentry);
        drop(a);
        let mut c: VisualQaCritic = injected();
        c.dispose();
        assert!(c.leak_check());
        assert_eq!(
            c.released_log,
            vec!["patch-workspace", "screenshot-buffers", "devtools-session"]
        );
    }

    #[test]
    fn run_before_inject_is_rejected() {
        let mut a: VisualQaCritic = VisualQaCritic::create();
        let err: LifecycleError = a.run(&clean_input()).expect_err("must reject");
        assert_eq!(err.reason, "run requires Injected");
    }

    #[test]
    fn inject_requires_preview_url() {
        let mut a: VisualQaCritic = VisualQaCritic::create();
        let ctx: RuntimeContext =
            RuntimeContext { workspace_root: ".".to_string(), preview_url: String::new() };
        let err: LifecycleError = a.inject(&ctx).expect_err("must reject");
        assert_eq!(err.state, Lifecycle::Created);
    }

    #[test]
    fn dirty_evidence_fails_and_demands_reentry() {
        let mut a: VisualQaCritic = injected();
        let dirty: CriticInput = CriticInput { console_errors: 2, ..clean_input() };
        let out: CriticOutput = a.run(&dirty).expect("run ok");
        assert_eq!(out.verdict, CriticVerdict::Fail);
        assert!(out.needs_layer_b_reentry);
        let overflow: CriticInput = CriticInput { horizontal_overflow: true, ..clean_input() };
        let out2: CriticOutput = a.run(&overflow).expect("run ok");
        assert_eq!(out2.verdict, CriticVerdict::Fail);
    }

    #[test]
    fn critic_patch_reenters_gate() {
        // Mirrors GLOBAL-LOGIC section 6 VERIFICATION RULE: a critic-introduced
        // change re-enters layer (b); Pass is possible only after the patched
        // tree reports layer-(b) clean. Patch -> Fail(re-entry owed) ->
        // mark_layer_b_clean -> Pass. Skipping re-entry NEVER passes.
        let mut a: VisualQaCritic = injected();
        let patched: CriticInput = CriticInput { patched: true, ..clean_input() };
        let first: CriticOutput = a.run(&patched).expect("run ok");
        assert_eq!(first.verdict, CriticVerdict::Fail);
        assert!(first.needs_layer_b_reentry);
        let second: CriticOutput = a.run(&patched).expect("run ok");
        assert_eq!(second.verdict, CriticVerdict::Fail);
        a.mark_layer_b_clean();
        let third: CriticOutput = a.run(&patched).expect("run ok");
        assert_eq!(third.verdict, CriticVerdict::Pass);
        assert!(!third.needs_layer_b_reentry);
    }

    #[test]
    fn dispose_is_idempotent_and_terminal() {
        let mut a: VisualQaCritic = injected();
        a.dispose();
        a.dispose();
        assert!(a.leak_check());
        let err: LifecycleError = a.run(&clean_input()).expect_err("disposed is terminal");
        assert_eq!(err.state, Lifecycle::Disposed);
    }

    #[test]
    fn tool_allowlist_preserved_sole_mcp_holder() {
        assert_eq!(
            TOOL_ALLOWLIST,
            ToolAllowlist { write: true, mcp: true, subagent: false }
        );
        assert_eq!(MCP_TOOL, "chrome-devtools-mcp");
    }

    #[test]
    fn viewports_cover_desktop_and_mobile() {
        assert_eq!(VIEWPORT_DESKTOP, (1440, 900));
        assert_eq!(VIEWPORT_MOBILE, (390, 844));
        assert!((SEMANTIC_ZOOM_K - 1.35).abs() < f32::EPSILON);
        assert_eq!(JUDGES.len(), 4);
    }

    #[test]
    fn escalation_routes_on_strike_exhaustion() {
        let e1: Escalation = escalate(Judge::CriticGate, 1);
        assert_eq!(e1.route, EscalationRoute::Archive);
        let e2: Escalation = escalate(Judge::CriticGate, 2);
        assert_eq!(e2.route, EscalationRoute::HumanQueue);
        assert_eq!(e2.agent, "kimi-visual-qa-critic");
    }
}
