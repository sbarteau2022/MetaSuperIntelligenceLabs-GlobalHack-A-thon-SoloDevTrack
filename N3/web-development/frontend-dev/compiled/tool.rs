//! frontend-dev gate \u2014 compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "div with role button".to_string(), rule: "RULES 11, 12".to_string() },
        BanRule { id: "B2".to_string(), pattern: "redundant role on native button".to_string(), rule: "RULES 12".to_string() },
        BanRule { id: "B3".to_string(), pattern: "redundant role list on ul".to_string(), rule: "RULES 12".to_string() },
        BanRule { id: "B4".to_string(), pattern: "fake heading: non-h element styled as heading".to_string(), rule: "RULES 13, 14".to_string() },
        BanRule { id: "B5".to_string(), pattern: "table used for layout".to_string(), rule: "RULES 15".to_string() },
        BanRule { id: "B6".to_string(), pattern: "aria-hidden true on focusable element or ancestor".to_string(), rule: "RULES 18".to_string() },
        BanRule { id: "B7".to_string(), pattern: "!important outside utilities cascade layer".to_string(), rule: "RULES 24".to_string() },
        BanRule { id: "B8".to_string(), pattern: "global reset selector".to_string(), rule: "RULES 32".to_string() },
        BanRule { id: "B9".to_string(), pattern: "literal tier custom property referenced in component code".to_string(), rule: "RULES 25, 26".to_string() },
        BanRule { id: "B10".to_string(), pattern: "hardcoded color, spacing, radius, or duration in component styles".to_string(), rule: "RULES 27".to_string() },
        BanRule { id: "B11".to_string(), pattern: "BEM naming to manage specificity".to_string(), rule: "RULES 31".to_string() },
        BanRule { id: "B12".to_string(), pattern: "nontrivial inline style".to_string(), rule: "RULES 20".to_string() },
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
    GateResult { pass, skill: "frontend-dev".to_string() }
}
