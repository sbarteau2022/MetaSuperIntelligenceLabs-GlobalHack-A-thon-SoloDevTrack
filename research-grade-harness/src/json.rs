//! Typed JSON boundary (P0-5 closure).
//!
//! Doctrine: the MCP wire is JSON-RPC, but this crate carries NO serde_json
//! (offline + auditability constraint — crates.io unreachable, hand scanners
//! untyped). This module is the single validated boundary: a strict parser
//! producing a typed [`Json`] value plus a canonical [`Writer`]. All
//! envelope/response handling in `mcp_client` goes through here; ad-hoc
//! substring scanners (`sfield`-style) are forbidden on new paths.
//!
//! Strictness rules (rejections are `Err`, never silent defaults):
//! - depth cap (32): deeper nests are `Err(TooDeep)`.
//! - duplicate object keys are `Err(DuplicateKey)`.
//! - numbers: integers lenient-parse to i64 when they fit, else f64;
//!   `NaN`/`Infinity` literals are rejected (JSON has none).
//! - strings: exact escapes `\" \\ \/ \b \f \n \r \t \uXXXX` (with UTF-16
//!   surrogate-pair combining); anything else after `\` is `Err(BadEscape)`.
//!   Byte-level `\x` escapes are rejected. Control chars (< 0x20) must be
//!   escaped — a raw control byte in a string is `Err(BadString)`.
//! - no trailing bytes after the top-level value (`Err(Trailing)`), except a
//!   single optional `\n` tolerated by [`parse_line`] (stdio framing).

use std::collections::BTreeMap;

/// A validated JSON value. `BTreeMap` keeps object iteration canonical.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    /// Object member lookup.
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.get(key),
            _ => None,
        }
    }

    /// `&str` when this is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    /// `true` forJSONObject/Object with a `"status"` member in the
    /// error/denied/refused family.
    pub fn is_error_status(&self) -> bool {
        matches!(
            self.get("status").and_then(|s| s.as_str()),
            Some("error" | "denied" | "fail-explained" | "refused")
        )
    }
}

/// Strictness failures. Every variant names the byte offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonError {
    Unexpected { offset: usize, what: &'static str },
    Unterminated { offset: usize },
    TooDeep { offset: usize },
    DuplicateKey { offset: usize, key: String },
    BadEscape { offset: usize },
    BadString { offset: usize },
    BadNumber { offset: usize },
    Trailing { offset: usize },
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Unexpected { offset, what } => {
                write!(f, "json: unexpected {} at byte {}", what, offset)
            }
            JsonError::Unterminated { offset } => {
                write!(f, "json: unterminated value at byte {}", offset)
            }
            JsonError::TooDeep { offset } => {
                write!(f, "json: nesting exceeds cap at byte {}", offset)
            }
            JsonError::DuplicateKey { offset, key } => {
                write!(f, "json: duplicate key {:?} at byte {}", key, offset)
            }
            JsonError::BadEscape { offset } => {
                write!(f, "json: bad escape at byte {}", offset)
            }
            JsonError::BadString { offset } => {
                write!(f, "json: bad string at byte {}", offset)
            }
            JsonError::BadNumber { offset } => {
                write!(f, "json: bad number at byte {}", offset)
            }
            JsonError::Trailing { offset } => {
                write!(f, "json: trailing bytes at byte {}", offset)
            }
        }
    }
}

impl std::error::Error for JsonError {}

