//! post_edit_verify.rs — PostToolUse warn-list appender. Compiled Rust port.
//!
//! Date: 2026-10-31. Status: MIGRATED (Rust port of /skills original).
//! Original: skills/web-development/hooks/scripts/post_edit_verify.py
//! Contract: HOOK_EVENT_SCHEMA.md (`post_write` event). Stdout `{}` always, exit 0,
//! side effect APPENDS (never overwrites) `.agent-state/pending-issues.jsonl`.
//!
//! Typed input: PostWriteEvent (envelope event/workspace_root/timestamp + path/content/operation).
//! Typed output: () — verdict-free by contract; findings are appended, not returned.
//!
//! TS-subset shim first: ts_check TRUE (strict, exit 0, 2026-10-31); ts_to_rust ok TRUE with
//! disclosed unsupported items (C-style for loops, struct literals) — completed by hand
//! below (explicit, not silent).
//!
//! Rule source: rules-table.json severity=warn rules, compiled in verbatim (VERIFIED REPO:
//! 9 warn rules in rules-table.json, 2026-10-31).
//!
//! No network, no process spawn. Only fs effect: append to
//! <workspace_root>/.agent-state/pending-issues.jsonl (created if missing).

use std::fs;
use std::path::{Path, PathBuf};

/// Typed input: one `post_write` hook event (common envelope + path/content/operation).
#[derive(Debug, Clone)]
pub struct PostWriteEvent {
    pub event: String,
    pub workspace_root: String,
    pub timestamp: String,
    pub path: String,
    pub content: Option<String>,
    pub operation: String,
}

/// One compiled warn rule: verbatim (id, scope globs, exclude, pattern, message, fix).
#[derive(Debug, Clone)]
pub struct WarnRule {
    pub id: &'static str,
    pub scope: &'static [&'static str],
    pub exclude: &'static [&'static str],
    pub pattern: &'static str,
    pub message: &'static str,
    pub fix: &'static str,
}

/// One finding: first match per rule (mirrors `_engine.scan` severities={"warn"}).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleHit {
    pub rule_id: String,
    pub message: String,
    pub matched: String,
    pub fix: Option<String>,
}

/// One pending-issue line: `{"ts","path","rule_id","message","fix"}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingIssue {
    pub ts: String,
    pub path: String,
    pub rule_id: String,
    pub message: String,
    pub fix: Option<String>,
}

