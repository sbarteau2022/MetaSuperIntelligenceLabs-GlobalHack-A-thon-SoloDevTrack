//! session_init.rs — SessionStart. Detects the stack, injects constraints. NEVER blocks.
//!
//! Date: 2026-10-31. Status: MIGRATED (Rust port of /skills original).
//! Original: skills/web-development/hooks/scripts/session_init.py
//! Contract: HOOK_EVENT_SCHEMA.md (`session_start` event).
//! Reads: common envelope (event/workspace_root/timestamp).
//! Writes: SessionVerdict { system_message, detected }. Exit 0 always.
//!
//! TS-subset shim first: ts_check TRUE (strict, exit 0, 2026-10-31); ts_to_rust ok TRUE with
//! one disclosed unsupported item ("object literal (no target struct known in expression)") —
//! the struct-return expression was completed by hand below (explicit, not silent).
//!
//! No network, no socket, no process spawn. Only reads `<workspace_root>/package.json`
//! (if present) and returns owned data; the caller decides where to write. All state
//! writes belong under workspace_root/.agent-state or temp (proof (a)).

use std::fs;
use std::path::{Path, PathBuf};

/// Typed input: the common `session_start` envelope.
#[derive(Debug, Clone)]
pub struct SessionEvent {
    pub event: String,
    pub workspace_root: String,
    pub timestamp: String,
}

/// Typed output: what the hook detected in the workspace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionDetected {
    pub package_json: bool,
    pub package_json_unreadable: bool,
    pub deps: Vec<String>,
    pub gates: Vec<String>,
}

/// Typed output: the hook verdict — constraints plus detections. Never blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionVerdict {
    pub system_message: String,
    pub detected: SessionDetected,
}

/// Gate script names surfaced from package.json `scripts` (mirrors session_init.py).
pub static KNOWN_GATES: &[&str] = &["build", "test", "lint", "typecheck", "audit"];

/// Base constraint text (mirrors session_init.py verbatim).
pub fn base_constraints() -> String {
    "Global logic: skills/web-development/GLOBAL-LOGIC.md. Binding rules: RULES.md. | Skill files load on trigger only; RULES.md loads always. | Every generative task declares BRIEF + dials before generating (GLOBAL-LOGIC.md section 7). | No secrets in files. No stubs without a tracked issue. Semantic tokens only in components.".to_string()
}

/// Full system message; appends the audit hint when the project has an audit gate.
/// Mirrors build_system_message in the TS-subset shim (ts_check TRUE 2026-10-31).
pub fn build_system_message(has_audit_gate: bool) -> String {
    let base: String = base_constraints();
    if has_audit_gate {
        return base + " | Batch audit available via the project's audit script before completion.";
    }
    base
}

/// Pure verdict constructor from already-gathered detections.
/// Mirrors decide_session in the TS-subset shim (hand-completed struct return, disclosed).
pub fn decide_session(detected: SessionDetected, has_audit_gate: bool) -> SessionVerdict {
    let message: String = build_system_message(has_audit_gate);
    SessionVerdict { system_message: message, detected }
}

/// Resolve the package.json path for a workspace root. Stays inside the root.
pub fn package_json_path(workspace_root: &str) -> PathBuf {
    Path::new(workspace_root).join("package.json")
}

/// Minimal JSON field extractor for the two shapes session_init needs:
/// string-array-ish maps under `dependencies`/`devDependencies` keys and `scripts` keys.
/// Parses quoted keys at any depth inside the named top-level object only.
fn object_string_keys(json: &str, object: &str) -> Option<Vec<String>> {
    let key_pat: String = format!("\"{}\"", object);
    let mut search_from: usize = 0;
    while let Some(rel) = json[search_from..].find(key_pat.as_str()) {
        let ks: usize = search_from + rel;
        let mut i: usize = ks + key_pat.len();
        while i < json.len() && (json.as_bytes()[i] == b' ' || json.as_bytes()[i] == b'\t' || json.as_bytes()[i] == b'\n' || json.as_bytes()[i] == b'\r') {
            i += 1;
        }
        if i < json.len() && json.as_bytes()[i] == b':' {
            i += 1;
            while i < json.len() && (json.as_bytes()[i] == b' ' || json.as_bytes()[i] == b'\t' || json.as_bytes()[i] == b'\n' || json.as_bytes()[i] == b'\r') {
                i += 1;
            }
            if i < json.len() && json.as_bytes()[i] == b'{' {
                return Some(collect_keys(json, i));
            }
        }
        search_from = ks + 1;
    }
    None
}

