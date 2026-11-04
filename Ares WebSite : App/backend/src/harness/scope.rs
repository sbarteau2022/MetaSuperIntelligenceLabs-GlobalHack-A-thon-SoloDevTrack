//! Server-held carved read capabilities for the one GlobalMind MCP child.
//! Workers receive an opaque handle; the governor supplies the well set.

use crate::roles::RoleId;
use std::collections::{BTreeSet, HashMap};

/// Replace any caller-selected wells with the governor's stored scope.
pub fn bind_scoped_recall_args(args_json: &str, wells: &[String]) -> Result<String, String> {
    let mut args: serde_json::Value =
        serde_json::from_str(args_json).map_err(|e| format!("recall args parse: {e}"))?;
    if !args.is_object() {
        return Err("recall arguments must be an object".into());
    }
    args["well_ids"] = serde_json::json!(wells);
    Ok(args.to_string())
}

const MAX_SCOPE_MS: u64 = 10 * 60 * 1000;
const MAX_ACTIVE_SCOPES: usize = 4096;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CarvedReadScope {
    pub role: RoleId,
    pub map_identity: String,
    pub prefix_identity: String,
    pub well_ids: Vec<String>,
    pub expires_ms: u64,
}

#[derive(Default)]
pub struct ScopeRegistry {
    entries: HashMap<String, CarvedReadScope>,
}

impl ScopeRegistry {
    pub fn validate(&self, scope: &CarvedReadScope, now_ms: u64) -> Result<(), String> {
        if scope.role == RoleId::Coordinator {
            return Err("scope must bind a carved worker role".into());
        }
        if scope.map_identity.is_empty() || scope.prefix_identity.is_empty() {
            return Err("scope identities must be nonempty".into());
        }
        if scope.expires_ms <= now_ms || scope.expires_ms - now_ms > MAX_SCOPE_MS {
            return Err("scope lifetime must be 1..=600000 ms".into());
        }
        if scope.well_ids.is_empty() || scope.well_ids.len() > 12 {
            return Err("scope must contain 1..=12 wells".into());
        }
        let mut seen = BTreeSet::new();
        for id in &scope.well_ids {
            if id.is_empty()
                || id.len() > 128
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
                || !seen.insert(id)
            {
                return Err("invalid or duplicate cognitive well ID".into());
            }
        }
        Ok(())
    }

    pub fn issue(&mut self, scope: CarvedReadScope, now_ms: u64) -> Result<String, String> {
        self.validate(&scope, now_ms)?;
        self.entries.retain(|_, entry| entry.expires_ms > now_ms);
        if self.entries.len() >= MAX_ACTIVE_SCOPES {
            return Err("active carved scope capacity reached".into());
        }
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|e| format!("scope entropy unavailable: {e}"))?;
        let token = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        if self.entries.contains_key(&token) {
            return Err("scope handle collision".into());
        }
        self.entries.insert(token.clone(), scope);
        Ok(token)
    }

    pub fn resolve(
        &mut self,
        token: &str,
        role: RoleId,
        map_identity: &str,
        prefix_identity: &str,
        now_ms: u64,
    ) -> Result<CarvedReadScope, String> {
        let scope = self
            .entries
            .get(token)
            .ok_or("unknown carved scope handle")?;
        if scope.expires_ms <= now_ms {
            self.entries.remove(token);
            return Err("carved scope handle expired".into());
        }
        if scope.role != role
            || scope.map_identity != map_identity
            || scope.prefix_identity != prefix_identity
        {
            return Err("carved scope handle identity mismatch".into());
        }
        Ok(scope.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_fails_closed_on_wrong_role_target_prefix_and_expiry() {
        let mut registry = ScopeRegistry::default();
        let scope = CarvedReadScope {
            role: RoleId::LaneBuilder,
            map_identity: "mind|ns|db".into(),
            prefix_identity: "model-prefix".into(),
            well_ids: vec!["well0".into()],
            expires_ms: 500,
        };
        let token = registry.issue(scope.clone(), 100).unwrap();
        assert_eq!(token.len(), 64);
        assert!(registry
            .resolve(
                &token,
                RoleId::LaneBuilder,
                "mind|ns|db",
                "model-prefix",
                101
            )
            .is_ok());
        assert!(registry
            .resolve(
                &token,
                RoleId::LaneVerifier,
                "mind|ns|db",
                "model-prefix",
                101
            )
            .is_err());
        assert!(registry
            .resolve(&token, RoleId::LaneBuilder, "wrong", "model-prefix", 101)
            .is_err());
        assert!(registry
            .resolve(&token, RoleId::LaneBuilder, "mind|ns|db", "wrong", 101)
            .is_err());
        assert!(registry
            .resolve(
                &token,
                RoleId::LaneBuilder,
                "mind|ns|db",
                "model-prefix",
                500
            )
            .is_err());
        assert!(registry
            .resolve(
                &token,
                RoleId::LaneBuilder,
                "mind|ns|db",
                "model-prefix",
                101
            )
            .is_err());
        let mut duplicate = scope;
        duplicate.well_ids.push("well0".into());
        assert!(registry.issue(duplicate, 100).is_err());
    }

    #[test]
    fn caller_cannot_substitute_scope_wells() {
        let args = bind_scoped_recall_args(
            r#"{"workspace":"globalmind","well_ids":["forged"]}"#,
            &["well0".into()],
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&args).unwrap();
        assert_eq!(value["well_ids"], serde_json::json!(["well0"]));
        assert!(bind_scoped_recall_args("[]", &["well0".into()]).is_err());
    }
}
