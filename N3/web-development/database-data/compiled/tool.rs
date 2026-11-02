//! database-data gate — compiled Rust port of tool.ts ban-list/contract.
//! Port map: string->String, boolean->bool, number->f64, const array->LazyLock static.
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct BanRule {
    pub id: String,
    pub pattern: String,
    pub rule: String,
}

#[derive(Debug, Clone)]
pub struct GateInput {
    pub banned_hit_count: f64,
    pub contract_on_disk: bool,
    pub audit_clean: bool,
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub pass: bool,
    pub skill: String,
}

pub static BAN_RULES: LazyLock<Vec<BanRule>> = LazyLock::new(|| {
    vec![
        BanRule { id: "B1".to_string(), pattern: "landed migration edited in place".to_string(), rule: "RULES 45".to_string() },
        BanRule { id: "B2".to_string(), pattern: "breaking schema change without expand/contract dual-compat".to_string(), rule: "RULES 46".to_string() },
        BanRule { id: "B3".to_string(), pattern: "hand-edited production rows instead of reviewed script".to_string(), rule: "RULES 47".to_string() },
        BanRule { id: "B4".to_string(), pattern: "hot-path filter/sort/join without index plus EXPLAIN record".to_string(), rule: "RULES 48".to_string() },
        BanRule { id: "B5".to_string(), pattern: "local-time storage without offset, server-local timezone reliance".to_string(), rule: "contract".to_string() },
        BanRule { id: "B6".to_string(), pattern: "ad-hoc deleted_at filter instead of shared query-layer scope".to_string(), rule: "contract".to_string() },
        BanRule { id: "B7".to_string(), pattern: "secret or credential inside migration or seed script".to_string(), rule: "RULES 44".to_string() },
    ]
});

pub fn evaluate(input: &GateInput) -> GateResult {
    let mut pass: bool = true;
    if input.banned_hit_count > 0.0 {
        pass = false;
    }
    if input.contract_on_disk {
    } else {
        pass = false;
    }
    if input.audit_clean {
    } else {
        pass = false;
    }
    GateResult { pass, skill: "database-data".to_string() }
}
