//! deny_rules — 9 compiled Deny-typed predicate fns from rules-table.json.
//!
//! Date: 2026-10-31. Agent C3 FINAL round (Wave-1 gap closure: adds no-div-button).
//! Source (read-only): N3/web-development/hooks/rules-table.json v1.0.0 (severity=deny, 9 rules).
//! Each predicate = scope glob-set + compiled literal/matcher + severity deny
//! + consequence (Deny + rule_id + fix). Pure functions of (path, content).
//!
//! No network, no fs, no process spawn.
//!
//! Contradiction check vs `../pre_tool_guard.rs` is recorded in RECEIPT.md
//! (this crate's 8 overlapping rules are field-by-field verbatim;
//! `no-div-button` is new).

#![deny(warnings)]

/// allow|deny — predicates return `Some(DenyVerdict)` on hit, `None` on allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

/// Typed consequence of a deny hit: decision + severity + rule_id + fix + evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyVerdict {
    pub decision: Decision,
    pub severity: &'static str,
    pub rule_id: &'static str,
    pub message: &'static str,
    pub fix: &'static str,
    pub matched: String,
}

fn deny(
    rule_id: &'static str,
    message: &'static str,
    fix: &'static str,
    matched_raw: String,
) -> DenyVerdict {
    let matched: String = matched_raw.chars().take(120).collect();
    DenyVerdict {
        decision: Decision::Deny,
        severity: "deny",
        rule_id,
        message,
        fix,
        matched,
    }
}

/// Minimal `**`-glob matcher over root-relative paths (ported verbatim from
/// ../pre_tool_guard.rs so scope semantics are identical).
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

