use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub name: String,
    pub version: String,
    pub reversible: bool,
}

#[derive(Clone, Debug)]
pub struct ReversibleEffect {
    pub component: String,
    pub forward: String, // description of effect
    pub inverse: String, // inverse held by runtime
    pub applied: bool,
}

impl ReversibleEffect {
    pub fn apply(&mut self) {
        self.applied = true;
    }
    pub fn revert(&mut self) {
        self.applied = false;
    } // runtime holds inverse
}

#[derive(Clone, Debug)]
pub struct Coeffect {
    pub key: String,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct Context {
    pub kv_tokens: usize,
    pub plan_nodes: usize,
    pub bundle_version: String,
    pub erased_bits_budget: u64,
    pub components: Vec<ComponentSpec>,
}

#[derive(Clone, Debug)]
pub struct Component {
    pub spec: ComponentSpec,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct HmrGate {
    pub signature_ok: bool,
    pub shadow_receipts_ok: bool,
    pub judge_suite_ok: bool,
    pub energy_ok: bool,
    pub rollback_retained: bool,
}

impl HmrGate {
    pub fn allows(&self) -> bool {
        self.signature_ok
            && self.shadow_receipts_ok
            && self.judge_suite_ok
            && self.energy_ok
            && self.rollback_retained
    }
}

impl Component {
    pub fn activate(&mut self) -> ReversibleEffect {
        self.active = true;
        ReversibleEffect {
            component: self.spec.name.clone(),
            forward: "activate".into(),
            inverse: "deactivate".into(),
            applied: true,
        }
    }
    pub fn deactivate(&mut self) -> ReversibleEffect {
        self.active = false;
        ReversibleEffect {
            component: self.spec.name.clone(),
            forward: "deactivate".into(),
            inverse: "activate".into(),
            applied: true,
        }
    }
}
