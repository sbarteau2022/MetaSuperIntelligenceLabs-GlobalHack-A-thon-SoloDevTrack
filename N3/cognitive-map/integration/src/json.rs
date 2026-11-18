//! Minimal JSON value + parser + serializer. Zero-dependency, hermetic like
//! Slices 1-3 (no serde): the tools layer speaks JSON at the belt boundary and
//! the persist layer stores event payloads as jsonb text. Numbers are f64;
//! integer-valued numbers serialize without a decimal point.
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj(pairs: Vec<(&str, Json)>) -> Json {
        Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Num(n) => Some(*n as i64),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&Vec<Json>> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn str_vec(&self) -> Option<Vec<String>> {
        self.as_arr().map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
    }
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) => {
                if n.fract() == 0.0 && n.abs() < 9.0e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Json::Str(s) => write!(f, "\"{}\"", esc(s)),
            Json::Arr(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Json::Obj(pairs) => {
                write!(f, "{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{v}", esc(k))?;
                }
                write!(f, "}}")
            }
        }
    }
}

pub fn parse(input: &str) -> Result<Json, String> {
    let bytes = input.as_bytes();
    let mut pos = 0usize;
    let v = parse_value(bytes, &mut pos)?;
    skip_ws(bytes, &mut pos);
    if pos != bytes.len() {
        return Err(format!("trailing input at byte {pos}"));
    }
    Ok(v)
}
fn skip_ws(b: &[u8], pos: &mut usize) {
    while *pos < b.len() && matches!(b[*pos], b' ' | b'\t' | b'\n' | b'\r') {
        *pos += 1;
    }
}
fn parse_value(b: &[u8], pos: &mut usize) -> Result<Json, String> {
    skip_ws(b, pos);
    match b.get(*pos) {
        Some(b'{') => {
            *pos += 1;
            let mut pairs = Vec::new();
            skip_ws(b, pos);
            if b.get(*pos) == Some(&b'}') {
                *pos += 1;
                return Ok(Json::Obj(pairs));
            }
            loop {
                skip_ws(b, pos);
                let key = match parse_value(b, pos)? {
                    Json::Str(s) => s,
                    _ => return Err("object key must be string".into()),
                };
                skip_ws(b, pos);
                if b.get(*pos) != Some(&b':') {
                    return Err("expected ':'".into());
                }
                *pos += 1;
                let val = parse_value(b, pos)?;
                pairs.push((key, val));
                skip_ws(b, pos);
                match b.get(*pos) {
                    Some(b',') => {
                        *pos += 1;
                    }
                    Some(b'}') => {
                        *pos += 1;
                        break;
                    }
                    _ => return Err("expected ',' or '}'".into()),
                }
            }
            Ok(Json::Obj(pairs))
        }
        Some(b'[') => {
            *pos += 1;
            let mut items = Vec::new();
            skip_ws(b, pos);
            if b.get(*pos) == Some(&b']') {
                *pos += 1;
                return Ok(Json::Arr(items));
            }
            loop {
                items.push(parse_value(b, pos)?);
                skip_ws(b, pos);
                match b.get(*pos) {
                    Some(b',') => {
                        *pos += 1;
                    }
                    Some(b']') => {
                        *pos += 1;
                        break;
                    }
                    _ => return Err("expected ',' or ']'".into()),
                }
            }
            Ok(Json::Arr(items))
        }
        Some(b'"') => Ok(Json::Str(parse_string(b, pos)?)),
        Some(b't') => {
            expect_lit(b, pos, "true")?;
            Ok(Json::Bool(true))
        }
        Some(b'f') => {
            expect_lit(b, pos, "false")?;
            Ok(Json::Bool(false))
        }
        Some(b'n') => {
            expect_lit(b, pos, "null")?;
            Ok(Json::Null)
        }
        Some(c) if *c == b'-' || c.is_ascii_digit() => parse_number(b, pos),
        _ => Err(format!("unexpected byte at {pos}")),
    }
}
fn expect_lit(b: &[u8], pos: &mut usize, lit: &str) -> Result<(), String> {
    if b[*pos..].starts_with(lit.as_bytes()) {
        *pos += lit.len();
        Ok(())
    } else {
        Err(format!("bad literal at {pos}"))
    }
}
fn parse_string(b: &[u8], pos: &mut usize) -> Result<String, String> {
    if b.get(*pos) != Some(&b'"') {
        return Err("expected string".into());
    }
    *pos += 1;
    let mut out = String::new();
    loop {
        match b.get(*pos) {
            None => return Err("unterminated string".into()),
            Some(b'"') => {
                *pos += 1;
                return Ok(out);
            }
            Some(b'\\') => {
                *pos += 1;
                match b.get(*pos) {
                    Some(b'"') => out.push('"'),
                    Some(b'\\') => out.push('\\'),
                    Some(b'/') => out.push('/'),
                    Some(b'n') => out.push('\n'),
                    Some(b'r') => out.push('\r'),
                    Some(b't') => out.push('\t'),
                    Some(b'u') => {
                        let hex = std::str::from_utf8(&b[*pos + 1..(*pos + 5).min(b.len())])
                            .map_err(|_| "bad \\u")?;
                        let code = u32::from_str_radix(hex, 16).map_err(|_| "bad \\u")?;
                        out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                        *pos += 4;
                    }
                    _ => return Err("bad escape".into()),
                }
                *pos += 1;
            }
            Some(_) => {
                // consume one UTF-8 char
                let s = std::str::from_utf8(&b[*pos..]).map_err(|_| "bad utf8")?;
                let ch = s.chars().next().ok_or("bad utf8")?;
                out.push(ch);
                *pos += ch.len_utf8();
            }
        }
    }
}
fn parse_number(b: &[u8], pos: &mut usize) -> Result<Json, String> {
    let start = *pos;
    while *pos < b.len() && matches!(b[*pos], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
        *pos += 1;
    }
    let text = std::str::from_utf8(&b[start..*pos]).map_err(|_| "bad number")?;
    text.parse::<f64>()
        .map(Json::Num)
        .map_err(|_| format!("bad number '{text}'"))
}
