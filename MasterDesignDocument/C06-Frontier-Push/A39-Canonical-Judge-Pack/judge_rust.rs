// Canonical judge for A39, Rust port of deterministic-judge v1.0 (Python, tested 29/29).
// Std only, no crates. Place at orchestrator/src/judge.rs and `mod judge;` in
// ablation_runner.rs. Every verdict is reproducible from the run file alone:
// the runner stores judge_version, judge_mode, verdict, decoded_sha256,
// canonical_sha256 (normalized mode) and per-check results in AblationResult.
//
// Modes, preregistered per task in the corpus, never switched after seeing results:
//   Exact      -- byte-for-byte equality.
//   Normalized -- MLIR-aware canonicalization, then equality (see canonicalize).
//   Rubric     -- pass iff every REQUIRED check passes; optional checks reported only.
// Verdicts: Pass | Fail | Error. Missing decoded, missing expected (exact/normalized),
// malformed rubric, unknown check, invalid regex -> Error, never Pass.
// Pass rate (analyze_a39.py) = passes / ALL tasks, errors in the denominator.
//
// Deliberate limits, same as the Python judge:
//   - SSA renaming is NOT done (first-appearance renaming is unsound across scopes).
//   - op-sequence checks are heuristics from the token stream, not parses.
//   - Number formatting is preserved (1.0 != 1.00) except in last_number_eq,
//     which compares numeric values.
//   - The judge compares text; it does not execute MLIR or prove equivalence.

pub const JUDGE_VERSION: &str = "deterministic-judge-1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub id: String,
    pub check: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct JudgeReceipt {
    pub version: &'static str,
    pub mode: String,
    pub verdict: Verdict,
    pub decoded_sha256: String,
    pub expected_sha256: Option<String>,
    pub canonical_sha256: Option<String>,
    pub first_diff_offset: Option<usize>,
    pub checks: Vec<CheckResult>,
    pub error: Option<String>,
}

// ---- SHA-256 (std only, FIPS 180-4) ----
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bitlen = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    let mut out = String::with_capacity(64);
    for v in h {
        out.push_str(&format!("{:08x}", v));
    }
    out
}

