use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const MASS_SCALE: i64 = 1_000_000;
pub const DEAD_THRESHOLD: i64 = 10_000; // mass below => reclaim-eligible

#[derive(Clone, Debug)]
pub struct Block {
    pub key: String,
    pub parent: Option<String>,
    pub token: String,
    pub tokens: usize,
    pub mass: i64,
    pub anchored: bool,
    pub refcount: u32,
    pub locked: bool,
    pub plan_covered: bool,
    pub leak_rate_ppm: u32, // parts per million per epoch
}

pub struct Cache {
    pub model_id: String,
    pub tokenizer_id: String,
    pub quant_id: String,
    pub capacity_blocks: usize,
    pub anchor_budget: usize,
    pub anchored_count: usize,
    pub blocks: HashMap<String, Block>,
    pub free_energy_cache: f64, // surrogate
}

impl Cache {
    pub fn new(model: &str, tok: &str, q: &str, cap: usize) -> Self {
        Self {
            model_id: model.into(),
            tokenizer_id: tok.into(),
            quant_id: q.into(),
            capacity_blocks: cap,
            anchor_budget: cap / 4,
            anchored_count: 0,
            blocks: HashMap::new(),
            free_energy_cache: 0.0,
        }
    }

    pub fn key(&self, parent: Option<&str>, token: &str) -> String {
        let mut h = Sha256::new();
        if let Some(p) = parent {
            h.update(p.as_bytes());
        }
        h.update(token.as_bytes());
        h.update(self.model_id.as_bytes());
        h.update(self.tokenizer_id.as_bytes());
        h.update(self.quant_id.as_bytes());
        hex::encode(h.finalize())
    }

    pub fn insert(
        &mut self,
        parent: Option<String>,
        token: String,
        tokens: usize,
        plan_covered: bool,
    ) -> String {
        let key = self.key(parent.as_deref(), &token);
        if self.blocks.contains_key(&key) {
            if let Some(b) = self.blocks.get_mut(&key) {
                b.refcount += 1;
            }
            return key;
        }
        // pressure: if full, evict
        while self.blocks.len() >= self.capacity_blocks {
            if !self.evict_one() {
                break;
            }
        }
        let leak = if plan_covered { 0 } else { 50_000 };
        let b = Block {
            key: key.clone(),
            parent,
            token,
            tokens,
            mass: MASS_SCALE,
            anchored: false,
            refcount: 1,
            locked: false,
            plan_covered,
            leak_rate_ppm: leak,
        };
        self.blocks.insert(key.clone(), b);
        key
    }

    pub fn ref_add(&mut self, key: &str) {
        if let Some(b) = self.blocks.get_mut(key) {
            b.refcount += 1;
        }
    }

    pub fn ref_release(&mut self, key: &str) {
        if let Some(b) = self.blocks.get_mut(key) {
            if b.refcount > 0 {
                b.refcount -= 1;
            }
        }
    }

    pub fn anchor(&mut self, key: &str) -> bool {
        if self.anchored_count >= self.anchor_budget {
            return false;
        } // degraded, not error here
        if let Some(b) = self.blocks.get_mut(key) {
            if !b.anchored {
                b.anchored = true;
                b.leak_rate_ppm = 0;
                self.anchored_count += 1;
            }
            return true;
        }
        false
    }

    pub fn release_anchor(&mut self, key: &str) {
        if let Some(b) = self.blocks.get_mut(key) {
            if b.anchored {
                b.anchored = false;
                if !b.plan_covered {
                    b.leak_rate_ppm = 50_000;
                }
                self.anchored_count = self.anchored_count.saturating_sub(1);
            }
        }
    }

    pub fn decay_epoch(&mut self) {
        // fixed-point Q1.31 style using integer ppm
        for b in self.blocks.values_mut() {
            if b.anchored {
                continue;
            }
            let leak = b.leak_rate_ppm as i64;
            // mass <- mass * (1 - leak/1e6)
            let new = b.mass - (b.mass * leak / 1_000_000);
            b.mass = new.max(0);
        }
    }

    pub fn evict_one(&mut self) -> bool {
        // find lowest mass, refcount 0, not anchored/locked, leaf (no resident children)
        let mut candidate: Option<(String, i64)> = None;
        for (k, b) in &self.blocks {
            if b.refcount == 0 && !b.anchored && !b.locked {
                // leaf check: no other block parent == k
                let has_child = self.blocks.values().any(|x| x.parent.as_deref() == Some(k));
                if has_child {
                    continue;
                }
                match candidate {
                    None => candidate = Some((k.clone(), b.mass)),
                    Some((_, m)) if b.mass < m => candidate = Some((k.clone(), b.mass)),
                    _ => {}
                }
            }
        }
        if let Some((k, _)) = candidate {
            self.blocks.remove(&k);
            return true;
        }
        false
    }

    /// exact coverage over declared tokenized context
    pub fn coverage_exact(&self, declared_tokens: &[String]) -> (usize, usize) {
        // declared_tokens are block keys expected; count present
        let total = declared_tokens.len();
        let covered = declared_tokens
            .iter()
            .filter(|t| self.blocks.contains_key(*t))
            .count();
        (covered, total.max(1))
    }

    pub fn resident_mass_total(&self) -> i64 {
        self.blocks.values().map(|b| b.mass).sum()
    }

    pub fn free_energy(&self) -> f64 {
        // surrogate: unanchored+uncovered mass / SCALE
        let mut f = 0.0;
        for b in self.blocks.values() {
            if !b.anchored && !b.plan_covered {
                f += b.mass as f64 / MASS_SCALE as f64;
            }
            if b.anchored {
                f += (b.mass as f64 / MASS_SCALE as f64) * 0.1; // anchor debt weight
            }
        }
        f
    }
}