/// The 9 warn rules from rules-table.json (VERIFIED REPO 2026-10-31), verbatim.
pub static WARN_RULES: &[WarnRule] = &[
    WarnRule {
        id: "no-fake-heading",
        scope: &[
            "**/*.tsx",
            "**/*.jsx",
            "**/*.html",
            "**/*.vue",
            "**/*.svelte",
        ],
        exclude: &[],
        pattern: "<(div|span)[^>]*class\\s*=\\s*[\"'][^\"']*(h1|h2|h3|heading|title)[^\"']*[\"']",
        message: "Possible fake heading (styled div/span instead of h1-h6).",
        fix: "Use a real heading element with sequential level.",
    },
    WarnRule {
        id: "no-global-reset",
        scope: &["**/*.css"],
        exclude: &[],
        pattern: "^\\s*\\*\\s*\\{",
        message: "Global * reset cannot be overridden by components or lower layers.",
        fix: "Target specific elements/conditions instead.",
    },
    WarnRule {
        id: "no-important",
        scope: &["**/*.css"],
        exclude: &[],
        pattern: "!important",
        message: "!important outside the utilities layer hides the real conflict.",
        fix: "Fix layer order or scope the rule; keep !important in utilities only.",
    },
    WarnRule {
        id: "no-transition-all",
        scope: &["**/*.css"],
        exclude: &[],
        pattern: "transition(-property)?\\s*:\\s*all\\b",
        message: "transition: all animates unintended properties and hurts performance.",
        fix: "List the animated properties explicitly (prefer transform/opacity).",
    },
    WarnRule {
        id: "no-linear-ease",
        scope: &["**/*.css"],
        exclude: &[],
        pattern: "transition-timing-function\\s*:\\s*(linear|ease-in-out)\\b",
        message: "linear/ease-in-out reads as default/generic motion.",
        fix: "Use a token easing curve (see frontend-design-language/tokens/tokens.css).",
    },
    WarnRule {
        id: "no-hardcoded-color",
        scope: &["**/*.css"],
        exclude: &["**/tokens/tokens.css"],
        pattern: "#[0-9a-fA-F]{3,8}\\b",
        message: "Literal color in CSS bypasses the token system.",
        fix: "Use a semantic token from frontend-design-language/tokens/.",
    },
    WarnRule {
        id: "no-todo-stub",
        scope: &[
            "**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx", "**/*.py", "**/*.css",
        ],
        exclude: &[],
        pattern: "(TODO|FIXME|XXX|HACK|//\\s*add logic here|\\.\\.\\.)",
        message: "Stub marker will ship if the stop-gate is bypassed.",
        fix: "Implement it now or file a tracked issue and reference its id.",
    },
    WarnRule {
        id: "no-console-log",
        scope: &["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"],
        exclude: &[],
        pattern: "console\\.(log|debug)\\s*\\(",
        message: "console.log/debug left in code.",
        fix: "Remove or route through the logging middleware.",
    },
    WarnRule {
        id: "no-select-star",
        scope: &["**/*.sql", "**/*.ts", "**/*.js", "**/*.py"],
        exclude: &[],
        pattern: "(?i)\\bselect\\s+\\*\\b",
        message: "SELECT * couples queries to schema order and over-fetches.",
        fix: "List the columns explicitly.",
    },
];

