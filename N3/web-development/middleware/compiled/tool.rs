//! middleware gate \u2014 compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "auth duplicated inline in route handlers instead of one guard".to_string(), rule: "RULES 39, 43".to_string() },
        BanRule { id: "B2".to_string(), pattern: "password, token, key, or card number in log line".to_string(), rule: "RULES 44".to_string() },
        BanRule { id: "B3".to_string(), pattern: "body parser without explicit size cap".to_string(), rule: "RULES 36".to_string() },
        BanRule { id: "B4".to_string(), pattern: "outbound call or handler without deadline".to_string(), rule: "contract".to_string() },
        BanRule { id: "B5".to_string(), pattern: "err.stack or driver message forwarded to client".to_string(), rule: "RULES 38".to_string() },
        BanRule { id: "B6".to_string(), pattern: "per-route security headers instead of one middleware".to_string(), rule: "contract".to_string() },
        BanRule { id: "B7".to_string(), pattern: "wildcard origin combined with credentialed requests".to_string(), rule: "contract".to_string() },
        BanRule { id: "B8".to_string(), pattern: "silent rate-limit bypass by IP trust".to_string(), rule: "contract".to_string() },
        BanRule { id: "B9".to_string(), pattern: "middleware importing domain services".to_string(), rule: "RULES 43".to_string() },
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
    GateResult { pass, skill: "middleware".to_string() }
}
