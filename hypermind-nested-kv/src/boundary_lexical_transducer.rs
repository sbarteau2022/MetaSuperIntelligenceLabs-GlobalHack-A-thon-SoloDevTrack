//! Perfect Boundary Symplectic Lexical Transducer (native rebuild, staged).
//! No silent drops: checked expansion, strict UTF-8, explicit drop receipts.

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct DropReceipt {
    pub egress_unhandled_ids: u64,
    pub expand_unknown_ids: u64,
    pub invalid_utf8: bool,
}

pub struct BoundaryLexicalTransducer {
    vocab_decode: HashMap<u32, Vec<u8>>,
    base_vocab_size: u32,
    multi_atoms: Vec<(u32, Vec<u32>)>,
    // 2026-10-30: the `silent: AtomicU64` loss counter is DELETED, not
    // exposed. It had no reader (no getter, no health/sink export) and its
    // function is fully covered by the replacements: per-call DropReceipt
    // counts + expand_checked Err naming the carrier ID (both pinned by
    // f1_silent_drop_probe) aggregate-side into policy_events (gate=egress)
    // + telemetry_delivery/dropped_count per the 76-table schema. Exposing
    // it would have created a second, non-receipted loss channel — F-1's
    // ghost. DropReceipt.expand_unknown_ids is reserved (always 0) until a
    // wiring caller translates expand_checked failures into the receipt.
}

impl BoundaryLexicalTransducer {
    pub fn new() -> Self {
        // Seed byte vocab 0-255 (vault parity: every byte transduces,
        // never a tokenizer error). base_vocab_size reserves the carrier
        // band; multi_atoms holds learned span folds (>= base).
        let mut vocab_decode = HashMap::new();
        for b in 0u16..256u16 {
            vocab_decode.insert(b as u32, vec![b as u8]);
        }
        Self {
            vocab_decode,
            base_vocab_size: 256,
            multi_atoms: vec![],
        }
    }

    /// Register a learned span fold (atom_id -> expansion). Mirrors the
    /// vault's AtomicIrAlphabet multi_atoms (e.g. struct.tool_call.open).
    pub fn register_span(&mut self, atom_id: u32, expansion: Vec<u32>) {
        if atom_id >= self.base_vocab_size && !self.multi_atoms.iter().any(|(a, _)| *a == atom_id) {
            self.multi_atoms.push((atom_id, expansion));
        }
    }

    /// INGRESS: UTF-8 bytes -> greedy longest-match carriers -> atomic fold.
    /// Total function: any bytes transduce (raw-byte fallback), never errors.
    pub fn transduce_ingress_text(&self, text: &str, prepend_bos: bool) -> Vec<u32> {
        let bytes = text.as_bytes();
        let mut raw: Vec<u32> = Vec::with_capacity(bytes.len() + 1);
        if prepend_bos {
            raw.push(0x10000); // BOS sentinel above carrier band
        }
        // Greedy longest-match over the decode table (vault parity, max 32).
        let mut encode: HashMap<&[u8], u32> = HashMap::new();
        for (id, piece) in &self.vocab_decode {
            encode.insert(piece.as_slice(), *id);
        }
        let mut i = 0;
        while i < bytes.len() {
            let max_len = (bytes.len() - i).min(32);
            let mut matched: Option<(usize, u32)> = None;
            for len in (1..=max_len).rev() {
                if let Some(&id) = encode.get(&bytes[i..i + len]) {
                    matched = Some((len, id));
                    break;
                }
            }
            match matched {
                Some((len, id)) => {
                    raw.push(id);
                    i += len;
                }
                None => {
                    raw.push(bytes[i] as u32);
                    i += 1;
                }
            }
        }
        // Atomic-IR fold: greedy longest-match over multi_atoms (vault
        // encode() parity: longest wins, ties -> lowest atom id).
        let mut out = Vec::with_capacity(raw.len());
        let mut j = 0;
        while j < raw.len() {
            let mut best: Option<(usize, u32)> = None;
            for (aid, exp) in &self.multi_atoms {
                let n = exp.len();
                if n > 0 && j + n <= raw.len() && raw[j..j + n] == exp[..] {
                    match best {
                        None => best = Some((n, *aid)),
                        Some((bl, bid)) if n > bl || (n == bl && *aid < bid) => {
                            best = Some((n, *aid))
                        }
                        _ => {}
                    }
                }
            }
            match best {
                Some((n, id)) => {
                    out.push(id);
                    j += n;
                }
                None => {
                    out.push(raw[j]);
                    j += 1;
                }
            }
        }
        out
    }

    pub fn expand_checked(&self, atoms: &[u32]) -> Result<Vec<u32>, String> {
        let mut out = Vec::with_capacity(atoms.len());
        for &id in atoms {
            if id < self.base_vocab_size {
                out.push(id);
            } else if let Some((_, exp)) = self.multi_atoms.iter().find(|(a, _)| *a == id) {
                out.extend_from_slice(exp);
            } else {
                return Err(format!("unknown Atomic-IR carrier {id}"));
            }
        }
        Ok(out)
    }

    pub fn transduce_egress_ops(
        &self,
        expanded: &[u32],
    ) -> Result<(Vec<u32>, String), DropReceipt> {
        let mut out_bytes = Vec::with_capacity(expanded.len() * 4);
        let mut receipt = DropReceipt::default();
        for &id in expanded {
            if let Some(piece) = self.vocab_decode.get(&id) {
                out_bytes.extend_from_slice(piece);
            } else if id < 256 {
                out_bytes.push(id as u8);
            } else {
                receipt.egress_unhandled_ids += 1;
                return Err(receipt);
            }
        }
        match String::from_utf8(out_bytes) {
            Ok(t) => Ok((expanded.to_vec(), t)),
            Err(_) => {
                receipt.invalid_utf8 = true;
                Err(receipt)
            }
        }
    }
}

impl Default for BoundaryLexicalTransducer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_checked_rejects_unknown() {
        let mut t = BoundaryLexicalTransducer::new();
        t.base_vocab_size = 1;
        assert!(t.expand_checked(&[0]).is_ok());
        assert!(t.expand_checked(&[999]).is_err());
    }

    #[test]
    fn transduce_roundtrip_bytes() {
        let mut t = BoundaryLexicalTransducer::new();
        t.base_vocab_size = 256;
        t.vocab_decode.insert(65, b"A".to_vec());
        let out = t.transduce_egress_ops(&[65]).expect("ok");
        assert_eq!(out.1, "A");
    }

    #[test]
    fn no_silent_drop() {
        let mut t = BoundaryLexicalTransducer::new();
        t.base_vocab_size = 1;
        let r = t.transduce_egress_ops(&[1000]);
        assert!(r.is_err());
    }
}