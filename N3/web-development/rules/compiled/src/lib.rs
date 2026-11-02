//! Compiled prose-law predicates — Agent C4 build 2026-10-31.
//!
//! Sources (read-only):
//! - `/Users/stewartbarteau/.Trash/skills-desktop-web-development-20261030/RULES.md`
//!   (rules 1–75 + 60a; §§0–11 with §8 absent)
//! - `.../GLOBAL-LOGIC.md` (§§3–9)
//! - `.../kimi-frontend-kit/rules/rules.md` (5 rules)
//! - `.../kimi-frontend-kit/rules/AGENTS.md` (§§1–3)
//!
//! Modality law: MUST → [`Verdict::Deny`] (+veto), SHOULD → [`Verdict::Warn`]
//! (+record), MAY → [`Verdict::Allow`]. Every predicate carries scope,
//! binding, and consequence. See `CONTRADICTION_LEDGER.md` for adjudications.

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Deontic modality of a source law.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    /// MUST: violation blocks. Compiled consequence: Deny + veto.
    Must,
    /// SHOULD: violation warns and is recorded. Does not veto.
    Should,
    /// MAY: permitted. Compiled consequence: Allow.
    May,
}

/// Enforcement verdict of a compiled predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Block the action; vetoes completion ("done").
    Deny,
    /// Record a pending issue; blocks only via the stop-gate accumulation.
    Warn,
    /// Explicitly permitted.
    Allow,
}

/// Where a law applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Applies everywhere in the scaffold.
    Global,
    /// Applies to tree layout and placement writes.
    Structure,
    /// Applies to UI surface and markup writes.
    Frontend,
    /// Applies to design-token and stylesheet writes.
    Tokens,
    /// Applies to endpoint, middleware, and API writes.
    Backend,
    /// Applies to schema and migration writes.
    Data,
    /// Applies to test coverage and checker-output claims.
    Testing,
    /// Applies to secrets, auth, and release-gate writes.
    Security,
    /// Applies to multi-agent routing, partitioning, and delegation.
    Swarm,
    /// Applies to workstream documentation completeness.
    Docs,
    /// Applies to conformance gates and checker invocations.
    Discipline,
    /// Applies to SKILL.md authoring and skeleton order.
    SkillShape,
    /// Applies to hook events and the shared rule table.
    HookLifecycle,
    /// Applies to retrieval-first and fail-fast working method.
    Workflow,
    /// Applies to pre-generation calibration declarations.
    Calibration,
    /// Applies to the kimi 5-stage execution loop.
    KimiLoop,
    /// Applies to kimi anti-slop visual and surface rules.
    KimiVisual,
    /// Applies to kimi frontier-stack pins (React/Tailwind/TS/etc).
    KimiStack,
}

/// Binding force of a compiled predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// Always loaded, always enforced.
    Always,
    /// Enforced when the trigger subsystem / file set is touched.
    OnTrigger,
    /// Enforced at the completion gate.
    AtGate,
}

/// What happens on violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Consequence {
    /// Deny the write + veto "done" until resolved.
    DenyAndVeto,
    /// Append to the pending-issues store; stop-gate converts to veto.
    WarnAndRecord,
    /// No action; explicitly allowed.
    Allow,
}

/// A single compiled law predicate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Predicate {
    /// Stable id, e.g. "R01", "G6-VERIFY", "K2-BAN".
    pub id: &'static str,
    /// Source rule reference, e.g. "RULES.md rule 1".
    pub source: &'static str,
    /// Deontic modality of the source law.
    pub modality: Modality,
    /// Subsystem or surface the law applies to.
    pub scope: Scope,
    /// When the predicate is enforced.
    pub binding: Binding,
    /// What a violation produces.
    pub consequence: Consequence,
    /// Short normative statement (mirrors source wording, not a rewrite).
    pub law: &'static str,
}

impl Predicate {
    /// The verdict this predicate yields on violation.
    #[must_use]
    pub const fn verdict(&self) -> Verdict {
        match self.modality {
            Modality::Must => Verdict::Deny,
            Modality::Should => Verdict::Warn,
            Modality::May => Verdict::Allow,
        }
    }
}

// ---------------------------------------------------------------------------
// Family A — RULES.md §§0–11 (rules 1–75 + 60a; §8 absent by source)
// ---------------------------------------------------------------------------

