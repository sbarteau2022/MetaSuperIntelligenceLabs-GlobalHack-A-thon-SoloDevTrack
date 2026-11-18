//! backend-systems gate — compiled Rust port of tool.ts ban-list/contract.
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
        BanRule { id: "B1".to_string(), pattern: "fat controller: model access plus domain rules plus shaping".to_string(), rule: "RULES 36-39".to_string() },
        BanRule { id: "B2".to_string(), pattern: "conditionals, calculations, or model access in route files".to_string(), rule: "contract".to_string() },
        BanRule { id: "B3".to_string(), pattern: "raw SQL string concatenation with input".to_string(), rule: "RULES 36, 38".to_string() },
        BanRule { id: "B4".to_string(), pattern: "req.body/query/params read without edge validator".to_string(), rule: "RULES 36".to_string() },
        BanRule { id: "B5".to_string(), pattern: "err.stack or driver message in production response".to_string(), rule: "RULES 38".to_string() },
        BanRule { id: "B6".to_string(), pattern: "server-side authorization missing (UI hiding only)".to_string(), rule: "RULES 39".to_string() },
        BanRule { id: "B7".to_string(), pattern: "mutating endpoint without Idempotency-Key or replay guard".to_string(), rule: "RULES 40".to_string() },
        BanRule { id: "B8".to_string(), pattern: "growable list endpoint without pagination".to_string(), rule: "RULES 41".to_string() },
        BanRule { id: "B9".to_string(), pattern: "endpoint outside /v1 with no breaking-change record".to_string(), rule: "RULES 42".to_string() },
        BanRule { id: "B10".to_string(), pattern: "secret or connection string in committed files, .env in VCS".to_string(), rule: "RULES 9, 53".to_string() },
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
    GateResult { pass, skill: "backend-systems".to_string() }
}