/// Minimal `**`-glob matcher over root-relative paths (same as Agent 3's port).
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
pub fn rule_applies(rule: &WarnRule, path: &str) -> bool {
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

fn contains_pos(haystack: &str, needle: &str) -> Option<usize> {
    haystack.find(needle)
}

fn is_hex(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

/// Literal scan with the table's regex semantics preserved per-rule (warn severities only).
pub fn find_match(rule_id: &str, content: &str) -> Option<String> {
    match rule_id {
        "no-fake-heading" => find_fake_heading(content),
        "no-global-reset" => find_global_reset(content),
        "no-important" => {
            if content.contains("!important") {
                Some("!important".to_string())
            } else {
                None
            }
        }
        "no-transition-all" => find_transition_all(content),
        "no-linear-ease" => find_linear_ease(content),
        "no-hardcoded-color" => find_hardcoded_color(content),
        "no-todo-stub" => find_todo_stub(content),
        "no-console-log" => find_console_log(content),
        "no-select-star" => find_select_star(content),
        _ => None,
    }
}

/// `<(div|span)[^>]*class\s*=\s*["'][^"']*(h1|h2|h3|heading|title)[^"']*["']`
fn find_fake_heading(content: &str) -> Option<String> {
    let mut idx: usize = 0;
    while idx < content.len() {
        let rel_div: Option<usize> = contains_pos(&content[idx..], "<div");
        let rel_span: Option<usize> = contains_pos(&content[idx..], "<span");
        let start: usize = match (rel_div, rel_span) {
            (Some(d), Some(s)) => idx + d.min(s),
            (Some(d), None) => idx + d,
            (None, Some(s)) => idx + s,
            (None, None) => return None,
        };
        let rest: &str = &content[start..];
        let tag_end: usize = rest.find('>')?;
        let tag: &str = &rest[..tag_end];
        if let Some(m) = fake_heading_class_match(tag) {
            let end: usize = start + tag_end + 1;
            let matched: String = content[start..end.min(content.len())]
                .chars()
                .take(120)
                .collect();
            let _ = m;
            return Some(matched);
        }
        idx = start + 4;
    }
    None
}

fn fake_heading_class_match(tag: &str) -> Option<String> {
    let class_pos: usize = tag.find("class")?;
    let after_class: &str = &tag[class_pos + "class".len()..];
    let after_ws: &str = after_class.trim_start_matches([' ', '\t', '\n', '\r']);
    if !after_ws.starts_with('=') {
        return None;
    }
    let after_eq: &str = after_ws[1..].trim_start();
    let q: char = after_eq.chars().next()?;
    if q != '"' && q != '\'' {
        return None;
    }
    let inner: &str = &after_eq[q.len_utf8()..];
    let end: usize = inner.find(q)?;
    let cls: &str = &inner[..end];
    let lower: String = cls.to_lowercase();
    for key in ["h1", "h2", "h3", "heading", "title"] {
        if lower.contains(key) {
            return Some(cls.to_string());
        }
    }
    None
}

/// `^\s*\*\s*\{` with MULTILINE: any line whose first non-ws char is `*` then optional ws then `{`.
fn find_global_reset(content: &str) -> Option<String> {
    for line in content.lines() {
        let t: &str = line.trim_start_matches([' ', '\t', '\r']);
        if !t.starts_with('*') {
            continue;
        }
        let after: &str = t[1..].trim_start_matches([' ', '\t']);
        if after.starts_with('{') {
            let matched: String = line.chars().take(120).collect();
            return Some(matched);
        }
    }
    None
}

/// `transition(-property)?\s*:\s*all\b`
fn find_transition_all(content: &str) -> Option<String> {
    let mut idx: usize = 0;
    while let Some(rel) = contains_pos(&content[idx..], "transition") {
        let s: usize = idx + rel;
        let rest: &str = &content[s + "transition".len()..];
        let prop_ok: bool = rest.starts_with("-property");
        let after_prop: &str = if prop_ok {
            &rest["-property".len()..]
        } else {
            rest
        };
        let trimmed: &str = after_prop.trim_start();
        if let Some(after_colon) = trimmed.strip_prefix(':') {
            let val: &str = after_colon.trim_start();
            if val == "all"
                || val.starts_with("all ")
                || val.starts_with("all\t")
                || val.starts_with("all\n")
                || val.starts_with("all;")
                || val.starts_with("all,")
            {
                let end: usize = s + "transition".len() + (after_prop.len() - trimmed.len()) + 1;
                let tail: usize = (end + 3).min(content.len());
                return Some(content[s..tail].to_string());
            }
        }
        idx = s + 1;
    }
    None
}

/// `transition-timing-function\s*:\s*(linear|ease-in-out)\b`
fn find_linear_ease(content: &str) -> Option<String> {
    let key: &str = "transition-timing-function";
    let mut idx: usize = 0;
    while let Some(rel) = contains_pos(&content[idx..], key) {
        let s: usize = idx + rel;
        let rest: &str = &content[s + key.len()..];
        let trimmed: &str = rest.trim_start();
        if let Some(after_colon) = trimmed.strip_prefix(':') {
            let val: &str = after_colon.trim_start();
            for cand in ["linear", "ease-in-out"] {
                if val == cand
                    || val.starts_with(&format!("{cand} "))
                    || val.starts_with(&format!("{cand};"))
                    || val.starts_with(&format!("{cand},"))
                    || val.starts_with(&format!("{cand}\t"))
                    || val.starts_with(&format!("{cand}\n"))
                {
                    let tail: usize =
                        (s + key.len() + (rest.len() - trimmed.len()) + 1 + cand.len())
                            .min(content.len());
                    return Some(content[s..tail].to_string());
                }
            }
        }
        idx = s + 1;
    }
    None
}

/// `#[0-9a-fA-F]{3,8}\b`
fn find_hardcoded_color(content: &str) -> Option<String> {
    let b: &[u8] = content.as_bytes();
    let mut i: usize = 0;
    while i < b.len() {
        if b[i] == b'#' {
            let mut j: usize = i + 1;
            while j < b.len() && j - (i + 1) < 8 && is_hex(b[j]) {
                j += 1;
            }
            let n: usize = j - (i + 1);
            if (3..=8).contains(&n) {
                let next_ok: bool = j >= b.len() || !is_word_char(b[j] as char);
                if next_ok {
                    return Some(content[i..j].to_string());
                }
            }
        }
        i += 1;
    }
    None
}

/// `(TODO|FIXME|XXX|HACK|//\s*add logic here|\.\.\.)`
fn find_todo_stub(content: &str) -> Option<String> {
    for key in ["TODO", "FIXME", "XXX", "HACK"] {
        if let Some(pos) = contains_pos(content, key) {
            let tail: usize = (pos + key.len() + 20).min(content.len());
            let head: usize = pos.saturating_sub(20);
            let ctx: String = content[head..tail].chars().take(120).collect();
            let _ = ctx;
            return Some(key.to_string());
        }
    }
    let mut idx: usize = 0;
    while let Some(rel) = contains_pos(&content[idx..], "//") {
        let s: usize = idx + rel + 2;
        let rest: &str = &content[s..];
        let trimmed: &str = rest.trim_start_matches([' ', '\t']);
        if trimmed.starts_with("add logic here") {
            return Some("// add logic here".to_string());
        }
        idx = s;
    }
    if content.contains("...") {
        return Some("...".to_string());
    }
    None
}

/// `console\.(log|debug)\s*\(`
fn find_console_log(content: &str) -> Option<String> {
    for key in ["console.log", "console.debug"] {
        let mut idx: usize = 0;
        while let Some(rel) = contains_pos(&content[idx..], key) {
            let s: usize = idx + rel;
            let rest: &str = &content[s + key.len()..];
            let trimmed: &str = rest.trim_start();
            if trimmed.starts_with('(') {
                return Some(format!("{key}("));
            }
            idx = s + 1;
        }
    }
    None
}

/// `(?i)\bselect\s+\*\b` — case-insensitive SELECT, whitespace run, `*` word boundary.
fn find_select_star(content: &str) -> Option<String> {
    let lower: String = content.to_lowercase();
    let mut idx: usize = 0;
    while let Some(rel) = contains_pos(&lower[idx..], "select") {
        let s: usize = idx + rel;
        let prev_ok: bool = s == 0 || !is_word_char(content[..s].chars().last().unwrap_or(' '));
        if prev_ok {
            let rest: &str = &content[s + "select".len()..];
            if rest.starts_with([' ', '\t', '\n', '\r']) {
                let trimmed: &str = rest.trim_start();
                if let Some(after) = trimmed.strip_prefix('*') {
                    let next_ok: bool = after
                        .chars()
                        .next()
                        .map(|c| !is_word_char(c))
                        .unwrap_or(true);
                    if next_ok {
                        let end: usize = s + "select".len() + (rest.len() - trimmed.len()) + 1;
                        return Some(content[s..end.min(content.len())].to_string());
                    }
                }
            }
        }
        idx = s + 1;
    }
    None
}

/// Full scan: one finding per warn rule on first match (rule-table order), mirrors `_engine.scan`.
pub fn scan(path: &str, content: &str) -> Vec<RuleHit> {
    let mut findings: Vec<RuleHit> = Vec::new();
    for rule in WARN_RULES {
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

/// Pure decision: event + hits → pending issues (mirrors TS shim `decide_post_write`).
/// Empty path or missing content → no issues (never blocks, per contract).
pub fn decide_post_write(event: &PostWriteEvent, hits: &[RuleHit]) -> Vec<PendingIssue> {
    let mut out: Vec<PendingIssue> = Vec::new();
    if event.path.is_empty() {
        return out;
    }
    if event.content.is_none() {
        return out;
    }
    for h in hits {
        out.push(PendingIssue {
            ts: event.timestamp.clone(),
            path: event.path.clone(),
            rule_id: h.rule_id.clone(),
            message: h.message.clone(),
            fix: h.fix.clone(),
        });
    }
    out
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

/// Pending-issues store path (append-only target).
pub fn pending_path(workspace_root: &str) -> PathBuf {
    state_dir(workspace_root).join("pending-issues.jsonl")
}

/// Escape a string for inclusion in a flat JSON object (minimal: `"` `\` + control chars).
pub fn json_escape(s: &str) -> String {
    let mut out: String = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Serialize one pending issue to a single JSONL line (field order per .py: ts,path,rule_id,message,fix).
pub fn issue_to_json_line(issue: &PendingIssue) -> String {
    let fix_part: String = match &issue.fix {
        Some(f) => format!("\"{}\"", json_escape(f)),
        None => "null".to_string(),
    };
    format!(
        "{{\"ts\":\"{}\",\"path\":\"{}\",\"rule_id\":\"{}\",\"message\":\"{}\",\"fix\":{}}}",
        json_escape(&issue.ts),
        json_escape(&issue.path),
        json_escape(&issue.rule_id),
        json_escape(&issue.message),
        fix_part
    )
}

/// Append issues to the store. Creates `.agent-state/` if missing. Never truncates.
/// Returns the number of lines appended. Missing content/path → 0 (no-op, exit 0).
pub fn post_edit_verify(event: &PostWriteEvent) -> usize {
    let content: &str = match event.content.as_deref() {
        Some(c) => c,
        None => return 0,
    };
    if event.path.is_empty() {
        return 0;
    }
    let hits: Vec<RuleHit> = scan(&event.path, content);
    let issues: Vec<PendingIssue> = decide_post_write(event, &hits);
    if issues.is_empty() {
        return 0;
    }
    let dir: PathBuf = state_dir(&event.workspace_root);
    if fs::create_dir_all(&dir).is_err() {
        return 0;
    }
    let store: PathBuf = dir.join("pending-issues.jsonl");
    let mut buf: String = String::new();
    if let Ok(existing) = fs::read_to_string(&store) {
        buf.push_str(&existing);
        if !existing.is_empty() && !existing.ends_with('\n') {
            buf.push('\n');
        }
    }
    for issue in &issues {
        buf.push_str(&issue_to_json_line(issue));
        buf.push('\n');
    }
    match fs::write(&store, buf) {
        Ok(()) => issues.len(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evt(path: &str, content: Option<&str>) -> PostWriteEvent {
        PostWriteEvent {
            event: "post_write".to_string(),
            workspace_root: "/tmp/x".to_string(),
            timestamp: "2026-10-31T00:00:00Z".to_string(),
            path: path.to_string(),
            content: content.map(|s| s.to_string()),
            operation: "write_file".to_string(),
        }
    }

    #[test]
    fn clean_ts_appends_nothing() {
        let hits: Vec<RuleHit> = scan("src/app.ts", "const x: number = 1;\n");
        assert!(hits.is_empty());
        let issues: Vec<PendingIssue> =
            decide_post_write(&evt("src/app.ts", Some("const x = 1;")), &hits);
        assert!(issues.is_empty());
    }

    #[test]
    fn important_in_css_warns() {
        let hits: Vec<RuleHit> = scan("a/b.css", ".x { color: red !important; }\n");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "no-important");
    }

    #[test]
    fn console_log_in_ts_warns() {
        let hits: Vec<RuleHit> = scan("src/a.ts", "console.log(x);\n");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "no-console-log");
    }

    #[test]
    fn hardcoded_color_excluded_under_tokens() {
        let hits: Vec<RuleHit> = scan("frontend/tokens/tokens.css", ":root { --c: #fff; }\n");
        assert!(hits.is_empty());
    }

    #[test]
    fn missing_content_is_noop() {
        let issues: Vec<PendingIssue> = decide_post_write(&evt("a/b.css", None), &[]);
        assert!(issues.is_empty());
    }

    #[test]
    fn issue_line_field_order() {
        let issue: PendingIssue = PendingIssue {
            ts: "t".to_string(),
            path: "p".to_string(),
            rule_id: "r".to_string(),
            message: "m".to_string(),
            fix: None,
        };
        assert_eq!(
            issue_to_json_line(&issue),
            "{\"ts\":\"t\",\"path\":\"p\",\"rule_id\":\"r\",\"message\":\"m\",\"fix\":null}"
        );
    }
}
