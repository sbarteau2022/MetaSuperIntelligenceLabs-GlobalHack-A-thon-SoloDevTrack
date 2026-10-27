# C20 — Configuration, Settings & Feature Flags

**Status.** Scaffolded; detail pending. **Parent:** Master Design Document index.

## Purpose

Data-driven configuration, per-agent settings, and feature flags.

## Subcategories (broad topics)

- C20.1 AgentConfig schema (N lanes, rag_collection, tools allow-list, approval policy)
- C20.2 Global settings (RAG, hooks, compaction budget, embedding endpoint)
- C20.3 Feature flags (kill switch, staged rollout per phase, optional components)
- C20.4 Environment/paths (ATOMA_WORKSPACE_ROOT, data root discovery)
- C20.5 Migration of configs (forward-only, serde defaults)
- C20.6 Secrets slots and endpoint enablement defaults

## Open questions / gaps

- Settings currently hand-wired; data-driven roster not yet implemented.

## Contract catalog (frozen for design)

| Key                         | Type            | Default         | Notes                         |
| --------------------------- | --------------- | --------------- | ----------------------------- |
| `agent_roster`              | array           | seed roster     | Data-driven lanes (F1)        |
| `rag_collection`            | string          | collection name | Per-lane configurable         |
| `embed_endpoint`            | optional string | none            | Honest BM25 if absent (NF6)   |
| `compaction_fold_threshold` | number          | 0.60            | 60% fold                      |
| `context_budget`            | enum            | 200-line        | 8k / 131k variants (A6)       |
| `atoma_workspace_root`      | optional string | discovered      | Overrides data root           |
| `acft_enabled`              | bool            | false           | Kill switch available (C10.4) |
| `hooks_manifest_mode`       | enum            | compiled        | compiled vs legacy json       |

### Migration rule

- Forward-only; serde defaults for new fields (as done for `rag_collection`).

## Typed Configuration as Effect System

### Configuration as Compile-Time Verified Schema

```rust
// config.rs — single source of truth, generates JSON Schema
#[derive(Serialize, Deserialize, JsonSchema, Validate)]
#[config(version = 3, migration = "config::migrate_v3")]
pub struct Config {
    #[serde(default = "default_agent_roster")]
    #[validate(length(min = 1))]
    pub agent_roster: Vec<AgentConfig>,

    #[serde(default = "default_rag_collection")]
    pub rag_collection: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embed_endpoint: Option<String>,

    #[serde(default = "default_compaction_fold")]
    #[validate(range(min = 0.1, max = 0.9))]
    pub compaction_fold_threshold: f32,

    #[serde(default = "default_context_budget")]
    pub context_budget: ContextBudget,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atoma_workspace_root: Option<PathBuf>,

    #[serde(default)]
    pub acft_enabled: bool,

    #[serde(default = "default_hooks_mode")]
    pub hooks_manifest_mode: HooksMode,

    #[serde(default)]
    pub feature_flags: FeatureFlags,
}

#[derive(Serialize, Deserialize, JsonSchema, Validate)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "lane_type")]
    pub lane_type: LaneType,
    pub rag_collection: String,
    pub tools: Vec<String>,
    pub approval_policy: ApprovalPolicy,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum LaneType {
    Builder,
    Researcher,
    Coder,
    Reviewer,
    Custom(String),
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    None,
    Hole,
    Human,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ContextBudget {
    #[serde(rename = "200-line")]
    Lines200,
    #[serde(rename = "8k")]
    K8,
    #[serde(rename = "131k")]
    K131,
}
```

### Feature Flags as Typed Effects

```rust
// Feature flags with rollout state
#[derive(Serialize, Deserialize, JsonSchema, Default)]
pub struct FeatureFlags {
    // Phase-gated rollout
    #[serde(default)]
    pub acft_rollout: RolloutState,

    #[serde(default)]
    pub hyperbolic_cache_rollout: RolloutState,

    #[serde(default)]
    pub speculative_decode_rollout: RolloutState,

    #[serde(default)]
    pub multi_agent_submanifolds_rollout: RolloutState,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "phase", content = "config")]
pub enum RolloutState {
    Disabled,
    Canary { traffic_percent: u8, duration_hours: u32 },
    Shadow { duration_hours: u32 },
    Enabled,
}

// Compile-time feature gate
#[macro_export]
macro_rules! feature_gate {
    ($flag:expr, $enabled:expr, $disabled:expr) => {
        if cfg!(feature = $flag) { $enabled } else { $disabled }
    };
}
```

### Environment Resolution (Priority Order)

```rust
fn resolve_workspace_root() -> PathBuf {
    // 1. Explicit override (highest priority)
    if let Ok(path) = std::env::var("ATOMA_WORKSPACE_ROOT") {
        return PathBuf::from(path);
    }

    // 2. .atoma_workspace marker file (walk up from cwd)
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join(".atoma_workspace").exists() {
            return dir;
        }
        if !dir.pop() { break; }
    }

    // 3. Cargo workspace root
    if let Ok(output) = std::process::Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
    {
        if let Ok(meta) = serde_json::from_slice::<CargoMetadata>(&output.stdout) {
            return PathBuf::from(meta.workspace_root);
        }
    }

    // 4. Git root
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if let Ok(path) = String::from_utf8(output.stdout) {
            return PathBuf::from(path.trim());
        }
    }

    // 5. Fallback
    dirs::home_dir().unwrap().join(".atoma")
}
```

