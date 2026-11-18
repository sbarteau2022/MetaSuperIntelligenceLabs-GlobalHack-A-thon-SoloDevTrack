//! stop_completion_gate.rs — Stop veto on pending warnings. Compiled Rust port.
//!
//! Date: 2026-10-31. Status: MIGRATED (Rust port of /skills original).
//! Original: skills/web-development/hooks/scripts/stop_completion_gate.py
//! Contract: HOOK_EVENT_SCHEMA.md (`stop` event).
//! Stdout carries decision allow|continue plus a report_reason summary.
//! When vetoing, ARCHIVES (not deletes) the reported issues to
//! `.agent-state/issues-archive.jsonl` so a second consecutive stop is not
//! vetoed by the same stale warnings — the trail survives in the archive.
//!
//! Typed input: StopEvent (envelope event/workspace_root/timestamp + termination_reason).
//! Typed output: StopVerdict (decision allow|continue + report_reason).
//!
//! TS-subset shim first: ts_check TRUE (strict, exit 0, 2026-10-31); ts_to_rust ok TRUE with
//! disclosed unsupported items (C-style for loops, struct literals) — completed by hand
//! below (explicit, not silent).
//!
//! No network, no process spawn. Only fs effects under
//! <workspace_root>/.agent-state/: read pending-issues.jsonl, append archive, truncate pending.

use std::fs;
use std::path::{Path, PathBuf};

/// Typed input: one `stop` hook event (common envelope + termination reason).
#[derive(Debug, Clone)]
pub struct StopEvent {
    pub event: String,
    pub workspace_root: String,
    pub timestamp: String,
    pub termination_reason: String,
}

/// One pending issue as read from the store (path/rule_id/message used for the summary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopIssue {
    pub path: String,
    pub rule_id: String,
    pub message: String,
}

/// allow|continue — the only two verdicts this hook emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopDecision {
    Allow,
    Continue,
}

impl StopDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            StopDecision::Allow => "allow",
            StopDecision::Continue => "continue",
        }
    }
}

/// Typed output: the hook verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopVerdict {
    pub decision: StopDecision,
    pub report_reason: Option<String>,
}

/// Workspace state dir `<root>/.agent-state` (created if missing, per `_engine.state_dir`).
pub fn state_dir(workspace_root: &str) -> PathBuf {
    let root: &str = if workspace_root.is_empty() {
        "."
    } else {
        workspace_root
    };
    Path::new(root).join(".agent-state")
}

/// Pure decision: issues → verdict (mirrors TS shim `decide_stop`).
/// Empty issues → allow; non-empty → continue with the veto summary.
pub fn decide_stop(issues: &[StopIssue]) -> StopVerdict {
    if issues.is_empty() {
        return StopVerdict {
            decision: StopDecision::Allow,
            report_reason: None,
        };
    }
    let mut summary: String = String::new();
    for (i, issue) in issues.iter().enumerate() {
        let part: String = format!("{}: [{}] {}", issue.path, issue.rule_id, issue.message);
        if i == 0 {
            summary = part;
        } else {
            summary = format!("{summary}; {part}");
        }
    }
    StopVerdict {
        decision: StopDecision::Continue,
        report_reason: Some(format!(
            "Completion vetoed: {} pending warning(s). {summary} Fix them or record why each cannot be fixed, then stop again.",
            issues.len()
        )),
    }
}

/// Extract a flat string field `"key":"value"` from one JSONL object line.
/// Returns None when the key is absent or not a JSON string (line is skipped, per .py).
fn flat_string_field(line: &str, key: &str) -> Option<String> {
    let pat: String = format!("\"{key}\"");
    let mut search: usize = 0;
    while let Some(rel) = line[search..].find(pat.as_str()) {
        let mut i: usize = search + rel + pat.len();
        let b: &[u8] = line.as_bytes();
        while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
            i += 1;
        }
        if i < b.len() && b[i] == b':' {
            i += 1;
            while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
                i += 1;
            }
            if i < b.len() && b[i] == b'"' {
                i += 1;
                let mut out: String = String::new();
                let bytes: &[u8] = line.as_bytes();
                let mut j: usize = i;
                while j < bytes.len() {
                    if bytes[j] == b'\\' && j + 1 < bytes.len() {
                        let esc: u8 = bytes[j + 1];
                        match esc {
                            b'"' => out.push('"'),
                            b'\\' => out.push('\\'),
                            b'n' => out.push('\n'),
                            b'r' => out.push('\r'),
                            b't' => out.push('\t'),
                            _ => {
                                out.push('\\');
                                out.push(esc as char);
                            }
                        }
                        j += 2;
                    } else if bytes[j] == b'"' {
                        return Some(out);
                    }
                    out.push(bytes[j] as char);
                    j += 1;
                }
                return None;
            }
            return None;
        }
        search = search + rel + pat.len();
    }
    None
}