const MAX_DEPTH: usize = 32;

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            b: s.as_bytes(),
            i: 0,
        }
    }

    fn err_unexpected(&self, what: &'static str) -> JsonError {
        JsonError::Unexpected {
            offset: self.i,
            what,
        }
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn value(&mut self, depth: usize) -> Result<Json, JsonError> {
        if depth > MAX_DEPTH {
            return Err(JsonError::TooDeep { offset: self.i });
        }
        self.ws();
        match self.peek() {
            Some(b'n') => self.lit("null", Json::Null),
            Some(b't') => self.lit("true", Json::Bool(true)),
            Some(b'f') => self.lit("false", Json::Bool(false)),
            Some(b'"') => Ok(Json::Str(self.string()?)),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err_unexpected("token")),
        }
    }

    fn lit(&mut self, word: &'static str, v: Json) -> Result<Json, JsonError> {
        if self.b.len() >= self.i + word.len()
            && &self.b[self.i..self.i + word.len()] == word.as_bytes()
        {
            // `nul` must not prefix a longer identifier (there is none in JSON,
            // but `truex` must not parse as `true` + trailing).
            let end = self.i + word.len();
            if end < self.b.len() && (self.b[end].is_ascii_alphanumeric() || self.b[end] == b'_') {
                return Err(self.err_unexpected("literal"));
            }
            self.i = end;
            Ok(v)
        } else {
            Err(self.err_unexpected("literal"))
        }
    }

    fn hex4(&mut self) -> Result<u32, JsonError> {
        if self.i + 4 > self.b.len() {
            return Err(JsonError::Unterminated { offset: self.i });
        }
        let mut v: u32 = 0;
        for k in 0..4 {
            let c = self.b[self.i + k];
            let d = (c as char)
                .to_digit(16)
                .ok_or(JsonError::BadEscape { offset: self.i + k })?;
            v = v * 16 + d;
        }
        self.i += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, JsonError> {
        // caller peeks `"`; consume it.
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.i += 1;
        let mut out = String::new();
        loop {
            let c = self
                .peek()
                .ok_or(JsonError::Unterminated { offset: self.i })?;
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    let esc_at = self.i;
                    self.i += 1;
                    let e = self
                        .peek()
                        .ok_or(JsonError::Unterminated { offset: self.i })?;
                    match e {
                        b'"' => {
                            out.push('"');
                            self.i += 1;
                        }
                        b'\\' => {
                            out.push('\\');
                            self.i += 1;
                        }
                        b'/' => {
                            out.push('/');
                            self.i += 1;
                        }
                        b'b' => {
                            out.push('\u{0008}');
                            self.i += 1;
                        }
                        b'f' => {
                            out.push('\u{000C}');
                            self.i += 1;
                        }
                        b'n' => {
                            out.push('\n');
                            self.i += 1;
                        }
                        b'r' => {
                            out.push('\r');
                            self.i += 1;
                        }
                        b't' => {
                            out.push('\t');
                            self.i += 1;
                        }
                        b'u' => {
                            self.i += 1;
                            let hi = self.hex4()?;
                            let ch: char = if (0xD800..0xDC00).contains(&hi) {
                                // lead surrogate: require `\uDC00..`.
                                if self.peek() == Some(b'\\')
                                    && self.b.get(self.i + 1) == Some(&b'u')
                                {
                                    self.i += 2;
                                    let lo = self.hex4()?;
                                    if !(0xDC00..0xE000).contains(&lo) {
                                        return Err(JsonError::BadEscape { offset: esc_at });
                                    }
                                    let cp = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                    char::from_u32(cp)
                                        .ok_or(JsonError::BadEscape { offset: esc_at })?
                                } else {
                                    return Err(JsonError::BadEscape { offset: esc_at });
                                }
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return Err(JsonError::BadEscape { offset: esc_at });
                            } else {
                                char::from_u32(hi).ok_or(JsonError::BadEscape { offset: esc_at })?
                            };
                            out.push(ch);
                        }
                        _ => return Err(JsonError::BadEscape { offset: esc_at }),
                    }
                }
                0x00..=0x1F => return Err(JsonError::BadString { offset: self.i }),
                _ => {
                    // UTF-8 scalar: decode from the str slice at self.i.
                    let rest = std::str::from_utf8(&self.b[self.i..])
                        .map_err(|_| JsonError::BadString { offset: self.i })?;
                    let ch = rest
                        .chars()
                        .next()
                        .ok_or(JsonError::Unterminated { offset: self.i })?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
    }

    fn number(&mut self) -> Result<Json, JsonError> {
        let start = self.i;
        if self.peek() == Some(b'-') {
            self.i += 1;
        }
        match self.peek() {
            Some(c) if c.is_ascii_digit() => {
                if c == b'0' {
                    self.i += 1;
                    // `01` is invalid JSON; a following digit rejects the number
                    // here and surfaces as trailing/unexpected downstream.
                    if matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                        return Err(JsonError::BadNumber { offset: start });
                    }
                } else {
                    while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                        self.i += 1;
                    }
                }
            }
            _ => return Err(JsonError::BadNumber { offset: start }),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.i += 1;
            if !matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                return Err(JsonError::BadNumber { offset: start });
            }
            while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                self.i += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.i += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                return Err(JsonError::BadNumber { offset: start });
            }
            while matches!(self.peek(), Some(d) if d.is_ascii_digit()) {
                self.i += 1;
            }
        }
        let text = std::str::from_utf8(&self.b[start..self.i])
            .map_err(|_| JsonError::BadNumber { offset: start })?;
        if !is_float {
            if let Ok(v) = text.parse::<i64>() {
                return Ok(Json::Int(v));
            }
            // integer overflow of i64 falls through to f64 (JSON has no int
            // width limit); unparseable is an error.
        }
        // Overflow saturates to inf in Rust's parser; JSON has no Infinity,
        // and silently retyping an overflowed number as a string would be
        // fail-open type confusion (swarm review 2026-11-01). Reject loudly.
        match text.parse::<f64>() {
            Ok(v) if v.is_finite() => Ok(Json::Float(v)),
            _ => Err(JsonError::BadNumber { offset: start }),
        }
    }

    fn array(&mut self, depth: usize) -> Result<Json, JsonError> {
        debug_assert_eq!(self.peek(), Some(b'['));
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value(depth + 1)?);
            self.ws();
            match self.peek() {
                Some(b',') => {
                    self.i += 1;
                    self.ws();
                    // trailing comma is invalid JSON.
                    if self.peek() == Some(b']') {
                        return Err(JsonError::Unexpected {
                            offset: self.i,
                            what: "comma",
                        });
                    }
                }
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(self.err_unexpected("comma-or-bracket")),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Json, JsonError> {
        debug_assert_eq!(self.peek(), Some(b'{'));
        self.i += 1;
        let mut map = BTreeMap::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(self.err_unexpected("key"));
            }
            let key_at = self.i;
            let key = self.string()?;
            self.ws();
            if self.peek() != Some(b':') {
                return Err(self.err_unexpected("colon"));
            }
            self.i += 1;
            let v = self.value(depth + 1)?;
            if map.contains_key(&key) {
                return Err(JsonError::DuplicateKey {
                    offset: key_at,
                    key,
                });
            }
            map.insert(key, v);
            self.ws();
            match self.peek() {
                Some(b',') => {
                    self.i += 1;
                    self.ws();
                    if self.peek() == Some(b'}') {
                        return Err(JsonError::Unexpected {
                            offset: self.i,
                            what: "comma",
                        });
                    }
                }
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Obj(map));
                }
                _ => return Err(self.err_unexpected("comma-or-brace")),
            }
        }
    }
}