/// RULES §0 Global Constraints (rules 1–5). Scope: Global. Binding: Always.
pub const RULES_S0: &[Predicate] = &[
    Predicate { id: "R01", source: "RULES.md rule 1", modality: Modality::Must, scope: Scope::Global, binding: Binding::Always, consequence: Consequence::DenyAndVeto, law: "Label every claim VERIFIED / REPORTED / INFERRED / PROPOSED." },
    Predicate { id: "R02", source: "RULES.md rule 2", modality: Modality::Must, scope: Scope::Global, binding: Binding::Always, consequence: Consequence::DenyAndVeto, law: "Date every document; name exact paths and versions." },
    Predicate { id: "R03", source: "RULES.md rule 3", modality: Modality::Must, scope: Scope::Global, binding: Binding::Always, consequence: Consequence::DenyAndVeto, law: "No performance numbers without hardware, settings, workload, baseline." },
    Predicate { id: "R04", source: "RULES.md rule 4", modality: Modality::Must, scope: Scope::Global, binding: Binding::Always, consequence: Consequence::DenyAndVeto, law: "No silent rewrites of wrong claims — append a dated correction entry." },
    Predicate { id: "R05", source: "RULES.md rule 5", modality: Modality::Must, scope: Scope::Global, binding: Binding::Always, consequence: Consequence::DenyAndVeto, law: "No bulk-delete, bulk-rewrite, or batch-claim; per-item disposition only." },
];

