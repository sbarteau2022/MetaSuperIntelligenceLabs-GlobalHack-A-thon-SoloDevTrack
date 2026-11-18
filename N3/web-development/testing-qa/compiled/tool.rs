//! testing-qa gate \u2014 compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "suite covering only the happy path".to_string(), rule: "RULES 50".to_string() },
        BanRule { id: "B2".to_string(), pattern: "unit test with network or filesystem access".to_string(), rule: "RULES 52".to_string() },
        BanRule { id: "B3".to_string(), pattern: "pass claim without pasted checker output".to_string(), rule: "RULES 51".to_string() },
        BanRule { id: "B4".to_string(), pattern: "failing test deleted or commented out to force green".to_string(), rule: "RULES 4, 49".to_string() },
        BanRule { id: "B5".to_string(), pattern: "bug fixed without reproduction test first".to_string(), rule: "RULES 49".to_string() },
        BanRule { id: "B6".to_string(), pattern: "unlabeled coverage claim without NOT-checked statement".to_string(), rule: "RULES 58".to_string() },
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
    GateResult { pass, skill: "testing-qa".to_string() }
}