/// Collect immediate `"key"` members of the object starting at `brace` (byte index of `{`).
fn collect_keys(json: &str, brace: usize) -> Vec<String> {
    let bytes: &[u8] = json.as_bytes();
    let mut keys: Vec<String> = Vec::new();
    let mut i: usize = brace + 1;
    let mut depth: usize = 0;
    while i < bytes.len() {
        let c: u8 = bytes[i];
        if c == b'{' || c == b'[' {
            depth += 1;
            i += 1;
            continue;
        }
        if c == b'}' || c == b']' {
            if depth == 0 {
                break;
            }
            depth -= 1;
            i += 1;
            continue;
        }
        if c == b'"' && depth == 0 {
            let mut j: usize = i + 1;
            let mut key: String = String::new();
            while j < bytes.len() {
                if bytes[j] == b'\\' && j + 1 < bytes.len() {
                    key.push(bytes[j + 1] as char);
                    j += 2;
                    continue;
                }
                if bytes[j] == b'"' {
                    break;
                }
                key.push(bytes[j] as char);
                j += 1;
            }
            let mut k: usize = j + 1;
            while k < bytes.len() && (bytes[k] == b' ' || bytes[k] == b'\t' || bytes[k] == b'\n' || bytes[k] == b'\r') {
                k += 1;
            }
            if k < bytes.len() && bytes[k] == b':' {
                keys.push(key);
            }
            i = j + 1;
            continue;
        }
        i += 1;
    }
    keys
}

/// Detect the stack from package.json text. Never throws: unreadable JSON flags
/// package_json_unreadable (mirrors the except branch in session_init.py).
pub fn detect_from_package_json_text(text: &str) -> SessionDetected {
    let deps: Vec<String> = match object_string_keys(text, "dependencies") {
        Some(v) => v,
        None => {
            let trimmed: &str = text.trim();
            if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
                return SessionDetected { package_json: true, package_json_unreadable: true, ..Default::default() };
            }
            return SessionDetected { package_json: true, ..Default::default() };
        }
    };
    let dev_deps: Vec<String> = object_string_keys(text, "devDependencies").unwrap_or_default();
    let mut all: Vec<String> = deps;
    for d in dev_deps {
        if !all.contains(&d) {
            all.push(d);
        }
    }
    all.sort();
    let scripts: Vec<String> = object_string_keys(text, "scripts").unwrap_or_default();
    let mut gates: Vec<String> = Vec::new();
    for gate in KNOWN_GATES {
        if scripts.iter().any(|s| s == gate) {
            gates.push(gate.to_string());
        }
    }
    SessionDetected { package_json: true, package_json_unreadable: false, deps: all, gates }
}

/// Top-level hook function: envelope in, verdict out. Never blocks, never fails.
pub fn session_init(event: &SessionEvent) -> SessionVerdict {
    let mut detected: SessionDetected = SessionDetected::default();
    let pkg: PathBuf = package_json_path(&event.workspace_root);
    if pkg.is_file() {
        match fs::read_to_string(&pkg) {
            Ok(text) => {
                detected = detect_from_package_json_text(&text);
            }
            Err(_) => {
                detected = SessionDetected { package_json: true, package_json_unreadable: true, ..Default::default() };
            }
        }
    }
    let has_audit_gate: bool = detected.gates.iter().any(|g| g == "audit");
    decide_session(detected, has_audit_gate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_workspace_never_blocks() {
        let event: SessionEvent = SessionEvent {
            event: "session_start".to_string(),
            workspace_root: "/tmp/hooks_a3/sess_empty".to_string(),
            timestamp: "2026-10-30T12:00:00Z".to_string(),
        };
        let v: SessionVerdict = session_init(&event);
        assert_eq!(v.detected, SessionDetected::default());
        assert!(!v.system_message.is_empty());
        assert!(!v.system_message.contains("Batch audit"));
    }

    #[test]
    fn detects_package_gates() {
        let text: &str = "{\"dependencies\":{\"react\":\"^19\"},\"devDependencies\":{\"vitest\":\"^3\"},\"scripts\":{\"build\":\"x\",\"test\":\"y\",\"lint\":\"z\",\"typecheck\":\"t\",\"audit\":\"a\"}}";
        let d: SessionDetected = detect_from_package_json_text(text);
        assert!(d.package_json);
        assert_eq!(d.deps, vec!["react".to_string(), "vitest".to_string()]);
        assert_eq!(d.gates, vec!["build".to_string(), "test".to_string(), "lint".to_string(), "typecheck".to_string(), "audit".to_string()]);
        let v: SessionVerdict = decide_session(d, true);
        assert!(v.system_message.contains("Batch audit"));
    }

    #[test]
    fn malformed_package_json_flags_unreadable() {
        let d: SessionDetected = detect_from_package_json_text("not json at all");
        assert!(d.package_json);
        assert!(d.package_json_unreadable);
    }
}