/// Parse one complete JSON document. Trailing whitespace allowed; any other
/// trailing byte is `Err(Trailing)`.
pub fn parse(text: &str) -> Result<Json, JsonError> {
    let mut p = Parser::new(text);
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(JsonError::Trailing { offset: p.i });
    }
    Ok(v)
}

/// Parse one stdio-framed line: a single trailing `\n` is tolerated.
/// Inner raw control bytes are still rejected by the string rule.
pub fn parse_line(line: &str) -> Result<Json, JsonError> {
    let trimmed = line.strip_suffix('\n').unwrap_or(line);
    parse(trimmed)
}

/// Canonical writer: `BTreeMap` order, shortest correct escapes.
pub fn write(v: &Json) -> String {
    let mut o = String::new();
    write_into(&mut o, v);
    o
}

fn write_into(o: &mut String, v: &Json) {
    match v {
        Json::Null => o.push_str("null"),
        Json::Bool(true) => o.push_str("true"),
        Json::Bool(false) => o.push_str("false"),
        Json::Int(n) => o.push_str(&n.to_string()),
        Json::Float(x) => {
            if x.is_finite() {
                o.push_str(&format!("{:?}", x));
            } else {
                o.push_str("null"); // JSON has no non-finite literals
            }
        }
        Json::Str(s) => write_str(o, s),
        Json::Arr(items) => {
            o.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    o.push(',');
                }
                write_into(o, it);
            }
            o.push(']');
        }
        Json::Obj(m) => {
            o.push('{');
            for (i, (k, val)) in m.iter().enumerate() {
                if i > 0 {
                    o.push(',');
                }
                write_str(o, k);
                o.push(':');
                write_into(o, val);
            }
            o.push('}');
        }
    }
}

fn write_str(o: &mut String, s: &str) {
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\u{0008}' => o.push_str("\\b"),
            '\u{000C}' => o.push_str("\\f"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                o.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => o.push(c),
        }
    }
    o.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives_round_trip() {
        for (text, want) in [
            ("null", Json::Null),
            ("true", Json::Bool(true)),
            ("false", Json::Bool(false)),
            ("0", Json::Int(0)),
            ("-17", Json::Int(-17)),
            ("1.5", Json::Float(1.5)),
            ("\"a\\\"b\\\\c\\/d\\u00e9\"", Json::Str("a\"b\\c/dé".into())),
        ] {
            let got = parse(text).expect(text);
            assert_eq!(got, want, "parse {text}");
            let back = parse(&write(&got)).expect("rewrite parses");
            assert_eq!(back, got, "round trip {text}");
        }
    }

    #[test]
    fn surrogate_pair_combines() {
        let got = parse("\"\\uD83D\\uDE00\"").expect("pair parses");
        assert_eq!(got, Json::Str("😀".into()));
    }

    #[test]
    fn rejections_are_errors_never_panics() {
        // Swarm review 2026-11-01: overflowed numbers (`1e999`, 400-digit
        // ints) must Err, never silently retype to Str.
        let giant_int = "9".repeat(400);
        for bad in [
            "",
            "{",
            "[1,",
            "{\"a\":1,}",
            "[1,2,]",
            "{\"a\":1,\"a\":2}",
            "01",
            "1.",
            "1e",
            "nul",
            "truex",
            "\"\\x41\"",
            "\"\\uD83D\"",
            "\"raw\u{1}byte\"",
            // NOTE: the parser rejects raw control bytes; the escaped form
            // above is accepted as U+0001. Raw-byte rejection is covered by
            // the BadString path (unrepresentable as a Rust str literal).
            "NaN",
            "1e999",
            "-1e999",
            giant_int.as_str(),
            &"[".repeat(40),
            "{\"a\" 1}",
            "[1 2]",
        ] {
            assert!(parse(bad).is_err(), "must reject {bad:?}");
        }
    }

    #[test]
    fn canonical_writer_orders_keys() {
        let v = parse("{\"b\":1,\"a\":2}").unwrap();
        assert_eq!(write(&v), "{\"a\":2,\"b\":1}");
    }
}
