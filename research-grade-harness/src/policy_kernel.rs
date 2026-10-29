#[derive(Clone, Debug)]
pub struct FrozenPolicyKernel {
    pub judges_frozen: bool,
    pub safety_frozen: bool,
    pub egress_frozen: bool,
    pub receipts_frozen: bool,
}

impl Default for FrozenPolicyKernel {
    fn default() -> Self {
        Self {
            judges_frozen: true,
            safety_frozen: true,
            egress_frozen: true,
            receipts_frozen: true,
        }
    }
}

impl FrozenPolicyKernel {
    pub fn check_mutation(&self, target: &str) -> Result<(), String> {
        if target == "judge" && self.judges_frozen {
            return Err("FPK: judges immutable".into());
        }
        if target == "safety" && self.safety_frozen {
            return Err("FPK: safety immutable".into());
        }
        if target == "egress" && self.egress_frozen {
            return Err("FPK: egress immutable".into());
        }
        if target == "receipt" && self.receipts_frozen {
            return Err("FPK: receipts immutable".into());
        }
        Ok(())
    }
}