### Secrets as Typed Slots

```rust
// Secret slot registry — compile-time verified
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SecretSlots {
    pub rapid_mlx_endpoint: SecretSlot,
    pub rapid_mlx_api_key: SecretSlot,
    pub omix_endpoint: SecretSlot,
    pub omix_api_key: SecretSlot,
    pub custom_endpoint: Option<SecretSlot>,
    pub custom_api_key: Option<SecretSlot>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SecretSlot {
    pub keychain_account: String,
    pub keychain_service: String,
    pub required: bool,
    pub description: String,
}

// Resolution at runtime — never in config files
fn resolve_secret(slot: &SecretSlot) -> Result<String> {
    keychain::get(slot.keychain_service, slot.keychain_account)
}
```

### Migration as Pure Functions

```rust
// Migration registry — pure, forward-only
pub type MigrationFn = fn(&mut serde_json::Value) -> Result<()>;

const MIGRATIONS: &[(u32, MigrationFn)] = &[
    (1, migrate_v1),  // Initial
    (2, migrate_v2),  // Add rag_collection
    (3, migrate_v3),  // Add feature_flags, acft_enabled
];

pub fn migrate_config(mut config: serde_json::Value) -> Result<serde_json::Value> {
    let current = config.get("schema_version").and_then(|v| v.as_u64()).unwrap_or(0);

    for (version, migrate) in MIGRATIONS {
        if version > current {
            migrate(&mut config)?;
            config["schema_version"] = serde_json::json!(version);
        }
    }

    Ok(config)
}
```

### Config Validation at Startup

```rust
fn validate_config(config: &Config) -> Result<()> {
    // 1. Schema validation (serde + validator)
    config.validate()?;

    // 2. Cross-field validation
    for agent in &config.agent_roster {
        // Tools must exist in registry
        for tool in &agent.tools {
            if !TOOL_REGISTRY.contains(tool) {
                return Err(ConfigError::UnknownTool(tool.clone()));
            }
        }
        // RAG collection must exist
        if !RAG_COLLECTIONS.contains(&agent.rag_collection) {
            return Err(ConfigError::UnknownRagCollection(agent.rag_collection.clone()));
        }
    }

    // 3. Feature flag consistency
    if config.acft_enabled && !config.feature_flags.acft_rollout.is_enabled() {
        return Err(ConfigError::FeatureFlagMismatch("acft_enabled but rollout disabled"));
    }

    Ok(())
}
```

---

## Contributor Deepening — Muse (2026-10-26)

### C20.7 Config-change blast radius (depth: which changes invalidate what)

Not all config changes are equal. Frozen blast-radius classes [PROPOSED — classes are my proposal; C09.8/C16.10 triggers are the authority]:

| Class      | Examples                                                                         | Effect on evidence                | Required action                                                              |
| ---------- | -------------------------------------------------------------------------------- | --------------------------------- | ---------------------------------------------------------------------------- |
| COSMETIC   | panel defaults, timeouts, log levels                                             | None                              | None (no re-validation)                                                      |
| BEHAVIORAL | compaction_fold_threshold, context_budget tier, D_max                            | Affects A2/A6/A1 judges           | Re-run affected ablations (C09.8 trigger 5 analogue: judge-affecting change) |
| STRUCTURAL | lane add/remove, allowlist change, approval-policy change, embed_endpoint change | Invalidates A5/A7/A10/A29/A30, A3 | Full C09.8 trigger list; threat-model rows re-open (C16.10)                  |
| FLAG       | acft_enabled, rollout state transitions                                          | Switches the measured path itself | Kill-switch drill (C10.8) + A1 re-run on the newly active path               |

Every config write emits a receipt with its blast class. STRUCTURAL and FLAG writes additionally require: (a) the C09.8 re-validation list computed and attached to the receipt, (b) human approval if the approval policy for ConfigureSystem (C16.2 Admin) demands it. A STRUCTURAL change applied without the attached re-validation list is a Gate 1 failure (unregistered evidence invalidation).

### C20.8 acft_enabled vs rollout-state precedence (depth: two flags, one switch)

C20 defines BOTH a flat `acft_enabled: bool` (default false) AND `feature_flags.acft_rollout: RolloutState` (Disabled/Canary/Shadow/Enabled), and `validate_config` errors when they disagree. That validation is correct but the precedence is unstated: which flag does the serving path actually read? Frozen precedence [PROPOSED — precedence is my proposal]:

1. The serving path reads ONLY `feature_flags.acft_rollout`. `acft_enabled` is a deprecated alias kept for migration (config_version 3 history): at load, `acft_enabled=true` maps to `rollout=Enabled`, `false` maps to `rollout=Disabled`, and a deprecation receipt is emitted.
2. The `validate_config` mismatch error is replaced by this deterministic mapping + receipt (errors on config load block startup; mapping preserves it).
3. Kill-switch drill (C10.8) flips `rollout` to Disabled, not the alias — the drill tests the flag the server reads.

Recommend a C36 correction entry recording the alias deprecation; original fields preserved per C03.3.

What was NOT checked here: current configurations.json contents (not in workspace); whether any code reads acft_enabled directly (implementation outside workspace).

## Open questions / gaps

- Settings currently hand-wired; data-driven roster not yet implemented.