/// RULES §1 Structure & Placement (rules 6–10). Binding: OnTrigger (tree writes).
pub const RULES_S1: &[Predicate] = &[
    Predicate { id: "R06", source: "RULES.md rule 6", modality: Modality::Must, scope: Scope::Structure, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Place code in the matching subsystem folder; create + README-update in the same change if none fits." },
    Predicate { id: "R07", source: "RULES.md rule 7", modality: Modality::Must, scope: Scope::Structure, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Keep shared contracts in ONE canonical location; never duplicate a schema." },
    Predicate { id: "R08", source: "RULES.md rule 8", modality: Modality::Must, scope: Scope::Structure, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No source code in docs/, templates/, or frontend-design-language/references/." },
    Predicate { id: "R09", source: "RULES.md rule 9", modality: Modality::Must, scope: Scope::Security, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No committed secrets, .env files, keys, or tokens; .env.example only." },
    Predicate { id: "R10", source: "RULES.md rule 10", modality: Modality::Must, scope: Scope::Structure, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Keep every directory non-empty in git via .gitkeep where no file exists yet." },
];

/// RULES §2 Frontend (rules 11–24). Binding: OnTrigger (UI surface writes).
pub const RULES_S2: &[Predicate] = &[
    Predicate { id: "R11", source: "RULES.md rule 11", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Native HTML before ARIA; no <div role=\"button\">, use <button>." },
    Predicate { id: "R12", source: "RULES.md rule 12", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No redundant ARIA (role=list on ul, role=button on button)." },
    Predicate { id: "R13", source: "RULES.md rule 13", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "One h1 per page, sequential headings, no skipped levels." },
    Predicate { id: "R14", source: "RULES.md rule 14", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No div/span styled to look like headings." },
    Predicate { id: "R15", source: "RULES.md rule 15", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Real table + th scope for tabular data; never tables for layout." },
    Predicate { id: "R16", source: "RULES.md rule 16", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Skip link to a focusable target (tabindex=-1) on pages with repeated nav." },
    Predicate { id: "R17", source: "RULES.md rule 17", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Accessible name on every interactive element." },
    Predicate { id: "R18", source: "RULES.md rule 18", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No aria-hidden=true on focusable elements or their ancestors." },
    Predicate { id: "R19", source: "RULES.md rule 19", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Respect prefers-reduced-motion for all non-essential animation." },
    Predicate { id: "R20", source: "RULES.md rule 20", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No nontrivial inline styling values — use design tokens." },
    Predicate { id: "R21", source: "RULES.md rule 21", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Contrast >= 4.5:1 body text; >= 3:1 large text and UI boundaries." },
    Predicate { id: "R22", source: "RULES.md rule 22", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Interactive targets >= 24x24 CSS px (44x44 preferred on touch)." },
    Predicate { id: "R23", source: "RULES.md rule 23", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Support light and dark themes unless a product decision is recorded." },
    Predicate { id: "R24", source: "RULES.md rule 24", modality: Modality::Must, scope: Scope::Frontend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No !important outside the utilities cascade layer." },
];

/// RULES §3 Design Language & Tokens (rules 25–35). Binding: OnTrigger (style writes).
pub const RULES_S3: &[Predicate] = &[
    Predicate { id: "R25", source: "RULES.md rule 25", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Use the 4-tier token model: literal -> semantic -> UI -> component." },
    Predicate { id: "R26", source: "RULES.md rule 26", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Reference semantic tokens in component code; literals only inside the token file." },
    Predicate { id: "R27", source: "RULES.md rule 27", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No hardcoded colors, spacing, radii, or durations in component styles." },
    Predicate { id: "R28", source: "RULES.md rule 28", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Define layer order explicitly: reset, base, theme, components, utilities." },
    Predicate { id: "R29", source: "RULES.md rule 29", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Prefer logical properties over physical ones." },
    Predicate { id: "R30", source: "RULES.md rule 30", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Use currentColor over a color variable when intent is identical." },
    Predicate { id: "R31", source: "RULES.md rule 31", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No BEM-style naming to manage specificity — cascade layers and :where()." },
    Predicate { id: "R32", source: "RULES.md rule 32", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No global resets (* { ... })." },
    Predicate { id: "R33", source: "RULES.md rule 33", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Use :not() for exclusion instead of override-then-restore, where intent is exclusion." },
    Predicate { id: "R34", source: "RULES.md rule 34", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Prefer @scope over :not() for excluding deeply nested subtrees." },
    Predicate { id: "R35", source: "RULES.md rule 35", modality: Modality::Must, scope: Scope::Tokens, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Feature-detect newly-available CSS before making it load-bearing; provide a fallback." },
];

/// RULES §4 Backend, Middleware & API (rules 36–44). Binding: OnTrigger (endpoint writes).
pub const RULES_S4: &[Predicate] = &[
    Predicate { id: "R36", source: "RULES.md rule 36", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Validate all input at the edge; never trust a client." },
    Predicate { id: "R37", source: "RULES.md rule 37", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Consistent error envelope across every endpoint." },
    Predicate { id: "R38", source: "RULES.md rule 38", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No leaked stack traces, SQL, or internal paths in production error responses." },
    Predicate { id: "R39", source: "RULES.md rule 39", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Authorize every request server-side; UI hiding is not authorization." },
    Predicate { id: "R40", source: "RULES.md rule 40", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Every state-changing endpoint idempotent or explicitly replay-guarded." },
    Predicate { id: "R41", source: "RULES.md rule 41", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Paginate any list endpoint that can grow without bound." },
    Predicate { id: "R42", source: "RULES.md rule 42", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Version the API contract (/v1); record breaking changes in api-integrations/docs/." },
    Predicate { id: "R43", source: "RULES.md rule 43", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Middleware order explicit and documented in middleware/README.md." },
    Predicate { id: "R44", source: "RULES.md rule 44", modality: Modality::Must, scope: Scope::Backend, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No logging of secrets, session tokens, or full card numbers." },
];

/// RULES §5 Data (rules 45–48). Binding: OnTrigger (schema/migration writes).
pub const RULES_S5: &[Predicate] = &[
    Predicate { id: "R45", source: "RULES.md rule 45", modality: Modality::Must, scope: Scope::Data, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Version every schema change as a forward-only migration with a documented rollback." },
    Predicate { id: "R46", source: "RULES.md rule 46", modality: Modality::Must, scope: Scope::Data, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Migrations safe to run while the old code is still serving traffic." },
    Predicate { id: "R47", source: "RULES.md rule 47", modality: Modality::Must, scope: Scope::Data, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No hand-edited production rows; use a reviewed script under database-data/." },
    Predicate { id: "R48", source: "RULES.md rule 48", modality: Modality::Must, scope: Scope::Data, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Index every field used for filtering, sorting, or joining on hot paths." },
];

/// RULES §6 Testing & Quality (rules 49–52; rule 52 is the sole SHOULD).
/// Binding: AtGate for MUSTs; OnTrigger for the SHOULD advisory.
pub const RULES_S6: &[Predicate] = &[
    Predicate { id: "R49", source: "RULES.md rule 49", modality: Modality::Must, scope: Scope::Testing, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "A failing test reproducing a bug must precede the fix (regression guard)." },
    Predicate { id: "R50", source: "RULES.md rule 50", modality: Modality::Must, scope: Scope::Testing, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Cover error and empty states, not only the happy path." },
    Predicate { id: "R51", source: "RULES.md rule 51", modality: Modality::Must, scope: Scope::Testing, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "No pass-claim without the checker's output shown." },
    Predicate { id: "R52", source: "RULES.md rule 52", modality: Modality::Should, scope: Scope::Testing, binding: Binding::OnTrigger, consequence: Consequence::WarnAndRecord, law: "Keep unit tests free of network and filesystem access." },
];

/// RULES §7 Security (rules 53–57). Binding: OnTrigger; R57 also AtGate (pre-release).
pub const RULES_S7: &[Predicate] = &[
    Predicate { id: "R53", source: "RULES.md rule 53", modality: Modality::Must, scope: Scope::Security, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Secrets in a secret manager or .env outside version control." },
    Predicate { id: "R54", source: "RULES.md rule 54", modality: Modality::Must, scope: Scope::Security, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Hash passwords with a memory-hard KDF (Argon2id/bcrypt) and per-user salt." },
    Predicate { id: "R55", source: "RULES.md rule 55", modality: Modality::Must, scope: Scope::Security, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Rotate session identifiers on privilege change (login, logout, role change)." },
    Predicate { id: "R56", source: "RULES.md rule 56", modality: Modality::Must, scope: Scope::Security, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Set HttpOnly, Secure, and SameSite on session cookies." },
    Predicate { id: "R57", source: "RULES.md rule 57", modality: Modality::Must, scope: Scope::Security, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "OWASP Top 10 review before any public release." },
];

/// RULES §9 Partition & Swarm Protocol (rules 58–63 + 60a).
/// Binding: OnTrigger for routing/partition; AtGate for the completion veto.
pub const RULES_S9: &[Predicate] = &[
    Predicate { id: "R58", source: "RULES.md rule 58", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Follow GLOBAL-LOGIC §6 swarm routing for tasks spanning >3 files or >1 engine." },
    Predicate { id: "R59", source: "RULES.md rule 59", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No parallel coders until shared contracts exist on disk and are reviewed." },
    Predicate { id: "R60", source: "RULES.md rule 60", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No coder self-approval; only deterministic gates and the critic pass work." },
    Predicate { id: "R60a", source: "RULES.md rule 60a", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Partition into small whole vertical slices, one mind each, front-to-back; never horizontal layers." },
    Predicate { id: "R61", source: "RULES.md rule 61", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Declare the one-line calibration before generating (dials or RISK/BLAST-RADIUS/CONTRACT-FIRST)." },
    Predicate { id: "R62", source: "RULES.md rule 62", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Condition-driven loop: on failure re-enter at the FAILING layer, never from the top." },
    Predicate { id: "R63", source: "RULES.md rule 63", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "No completion claim while the stop-gate reports pending issues; the gate vetoes done." },
];

/// RULES §10 Documentation (rules 64–68). Binding: AtGate (docs completeness).
pub const RULES_S10: &[Predicate] = &[
    Predicate { id: "R64", source: "RULES.md rule 64", modality: Modality::Must, scope: Scope::Docs, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Date every document and state what was NOT checked." },
    Predicate { id: "R65", source: "RULES.md rule 65", modality: Modality::Must, scope: Scope::Docs, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Record failures — a recorded failure beats an asserted success." },
    Predicate { id: "R66", source: "RULES.md rule 66", modality: Modality::Must, scope: Scope::Docs, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Keep the five-file set per workstream (Design, Pre-Registration, CLAIMS_REGISTER, Successes, Failures)." },
    Predicate { id: "R67", source: "RULES.md rule 67", modality: Modality::Must, scope: Scope::Docs, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Mermaid for flow diagrams; no hardcoded hex in diagrams." },
    Predicate { id: "R68", source: "RULES.md rule 68", modality: Modality::Must, scope: Scope::Docs, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Append dated correction entries; never rewrite a wrong claim in place." },
];

/// RULES §11 Discipline Enforcement (rules 69–75). Binding: AtGate, except R73–R75 OnTrigger.
pub const RULES_S11: &[Predicate] = &[
    Predicate { id: "R69", source: "RULES.md rule 69", modality: Modality::Must, scope: Scope::Discipline, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Run the conformance gate (discipline.py exit 0) before claiming done; paste into release checklist." },
    Predicate { id: "R70", source: "RULES.md rule 70", modality: Modality::Must, scope: Scope::Discipline, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Run the batch audit (audit.sh) in the same change; exit 0." },
    Predicate { id: "R71", source: "RULES.md rule 71", modality: Modality::Must, scope: Scope::Discipline, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "No completion while the stop-gate reports pending issues; skipping the gate is a violation." },
    Predicate { id: "R72", source: "RULES.md rule 72", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Task subagents as siloed whole-slice owners behind one contract gate; coders MUST NOT self-approve." },
    Predicate { id: "R73", source: "RULES.md rule 73", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Scope every delegation to its partition; no duplicate delegations for the same unresolved subproblem." },
    Predicate { id: "R74", source: "RULES.md rule 74", modality: Modality::Must, scope: Scope::Discipline, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Verify agent-reported structural claims independently before recording as VERIFIED (REPORTED until checked)." },
    Predicate { id: "R75", source: "RULES.md rule 75", modality: Modality::Must, scope: Scope::Discipline, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No invented scripts, hooks, or gates; §6 of every SKILL.md references only audit.sh, discipline.py, rules-table.json." },
];

/// All RULES predicates in source order (76 entries: rules 1–75 + 60a; §8 absent).
pub const ALL_RULES: &[&[Predicate]] =
    &[RULES_S0, RULES_S1, RULES_S2, RULES_S3, RULES_S4, RULES_S5, RULES_S6, RULES_S7, RULES_S9, RULES_S10, RULES_S11];

// ---------------------------------------------------------------------------
// Family B — GLOBAL-LOGIC.md §§3–9
// ---------------------------------------------------------------------------

/// GLOBAL-LOGIC §3: universal skill-file skeleton (mandatory shape §§1–9 order;
/// flat-tier exception for database-data / api-integrations / devops-deployment / security-auth).
pub const GLOGIC_S3: &[Predicate] = &[
    Predicate { id: "G3-SKEL", source: "GLOBAL-LOGIC.md §3", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Every SKILL.md follows the §§1–9 skeleton in order (full tier)." },
    Predicate { id: "G3-FLAT", source: "GLOBAL-LOGIC.md §3 flat-tier exception", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Flat tier allowed ONLY for database-data, api-integrations, devops-deployment, security-auth (frontmatter + Conventions + examples)." },
    Predicate { id: "G3-DATA", source: "GLOBAL-LOGIC.md §3 + §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Skill files are query interfaces over data: tables or scripts where a section can be one." },
    Predicate { id: "G3-BAN", source: "GLOBAL-LOGIC.md §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Ban-lists use exact values, never vibes." },
    Predicate { id: "G3-GATE", source: "GLOBAL-LOGIC.md §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Every skill ends in a checkbox gate; nothing ships with an unchecked box." },
    Predicate { id: "G3-PATH", source: "GLOBAL-LOGIC.md §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Scripts invoked by absolute skill path; CWD stays at project root." },
    Predicate { id: "G3-DEPTH", source: "GLOBAL-LOGIC.md §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "RULES.md is the always-loaded enforcement stub; area SKILL.md is the on-trigger spec; never invert." },
    Predicate { id: "G3-PIN", source: "GLOBAL-LOGIC.md §8", modality: Modality::Must, scope: Scope::SkillShape, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Templates pin versions; `latest` tags are FORBIDDEN." },
];

/// GLOBAL-LOGIC §4: hook lifecycle (session_start → pre_write DENY →
/// post_write APPEND → stop CONTINUE → batch_audit; fail closed; append-store;
/// one shared rule table; state in .agent-state/).
pub const GLOGIC_S4: &[Predicate] = &[
    Predicate { id: "G4-START", source: "GLOBAL-LOGIC.md §4 cl.1", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "session_start injects stack constraints as context; NEVER blocks." },
    Predicate { id: "G4-PRE", source: "GLOBAL-LOGIC.md §4 cl.2", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "pre_write on deny-list match → DENY + reason + fix; fail CLOSED on parse error." },
    Predicate { id: "G4-POST", source: "GLOBAL-LOGIC.md §4 cl.3", modality: Modality::Should, scope: Scope::HookLifecycle, binding: Binding::OnTrigger, consequence: Consequence::WarnAndRecord, law: "post_write on warn-list match → APPEND to pending-issues store; never blocks, never judges." },
    Predicate { id: "G4-STOP", source: "GLOBAL-LOGIC.md §4 cl.4", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "stop(idle) with non-empty pending-issues → CONTINUE + issues; delete store only after read." },
    Predicate { id: "G4-BATCH", source: "GLOBAL-LOGIC.md §4 cl.5", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "batch_audit applies the SAME deny+warn rules over a tree; exit 2 + machine-readable report." },
    Predicate { id: "G4-TABLE", source: "GLOBAL-LOGIC.md §4 kit-gap fix", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Live hooks and batch audit share ONE rule table sourced from a single file; no drift." },
    Predicate { id: "G4-STATE", source: "GLOBAL-LOGIC.md §4 kit-gap fix", modality: Modality::Must, scope: Scope::HookLifecycle, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Hook state lives in .agent-state/ (gitignored), never the workspace root." },
];

/// GLOBAL-LOGIC §5: workflow optimisation (retrieval-first, negative-contracts,
/// defaults, fail-fast, amortise, defer-subjective/deny-objective).
pub const GLOGIC_S5: &[Predicate] = &[
    Predicate { id: "G5-RETRIEVE", source: "GLOBAL-LOGIC.md §5.1", modality: Modality::Must, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Decisions live in queryable files; look up instead of inventing." },
    Predicate { id: "G5-NEGATIVE", source: "GLOBAL-LOGIC.md §5.2", modality: Modality::Must, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Banned lists first: prune generic outputs by literal match." },
    Predicate { id: "G5-DEFAULTS", source: "GLOBAL-LOGIC.md §5.3", modality: Modality::Must, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "One import path, one state pattern, one easing curve, one error envelope — as recorded." },
    Predicate { id: "G5-FAILFAST", source: "GLOBAL-LOGIC.md §5.4", modality: Modality::Must, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Deny at write-time, verify per-edit, gate the phase, refuse dirty completion." },
    Predicate { id: "G5-AMORTISE", source: "GLOBAL-LOGIC.md §5.5", modality: Modality::Should, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::WarnAndRecord, law: "Persist decisions (MASTER.md, tokens, contracts) so later tasks start from record." },
    Predicate { id: "G5-DEFER", source: "GLOBAL-LOGIC.md §5.6", modality: Modality::Must, scope: Scope::Workflow, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Objective violations block immediately; subjective ones accumulate as warnings and veto completion." },
];

/// GLOBAL-LOGIC §6: partition & swarm protocol (clauses 1–8; INCLUDES the §6.6
/// re-entry adjudication of ledger Entry 2).
pub const GLOGIC_S6: &[Predicate] = &[
    Predicate { id: "G6-ROLES", source: "GLOBAL-LOGIC.md §6.1", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Roles: Explorer (read-only) → Architect (shared contracts) → N Coders (one partition each) → Critic (sole live-tool verifier)." },
    Predicate { id: "G6-ROUTING", source: "GLOBAL-LOGIC.md §6.2", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Task <=3 files single engine → solo; else explorer → architect → parallel coders." },
    Predicate { id: "G6-PARTITION", source: "GLOBAL-LOGIC.md §6.3", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Many small VERTICAL slices (feature/surface/contract), one mind each front-to-back; never horizontal layers." },
    Predicate { id: "G6-CONTRACT", source: "GLOBAL-LOGIC.md §6.4", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No coder starts until shared contracts exist ON DISK and are reviewed." },
    Predicate { id: "G6-OWNERSHIP", source: "GLOBAL-LOGIC.md §6.5", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Exclusive partition ownership; everything outside is READ-ONLY." },
    Predicate { id: "G6-VERIFY", source: "GLOBAL-LOGIC.md §6.6 (+ ledger Entry 2)", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Coders never self-approve; critic patch lawful ONLY IF the change re-enters layer (b) and is passed by gates or a fresh check." },
    Predicate { id: "G6-EXIT", source: "GLOBAL-LOGIC.md §6.7", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Done IFF typecheck ∧ build ∧ audit clean AND zero runtime errors AND critic checks pass; else re-enter at the FAILING layer." },
    Predicate { id: "G6-LOCK", source: "GLOBAL-LOGIC.md §6.8", modality: Modality::Must, scope: Scope::Swarm, binding: Binding::AtGate, consequence: Consequence::DenyAndVeto, law: "Stop-hook re-checks all gates; any failure vetoes done." },
];

/// GLOBAL-LOGIC §7: calibration dials (declared BEFORE generating).
pub const GLOGIC_S7: &[Predicate] = &[
    Predicate { id: "G7-DIALS", source: "GLOBAL-LOGIC.md §7", modality: Modality::Must, scope: Scope::Calibration, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Every generative task declares BRIEF + STACK + DESIGN_VARIANCE/MOTION_INTENSITY/VISUAL_DENSITY 1–10 up front." },
];

/// GLOBAL-LOGIC §9: compliance map (routing — each law lives in exactly one place).
pub const GLOGIC_S9: &[Predicate] = &[
    Predicate { id: "G9-MAP", source: "GLOBAL-LOGIC.md §9", modality: Modality::Must, scope: Scope::Docs, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Each law lives in its mapped canonical file; compliance map is the router, not a duplicate." },
];

/// All GLOBAL-LOGIC predicates in source order (§§3–9; §§0–2, 8, 10 narrative/provenance, no laws).
pub const ALL_GLOGIC: &[&[Predicate]] =
    &[GLOGIC_S3, GLOGIC_S4, GLOGIC_S5, GLOGIC_S6, GLOGIC_S7, GLOGIC_S9];

// ---------------------------------------------------------------------------
// Family C — kimi-frontend-kit rules (rules.md 5 rules + AGENTS.md §§1–3)
// ---------------------------------------------------------------------------

/// kimi `rules.md`: the 5 quick-reference laws (workflow-first MUST chain,
/// anti-slop bans, React 19 / Tailwind v4 / TS 7.1 stack pins).
pub const KIMI_RULES: &[Predicate] = &[
    Predicate { id: "K1-LOOP", source: "kimi rules.md rule 1", modality: Modality::Must, scope: Scope::KimiLoop, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Execute the 5-stage Kimi loop: calibration → token spec (OKLCH) → decomposition → zero-stub → browser vision QA." },
    Predicate { id: "K2-BAN", source: "kimi rules.md rule 2", modality: Modality::Must, scope: Scope::KimiVisual, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No AI slop: no default Inter/Roboto, no purple-blue gradients, no flat 1px gray cards, no linear/ease-in-out; double-bezel surfaces + spring/cubic-bezier(0.16,1,0.3,1) physics." },
    Predicate { id: "K3-REACT", source: "kimi rules.md rule 3", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "React 19: ref-as-prop (no forwardRef), use(), useActionState, useOptimistic, Compiler-pure renders." },
    Predicate { id: "K4-TAILWIND", source: "kimi rules.md rule 4", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Tailwind v4 CSS-first: @import tailwindcss + @theme with oklch() tokens; no tailwind.config.js." },
    Predicate { id: "K5-TS", source: "kimi rules.md rule 5", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "TypeScript 7.1: verbatimModuleSyntax + erasableSyntaxOnly, no enum/namespace (as const + satisfies), explicit import type." },
];

/// kimi `AGENTS.md` §§1–3 (execution loop, surface architecture, frontier stack).
pub const KIMI_AGENTS: &[Predicate] = &[
    Predicate { id: "KA1-LOOP", source: "kimi AGENTS.md §1", modality: Modality::Must, scope: Scope::KimiLoop, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Mandatory 5-step loop incl. token lock before code, subagent delegation for multi-file work, vision-in-the-loop QA at 1440x900 + 390x844." },
    Predicate { id: "KA1-NOSTUB", source: "kimi AGENTS.md §1 cl.4", modality: Modality::Must, scope: Scope::KimiLoop, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No stubs or dead states: no TODO/implement-later; every control wired to live state with all 5 interaction states." },
    Predicate { id: "KA2-SURFACE", source: "kimi AGENTS.md §2 double-bezel", modality: Modality::Must, scope: Scope::KimiVisual, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Major surfaces use double-bezel nested enclosures (outer ring + inner inset highlight; inner_radius = outer_radius - padding)." },
    Predicate { id: "KA2-GPU", source: "kimi AGENTS.md §2 compositor physics", modality: Modality::Must, scope: Scope::KimiVisual, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Animate transform/opacity only (never layout props); backdrop-blur only on fixed/sticky chrome." },
    Predicate { id: "KA2-TYPE", source: "kimi AGENTS.md §2 banned defaults", modality: Modality::Must, scope: Scope::KimiVisual, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Distinct type pairing (display + body + mono); never unstyled Inter/Roboto/Arial as default." },
    Predicate { id: "KA3-ZUSTAND", source: "kimi AGENTS.md §3 Zustand v5", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Curried create<AppState>()(...); useShallow for object/array selectors; transient getState/subscribe for high-frequency frame data." },
    Predicate { id: "KA3-MOTION", source: "kimi AGENTS.md §3 Motion v12", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Import from motion/react, never the legacy framer-motion entrypoint in new code." },
    Predicate { id: "KA3-R3F", source: "kimi AGENTS.md §3 R3F/WebGL", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "No setState inside useFrame (mutate refs); instancedMesh past 150 repeats; dispose on unmount; frameloop=demand when static." },
    Predicate { id: "KA3-EDITORS", source: "kimi AGENTS.md §3 Dockview/Monaco/CodeMirror", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Dockview api persistence + full-bleed panel containers; Monaco workers + file:// model URIs + automaticLayout; CodeMirror Compartments + ViewPlugin decorations." },
    Predicate { id: "KA3-GRAPHS", source: "kimi AGENTS.md §3 graphs/DAGs", modality: Modality::Must, scope: Scope::KimiStack, binding: Binding::OnTrigger, consequence: Consequence::DenyAndVeto, law: "Graphs >200 nodes on canvas/WebGL with LOD/quadtree/ego-highlight; DAGs via @xyflow/react v12 + elkjs layered layout." },
];

/// All kimi predicates in source order.
pub const ALL_KIMI: &[&[Predicate]] = &[KIMI_RULES, KIMI_AGENTS];

// ---------------------------------------------------------------------------
// Dispatch: every predicate, modality law, family counts
// ---------------------------------------------------------------------------

/// Every compiled predicate: 76 RULES + 31 GLOBAL-LOGIC + 15 kimi = 122.
pub fn all_predicates() -> Vec<&'static Predicate> {
    let mut out = Vec::with_capacity(122);
    for fam in ALL_RULES.iter().chain(ALL_GLOGIC.iter()).chain(ALL_KIMI.iter()) {
        out.extend(fam.iter());
    }
    out
}

/// Count predicates by modality across the whole compiled set.
#[must_use]
pub fn modality_counts() -> (usize, usize, usize) {
    let mut must = 0;
    let mut should = 0;
    let mut may = 0;
    for fam in ALL_RULES.iter().chain(ALL_GLOGIC.iter()).chain(ALL_KIMI.iter()) {
        for p in fam.iter() {
            match p.modality {
                Modality::Must => must += 1,
                Modality::Should => should += 1,
                Modality::May => may += 1,
            }
        }
    }
    (must, should, may)
}

/// Evaluate the modality law: MUST→Deny+veto, SHOULD→Warn+record, MAY→Allow.
#[must_use]
pub const fn evaluate(modality: Modality) -> (Verdict, Consequence) {
    match modality {
        Modality::Must => (Verdict::Deny, Consequence::DenyAndVeto),
        Modality::Should => (Verdict::Warn, Consequence::WarnAndRecord),
        Modality::May => (Verdict::Allow, Consequence::Allow),
    }
}

// ---------------------------------------------------------------------------
// Tests — pass AND violate per family (violate paths assert the veto fires)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_family_count_is_76_no_section_8() {
        let n: usize = ALL_RULES.iter().map(|s| s.len()).sum();
        assert_eq!(n, 76, "RULES family must hold 76 predicates (rules 1-75 + 60a)");
        assert!(all_predicates().iter().all(|p| p.scope != Scope::KimiLoop || p.id.starts_with('K') || p.id.starts_with("KA")));
    }

    #[test]
    fn glogic_family_count_is_30() {
        let n: usize = ALL_GLOGIC.iter().map(|s| s.len()).sum();
        assert_eq!(n, 31, "GLOBAL-LOGIC family must hold 31 predicates");
    }

    #[test]
    fn kimi_family_count_is_15() {
        let n: usize = ALL_KIMI.iter().map(|s| s.len()).sum();
        assert_eq!(n, 15, "kimi family must hold 15 predicates (5 rules.md + 10 AGENTS.md)");
    }

    #[test]
    fn total_is_121() {
        assert_eq!(all_predicates().len(), 122);
    }

    #[test]
    fn modality_law_holds_for_every_predicate() {
        for p in all_predicates() {
            let (v, c) = evaluate(p.modality);
            assert_eq!(v, p.verdict(), "verdict mismatch on {}", p.id);
            assert_eq!(c, p.consequence, "consequence mismatch on {}", p.id);
        }
    }

    #[test]
    fn must_means_deny_and_veto() {
        let (v, c) = evaluate(Modality::Must);
        assert_eq!((v, c), (Verdict::Deny, Consequence::DenyAndVeto));
    }

    #[test]
    fn should_means_warn_and_record() {
        // PASS path: the sole SHOULD (R52) warns, never vetoes.
        let r52 = RULES_S6.iter().find(|p| p.id == "R52").expect("R52 exists");
        assert_eq!(r52.modality, Modality::Should);
        assert_eq!(r52.verdict(), Verdict::Warn);
        // VIOLATE path: a SHOULD violation must NOT yield Deny.
        let (v, _) = evaluate(r52.modality);
        assert_ne!(v, Verdict::Deny, "SHOULD violation must not veto");
    }

    #[test]
    fn violate_must_fires_veto_per_family() {
        // VIOLATE path per family: a MUST violation in each family yields Deny+veto.
        for (fam, slice) in [("RULES", RULES_S4), ("GLOGIC", GLOGIC_S6), ("KIMI", KIMI_RULES)] {
            let first_must = slice.iter().find(|p| p.modality == Modality::Must).expect("has MUST");
            let (v, c) = evaluate(first_must.modality);
            assert_eq!((v, c), (Verdict::Deny, Consequence::DenyAndVeto), "veto must fire for {fam} {}", first_must.id);
        }
    }

    #[test]
    fn ledger_entry_2_reentry_predicate_exists() {
        // The §6.6 re-entry adjudication compiles to an enforceable predicate.
        let g = GLOGIC_S6.iter().find(|p| p.id == "G6-VERIFY").expect("G6-VERIFY exists");
        assert_eq!(g.verdict(), Verdict::Deny);
        assert!(g.law.contains("re-enters"), "G6-VERIFY must encode the re-entry condition");
    }

    #[test]
    fn single_table_convergence_predicates_exist() {
        // Ledger Entry 1: R07 + G4-TABLE both enforce singularity (convergent).
        let r07 = RULES_S1.iter().find(|p| p.id == "R07").expect("R07 exists");
        let g4t = GLOGIC_S4.iter().find(|p| p.id == "G4-TABLE").expect("G4-TABLE exists");
        assert_eq!(r07.verdict(), Verdict::Deny);
        assert_eq!(g4t.verdict(), Verdict::Deny);
    }

    #[test]
    fn modality_counts_match_sources() {
        // 119 MUST + 3 SHOULD (R52, G4-POST, G5-AMORTISE) + 0 MAY.
        assert_eq!(modality_counts(), (119, 3, 0));
    }

    #[test]
    fn no_duplicate_predicate_ids() {
        let all = all_predicates();
        let mut ids: Vec<&str> = all.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), before, "predicate ids must be unique (no-duplicate-schema holds)");
    }
}
