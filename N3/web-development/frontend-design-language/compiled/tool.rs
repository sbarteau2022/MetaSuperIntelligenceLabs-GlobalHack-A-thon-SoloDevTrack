//! frontend-design-language gate \u2014 compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "tier-1 literal value in component code".to_string(), rule: "RULES 26".to_string() },
        BanRule { id: "B2".to_string(), pattern: "hardcoded color, spacing, radius, or duration in component styles".to_string(), rule: "RULES 27".to_string() },
        BanRule { id: "B3".to_string(), pattern: "new literal added to component without token registration first".to_string(), rule: "RULES 7".to_string() },
        BanRule { id: "B4".to_string(), pattern: "!important outside utilities cascade layer".to_string(), rule: "RULES 24".to_string() },
        BanRule { id: "B5".to_string(), pattern: "global reset selector".to_string(), rule: "RULES 32".to_string() },
        BanRule { id: "B6".to_string(), pattern: "BEM naming to manage specificity".to_string(), rule: "RULES 31".to_string() },
        BanRule { id: "B7".to_string(), pattern: "redundant ARIA forwarded from spec".to_string(), rule: "RULES 11, 12".to_string() },
        BanRule { id: "B8".to_string(), pattern: "physical property where a logical property exists".to_string(), rule: "RULES 29".to_string() },
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
    GateResult { pass, skill: "frontend-design-language".to_string() }
}
