//! pre_tool_guard.rs — PreToolUse deny-list on write content. Compiled Rust port.
//!
//! Date: 2026-10-31. Status: MIGRATED (Rust port of /skills original).
//! Original: skills/web-development/hooks/scripts/pre_tool_guard.py
//! Contract: HOOK_EVENT_SCHEMA.md (`pre_write` event). Fail CLOSED on schema errors.
//! Rule source: rules-table.json severity=deny rules, compiled in verbatim (VERIFIED REPO:
//! 8 deny rules in rules-table.json, 2026-10-31).
//!
//! Typed input: PreWriteEvent (envelope event/workspace_root/timestamp + path/content/operation).
//! Typed output: PreVerdict (allow|deny + rule_id + fix + report_reason).
//!
//! TS-subset shim first: ts_check TRUE (strict, exit 0, 2026-10-31); ts_to_rust ok TRUE with
//! one disclosed unsupported item ("object literal (no target struct known in expression)") —
//! the two struct-return expressions were completed by hand below (explicit, not silent).
//!
//! No network, no file writes, no process spawn: pure function of (event, rules).
//! All state writes (if any future caller) belong under workspace_root/.agent-state or temp.

/// Typed input: one `pre_write` hook event (common envelope + path/content/operation).
#[derive(Debug, Clone)]
pub struct PreWriteEvent {
    pub event: String,
    pub workspace_root: String,
    pub timestamp: String,
    pub path: String,
    pub content: Option<String>,
    pub operation: String,
}

/// One compiled deny rule: verbatim (id, scope globs, pattern, message, fix).
#[derive(Debug, Clone)]
pub struct DenyRule {
    pub id: &'static str,
    pub scope: &'static [&'static str],
    pub exclude: &'static [&'static str],
    pub pattern: &'static str,
    pub message: &'static str,
    pub fix: &'static str,
}

/// Typed output: the hook verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreVerdict {
    pub decision: PreDecision,
    pub report_reason: Option<String>,
    pub rule_id: Option<String>,
    pub fix: Option<String>,
}

/// allow|deny — the only two verdicts this hook emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreDecision {
    Allow,
    Deny,
}

impl PreDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            PreDecision::Allow => "allow",
            PreDecision::Deny => "deny",
        }
    }
}

/// First-match-wins finding: rule hit with the exact matched text (max 120 chars, per _engine.scan).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleHit {
    pub rule_id: String,
    pub message: String,
    pub matched: String,
    pub fix: Option<String>,
}

