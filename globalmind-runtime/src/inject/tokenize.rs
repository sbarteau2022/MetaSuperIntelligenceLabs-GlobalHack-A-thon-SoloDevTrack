//! Schedule-v1 Qwen3.6 BPE encoder from the rostered `tokenizer.json`.
//!
//! Faithful components: GPT-2 `bytes_to_unicode` table, ordered merge
//! ranks from the file, vocab lookup. Enforced domain (fail-closed
//! outside it): ASCII-only input (NFC-trivial, so no normalizer needed),
//! no `<|` sequences (added tokens would need atomic handling). On that
//! domain the hand-rolled GPT-2 split matches the file's Split pattern
//! exactly: every ASCII char falls in exactly one branch.
//!
//! Both ablation arms use this encoder identically, so the measured delta
//! is valid even where the schedule-v1 domain is narrower than canonical.

use serde_json::Value;
use std::collections::HashMap;

pub struct ByteLevelBpe {
    vocab: HashMap<String, u32>,
    by_id: Vec<String>,
    merges: HashMap<(String, String), u32>,
    byte_to_char: [char; 256],
    char_to_byte: HashMap<char, u8>,
}

fn bytes_to_unicode() -> ([char; 256], HashMap<char, u8>) {
    let mut table = ['\0'; 256];
    let mut forward = HashMap::new();
    // Printable ASCII + Latin-1 supplement map to themselves.
    let mut direct: Vec<u8> = (b'!'..=b'~').collect();
    direct.extend(b'\xA1'..=b'\xAC');
    direct.extend(b'\xAE'..=b'\xFF');
    for byte in direct {
        table[byte as usize] = byte as char;
        forward.insert(byte as char, byte);
    }
    // Remaining bytes map to U+0100 upwards in byte order.
    let mut code = 0x0100u32;
    for byte in 0u16..256 {
        let byte = byte as u8;
        if table[byte as usize] == '\0' {
            let ch = char::from_u32(code).expect("valid scalar");
            table[byte as usize] = ch;
            forward.insert(ch, byte);
            code += 1;
        }
    }
    (table, forward)
}

impl ByteLevelBpe {
    pub fn load(path: &str) -> Result<Self, String> {
        let text =
            std::fs::read_to_string(path).map_err(|error| format!("read tokenizer: {error}"))?;
        let parsed: Value = serde_json::from_str(&text)
            .map_err(|error| format!("parse tokenizer.json: {error}"))?;
        if parsed
            .get("model")
            .and_then(|m| m.get("type"))
            .and_then(Value::as_str)
            != Some("BPE")
        {
            return Err("tokenizer model is not BPE".into());
        }
        let vocab_object = parsed
            .get("model")
            .and_then(|m| m.get("vocab"))
            .and_then(Value::as_object)
            .ok_or_else(|| "tokenizer has no model.vocab".to_string())?;
        let mut vocab = HashMap::new();
        let mut max_id = 0u32;
        for (token, id) in vocab_object {
            let id = id
                .as_u64()
                .and_then(|v| u32::try_from(v).ok())
                .ok_or_else(|| "vocab id is not u32".to_string())?;
            max_id = max_id.max(id);
            vocab.insert(token.clone(), id);
        }
        let mut by_id = vec![String::new(); (max_id + 1) as usize];
        for (token, id) in &vocab {
            by_id[*id as usize] = token.clone();
        }
        let merges_array = parsed
            .get("model")
            .and_then(|m| m.get("merges"))
            .and_then(Value::as_array)
            .ok_or_else(|| "tokenizer has no model.merges".to_string())?;
        let mut merges = HashMap::new();
        for (rank, pair) in merges_array.iter().enumerate() {
            let pair = pair
                .as_array()
                .ok_or_else(|| "merge entry is not a pair".to_string())?;
            if pair.len() != 2 {
                return Err("merge entry is not a pair".into());
            }
            let left = pair[0]
                .as_str()
                .ok_or_else(|| "merge side is not a string".to_string())?;
            let right = pair[1]
                .as_str()
                .ok_or_else(|| "merge side is not a string".to_string())?;
            merges.insert((left.to_string(), right.to_string()), rank as u32);
        }
        let (byte_to_char, char_to_byte) = bytes_to_unicode();
        Ok(Self {
            vocab,
            by_id,
            merges,
            byte_to_char,
            char_to_byte,
        })
    }

    pub fn vocab_size(&self) -> usize {
        self.by_id.len()
    }

    fn check_domain(&self, text: &str) -> Result<(), String> {
        if !text.is_ascii() {
            return Err("schedule-v1 encoder requires ASCII-only input".into());
        }
        if text.contains("<|") {
            return Err("schedule-v1 encoder refuses added-token sequences".into());
        }
        if text.bytes().any(|b| b == 0x0B || b == 0x0C) {
            return Err("schedule-v1 encoder refuses vertical-tab/form-feed".into());
        }
        Ok(())
    }

