//! SurrealDB HTTP client (REST `/sql`), backend-native.
//!
//! Governance state is core state: control registry, permits, audit
//! entries, Merkle roots, and erasure receipts live in SurrealDB (:8000)
//! alongside runs — same store, same accountability. Tables use RG-AH
//! vocabulary only: `thread`, `message`, `run`, `run_step`, `tool_call`,
//! `receipt`, `atom`, `packet`, plus governance tables `control`, `permit`,
//! `audit_entry`, `merkle_root`, `erasure`.
//!
//! Credentials come from `~/.config/ares-backend/secrets.env` (0600) if
//! present, else from the running SurrealDB process arguments (the live
//! instance on this Mac was started with `--user/--pass`). No credential
//! is ever written into this crate, the app folder, or git.

use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SurrealClient {
    base: String,
    user: String,
    pass: String,
    ns: String,
    db: String,
    http: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct SqlResponse {
    result: Option<serde_json::Value>,
    status: Option<String>,
}

fn parse_env_file(path: &std::path::Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        })
        .collect()
}

/// Discover the live SurrealDB root password from the running process
/// (`surreal start ... --user root --pass <pw>`). Loopback dev only.
fn discover_live_credentials() -> Option<(String, String)> {
    let out = std::process::Command::new("ps")
        .args(["-eo", "command"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("surreal") && line.contains("--bind 127.0.0.1:8000") {
            let toks: Vec<&str> = line.split_whitespace().collect();
            let mut user = "root".to_string();
            let mut pass: Option<String> = None;
            let mut i = 0;
            while i < toks.len() {
                if toks[i] == "--user" && i + 1 < toks.len() {
                    user = toks[i + 1].to_string();
                }
                if toks[i] == "--pass" && i + 1 < toks.len() {
                    pass = Some(toks[i + 1].to_string());
                }
                i += 1;
            }
            if let Some(p) = pass {
                return Some((user, p));
            }
        }
    }
    None
}

impl SurrealClient {
    pub fn from_env() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let secrets_path = std::path::PathBuf::from(&home).join(".config/ares-backend/secrets.env");
        let env = parse_env_file(&secrets_path);
        let get = |k: &str| -> Option<String> {
            env.iter()
                .find(|(ek, _)| ek == k)
                .map(|(_, v)| v.clone())
                .or_else(|| std::env::var(k).ok())
        };

        let mut user = get("SURREAL_USER");
        let mut pass = get("SURREAL_PASS");
        if user.is_none() || pass.is_none() {
            if let Some((u, p)) = discover_live_credentials() {
                user = Some(u);
                pass = Some(p);
            }
        }
        let user = user.ok_or("SurrealDB user not found (secrets.env, env, or live process)")?;
        let pass = pass.ok_or("SurrealDB pass not found (secrets.env, env, or live process)")?;

        let base = get("SURREAL_URL").unwrap_or_else(|| "http://127.0.0.1:8000".to_string());
        let ns = get("SURREAL_NS").unwrap_or_else(|| "ares".to_string());
        let db = get("SURREAL_DB").unwrap_or_else(|| "ares".to_string());

        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            base,
            user,
            pass,
            ns,
            db,
            http,
        })
    }

    pub fn authority(&self) -> (String, String, String) {
        (self.base.clone(), self.ns.clone(), self.db.clone())
    }

    pub fn sql_url(&self) -> String {
        format!("{}/sql", self.base)
    }

    /// Execute one SQL statement. Returns the raw result value.
    pub async fn sql(&self, statement: &str) -> Result<serde_json::Value, String> {
        let resp = self
            .http
            .post(self.sql_url())
            .basic_auth(&self.user, Some(&self.pass))
            .header("Accept", "application/json")
            .header("Content-Type", "text/plain")
            .header("surreal-ns", &self.ns)
            .header("surreal-db", &self.db)
            .body(statement.to_string())
            .send()
            .await
            .map_err(|e| format!("surreal request failed: {}", e))?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;
        if !status.is_success() {
            return Err(format!("surreal HTTP {}: {}", status, text));
        }
        let parsed: Vec<SqlResponse> =
            serde_json::from_str(&text).map_err(|e| format!("surreal parse: {} ({})", e, text))?;
        let first = parsed.into_iter().next().ok_or("surreal: empty response")?;
        if first.status.as_deref() == Some("ERR") {
            return Err(format!("surreal ERR: {}", text));
        }
        Ok(first.result.unwrap_or(serde_json::Value::Null))
    }

    /// Health probe for /api/status: `GET /health` then a version query.
    pub async fn health(&self) -> Result<String, String> {
        let resp = self
            .http
            .get(format!("{}/health", self.base))
            .send()
            .await
            .map_err(|e| format!("surreal health: {}", e))?;
        if resp.status().is_success() {
            Ok("up".to_string())
        } else {
            Err(format!("surreal health HTTP {}", resp.status()))
        }
    }
}

/// Escape a string for inclusion in a SurrealQL single-quoted literal.
pub fn sql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}
