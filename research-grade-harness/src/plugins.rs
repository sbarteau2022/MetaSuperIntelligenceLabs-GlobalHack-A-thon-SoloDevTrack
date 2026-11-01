//! Item 5a: reversible, hot-swappable plugins, Cordis-style.
//!
//! Unload fully undoes a plugin (inverse held by the runtime, not the
//! plugin). Dependencies declared up front; a model, sandbox, or loop can
//! be swapped while a session continues. Swap = deactivate + activate with
//! receipts at every step; a failed swap rolls back to the prior active set.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub name: String,
    pub version: String,
    /// Declared dependencies (other plugin names). Checked at load.
    pub depends_on: Vec<String>,
    /// What this plugin provides (capability names for dependents).
    pub provides: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReceipt {
    pub plugin: String,
    pub op: String, // "load" | "unload" | "swap_in" | "swap_out" | "rollback"
    pub inverse_held: bool,
    pub session_continued: bool,
}

/// Hot-swappable registry. The runtime (not the plugin) holds every inverse.
pub struct PluginRegistry {
    pub active: HashMap<String, PluginSpec>,
    pub log: Vec<PluginReceipt>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            active: HashMap::new(),
            log: vec![],
        }
    }

    fn deps_satisfied(&self, spec: &PluginSpec) -> bool {
        spec.depends_on.iter().all(|d| {
            self.active
                .values()
                .any(|a| a.provides.iter().any(|p| p == d) || &a.name == d)
        })
    }

    /// Load: check deps, activate, runtime holds the inverse (unload).
    pub fn load(&mut self, spec: PluginSpec) -> Result<PluginReceipt, String> {
        if self.active.contains_key(&spec.name) {
            return Err(format!("already loaded: {}", spec.name));
        }
        if !self.deps_satisfied(&spec) {
            return Err(format!(
                "unsatisfied deps for {}: {:?}",
                spec.name, spec.depends_on
            ));
        }
        self.active.insert(spec.name.clone(), spec.clone());
        let r = PluginReceipt {
            plugin: spec.name,
            op: "load".into(),
            inverse_held: true,
            session_continued: true,
        };
        self.log.push(r.clone());
        Ok(r)
    }

    fn provided_elsewhere(&self, cap: &str, except: &str) -> bool {
        self.active
            .values()
            .any(|a| a.name != except && (a.provides.iter().any(|p| p == cap) || a.name == cap))
    }

    /// Unload: fully undoes the plugin. Blocked when any active dependent
    /// needs a capability only this plugin provides (capability match, not
    /// name match — dependents declare what they need, not who provides it).
    pub fn unload(&mut self, name: &str) -> Result<PluginReceipt, String> {
        let provided: Vec<String> = self
            .active
            .get(name)
            .map(|s| s.provides.clone())
            .unwrap_or_default();
        let dependents: Vec<String> = self
            .active
            .values()
            .filter(|a| a.name != name)
            .filter(|a| {
                a.depends_on.iter().any(|d| {
                    d == name || (provided.contains(d) && !self.provided_elsewhere(d, name))
                })
            })
            .map(|a| a.name.clone())
            .collect();
        if !dependents.is_empty() {
            return Err(format!(
                "cannot unload {}: dependents {:?}",
                name, dependents
            ));
        }
        match self.active.remove(name) {
            Some(_) => {
                let r = PluginReceipt {
                    plugin: name.into(),
                    op: "unload".into(),
                    inverse_held: true,
                    session_continued: true,
                };
                self.log.push(r.clone());
                Ok(r)
            }
            None => Err(format!("not loaded: {}", name)),
        }
    }

    /// Swap while a session continues: deactivate old, activate new, with
    /// rollback to the old on any failure. The session never stops.
    pub fn swap(&mut self, old: &str, new_spec: PluginSpec) -> Result<Vec<PluginReceipt>, String> {
        let old_spec = self.active.get(old).cloned();
        // Dependents of old must be satisfiable by new or also swapped.
        let blocked: Vec<String> = self
            .active
            .values()
            .filter(|a| a.name != old && a.depends_on.iter().any(|d| d == old))
            .filter(|a| {
                !new_spec.provides.iter().any(|p| a.depends_on.contains(p))
                    && !a.depends_on.contains(&new_spec.name)
            })
            .map(|a| a.name.clone())
            .collect();
        if !blocked.is_empty() {
            return Err(format!("swap blocked by dependents: {:?}", blocked));
        }
        let mut receipts = vec![];
        if old_spec.is_some() {
            // Temporarily bypass dependent check: swap guarantees continuity.
            self.active.remove(old);
            receipts.push(PluginReceipt {
                plugin: old.into(),
                op: "swap_out".into(),
                inverse_held: true,
                session_continued: true,
            });
        }
        match self.load(new_spec.clone()) {
            Ok(r) => {
                receipts.push(PluginReceipt {
                    op: "swap_in".into(),
                    ..r
                });
                self.log.extend(receipts.clone());
                Ok(receipts)
            }
            Err(e) => {
                // Rollback: restore old.
                if let Some(o) = old_spec {
                    self.active.insert(o.name.clone(), o);
                }
                let rb = PluginReceipt {
                    plugin: new_spec.name,
                    op: "rollback".into(),
                    inverse_held: true,
                    session_continued: true,
                };
                self.log.push(rb);
                Err(format!("swap failed, rolled back: {}", e))
            }
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