/// Scope gate: excluded wins, else any scope glob must match.
pub fn scope_applies(scope: &[&str], exclude: &[&str], path: &str) -> bool {
    if exclude.iter().any(|g| glob_match(g, path)) {
        return false;
    }
    if scope.is_empty() {
        return true;
    }
    scope.iter().any(|g| glob_match(g, path))
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

// ---------------------------------------------------------------------------
// Rule 1: no-secrets-aws-key — scope ["**"], pattern `AKIA[0-9A-Z]{16}`
// ---------------------------------------------------------------------------

/// Scope glob-set, verbatim from rules-table.json.
pub const NO_SECRETS_AWS_KEY_SCOPE: &[&str] = &["**"];
/// Pattern source, verbatim from rules-table.json.
pub const NO_SECRETS_AWS_KEY_PATTERN: &str = "AKIA[0-9A-Z]{16}";
pub const NO_SECRETS_AWS_KEY_MESSAGE: &str = "Possible AWS access key in file content.";
pub const NO_SECRETS_AWS_KEY_FIX: &str =
    "Remove the key, rotate it, and load it from a secret manager or .env (never committed).";

fn match_aws_key(content: &str) -> Option<String> {
    let b: &[u8] = content.as_bytes();
    let mut i: usize = 0;
    while i + 20 <= b.len() {
        if &b[i..i + 4] == b"AKIA"
            && b[i + 4..i + 20]
                .iter()
                .all(|c| c.is_ascii_digit() || c.is_ascii_uppercase())
        {
            return Some(content[i..i + 20].to_string());
        }
        i += 1;
    }
    None
}

/// Deny predicate: `Some` (deny + rule_id + fix) on hit, `None` (allow) otherwise.
pub fn no_secrets_aws_key(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_SECRETS_AWS_KEY_SCOPE, &[], path) {
        return None;
    }
    match_aws_key(content).map(|m| {
        deny(
            "no-secrets-aws-key",
            NO_SECRETS_AWS_KEY_MESSAGE,
            NO_SECRETS_AWS_KEY_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 2: no-private-key — scope ["**"]
// ---------------------------------------------------------------------------

pub const NO_PRIVATE_KEY_SCOPE: &[&str] = &["**"];
pub const NO_PRIVATE_KEY_PATTERN: &str = "-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----";
pub const NO_PRIVATE_KEY_MESSAGE: &str = "Private key material in file content.";
pub const NO_PRIVATE_KEY_FIX: &str = "Remove immediately, rotate the key, use a secret manager.";

fn match_private_key(content: &str) -> Option<String> {
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

pub fn no_private_key(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_PRIVATE_KEY_SCOPE, &[], path) {
        return None;
    }
    match_private_key(content).map(|m| {
        deny(
            "no-private-key",
            NO_PRIVATE_KEY_MESSAGE,
            NO_PRIVATE_KEY_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 3: no-hardcoded-password — scoped to code/config files
// ---------------------------------------------------------------------------

pub const NO_HARDCODED_PASSWORD_SCOPE: &[&str] = &[
    "**/*.ts",
    "**/*.tsx",
    "**/*.js",
    "**/*.jsx",
    "**/*.py",
    "**/*.json",
    "**/*.yml",
    "**/*.yaml",
];
pub const NO_HARDCODED_PASSWORD_PATTERN: &str =
    "(?i)(password|passwd|pwd)\\s*[:=]\\s*['\"][^'\"]{3,}['\"]";
pub const NO_HARDCODED_PASSWORD_MESSAGE: &str = "Hardcoded password/secret assignment.";
pub const NO_HARDCODED_PASSWORD_FIX: &str =
    "Read from environment or secret manager; keep only a placeholder in .env.example.";

fn match_hardcoded_password(content: &str) -> Option<String> {
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

pub fn no_hardcoded_password(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_HARDCODED_PASSWORD_SCOPE, &[], path) {
        return None;
    }
    match_hardcoded_password(content).map(|m| {
        deny(
            "no-hardcoded-password",
            NO_HARDCODED_PASSWORD_MESSAGE,
            NO_HARDCODED_PASSWORD_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 4: no-env-commit — scope ["**/.env", "**/.env.*"], pattern `.`
// ---------------------------------------------------------------------------

pub const NO_ENV_COMMIT_SCOPE: &[&str] = &["**/.env", "**/.env.*"];
pub const NO_ENV_COMMIT_PATTERN: &str = ".";
pub const NO_ENV_COMMIT_MESSAGE: &str = "Real .env file caught by the write guard.";
pub const NO_ENV_COMMIT_FIX: &str = "Unstage it; only .env.example belongs in version control.";

fn match_env_commit(content: &str) -> Option<String> {
    if content.is_empty() {
        None
    } else {
        Some(content.chars().take(120).collect())
    }
}

pub fn no_env_commit(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_ENV_COMMIT_SCOPE, &[], path) {
        return None;
    }
    match_env_commit(content).map(|m| {
        deny(
            "no-env-commit",
            NO_ENV_COMMIT_MESSAGE,
            NO_ENV_COMMIT_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 5: no-eval — scope TS/JS, pattern `\beval\s*\(`
// ---------------------------------------------------------------------------

pub const NO_EVAL_SCOPE: &[&str] = &["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"];
pub const NO_EVAL_PATTERN: &str = "\\beval\\s*\\(";
pub const NO_EVAL_MESSAGE: &str = "eval() is a code-injection surface.";
pub const NO_EVAL_FIX: &str = "Replace with JSON.parse, a lookup table, or a proper parser.";

fn match_eval(content: &str) -> Option<String> {
    let b: &[u8] = content.as_bytes();
    let mut i: usize = 0;
    while i + 4 <= b.len() {
        if &b[i..i + 4] == b"eval" {
            let prev_ok: bool = i == 0 || !is_word_char(b[i - 1] as char);
            if prev_ok {
                let mut j: usize = i + 4;
                while j < b.len()
                    && (b[j] == b' ' || b[j] == b'\t' || b[j] == b'\n' || b[j] == b'\r')
                {
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

pub fn no_eval(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_EVAL_SCOPE, &[], path) {
        return None;
    }
    match_eval(content)
        .map(|m| deny("no-eval", NO_EVAL_MESSAGE, NO_EVAL_FIX, m))
}

// ---------------------------------------------------------------------------
// Rule 6: no-dangerous-html — scope TSX/JSX, literal `dangerouslySetInnerHTML`
// ---------------------------------------------------------------------------

pub const NO_DANGEROUS_HTML_SCOPE: &[&str] = &["**/*.tsx", "**/*.jsx"];
pub const NO_DANGEROUS_HTML_PATTERN: &str = "dangerouslySetInnerHTML";
pub const NO_DANGEROUS_HTML_MESSAGE: &str = "Raw HTML injection in JSX.";
pub const NO_DANGEROUS_HTML_FIX: &str =
    "Render structured content or sanitize with a reviewed sanitizer at the sink.";

fn match_dangerous_html(content: &str) -> Option<String> {
    if content.contains("dangerouslySetInnerHTML") {
        Some("dangerouslySetInnerHTML".to_string())
    } else {
        None
    }
}

pub fn no_dangerous_html(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_DANGEROUS_HTML_SCOPE, &[], path) {
        return None;
    }
    match_dangerous_html(content).map(|m| {
        deny(
            "no-dangerous-html",
            NO_DANGEROUS_HTML_MESSAGE,
            NO_DANGEROUS_HTML_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 7: no-innerhtml-sink — scope TS/JS, pattern `\.innerHTML\s*=`
// ---------------------------------------------------------------------------

pub const NO_INNERHTML_SINK_SCOPE: &[&str] = &["**/*.ts", "**/*.js"];
pub const NO_INNERHTML_SINK_PATTERN: &str = "\\.innerHTML\\s*=";
pub const NO_INNERHTML_SINK_MESSAGE: &str = "innerHTML assignment is an XSS sink.";
pub const NO_INNERHTML_SINK_FIX: &str = "Use textContent / DOM construction or a sanitizer.";

fn match_innerhtml_sink(content: &str) -> Option<String> {
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

pub fn no_innerhtml_sink(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_INNERHTML_SINK_SCOPE, &[], path) {
        return None;
    }
    match_innerhtml_sink(content).map(|m| {
        deny(
            "no-innerhtml-sink",
            NO_INNERHTML_SINK_MESSAGE,
            NO_INNERHTML_SINK_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 8 (Wave-1 gap closure): no-div-button — scope TSX/JSX/HTML/Vue/Svelte
// Pattern: `<div[^>]*role\s*=\s*["']button["']`
// ---------------------------------------------------------------------------

pub const NO_DIV_BUTTON_SCOPE: &[&str] = &[
    "**/*.tsx",
    "**/*.jsx",
    "**/*.html",
    "**/*.vue",
    "**/*.svelte",
];
pub const NO_DIV_BUTTON_PATTERN: &str = "<div[^>]*role\\s*=\\s*[\"']button[\"']";
pub const NO_DIV_BUTTON_MESSAGE: &str =
    "div-as-button breaks keyboard and screen-reader interaction.";
pub const NO_DIV_BUTTON_FIX: &str = "Use a native <button>.";

fn find_role_button_in_tag(tag: &str) -> Option<String> {
    let b: &[u8] = tag.as_bytes();
    let mut i: usize = 0;
    while i + 4 <= b.len() {
        if &b[i..i + 4] == b"role" {
            let mut j: usize = i + 4;
            while j < b.len() && (b[j] == b' ' || b[j] == b'\t' || b[j] == b'\n' || b[j] == b'\r')
            {
                j += 1;
            }
            if j < b.len() && b[j] == b'=' {
                j += 1;
                while j < b.len()
                    && (b[j] == b' ' || b[j] == b'\t' || b[j] == b'\n' || b[j] == b'\r')
                {
                    j += 1;
                }
                if j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q: u8 = b[j];
                    let vs: usize = j + 1;
                    if vs + 6 <= b.len() && &b[vs..vs + 6] == b"button" {
                        let ve: usize = vs + 6;
                        if ve < b.len() && b[ve] == q {
                            return Some(tag[..ve + 1].to_string());
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn match_div_button(content: &str) -> Option<String> {
    let mut idx: usize = 0;
    while let Some(rel) = content[idx..].find("<div") {
        let s: usize = idx + rel;
        let after: &str = &content[s + "<div".len()..];
        let tag_end_rel: usize = after.find('>').unwrap_or(after.len());
        let tag: &str = &content[s..s + "<div".len() + tag_end_rel];
        if let Some(m) = find_role_button_in_tag(tag) {
            return Some(m);
        }
        idx = s + 1;
    }
    None
}

pub fn no_div_button(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_DIV_BUTTON_SCOPE, &[], path) {
        return None;
    }
    match_div_button(content).map(|m| {
        deny(
            "no-div-button",
            NO_DIV_BUTTON_MESSAGE,
            NO_DIV_BUTTON_FIX,
            m,
        )
    })
}

// ---------------------------------------------------------------------------
// Rule 9: no-debugger — scope TS/JS, pattern `\bdebugger\b`
// ---------------------------------------------------------------------------

pub const NO_DEBUGGER_SCOPE: &[&str] = &["**/*.ts", "**/*.tsx", "**/*.js", "**/*.jsx"];
pub const NO_DEBUGGER_PATTERN: &str = "\\bdebugger\\b";
pub const NO_DEBUGGER_MESSAGE: &str = "debugger statement halts execution for every user.";
pub const NO_DEBUGGER_FIX: &str = "Remove it.";

fn match_debugger(content: &str) -> Option<String> {
    let mut idx: usize = 0;
    while let Some(rel) = content[idx..].find("debugger") {
        let s: usize = idx + rel;
        let prev_ok: bool = s == 0
            || !is_word_char(content[..s].chars().last().unwrap_or(' '));
        let after: &str = &content[s + "debugger".len()..];
        let next_ok: bool = after
            .chars()
            .next()
            .map(|c| !is_word_char(c))
            .unwrap_or(true);
        if prev_ok && next_ok {
            return Some("debugger".to_string());
        }
        idx = s + 1;
    }
    None
}

pub fn no_debugger(path: &str, content: &str) -> Option<DenyVerdict> {
    if !scope_applies(NO_DEBUGGER_SCOPE, &[], path) {
        return None;
    }
    match_debugger(content)
        .map(|m| deny("no-debugger", NO_DEBUGGER_MESSAGE, NO_DEBUGGER_FIX, m))
}

// ---------------------------------------------------------------------------
// First-match-wins scan in rules-table.json deny order.
// ---------------------------------------------------------------------------

/// Table order scan (deny only): first hit wins, mirroring `_engine.scan severities={"deny"}`.
pub fn scan(path: &str, content: &str) -> Option<DenyVerdict> {
    no_secrets_aws_key(path, content)
        .or_else(|| no_private_key(path, content))
        .or_else(|| no_hardcoded_password(path, content))
        .or_else(|| no_env_commit(path, content))
        .or_else(|| no_eval(path, content))
        .or_else(|| no_dangerous_html(path, content))
        .or_else(|| no_innerhtml_sink(path, content))
        .or_else(|| no_div_button(path, content))
        .or_else(|| no_debugger(path, content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_no_secrets_aws_key() {
        assert_eq!(no_secrets_aws_key("src/clean.ts", "export const x: number = 1;\n"), None);
    }

    #[test]
    fn deny_no_secrets_aws_key() {
        let v = no_secrets_aws_key("src/keys.ts", "export const K = \"AKIAIOSFODNN7EXAMPLE\";\n")
            .expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-secrets-aws-key");
        assert_eq!(v.matched, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(
            v.fix,
            "Remove the key, rotate it, and load it from a secret manager or .env (never committed)."
        );
    }

    #[test]
    fn allow_no_private_key() {
        assert_eq!(no_private_key("notes.txt", "no keys here\n"), None);
    }

    #[test]
    fn deny_no_private_key() {
        let v = no_private_key("notes.txt", "-----BEGIN RSA PRIVATE KEY-----\nabc\n")
            .expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-private-key");
        assert_eq!(v.matched, "-----BEGIN RSA PRIVATE KEY-----");
        assert_eq!(
            v.fix,
            "Remove immediately, rotate the key, use a secret manager."
        );
    }

    #[test]
    fn allow_no_hardcoded_password() {
        assert_eq!(
            no_hardcoded_password("src/app.ts", "export const x: number = 1;\n"),
            None
        );
    }

    #[test]
    fn deny_no_hardcoded_password() {
        let v = no_hardcoded_password("src/app.ts", "const password = \"s3cret!\";\n")
            .expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-hardcoded-password");
        assert_eq!(v.matched, "password = \"s3cret!\"");
        assert_eq!(
            v.fix,
            "Read from environment or secret manager; keep only a placeholder in .env.example."
        );
    }

    #[test]
    fn allow_no_env_commit() {
        assert_eq!(
            no_env_commit("src/app.ts", "SECRET=abc\n"),
            None,
            "out-of-scope path must allow"
        );
    }

    #[test]
    fn deny_no_env_commit() {
        let v = no_env_commit(".env", "SECRET=abc\n").expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-env-commit");
        assert_eq!(v.matched, "SECRET=abc\n");
        assert_eq!(
            v.fix,
            "Unstage it; only .env.example belongs in version control."
        );
    }

    #[test]
    fn allow_no_eval() {
        assert_eq!(
            no_eval("src/app.ts", "const r = evaluate(x);\n"),
            None,
            "evaluate must not match word-boundary eval"
        );
    }

    #[test]
    fn deny_no_eval() {
        let v = no_eval("src/app.ts", "const r = eval(userInput);\n").expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-eval");
        assert_eq!(v.matched, "eval(");
        assert_eq!(
            v.fix,
            "Replace with JSON.parse, a lookup table, or a proper parser."
        );
    }

    #[test]
    fn allow_no_dangerous_html() {
        assert_eq!(
            no_dangerous_html("src/c.ts", "dangerouslySetInnerHTML(x);\n"),
            None,
            "out-of-scope .ts must allow"
        );
    }

    #[test]
    fn deny_no_dangerous_html() {
        let v = no_dangerous_html(
            "src/c.tsx",
            "const el = <div dangerouslySetInnerHTML={{__html: x}} />;\n",
        )
        .expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-dangerous-html");
        assert_eq!(v.matched, "dangerouslySetInnerHTML");
        assert_eq!(
            v.fix,
            "Render structured content or sanitize with a reviewed sanitizer at the sink."
        );
    }

    #[test]
    fn allow_no_innerhtml_sink() {
        assert_eq!(
            no_innerhtml_sink("src/a.tsx", "el.innerHTML = x;\n"),
            None,
            "out-of-scope .tsx must allow"
        );
    }

    #[test]
    fn deny_no_innerhtml_sink() {
        let v = no_innerhtml_sink("src/a.ts", "el.innerHTML = x;\n").expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-innerhtml-sink");
        assert_eq!(v.matched, ".innerHTML =");
        assert_eq!(
            v.fix,
            "Use textContent / DOM construction or a sanitizer."
        );
    }

    #[test]
    fn allow_no_div_button() {
        assert_eq!(
            no_div_button("src/b.tsx", "<button onClick={h}>x</button>\n"),
            None,
            "native button must allow"
        );
    }

    #[test]
    fn deny_no_div_button() {
        let v = no_div_button(
            "src/b.tsx",
            "<div role=\"button\" onClick={h}>x</div>\n",
        )
        .expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-div-button");
        assert!(v.matched.contains("role=\"button\""), "matched={}", v.matched);
        assert_eq!(v.fix, "Use a native <button>.");
    }

    #[test]
    fn allow_no_debugger() {
        assert_eq!(
            no_debugger("src/a.ts", "// no breakpoints here\n"),
            None
        );
    }

    #[test]
    fn deny_no_debugger() {
        let v = no_debugger("src/a.ts", "debugger;\n").expect("must hit");
        assert_eq!(v.decision, Decision::Deny);
        assert_eq!(v.severity, "deny");
        assert_eq!(v.rule_id, "no-debugger");
        assert_eq!(v.matched, "debugger");
        assert_eq!(v.fix, "Remove it.");
    }
}
