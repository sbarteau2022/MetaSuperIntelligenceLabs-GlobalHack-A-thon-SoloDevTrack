//! SHA-256 digest type (backend-native equivalent of `refraction::Sha256Digest`).

use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sha256Digest(pub [u8; 32]);

impl Sha256Digest {
    pub fn compute(data: &[u8]) -> Self {
        let mut h = Sha256::new();
        h.update(data);
        let out = h.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&out);
        Self(bytes)
    }

    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for b in self.0.iter() {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for i in 0..32 {
            out[i] = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()?;
        }
        Some(Self(out))
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Constant-time digest comparison guarded by `black_box` barriers
/// (carried from the source vault; GDPR Art. 32 / AIA-15 integrity posture).
pub fn digest_ct_eq(a: &Sha256Digest, b: &Sha256Digest) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= std::hint::black_box(a.0[i] ^ b.0[i]);
    }
    std::hint::black_box(diff) == 0
}
