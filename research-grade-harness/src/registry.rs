//! Native-name registry for everything RG-AH imports.
//!
//! Naming rule, Stewart 2026-10-29: names are native to the harness and the
//! model, not to where they currently sit. No vendor, host app, file path,
//! or IDE/SDK name appears in a native name. A native name comes from what
//! the thing does in RG-AH: mission, packet, lane, judge, receipt, atom,
//! arc, basin, cache, tap, gate.
//!
//! This registry does NOT copy external servers, skills, or docs into Rust.
//! Servers stay servers, skills stay docs, runbooks stay docs. RG-AH owns
//! the registry entry, the live check, and the receipt. An entry marked
//! `live: false` stops a run at Step 0, with the entry named, rather than
//! failing silently mid-mission.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegistryKind {
    McpServer,
    Tool,
    Skill,
    Hook,
    Runbook,
    Workflow,
    Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Native name, RG-AH vocabulary only.
    pub native: String,
    /// Current name, where it sits today. Kept as an alias during transition.
    pub current: String,
    pub kind: RegistryKind,
    /// What it does in RG-AH, in one line.
    pub role: String,
    /// Whether it is verified live. False means Step 0 stops the run.
    pub live: bool,
}

pub fn registry() -> Vec<RegistryEntry> {
    let mut entries: Vec<RegistryEntry> = Vec::new();
    let mut add = |native: &str, current: &str, kind: RegistryKind, role: &str, live: bool| {
        entries.push(RegistryEntry {
            native: native.into(),
            current: current.into(),
            kind,
            role: role.into(),
            live,
        });
    };
    // MCP servers, native by function served.
    add(
        "route",
        "atoma-tool-router",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) route, search catalog, execute, lane.mint",
        false,
    );
    add(
        "check",
        "ts-to-rust",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) TypeScript strict check first, transpile stated subset only",
        false,
    );
    add(
        "lazy",
        "mcp-lazy",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) find and execute pinned tools only, adds no servers",
        false,
    );
    add(
        "runbook",
        "qwen36-autonomous-runbook",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) autonomous runbook entry point and verified patch tools",
        false,
    );
    add(
        "toolchain",
        "zed-engineering-toolchain",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) detect toolchains, plan and run language checks",
        false,
    );
    add(
        "design",
        "open-design",
        RegistryKind::McpServer,
        "(unverified: no in-repo implementation as of 2026-11-01) design system server, only where a run names it",
        false,
    );
    // Tools.
    add(
        "judge",
        "deterministic-judge",
        RegistryKind::Tool,
        "judge of record for code tasks",
        true,
    );
    add(
        "telemetry",
        "tool-telemetry",
        RegistryKind::Tool,
        "tool call records, metadata and hashes only",
        true,
    );
    add(
        "tap",
        "taps",
        RegistryKind::Tool,
        "(unverified: no in-repo implementation as of 2026-11-01) per-category tap gates, dropped count",
        false,
    );
    // Skills, native by role. The 7 MasterSkills are a group until the
    // rename map gives each one a native name; no name is invented here.
    add(
        "carve-swarm",
        "carving-the-swarm",
        RegistryKind::Skill,
        "(unverified: no in-repo implementation as of 2026-11-01) carve weights into the swarm, claims traced to live code",
        false,
    );
    add(
        "task-decomp",
        "task-decomp",
        RegistryKind::Skill,
        "(unverified: no in-repo implementation as of 2026-11-01) decompose a task into judged packets",
        false,
    );
    add(
        "tactical",
        "tactical-orchestration",
        RegistryKind::Skill,
        "(unverified: no in-repo implementation as of 2026-11-01) tactical orchestration of lanes and packets",
        false,
    );
    add(
        "allocate-weights",
        "allocating-weights",
        RegistryKind::Skill,
        "(unverified: no in-repo implementation as of 2026-11-01) allocate weights across lanes and packets",
        false,
    );
    add(
        "master-skill",
        "MasterSkills",
        RegistryKind::Skill,
        "7 necessary skills, group entry, native names pending rename map",
        false,
    );
    // Hooks, N3 order: TreeSitter, Top MCP, Plugins, Skills, Hooks, before completion.
    add(
        "tree-sitter",
        "Capability Tree Sitter",
        RegistryKind::Hook,
        "capability tree, first in the hook order",
        false,
    );
    add(
        "top-mcp",
        "Top MCP",
        RegistryKind::Hook,
        "top MCP layer in the hook order",
        false,
    );
    add(
        "plugin",
        "Plugins",
        RegistryKind::Hook,
        "plugin layer in the hook order",
        false,
    );
    add(
        "skill",
        "Skills.md",
        RegistryKind::Hook,
        "skill layer in the hook order",
        false,
    );
    add(
        "hook",
        "Hooks",
        RegistryKind::Hook,
        "hook layer, before completion",
        false,
    );
    add(
        "tail",
        "Tails",
        RegistryKind::Hook,
        "tail layer, after completion",
        false,
    );
    // Runbooks, pinned docs with gates intact.
    add(
        "frontend-refinement",
        "frontend-refinement-runbook",
        RegistryKind::Runbook,
        "(unverified: no in-repo implementation as of 2026-11-01) Beam loops a real frontend until a deterministic gate passes",
        false,
    );
    add(
        "master-todo",
        "master-todo-2026-10-28",
        RegistryKind::Runbook,
        "(unverified: no in-repo implementation as of 2026-11-01) Delta-H lock, extraction, representation check, in order",
        false,
    );
    add(
        "overnight-audit",
        "overnight-audit-certify-loop",
        RegistryKind::Runbook,
        "(unverified: no in-repo implementation as of 2026-11-01) audit, certify, morning report, no auto-post or submit",
        false,
    );
    add(
        "delta-h-ablation",
        "delta-h-spec-ablation-protocol",
        RegistryKind::Runbook,
        "Delta-H spec and ablation protocol, proposed until locked",
        false,
    );
    // Workflows.
    add(
        "autorunner",
        "MadMind autorunner",
        RegistryKind::Workflow,
        "(unverified: no in-repo implementation as of 2026-11-01) poll ready packets, judge-gated, escalation, resume",
        false,
    );
    add(
        "mission",
        "rgah mission",
        RegistryKind::Workflow,
        "run, dry-run, resume, from a brief and a plan",
        true,
    );
    // Policy, already in the crate plus the mission rules.
    add(
        "frozen-kernel",
        "FrozenPolicyKernel",
        RegistryKind::Policy,
        "kernel mutation refusals, checked before every run",
        true,
    );
    add(
        "go-no-go",
        "go-no-go",
        RegistryKind::Policy,
        "brief, judges, tools, budgets, kernel, all present or no-go",
        true,
    );
    add(
        "two-strike",
        "two-strike",
        RegistryKind::Policy,
        "first identical fail retries once, second escalates",
        true,
    );
    add(
        "abort",
        "abort",
        RegistryKind::Policy,
        "(unverified: no in-repo implementation as of 2026-11-01) abort on judge failure, kernel violation, budget breach, or operator command",
        false,
    );
    add(
        "budget",
        "energy-budget",
        RegistryKind::Policy,
        "per-packet and per-mission token and energy budgets",
        true,
    );
    entries
}