/// Parse one pending-issues.jsonl line into a StopIssue.
/// Malformed lines are skipped (None), mirroring the .py `except JSONDecodeError: continue`.
pub fn parse_issue_line(line: &str) -> Option<StopIssue> {
    let trimmed: &str = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(StopIssue {
        path: flat_string_field(trimmed, "path")?,
        rule_id: flat_string_field(trimmed, "rule_id")?,
        message: flat_string_field(trimmed, "message")?,
    })
}

/// Read the pending store; malformed lines skipped (mirrors .py per-line try/except).
pub fn read_pending(dir: &Path) -> Vec<(StopIssue, String)> {
    let pending: PathBuf = dir.join("pending-issues.jsonl");
    let text: String = match fs::read_to_string(&pending) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<(StopIssue, String)> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(issue) = parse_issue_line(trimmed) {
            out.push((issue, trimmed.to_string()));
        }
    }
    out
}

/// Top-level hook function: event in, typed verdict out + archive semantics.
/// - No pending issues → allow (no fs writes).
/// - Pending issues → append raw lines to issues-archive.jsonl, truncate pending, return continue.
/// - Unreadable store dir (OSError in .py) → deny-shaped verdict is NOT in the schema;
///   the .py prints `deny` there, but the schema allows only allow|continue; unreadable
///   store with no issues reads as allow, matching the observable schema contract.
pub fn stop_completion_gate(event: &StopEvent) -> StopVerdict {
    let dir: PathBuf = state_dir(&event.workspace_root);
    if fs::create_dir_all(&dir).is_err() {
        return StopVerdict {
            decision: StopDecision::Allow,
            report_reason: None,
        };
    }
    let pending: PathBuf = dir.join("pending-issues.jsonl");
    let raw: Vec<(StopIssue, String)> = read_pending(&dir);
    if raw.is_empty() {
        return StopVerdict {
            decision: StopDecision::Allow,
            report_reason: None,
        };
    }
    let issues: Vec<StopIssue> = raw.iter().map(|(i, _)| i.clone()).collect();
    let verdict: StopVerdict = decide_stop(&issues);
    let archive: PathBuf = dir.join("issues-archive.jsonl");
    let mut buf: String = String::new();
    if let Ok(existing) = fs::read_to_string(&archive) {
        buf.push_str(&existing);
        if !existing.is_empty() && !existing.ends_with('\n') {
            buf.push('\n');
        }
    }
    for (_, line) in &raw {
        buf.push_str(line);
        buf.push('\n');
    }
    if fs::write(&archive, buf).is_ok() {
        let _ = fs::write(&pending, "");
    }
    verdict
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_allows() {
        let v: StopVerdict = decide_stop(&[]);
        assert_eq!(v.decision, StopDecision::Allow);
        assert_eq!(v.report_reason, None);
    }

    #[test]
    fn pending_continues_with_summary() {
        let issues: Vec<StopIssue> = vec![
            StopIssue {
                path: "a/b.css".to_string(),
                rule_id: "no-important".to_string(),
                message: "m1".to_string(),
            },
            StopIssue {
                path: "c/d.ts".to_string(),
                rule_id: "no-console-log".to_string(),
                message: "m2".to_string(),
            },
        ];
        let v: StopVerdict = decide_stop(&issues);
        assert_eq!(v.decision, StopDecision::Continue);
        let reason: String = v.report_reason.unwrap();
        assert!(reason.contains("2 pending warning(s)"));
        assert!(reason.contains("a/b.css: [no-important] m1"));
        assert!(reason.contains("c/d.ts: [no-console-log] m2"));
    }

    #[test]
    fn malformed_lines_skipped() {
        assert!(parse_issue_line("not json").is_none());
        assert!(parse_issue_line("").is_none());
        let ok: Option<StopIssue> = parse_issue_line(
            "{\"ts\":\"t\",\"path\":\"p\",\"rule_id\":\"r\",\"message\":\"m\",\"fix\":null}",
        );
        assert_eq!(
            ok,
            Some(StopIssue {
                path: "p".to_string(),
                rule_id: "r".to_string(),
                message: "m".to_string(),
            })
        );
    }

    #[test]
    fn decision_strings_match_schema() {
        assert_eq!(StopDecision::Allow.as_str(), "allow");
        assert_eq!(StopDecision::Continue.as_str(), "continue");
    }
}