/// The 8 deny rules from rules-table.json (VERIFIED REPO 2026-10-31), verbatim.
pub static DENY_RULES: &[DenyRule] = &[
    DenyRule {
        id: "no-secrets-aws-key",
        scope: &["**"],
        exclude: &[],
        pattern: "AKIA[0-9A-Z]{16}",
        message: "Possible AWS access key in file content.",
        fix: "Remove the key, rotate it, and load it from a secret manager or .env (never committed).",
    },
    DenyRule {
        id: "no-private-key",
        scope: &["**"],
        exclude: &[],
        pattern: "-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----",
        message: "Private key material in file content.",
        fix: "Remove immediately, rotate the key, use a secret manager.",
    },
    DenyRule {
        id: "no-hardcoded-password",
        scope: &[
            "**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx", "**/*.py", "**/*.json", "**/*.yml",
            "**/*.yaml",
        ],
        exclude: &[],
        pattern: "(?i)(password|passwd|pwd)\\s*[:=]\\s*['\"][^'\"]{3,}['\"]",
        message: "Hardcoded password/secret assignment.",
        fix: "Read from environment or secret manager; keep only a placeholder in .env.example.",
    },
    DenyRule {
        id: "no-env-commit",
        scope: &["**/.env", "**/.env.*"],
        exclude: &[],
        pattern: ".",
        message: "Real .env file caught by the write guard.",
        fix: "Unstage it; only .env.example belongs in version control.",
    },
    DenyRule {
        id: "no-eval",
        scope: &["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
        exclude: &[],
        pattern: "\\beval\\s*\\(",
        message: "eval() is a code-injection surface.",
        fix: "Replace with JSON.parse, a lookup table, or a proper parser.",
    },
    DenyRule {
        id: "no-dangerous-html",
        scope: &["**/*.tsx", "**/*.jsx"],
        exclude: &[],
        pattern: "dangerouslySetInnerHTML",
        message: "Raw HTML injection in JSX.",
        fix: "Render structured content or sanitize with a reviewed sanitizer at the sink.",
    },
    DenyRule {
        id: "no-innerhtml-sink",
        scope: &["**/*.ts", "**/*.js"],
        exclude: &[],
        pattern: "\\.innerHTML\\s*=",
        message: "innerHTML assignment is an XSS sink.",
        fix: "Use textContent / DOM construction or a sanitizer.",
    },
    DenyRule {
        id: "no-debugger",
        scope: &["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
        exclude: &[],
        pattern: "\\bdebugger\\b",
        message: "debugger statement halts execution for every user.",
        fix: "Remove it.",
    },
];

/// schema-error deny constructor (fail closed), mirrors pre_tool_guard.py::deny.
pub fn schema_deny(reason: String, fix: String) -> PreVerdict {
    PreVerdict {
        decision: PreDecision::Deny,
        report_reason: Some(reason),
        rule_id: Some("schema-error".to_string()),
        fix: Some(fix),
    }
}

/// Pure verdict from an event plus pre-computed rule hits (first hit wins).
/// Mirrors decide_pre_write in the TS-subset shim (ts_check TRUE 2026-10-31).
pub fn decide_pre_write(event: &PreWriteEvent, hits: &[RuleHit]) -> PreVerdict {
    if event.path.is_empty() {
        return schema_deny(
            "Pre-guard received no path. Failing closed.".to_string(),
            "Provide the root-relative path in the hook event.".to_string(),
        );
    }
    if event.content.is_none() {
        return schema_deny(
            format!("Pre-guard received no content for {}. Failing closed.", event.path),
            "Provide the full post-write content in the hook event.".to_string(),
        );
    }
    if hits.is_empty() {
        return PreVerdict {
            decision: PreDecision::Allow,
            report_reason: None,
            rule_id: None,
            fix: None,
        };
    }
    let first: &RuleHit = &hits[0];
    let reason: String = format!("Blocked {}: {} Matched: {:?}.", event.path, first.message, first.matched);
    PreVerdict {
        decision: PreDecision::Deny,
        report_reason: Some(reason),
        rule_id: Some(first.rule_id.clone()),
        fix: first.fix.clone(),
    }
}

/// Minimal `**`-glob matcher over root-relative paths (enough for the table's scopes).
/// `**` matches any run including `/`; `*` matches any run except `/`; `?` one non-`/`.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    glob_match_inner(pattern.as_bytes(), path.as_bytes())
}

fn glob_match_inner(pat: &[u8], s: &[u8]) -> bool {
    if pat.is_empty() {
        return s.is_empty();
    }
    if pat.len() >= 2 && pat[0] == b'*' && pat[1] == b'*' {
        let mut rest: &[u8] = &pat[2..];
        if !rest.is_empty() && rest[0] == b'/' {
            rest = &rest[1..];
            if glob_match_inner(rest, s) {
                return true;
            }
        }
        let mut i: usize = 0;
        while i <= s.len() {
            if glob_match_inner(rest, &s[i..]) {
                return true;
            }
            i += 1;
        }
        return false;
    }
    if pat[0] == b'*' {
        let mut i: usize = 0;
        while i <= s.len() && (i == s.len() || s[i] != b'/') {
            if glob_match_inner(&pat[1..], &s[i..]) {
                return true;
            }
            i += 1;
        }
        return false;
    }
    if pat[0] == b'?' {
        return !s.is_empty() && s[0] != b'/' && glob_match_inner(&pat[1..], &s[1..]);
    }
    if s.is_empty() || pat[0] != s[0] {
        return false;
    }
    glob_match_inner(&pat[1..], &s[1..])
}

/// Scope gate: excluded wins, else any scope glob must match (default `**`).
pub fn rule_applies(rule: &DenyRule, path: &str) -> bool {
    if rule.exclude.iter().any(|g| glob_match(g, path)) {
        return false;
    }
    if rule.scope.is_empty() {
        return true;
    }
    rule.scope.iter().any(|g| glob_match(g, path))
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Literal scan with the table's regex semantics preserved per-rule:
/// - no-secrets-aws-key: `AKIA` + 16 [0-9A-Z]
/// - no-private-key: `-----BEGIN [RSA |EC |OPENSSH ]PRIVATE KEY-----`
/// - no-hardcoded-password: case-insensitive password|passwd|pwd, `:`/`=`, quoted 3+ chars
/// - no-env-commit: any content (single `.` = any char incl. newline)
/// - no-eval: word-boundary `eval` + optional spaces + `(`
/// - no-dangerous-html: literal `dangerouslySetInnerHTML`
/// - no-innerhtml-sink: `.innerHTML` + optional spaces + `=`
/// - no-debugger: word-boundary `debugger`
pub fn find_match(rule_id: &str, content: &str) -> Option<String> {
    match rule_id {
        "no-secrets-aws-key" => {
            let b: &[u8] = content.as_bytes();
            let mut i: usize = 0;
            while i + 20 <= b.len() {
                if &b[i..i + 4] == b"AKIA"
                    && b[i + 4..i + 20].iter().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
                {
                    return Some(content[i..i + 20].to_string());
                }
                i += 1;
            }
            None
        }
        "no-private-key" => {
            let markers: [&str; 4] = [
                "-----BEGIN PRIVATE KEY-----",
                "-----BEGIN RSA PRIVATE KEY-----",
                "-----BEGIN EC PRIVATE KEY-----",
                "-----BEGIN OPENSSH PRIVATE KEY-----",
            ];
            for m in markers {
                if content.contains(m) {
                    return Some(m.to_string());
                }
            }
            None
        }
        "no-hardcoded-password" => {
            let keys: [&str; 3] = ["password", "passwd", "pwd"];
            let lower: String = content.to_lowercase();
            for key in keys {
                let mut start: usize = 0;
                while let Some(rel) = lower[start..].find(key) {
                    let ks: usize = start + rel;
                    let ke: usize = ks + key.len();
                    let rest: &str = &content[ke..];
                    let rest_trim: &str = rest.trim_start_matches([' ', '\t']);
                    if let Some(sep) = rest_trim.chars().next() {
                        if sep == ':' || sep == '=' {
                            let after: &str = rest_trim[sep.len_utf8()..].trim_start();
                            if let Some(q) = after.chars().next() {
                                if q == '\'' || q == '"' {
                                    let inner: &str = &after[q.len_utf8()..];
                                    if let Some(end) = inner.find(q) {
                                        if inner[..end].chars().count() >= 3 {
                                            let mut m: String = content[ks..ke].to_string();
                                            m.push_str(&rest[..rest.len() - after.len()]);
                                            m.push(q);
                                            m.push_str(&inner[..end]);
                                            m.push(q);
                                            return Some(m);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    start = ke;
                }
            }
            None
        }
        "no-env-commit" => {
            if content.is_empty() {
                None
            } else {
                Some(content.chars().take(120).collect())
            }
        }
        "no-eval" => {
            let b: &[u8] = content.as_bytes();
            let mut i: usize = 0;
            while i + 4 <= b.len() {
                if &b[i..i + 4] == b"eval" {
                    let prev_ok: bool = i == 0 || !is_word_char(b[i - 1] as char);
                    if prev_ok {
                        let mut j: usize = i + 4;
                        while j < b.len() && (b[j] == b' ' || b[j] == b'\t' || b[j] == b'\n' || b[j] == b'\r') {
                            j += 1;
                        }
                        if j < b.len() && b[j] == b'(' {
                            return Some(content[i..j + 1].to_string());
                        }
                    }
                }
                i += 1;
            }
            None
        }
        "no-dangerous-html" => {
            if content.contains("dangerouslySetInnerHTML") {
                Some("dangerouslySetInnerHTML".to_string())
            } else {
                None
            }
        }
        "no-innerhtml-sink" => {
            let mut idx: usize = 0;
            while let Some(rel) = content[idx..].find(".innerHTML") {
                let s: usize = idx + rel;
                let rest: &str = &content[s + ".innerHTML".len()..];
                let trimmed: &str = rest.trim_start();
                if trimmed.starts_with('=') {
                    let mut m: String = ".innerHTML".to_string();
                    m.push_str(&rest[..rest.len() - trimmed.len()]);
                    m.push('=');
                    return Some(m);
                }
                idx = s + 1;
            }
            None
        }
        "no-debugger" => {
            let mut idx: usize = 0;
            while let Some(rel) = contains_pos(&content[idx..], "debugger") {
                let s: usize = idx + rel;
                let prev_ok: bool = s == 0 || !is_word_char(content[..s].chars().last().unwrap_or(' '));
                let after: &str = &content[s + "debugger".len()..];
                let next_ok: bool = after.chars().next().map(|c| !is_word_char(c)).unwrap_or(true);
                if prev_ok && next_ok {
                    return Some("debugger".to_string());
                }
                idx = s + 1;
            }
            None
        }
        _ => None,
    }
}

fn contains_pos(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

/// Full scan: first deny-rule match wins (rule-table order), mirrors _engine.scan severities={"deny"}.
pub fn scan(path: &str, content: &str) -> Vec<RuleHit> {
    let mut findings: Vec<RuleHit> = Vec::new();
    for rule in DENY_RULES {
        if !rule_applies(rule, path) {
            continue;
        }
        if let Some(m) = find_match(rule.id, content) {
            let mut matched: String = m.chars().take(120).collect();
            if matched.is_empty() {
                matched = m;
            }
            findings.push(RuleHit {
                rule_id: rule.id.to_string(),
                message: rule.message.to_string(),
                matched,
                fix: Some(rule.fix.to_string()),
            });
        }
    }
    findings
}

/// Top-level hook function: event in, typed verdict out. Fails closed on schema errors.
pub fn pre_tool_guard(event: &PreWriteEvent) -> PreVerdict {
    if event.path.is_empty() {
        return schema_deny(
            "Pre-guard received no path. Failing closed.".to_string(),
            "Provide the root-relative path in the hook event.".to_string(),
        );
    }
    let content: &str = match event.content.as_deref() {
        Some(c) => c,
        None => {
            return schema_deny(
                format!("Pre-guard received no content for {}. Failing closed.", event.path),
                "Provide the full post-write content in the hook event.".to_string(),
            )
        }
    };
    let hits: Vec<RuleHit> = scan(&event.path, content);
    decide_pre_write(event, &hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_clean_ts() {
        let event: PreWriteEvent = PreWriteEvent {
            event: "pre_write".to_string(),
            workspace_root: "/tmp/WR".to_string(),
            timestamp: "2026-10-30T12:00:00Z".to_string(),
            path: "src/clean.ts".to_string(),
            content: Some("export const x: number = 1;\n".to_string()),
            operation: "write_file".to_string(),
        };
        let v: PreVerdict = pre_tool_guard(&event);
        assert_eq!(v.decision, PreDecision::Allow);
        assert_eq!(v.rule_id, None);
    }

    #[test]
    fn deny_eval_ts() {
        let event: PreWriteEvent = PreWriteEvent {
            event: "pre_write".to_string(),
            workspace_root: "/tmp/WR".to_string(),
            timestamp: "2026-10-30T12:00:00Z".to_string(),
            path: "src/app.ts".to_string(),
            content: Some("const r = eval(userInput);\n".to_string()),
            operation: "write_file".to_string(),
        };
        let v: PreVerdict = pre_tool_guard(&event);
        assert_eq!(v.decision, PreDecision::Deny);
        assert_eq!(v.rule_id, Some("no-eval".to_string()));
    }

    #[test]
    fn deny_akia_key() {
        let event: PreWriteEvent = PreWriteEvent {
            event: "pre_write".to_string(),
            workspace_root: "/tmp/WR".to_string(),
            timestamp: "2026-10-30T12:00:00Z".to_string(),
            path: "src/keys.ts".to_string(),
            content: Some("export const K = \"AKIAIOSFODNN7EXAMPLE\";\n".to_string()),
            operation: "write_file".to_string(),
        };
        let v: PreVerdict = pre_tool_guard(&event);
        assert_eq!(v.decision, PreDecision::Deny);
        assert_eq!(v.rule_id, Some("no-secrets-aws-key".to_string()));
    }

    #[test]
    fn deny_missing_path_fails_closed() {
        let event: PreWriteEvent = PreWriteEvent {
            event: "pre_write".to_string(),
            workspace_root: "/tmp/WR".to_string(),
            timestamp: "2026-10-30T12:00:00Z".to_string(),
            path: "".to_string(),
            content: Some("x".to_string()),
            operation: "write_file".to_string(),
        };
        let v: PreVerdict = pre_tool_guard(&event);
        assert_eq!(v.decision, PreDecision::Deny);
        assert_eq!(v.rule_id, Some("schema-error".to_string()));
    }
}