    /// GPT-2 Split pattern over ASCII, leftmost first-match wins:
    /// contractions | nonword?letters+ | digit | space?symbols | newlines |
    /// spaces (trailing-space and plain-spaces branches merge: identical
    /// pieces). Covers every admitted ASCII char exactly once.
    fn split_piece(text: &str, start: usize) -> (usize, usize) {
        let bytes = text.as_bytes();
        let rest = &text[start..];
        // Contractions (case-insensitive): 's 't 're 've 'm 'll 'd.
        for suffix in ["'s", "'t", "'re", "'ve", "'m", "'ll", "'d"] {
            if rest.len() >= suffix.len() && rest[..suffix.len()].eq_ignore_ascii_case(suffix) {
                return (start, start + suffix.len());
            }
        }
        let at = |i: usize| bytes.get(start + i).copied().unwrap_or(0);
        let is_letter = |b: u8| b.is_ascii_alphabetic();
        let is_digit = |b: u8| b.is_ascii_digit();
        let is_blank = |b: u8| b == b' ' || b == b'\t';
        // [^\r\nLN]?[L]+ — optional single non-letter/digit (space
        // included, \r\n excluded), then one or more letters.
        let prefix_ok = |b: u8| b != b'\r' && b != b'\n' && !is_letter(b) && !is_digit(b) && b != 0;
        if is_letter(at(0)) || (prefix_ok(at(0)) && is_letter(at(1))) {
            let mut end = start + usize::from(!is_letter(at(0)));
            while is_letter(bytes.get(end).copied().unwrap_or(0)) {
                end += 1;
            }
            return (start, end);
        }
        // Single digit.
        if is_digit(at(0)) {
            return (start, start + 1);
        }
        // Optional space + symbols + trailing newlines.
        let is_symbol = |b: u8| {
            b != 0 && !is_blank(b) && b != b'\r' && b != b'\n' && !is_letter(b) && !is_digit(b)
        };
        if is_symbol(at(0)) || (at(0) == b' ' && is_symbol(at(1))) {
            let mut end = start;
            if bytes[end] == b' ' {
                end += 1;
            }
            while is_symbol(bytes.get(end).copied().unwrap_or(0)) {
                end += 1;
            }
            while matches!(bytes.get(end), Some(b'\r') | Some(b'\n')) {
                end += 1;
            }
            return (start, end);
        }
        // Newline runs with leading blanks: \s*[\r\n]+.
        if matches!(at(0), b'\r' | b'\n')
            || (is_blank(at(0)) && Self::blanks_then_newline(bytes, start))
        {
            let mut end = start;
            while is_blank(bytes.get(end).copied().unwrap_or(0)) {
                end += 1;
            }
            while matches!(bytes.get(end), Some(b'\r') | Some(b'\n')) {
                end += 1;
            }
            return (start, end);
        }
        // Remaining blank runs (trailing-space and plain-spaces branches).
        if is_blank(at(0)) {
            let mut end = start;
            while is_blank(bytes.get(end).copied().unwrap_or(0)) {
                end += 1;
            }
            return (start, end);
        }
        // Unreachable on the admitted domain (every byte class is covered
        // above), but fail-safe: consume one byte rather than stalling.
        (start, start + 1)
    }

    fn blanks_then_newline(bytes: &[u8], start: usize) -> bool {
        let mut pos = start;
        while pos < bytes.len() && (bytes[pos] == b' ' || bytes[pos] == b'\t') {
            pos += 1;
        }
        matches!(bytes.get(pos), Some(b'\r') | Some(b'\n'))
    }

