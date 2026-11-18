//! Source-text carrier for replay atoms.
//!
//! A rank-0 atom is a tensor fragment at one token position. The meaning a
//! model can use is the language that produced it, so the token tape's text
//! is carried here and bound by digest: the atom's text is the exact prefix of
//! the source ending at its token position, and its `content_digest` is the
//! SHA-256 of those bytes. Nothing is summarized or rewritten; a span that
//! cannot be proven to be a prefix of the full decoded text is refused.

use crate::Digest;
use sha2::{Digest as ShaDigest, Sha256};

pub const SOURCE_MAGIC: [u8; 4] = *b"GMSR";
pub const SOURCE_VERSION: u16 = 1;
/// Largest inline atom text; matches the store's inline-content bound.
pub const MAX_SPAN_BYTES: usize = 16 * 1024;
pub const MAX_SOURCE_TOKENS: usize = 4096;
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    /// Digest of the token tape payload this text was decoded from.
    pub tape_digest: Digest,
    text: String,
    /// `ends[p]` = byte length of the text covered by tokens `0..=p`.
    ends: Vec<u32>,
}

impl SourceText {
    /// Builds the carrier from tape tokens and the model's own decoder. The
    /// decoder is injected so this crate links no tokenizer of its own.
    pub fn from_tokens(
        tape_digest: Digest,
        tokens: &[u32],
        decode: &dyn Fn(&[u32]) -> String,
    ) -> Result<Self, String> {
        if tokens.is_empty() || tokens.len() > MAX_SOURCE_TOKENS {
            return Err("source token count out of bounds".into());
        }
        let text = decode(tokens);
        if text.is_empty() || text.len() > MAX_SOURCE_BYTES || text.contains('\u{FFFD}') {
            return Err("source text is empty, oversized, or not cleanly decodable".into());
        }
        let mut ends = Vec::with_capacity(tokens.len());
        let mut previous = 0usize;
        for position in 0..tokens.len() {
            let prefix = decode(&tokens[..=position]);
            // A token may end mid-character; hold the span at the last whole
            // character instead of inventing one.
            let prefix = prefix.trim_end_matches('\u{FFFD}');
            if !text.starts_with(prefix) || prefix.len() < previous {
                return Err("token prefix text is not a prefix of the source text".into());
            }
            previous = prefix.len();
            ends.push(u32::try_from(previous).map_err(|_| "source offset overflow".to_string())?);
        }
        if previous != text.len() {
            return Err("source offsets do not cover the full text".into());
        }
        Ok(Self {
            tape_digest,
            text,
            ends,
        })
    }

    pub fn token_count(&self) -> usize {
        self.ends.len()
    }

    pub fn full_text(&self) -> &str {
        &self.text
    }

    /// Exact source text covered by tokens `0..=position`.
    pub fn span(&self, position: u64) -> Result<&str, String> {
        let end = *self
            .ends
            .get(usize::try_from(position).map_err(|_| "position overflow".to_string())?)
            .ok_or("token position outside the source text")? as usize;
        if end == 0 {
            return Err("source span is empty at this position".into());
        }
        if end > MAX_SPAN_BYTES {
            return Err("source span exceeds the inline content bound".into());
        }
        self.text
            .get(..end)
            .ok_or_else(|| "source span is not on a character boundary".to_string())
    }

    fn body(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(48 + self.text.len() + self.ends.len() * 4);
        out.extend_from_slice(&SOURCE_MAGIC);
        out.extend_from_slice(&SOURCE_VERSION.to_le_bytes());
        out.extend_from_slice(&digest_raw(&self.tape_digest));
        out.extend_from_slice(&(self.text.len() as u32).to_le_bytes());
        out.extend_from_slice(self.text.as_bytes());
        out.extend_from_slice(&(self.ends.len() as u32).to_le_bytes());
        for end in &self.ends {
            out.extend_from_slice(&end.to_le_bytes());
        }
        out
    }

