//! ui-ux-flow gate \u2014 compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "placeholder copy presented as final spec text".to_string(), rule: "contract".to_string() },
        BanRule { id: "B2".to_string(), pattern: "flow missing happy, error, empty, or loading state".to_string(), rule: "RULES 50".to_string() },
        BanRule { id: "B3".to_string(), pattern: "mockup motion not achievable with registered tokens".to_string(), rule: "contract".to_string() },
        BanRule { id: "B4".to_string(), pattern: "component spec without a11y notes block".to_string(), rule: "RULES 11-24".to_string() },
        BanRule { id: "B5".to_string(), pattern: "non-native control pattern in wireframes or prototypes".to_string(), rule: "RULES 11, 12".to_string() },
        BanRule { id: "B6".to_string(), pattern: "table for layout, tabular data without table plus th scope".to_string(), rule: "RULES 15".to_string() },
        BanRule { id: "B7".to_string(), pattern: "skipped heading level or div styled as heading".to_string(), rule: "RULES 13, 14".to_string() },
        BanRule { id: "B8".to_string(), pattern: "interactive target below 24x24 CSS px".to_string(), rule: "RULES 22".to_string() },
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
    GateResult { pass, skill: "ui-ux-flow".to_string() }
}