/// Native names only, in registry order. Used to check that no native name
/// carries a vendor, host, path, or IDE name.
pub fn native_names() -> Vec<String> {
    registry().into_iter().map(|e| e.native).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_kinds() {
        let entries = registry();
        for kind in [
            RegistryKind::McpServer,
            RegistryKind::Tool,
            RegistryKind::Skill,
            RegistryKind::Hook,
            RegistryKind::Runbook,
            RegistryKind::Workflow,
            RegistryKind::Policy,
        ] {
            assert!(
                entries.iter().any(|e| e.kind == kind),
                "missing kind: {:?}",
                kind
            );
        }
    }

    #[test]
    fn native_names_carry_no_vendor_or_path() {
        let banned = [
            "claude",
            "codex",
            "antigravity",
            "grok",
            "zed",
            "open-design",
            "/",
            ":",
            "Desktop",
        ];
        for name in native_names() {
            for bad in banned {
                assert!(
                    !name.to_lowercase().contains(&bad.to_lowercase()),
                    "native name {} carries {}",
                    name,
                    bad
                );
            }
        }
    }

    #[test]
    fn missing_entries_are_marked_not_live() {
        // Hooks and the Delta-H runbook are proposed, not live, by design.
        // The registry must say so, not pretend otherwise.
        let entries = registry();
        assert!(entries.iter().any(|e| e.native == "hook" && !e.live));
        assert!(
            entries
                .iter()
                .any(|e| e.native == "delta-h-ablation" && !e.live)
        );
    }
}