// ---- Tokenizer: strings preserved exactly, // comments stripped outside strings ----
fn strip_comments(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut in_str = false;
    while i < b.len() {
        let c = b[i] as char;
        if in_str {
            out.push(b[i]);
            if c == '\\' && i + 1 < b.len() {
                out.push(b[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(b[i]);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn tokenize(text: &str) -> Vec<String> {
    let stripped = strip_comments(text);
    let b = stripped.as_bytes();
    let mut toks = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '"' {
            let start = i;
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if b[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            toks.push(stripped[start..i].to_string());
            continue;
        }
        if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '%' || c == '@' {
            let start = i;
            while i < b.len() {
                let d = b[i] as char;
                if d.is_alphanumeric() || d == '_' || d == '.' || d == '-' || d == '%' || d == '@' {
                    i += 1;
                } else {
                    break;
                }
            }
            toks.push(stripped[start..i].to_string());
            continue;
        }
        toks.push(c.to_string());
        i += 1;
    }
    toks
}

fn strip_loc(text: &str) -> String {
    // Remove loc(...) with balanced nested parens, outside strings (best-effort:
    // a loc( inside a string literal is left alone by the tokenizer path in
    // canonicalize, which strips comments first and treats strings as tokens).
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 4 <= chars.len() && chars[i..i + 4].iter().collect::<String>() == "loc(" {
            let mut depth = 0;
            let mut j = i + 3;
            while j < chars.len() {
                if chars[j] == '(' {
                    depth += 1;
                }
                if chars[j] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            i = j + 1;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn canonicalize(text: &str) -> String {
    let no_loc = strip_loc(text);
    tokenize(&no_loc).join(" ")
}

fn first_diff(a: &str, b: &str) -> Option<usize> {
    let (ab, bb) = (a.as_bytes(), b.as_bytes());
    let mut i = 0;
    while i < ab.len() && i < bb.len() {
        if ab[i] != bb[i] {
            return Some(i);
        }
        i += 1;
    }
    if ab.len() != bb.len() {
        return Some(i);
    }
    None
}

fn err_receipt(mode: &str, decoded: &str, msg: &str) -> JudgeReceipt {
    JudgeReceipt {
        version: JUDGE_VERSION,
        mode: mode.to_string(),
        verdict: Verdict::Error,
        decoded_sha256: sha256_hex(decoded.as_bytes()),
        expected_sha256: None,
        canonical_sha256: None,
        first_diff_offset: None,
        checks: vec![],
        error: Some(msg.to_string()),
    }
}

pub fn judge_exact(decoded: &str, expected: &str) -> JudgeReceipt {
    let verdict = if decoded == expected { Verdict::Pass } else { Verdict::Fail };
    JudgeReceipt {
        version: JUDGE_VERSION,
        mode: "exact".to_string(),
        verdict,
        decoded_sha256: sha256_hex(decoded.as_bytes()),
        expected_sha256: Some(sha256_hex(expected.as_bytes())),
        canonical_sha256: None,
        first_diff_offset: first_diff(decoded, expected),
        checks: vec![],
        error: None,
    }
}

pub fn judge_normalized(decoded: &str, expected: &str) -> JudgeReceipt {
    let cd = canonicalize(decoded);
    let ce = canonicalize(expected);
    let verdict = if cd == ce { Verdict::Pass } else { Verdict::Fail };
    JudgeReceipt {
        version: JUDGE_VERSION,
        mode: "normalized".to_string(),
        verdict,
        decoded_sha256: sha256_hex(decoded.as_bytes()),
        expected_sha256: Some(sha256_hex(expected.as_bytes())),
        canonical_sha256: Some(sha256_hex(cd.as_bytes())),
        first_diff_offset: first_diff(&cd, &ce),
        checks: vec![],
        error: None,
    }
}

// token_contains: expected must be a WHOLE token in the comment-stripped token
// stream. Comments and string contents cannot pass it; substrings of longer
// identifiers cannot pass it. Case-sensitive unless case_sensitive=false.
pub fn token_contains(decoded: &str, target: &str, case_sensitive: bool) -> (bool, String) {
    let toks = tokenize(decoded);
    let found = if case_sensitive {
        toks.iter().any(|t| t == target)
    } else {
        let lt = target.to_lowercase();
        toks.iter().any(|t| t.to_lowercase() == lt)
    };
    (
        found,
        format!("token {} (case_sensitive={})", if found { "present" } else { "absent" }, case_sensitive),
    )
}

// last_number_eq: the LAST number token in decoded text must equal target
// numerically. A correct number mentioned only in working does not pass.
pub fn last_number_eq(decoded: &str, target: &str) -> (bool, String) {
    let mut last: Option<(String, f64)> = None;
    let mut cur = String::new();
    let mut flush = |cur: &mut String, last: &mut Option<(String, f64)>| {
        if !cur.is_empty() {
            if let Ok(v) = cur.parse::<f64>() {
                *last = Some((cur.clone(), v));
            }
            cur.clear();
        }
    };
    for ch in decoded.chars() {
        if ch.is_ascii_digit() || ch == '.' || (ch == '-' && cur.is_empty()) {
            cur.push(ch);
        } else {
            flush(&mut cur, &mut last);
        }
    }
    flush(&mut cur, &mut last);
    match last {
        None => (false, "no number token in decoded text".to_string()),
        Some((s, v)) => match target.parse::<f64>() {
            Ok(w) => (v == w, format!("last number {} {} {}", s, if v == w { "==" } else { "!=" }, target)),
            Err(_) => (false, "last_number_eq needs numeric target".to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fn_token_not_in_function() {
        assert_eq!(token_contains("func function() {}", "fn", true).0, false);
        assert_eq!(token_contains("fn main() {}", "fn", true).0, true);
    }
    #[test]
    fn comment_and_string_do_not_pass() {
        assert_eq!(token_contains("// CWE here\nfn main() {}", "CWE", true).0, false);
        assert_eq!(token_contains("\"CWE\" fn main() {}", "CWE", true).0, false);
    }
    #[test]
    fn last_number() {
        assert_eq!(last_number_eq("255 minus 85 is 170", "170").0, true);
        assert_eq!(last_number_eq("170 is not the answer, the answer is 171", "170").0, false);
    }
    #[test]
    fn normalized_reformat_passes() {
        let a = "func.func @add(%a: i32) -> i32 { arith.addi %a, %a : i32 }";
        let b = "func.func @add ( %a : i32 ) -> i32 { // note\n arith.addi %a , %a : i32 loc(\"f\":1:1) }";
        assert_eq!(judge_normalized(a, b).verdict, Verdict::Pass);
    }
    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