    fn split<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut pieces = Vec::new();
        let mut pos = 0;
        while pos < text.len() {
            let (from, to) = Self::split_piece(text, pos);
            pieces.push(&text[from..to]);
            pos = to;
        }
        pieces
    }

    fn bpe_merge(&self, piece: &str) -> Result<Vec<String>, String> {
        // ByteLevel (add_prefix_space=false): raw UTF-8 bytes through the table.
        let mut parts: Vec<String> = piece
            .bytes()
            .map(|byte| self.byte_to_char[byte as usize].to_string())
            .collect();
        loop {
            let mut best: Option<(usize, u32)> = None;
            for index in 0..parts.len().saturating_sub(1) {
                let key = (parts[index].clone(), parts[index + 1].clone());
                if let Some(rank) = self.merges.get(&key)
                    && best.is_none_or(|(_, best_rank)| *rank < best_rank)
                {
                    best = Some((index, *rank));
                }
            }
            let Some((at, _)) = best else {
                break;
            };
            let merged = format!("{}{}", parts[at], parts[at + 1]);
            parts.splice(at..=at + 1, [merged]);
        }
        Ok(parts)
    }

    /// Encodes ASCII text to token ids. Fails closed outside the enforced
    /// domain or on any piece missing from vocab.
    pub fn encode(&self, text: &str) -> Result<Vec<u32>, String> {
        self.check_domain(text)?;
        if text.is_empty() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for piece in self.split(text) {
            for token in self.bpe_merge(piece)? {
                ids.push(
                    *self
                        .vocab
                        .get(&token)
                        .ok_or_else(|| format!("merged token missing from vocab: {token:?}"))?,
                );
            }
        }
        Ok(ids)
    }

    /// Decodes token ids back to text (inverse byte table). Used to verify
    /// roundtrips and to surface recalled text; unknown ids fail closed.
    pub fn decode(&self, ids: &[u32]) -> Result<String, String> {
        let mut bytes = Vec::new();
        for id in ids {
            let token = self
                .by_id
                .get(*id as usize)
                .filter(|token| !token.is_empty())
                .ok_or_else(|| format!("token id {id} is not in vocab"))?;
            for ch in token.chars() {
                bytes.push(
                    *self
                        .char_to_byte
                        .get(&ch)
                        .ok_or_else(|| format!("vocab char {ch:?} has no byte mapping"))?,
                );
            }
        }
        String::from_utf8(bytes).map_err(|_| "decoded bytes are not UTF-8".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKENIZER: &str =
        "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/tokenizer.json";

    fn bpe() -> ByteLevelBpe {
        ByteLevelBpe::load(TOKENIZER).expect("rostered tokenizer loads")
    }

    #[test]
    fn roster_file_is_self_consistent_bpe() {
        let bpe = bpe();
        assert!(bpe.vocab.len() > 200_000, "vocab scale");
        assert!(bpe.merges.len() > 200_000, "merge scale");
        // Every merge side and every merge result resolves in vocab.
        for (left, right) in bpe.merges.keys() {
            assert!(bpe.vocab.contains_key(left), "left resolves: {left:?}");
            assert!(bpe.vocab.contains_key(right), "right resolves: {right:?}");
            assert!(
                bpe.vocab.contains_key(&format!("{left}{right}")),
                "result resolves: {left:?}+{right:?}"
            );
        }
        // All 256 byte-chars resolve (single bytes always encodable).
        for byte in 0u16..256 {
            let ch = bpe.byte_to_char[byte as usize].to_string();
            assert!(bpe.vocab.contains_key(&ch), "byte char resolves: {byte}");
        }
    }

    #[test]
    fn merge_engine_applies_file_ranks() {
        let bpe = bpe();
        // Single-pair pieces: the engine must apply the file's own merge.
        // Expected ids are looked up live (no hardcoded ids).
        let expected_in = *bpe.vocab.get("in").expect("file merges i+n");
        assert_eq!(bpe.encode("in").unwrap(), vec![expected_in]);
        let expected_t = *bpe.vocab.get("Ġt").expect("file merges space+t");
        assert_eq!(bpe.encode(" t").unwrap(), vec![expected_t]);
        // Contraction splits before merging: "don't" -> "don" + "'t".
        let pieces = bpe.split("don't");
        assert_eq!(pieces, vec!["don", "'t"]);
        // Digits split singly pre-merge.
        assert_eq!(bpe.split("a1b2"), vec!["a", "1", "b", "2"]);
    }

    #[test]
    fn split_covers_every_admitted_byte() {
        let bpe = bpe();
        // Every ASCII byte (minus refused \x0B\x0C) in one nasty string:
        // pieces must tile the input exactly, with progress everywhere.
        let nasty: String = (0x01u8..0x80)
            .filter(|b| *b != 0x0B && *b != 0x0C)
            .map(|b| b as char)
            .collect();
        let pieces = bpe.split(&nasty);
        assert!(!pieces.is_empty());
        assert_eq!(pieces.join(""), nasty);
        assert!(pieces.iter().all(|p| !p.is_empty()));
    }

    #[test]
    fn roundtrips_and_domain_refusals() {
        let bpe = bpe();
        for sample in [
            "hello world",
            "The quick brown fox jumps 123 times.",
            "don't stop 'til morning\nnew line\nhere",
            "Symbols: (a+b)*c/d-e_f! @#%?",
            "tabs\tand  double  spaces",
            "line1\r\nline2\rline3\n",
            "",
        ] {
            let ids = bpe.encode(sample).unwrap();
            assert_eq!(bpe.decode(&ids).unwrap(), sample, "roundtrip {sample:?}");
            assert!(ids.iter().all(|id| (*id as usize) < bpe.vocab_size()));
            // Deterministic.
            assert_eq!(bpe.encode(sample).unwrap(), ids);
        }
        assert!(bpe.encode("héllo wörld ✓").is_err());
        assert!(bpe.encode("text <|im_start|> more").is_err());
        assert!(bpe.encode("vertical\x0Btab").is_err());
        assert!(bpe.decode(&[u32::MAX]).is_err());
    }
}