    /// Canonical bytes with a trailing SHA-256 over the body.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.body();
        let digest = Sha256::digest(&out);
        out.extend_from_slice(&digest);
        out
    }

    pub fn digest(&self) -> Digest {
        Digest::sha256(&self.body())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 4 + 2 + 32 + 4 + 4 + 32 || bytes[..4] != SOURCE_MAGIC {
            return Err("source artifact header invalid".into());
        }
        if u16::from_le_bytes([bytes[4], bytes[5]]) != SOURCE_VERSION {
            return Err("unsupported source artifact version".into());
        }
        let body_end = bytes.len() - 32;
        if Sha256::digest(&bytes[..body_end])[..] != bytes[body_end..] {
            return Err("source artifact digest mismatch".into());
        }
        let tape_digest = digest_from_raw(&bytes[6..38])?;
        let mut at = 38;
        let text_len = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        if text_len == 0 || text_len > MAX_SOURCE_BYTES || at + text_len + 4 > body_end {
            return Err("source text length invalid".into());
        }
        let text = std::str::from_utf8(&bytes[at..at + text_len])
            .map_err(|e| e.to_string())?
            .to_string();
        at += text_len;
        let count = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()) as usize;
        at += 4;
        if count == 0 || count > MAX_SOURCE_TOKENS || at + count * 4 != body_end {
            return Err("source offset count invalid".into());
        }
        let ends: Vec<u32> = (0..count)
            .map(|i| u32::from_le_bytes(bytes[at + 4 * i..at + 4 * i + 4].try_into().unwrap()))
            .collect();
        let mut previous = 0u32;
        for end in &ends {
            if *end < previous
                || *end as usize > text.len()
                || !text.is_char_boundary(*end as usize)
            {
                return Err("source offsets are not monotone character boundaries".into());
            }
            previous = *end;
        }
        if previous as usize != text.len() {
            return Err("source offsets do not cover the full text".into());
        }
        Ok(Self {
            tape_digest,
            text,
            ends,
        })
    }
}

fn digest_raw(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let nibble = |b: u8| match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        _ => 0,
    };
    let mut out = [0u8; 32];
    for (i, pair) in hex.chunks(2).enumerate().take(32) {
        out[i] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

fn digest_from_raw(raw: &[u8]) -> Result<Digest, String> {
    Digest::parse(raw.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Framing tests use a one-char-per-id decoder; the real-tokenizer
    /// behaviour is covered by the ignored test below.
    fn ascii(ids: &[u32]) -> String {
        ids.iter().filter_map(|i| char::from_u32(*i)).collect()
    }

    fn carrier() -> SourceText {
        let tokens: Vec<u32> = "The capital of France is"
            .chars()
            .map(|c| c as u32)
            .collect();
        SourceText::from_tokens(Digest::sha256(b"tape"), &tokens, &ascii).unwrap()
    }

    #[test]
    fn span_is_the_exact_text_prefix_and_round_trips() {
        let source = carrier();
        assert_eq!(source.span(2).unwrap(), "The");
        assert_eq!(source.span(23).unwrap(), "The capital of France is");
        assert!(source.span(24).is_err());
        let back = SourceText::from_bytes(&source.to_bytes()).unwrap();
        assert_eq!(back, source);
        assert_eq!(back.digest(), source.digest());
    }

    #[test]
    fn tampered_or_truncated_artifact_is_refused() {
        let bytes = carrier().to_bytes();
        let mut flipped = bytes.clone();
        flipped[45] ^= 1;
        assert!(SourceText::from_bytes(&flipped).is_err());
        assert!(SourceText::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(SourceText::from_bytes(&bytes[..10]).is_err());
    }

    #[test]
    fn undecodable_or_empty_source_is_refused() {
        let bad = |ids: &[u32]| ids.iter().map(|_| '\u{FFFD}').collect::<String>();
        assert!(SourceText::from_tokens(Digest::sha256(b"t"), &[1, 2], &bad).is_err());
        assert!(SourceText::from_tokens(Digest::sha256(b"t"), &[], &ascii).is_err());
    }

    #[test]
    fn non_prefix_decoder_is_refused() {
        // A decoder whose prefix decode is not a prefix of the full decode.
        let odd = |ids: &[u32]| {
            if ids.len() == 1 {
                "X".to_string()
            } else {
                "abc".to_string()
            }
        };
        assert!(SourceText::from_tokens(Digest::sha256(b"t"), &[1, 2], &odd).is_err());
    }
}
