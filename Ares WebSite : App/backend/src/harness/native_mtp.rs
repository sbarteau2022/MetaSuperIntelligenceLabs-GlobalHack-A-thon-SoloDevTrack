//! Native MTP automaton executor — the backend's model-seat boundary.
//!
//! The native MTP server IS the Meta Automaton, and it is model
//! agnostic: this executor never hardcodes a checkpoint, a tokenizer,
//! or a model name. The seat identity comes from the server itself,
//! in the metadata frame of every `/v1/generate` stream, and is bound
//! into the receipt digest below. A different seat loaded into the
//! native MTP server therefore changes the identity, and the receipt,
//! without any code change here.
//!
//! Wire format: native MTP streams size-prefixed FlatBuffers frames
//! (AMST identifier, schema `stream_wire::Frame`). This module parses
//! that format directly — no FlatBuffers crate is added to the backend,
//! and no frame is accepted on a partial or identifier-mismatched
//! parse. If the server is unreachable, or the stream ends without a
//! finish frame, `call` returns an error and the harness escalates
//! fail-closed. Nothing here synthesises a PASS.
//!
//! The executor input is the packet's tool name and its args JSON,
//! exactly as the harness hands it over. The capability gate and the
//! compliance permit are enforced by the caller before this code runs;
//! this executor does not invent a handle, a permit, or a gate, and it
//! does not simulate a tool by prompt. It asks the automaton seat to
//! generate packet output, and the harness judges that output.
//!
//! Receipt digest: SHA-256 over
//! `seat_identity + "\n" + tool + "\n" + args_json + "\n" + output`.
//! The identity is server-supplied, the output is server-generated,
//! so the digest binds this execution to this seat and this output.
//! It is a backend-computed binding digest, documented as such — it
//! is not a receipt the server issued, and it must never be described
//! as one.

use crate::digest::Sha256Digest;
use crate::harness::runtime::{CallOutcome, Executor};
use std::time::{Duration, Instant};

pub const DEFAULT_NATIVE_MTP_URL: &str = "http://127.0.0.1:8010";
pub const MAX_FRAME_BYTES: usize = 72 * 1024 + 4;
pub const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

// FlatBuffers vtable offsets for stream_wire::Frame.
const VT_VERSION: u16 = 4;
const VT_KIND: u16 = 6;
const VT_SEQUENCE: u16 = 8;
const VT_INDEX: u16 = 10;
const VT_TOKEN_ID: u16 = 12;
const VT_ELAPSED_US: u16 = 14;
const VT_TEXT: u16 = 16;
const VT_MODEL_IDENTITY: u16 = 18;
const VT_PROFILE: u16 = 20;
const VT_OUTPUT_TOKENS: u16 = 22;
const VT_STOP: u16 = 24;
const VT_ERROR_CODE: u16 = 26;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmstFrame {
    pub version: u16,
    pub kind: u8,
    pub sequence: u64,
    pub index: u32,
    pub token_id: u32,
    pub elapsed_us: u64,
    pub text: String,
    pub model_identity: String,
    pub profile: String,
    pub output_tokens: u32,
    pub stop: u8,
    pub error_code: u32,
}

fn u16_at(b: &[u8], p: usize) -> Result<u16, String> {
    let s = b
        .get(p..p + 2)
        .ok_or_else(|| format!("read out of bounds at {}", p))?;
    Ok(u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], p: usize) -> Result<u32, String> {
    let s = b
        .get(p..p + 4)
        .ok_or_else(|| format!("read out of bounds at {}", p))?;
    Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn u64_at(b: &[u8], p: usize) -> Result<u64, String> {
    let s = b
        .get(p..p + 8)
        .ok_or_else(|| format!("read out of bounds at {}", p))?;
    let mut a = [0u8; 8];
    a.copy_from_slice(s);
    Ok(u64::from_le_bytes(a))
}

fn i32_at(b: &[u8], p: usize) -> Result<i32, String> {
    Ok(u32_at(b, p)? as i32)
}

/// Offset of a field from the table start, or None when absent/default.
fn field_offset(
    b: &[u8],
    _table_pos: usize,
    vtable_pos: usize,
    vtable_len: usize,
    voffset: u16,
) -> Result<Option<usize>, String> {
    let idx = ((voffset - 4) / 2) as usize;
    let entry = vtable_pos + 4 + idx * 2;
    if entry + 2 > vtable_pos + vtable_len {
        return Ok(None);
    }
    let off = u16_at(b, entry)? as usize;
    if off == 0 {
        Ok(None)
    } else {
        Ok(Some(off))
    }
}

fn scalar_u16(b: &[u8], table_pos: usize, off: Option<usize>, def: u16) -> Result<u16, String> {
    match off {
        Some(o) => u16_at(b, table_pos + o),
        None => Ok(def),
    }
}

fn scalar_u8(b: &[u8], table_pos: usize, off: Option<usize>, def: u8) -> Result<u8, String> {
    match off {
        Some(o) => b
            .get(table_pos + o)
            .copied()
            .ok_or_else(|| "read out of bounds".to_string()),
        None => Ok(def),
    }
}

fn scalar_u32(b: &[u8], table_pos: usize, off: Option<usize>, def: u32) -> Result<u32, String> {
    match off {
        Some(o) => u32_at(b, table_pos + o),
        None => Ok(def),
    }
}

fn scalar_u64(b: &[u8], table_pos: usize, off: Option<usize>, def: u64) -> Result<u64, String> {
    match off {
        Some(o) => u64_at(b, table_pos + o),
        None => Ok(def),
    }
}

fn string_field(b: &[u8], table_pos: usize, off: Option<usize>) -> Result<String, String> {
    match off {
        None => Ok(String::new()),
        Some(o) => {
            let field_pos = table_pos + o;
            let rel = u32_at(b, field_pos)? as usize;
            let str_pos = field_pos
                .checked_add(rel)
                .ok_or_else(|| "string offset overflow".to_string())?;
            let len = u32_at(b, str_pos)? as usize;
            let bytes = b
                .get(str_pos + 4..str_pos + 4 + len)
                .ok_or_else(|| "string out of bounds".to_string())?;
            String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
        }
    }
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit())
}

fn pin_mission_identity(pinned: &mut Option<String>, observed: &str) -> Result<(), String> {
    if !is_hex64(observed) {
        return Err("native-mtp invalid model identity".to_string());
    }
    if let Some(current) = pinned {
        if current != observed {
            return Err("native-mtp model identity changed within mission".to_string());
        }
    } else {
        *pinned = Some(observed.to_string());
    }
    Ok(())
}

/// Parse one size-prefixed AMST frame, with the server's own validation
/// rules (version, kinds, bounds) enforced, not just the layout.
pub fn parse_frame(bytes: &[u8]) -> Result<AmstFrame, String> {
    if bytes.len() < 16 || bytes.len() > MAX_FRAME_BYTES {
        return Err(format!("frame byte bounds: {}", bytes.len()));
    }
    let size = u32_at(bytes, 0)? as usize;
    if size != bytes.len() - 4 {
        return Err("frame length mismatch".to_string());
    }
    if &bytes[8..12] != b"AMST" {
        return Err("frame identifier mismatch".to_string());
    }
    let root_offset = u32_at(bytes, 4)? as usize;
    let table_pos = 4usize
        .checked_add(root_offset)
        .ok_or_else(|| "table offset overflow".to_string())?;
    let soffset = i32_at(bytes, table_pos)?;
    if soffset <= 0 {
        return Err("bad vtable offset".to_string());
    }
    let vtable_pos = table_pos
        .checked_sub(soffset as usize)
        .ok_or_else(|| "vtable before buffer".to_string())?;
    let vtable_len = u16_at(bytes, vtable_pos)? as usize;
    if vtable_len < 4 || vtable_pos + vtable_len > bytes.len() {
        return Err("vtable out of bounds".to_string());
    }
    let off = |v: u16| field_offset(bytes, table_pos, vtable_pos, vtable_len, v);
    let version = scalar_u16(bytes, table_pos, off(VT_VERSION)?, 1)?;
    let kind = scalar_u8(bytes, table_pos, off(VT_KIND)?, 0)?;
    let sequence = scalar_u64(bytes, table_pos, off(VT_SEQUENCE)?, 0)?;
    let index = scalar_u32(bytes, table_pos, off(VT_INDEX)?, 0)?;
    let token_id = scalar_u32(bytes, table_pos, off(VT_TOKEN_ID)?, 0)?;
    let elapsed_us = scalar_u64(bytes, table_pos, off(VT_ELAPSED_US)?, 0)?;
    let text = string_field(bytes, table_pos, off(VT_TEXT)?)?;
    let model_identity = string_field(bytes, table_pos, off(VT_MODEL_IDENTITY)?)?;
    let profile = string_field(bytes, table_pos, off(VT_PROFILE)?)?;
    let output_tokens = scalar_u32(bytes, table_pos, off(VT_OUTPUT_TOKENS)?, 0)?;
    let stop = scalar_u8(bytes, table_pos, off(VT_STOP)?, 0)?;
    let error_code = scalar_u32(bytes, table_pos, off(VT_ERROR_CODE)?, 0)?;
    let frame = AmstFrame {
        version,
        kind,
        sequence,
        index,
        token_id,
        elapsed_us,
        text,
        model_identity,
        profile,
        output_tokens,
        stop,
        error_code,
    };
    validate_frame(&frame)?;
    Ok(frame)
}

fn validate_frame(f: &AmstFrame) -> Result<(), String> {
    if f.version != 1 {
        return Err("stream version".to_string());
    }
    if f.text.len() > 64 * 1024 || f.profile.len() > 4096 {
        return Err("stream string bounds".to_string());
    }
    match f.kind {
        0 => {
            if !is_hex64(&f.model_identity)
                || f.profile.is_empty()
                || f.sequence != 0
                || f.index != 0
                || f.token_id != 0
                || f.elapsed_us != 0
                || !f.text.is_empty()
                || f.output_tokens != 0
                || f.stop != 0
                || f.error_code != 0
            {
                return Err("metadata fields".to_string());
            }
        }
        1 => {
            if !f.model_identity.is_empty()
                || !f.profile.is_empty()
                || f.sequence == 0
                || f.index >= 128
                || f.output_tokens != 0
                || f.stop != 0
                || f.error_code != 0
            {
                return Err("token fields".to_string());
            }
        }
        2 => {
            if !f.model_identity.is_empty()
                || !f.profile.is_empty()
                || f.sequence == 0
                || f.index != 0
                || f.token_id != 0
                || !(1..=128).contains(&f.output_tokens)
                || !(1..=2).contains(&f.stop)
                || f.error_code != 0
            {
                return Err("finish fields".to_string());
            }
        }
        3 => {
            if !f.model_identity.is_empty()
                || !f.profile.is_empty()
                || f.index != 0
                || f.token_id != 0
                || f.output_tokens > 128
                || f.stop != 0
                || !(1..=7).contains(&f.error_code)
                || f.text.is_empty()
                || f.text.len() > 4096
            {
                return Err("error fields".to_string());
            }
        }
        _ => return Err("unknown stream kind".to_string()),
    }
    Ok(())
}

/// Parse a whole `/v1/generate` body into its frames, in order.
pub fn parse_stream(body: &[u8]) -> Result<Vec<AmstFrame>, String> {
    let mut frames = Vec::new();
    let mut pos = 0usize;
    while pos < body.len() {
        if body.len() - pos < 4 {
            return Err("truncated stream".to_string());
        }
        let size = u32_at(body, pos)? as usize;
        let total = 4usize
            .checked_add(size)
            .ok_or_else(|| "frame size overflow".to_string())?;
        if pos + total > body.len() {
            return Err("truncated stream".to_string());
        }
        frames.push(parse_frame(&body[pos..pos + total])?);
        pos += total;
    }
    if frames.is_empty() {
        return Err("empty stream".to_string());
    }
    Ok(frames)
}

pub fn parse_info_body(body: &[u8]) -> Result<(String, String), String> {
    let mut frames = parse_stream(body)?;
    if frames.len() != 1 || frames[0].kind != 0 {
        return Err("native model info must contain exactly one metadata frame".into());
    }
    let frame = frames.remove(0);
    Ok((frame.model_identity, frame.profile))
}

/// Attest the actual loaded native seat through its binary AMST info frame.
pub async fn probe_seat(base_url: &str) -> Result<(String, String), String> {
    if !base_url.starts_with("http://127.0.0.1:") && !base_url.starts_with("http://localhost:") {
        return Err("native model probe requires loopback".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1500))
        .build()
        .map_err(|e| e.to_string())?;
    let url = format!("{}/v1/info", base_url.trim_end_matches('/'));
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("native model info status {}", response.status()));
    }
    let body = response.bytes().await.map_err(|e| e.to_string())?;
    if body.len() > MAX_FRAME_BYTES {
        return Err("native model info frame exceeds cap".into());
    }
    parse_info_body(&body)
}

#[derive(Debug, Clone)]
pub struct NativeMtpConfig {
    pub base_url: String,
    pub max_new_tokens: usize,
    pub deadline_ms: u64,
}

impl NativeMtpConfig {
    pub fn from_env() -> Result<Self, String> {
        let base_url = std::env::var("ARES_NATIVE_MTP_URL")
            .unwrap_or_else(|_| DEFAULT_NATIVE_MTP_URL.to_string());
        if !base_url.starts_with("http://127.0.0.1:") && !base_url.starts_with("http://localhost:")
        {
            return Err("native MTP URL must be loopback".to_string());
        }
        let max_new_tokens = std::env::var("ARES_NATIVE_MTP_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(32);
        if !(1..=128).contains(&max_new_tokens) {
            return Err("max_new_tokens out of range 1..=128".to_string());
        }
        let deadline_ms = std::env::var("ARES_NATIVE_MTP_DEADLINE_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30_000);
        if !(1..=300_000).contains(&deadline_ms) {
            return Err("deadline_ms out of range 1..=300000".to_string());
        }
        Ok(Self {
            base_url,
            max_new_tokens,
            deadline_ms,
        })
    }
}

/// Executor bound to the native MTP automaton seat at `config.base_url`.
pub struct NativeMtpExecutor {
    config: NativeMtpConfig,
    client: reqwest::Client,
    last_identity: Option<String>,
}

impl NativeMtpExecutor {
    pub fn new(config: NativeMtpConfig) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.deadline_ms + 10_000))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            config,
            client,
            last_identity: None,
        })
    }

    pub fn from_env() -> Result<Self, String> {
        Self::new(NativeMtpConfig::from_env()?)
    }

    pub fn seat_identity(&self) -> Option<String> {
        self.last_identity.clone()
    }

    async fn generate(&mut self, tool: &str, args_json: &str) -> Result<(String, String), String> {
        let url = format!("{}/v1/generate", self.config.base_url.trim_end_matches('/'));
        let prompt = format!("tool: {}\nargs: {}\n", tool, args_json);
        let resp = self
            .client
            .post(&url)
            .header(
                "x-rust-max-new-tokens",
                self.config.max_new_tokens.to_string(),
            )
            .header("x-rust-deadline-ms", self.config.deadline_ms.to_string())
            .header("content-type", "text/plain")
            .body(prompt)
            .send()
            .await
            .map_err(|e| format!("native-mtp transport: {}", e))?;
        if !resp.status().is_success() {
            return Err(format!("native-mtp status {}", resp.status()));
        }
        let body = resp
            .bytes()
            .await
            .map_err(|e| format!("native-mtp body: {}", e))?;
        if body.len() > MAX_BODY_BYTES {
            return Err("native-mtp body over cap".to_string());
        }
        let frames = parse_stream(&body)?;
        let mut identity: Option<String> = None;
        let mut output = String::new();
        let mut finished = false;
        for f in frames {
            match f.kind {
                0 => identity = Some(f.model_identity),
                1 => output.push_str(&f.text),
                2 => finished = true,
                3 => return Err(format!("native-mtp error {}: {}", f.error_code, f.text)),
                _ => return Err("unknown stream kind".to_string()),
            }
        }
        if !finished {
            return Err("native-mtp stream ended without finish".to_string());
        }
        let identity = identity.ok_or_else(|| "native-mtp stream without metadata".to_string())?;
        // A fresh executor is a new mission seat. Once a mission has received
        // one frame, its model cannot silently change under its prefix/K/V and
        // receipt chain. Construct a new executor for an intentional swap.
        pin_mission_identity(&mut self.last_identity, &identity)?;
        Ok((output, identity))
    }
}

impl Executor for NativeMtpExecutor {
    fn call(&mut self, tool: &str, args_json: &str) -> Result<CallOutcome, String> {
        let started = Instant::now();
        // Prefer the caller's runtime (axum workers); fall back to a
        // fresh runtime when there is none (unit / live tests). A new
        // runtime per call is heavier, but it never deadlocks, and
        // executor calls are model generations, not hot-path calls.
        let result = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                tokio::task::block_in_place(|| handle.block_on(self.generate(tool, args_json)))
            }
            Err(_) => match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt.block_on(self.generate(tool, args_json)),
                Err(e) => Err(format!("runtime build: {}", e)),
            },
        };
        let dt_ns = started.elapsed().as_nanos() as u64;
        let (output, identity) = result?;
        let receipt = Sha256Digest::compute(
            format!("{}\n{}\n{}\n{}", identity, tool, args_json, output).as_bytes(),
        )
        .to_hex();
        Ok(CallOutcome {
            inner_text: output,
            receipt_sha256: receipt,
            dt_ns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_identity_cannot_change_midstream() {
        let mut pinned = None;
        let a = "a".repeat(64);
        let b = "b".repeat(64);
        assert!(pin_mission_identity(&mut pinned, &a).is_ok());
        assert!(pin_mission_identity(&mut pinned, &a).is_ok());
        assert!(pin_mission_identity(&mut pinned, &b).is_err());
        assert_eq!(pinned.as_deref(), Some(a.as_str()));
        assert!(pin_mission_identity(&mut None, &b).is_ok());
    }

    // Manual FlatBuffers builder for the parser tests. It lays out one
    // frame by hand (vtable, table, strings) following the same schema
    // the parser reads. It tests the parser's offset math; it is not a
    // substitute for a frame produced by the native MTP server itself,
    // which is why live verification against a running server is still
    // required before any live receipt is claimed.
    #[allow(clippy::too_many_arguments)]
    fn build_frame(
        kind: u8,
        sequence: u64,
        text: &str,
        identity: &str,
        profile: &str,
        output_tokens: u32,
        stop: u8,
        error_code: u32,
    ) -> Vec<u8> {
        // Table field offsets from the table start.
        let (o_version, o_kind, o_seq, o_text, o_ident, o_prof, o_out, o_stop, o_err) =
            (4usize, 6, 8, 16, 20, 24, 28, 32, 36);
        let object_len = 40usize;
        let vtable_len = 28usize;
        let vtable_pos = 12usize; // after size + root offset + identifier
        let table_pos = vtable_pos + vtable_len;
        let mut table = vec![0u8; object_len];
        table[0..4].copy_from_slice(&(vtable_len as i32).to_le_bytes()); // soffset = table - vtable
        table[o_version..o_version + 2].copy_from_slice(&1u16.to_le_bytes());
        table[o_kind] = kind;
        table[o_seq..o_seq + 8].copy_from_slice(&sequence.to_le_bytes());
        table[o_out..o_out + 4].copy_from_slice(&output_tokens.to_le_bytes());
        table[o_stop] = stop;
        table[o_err..o_err + 4].copy_from_slice(&error_code.to_le_bytes());
        // Strings follow the table.
        let mut strings = Vec::new();
        let put_str = |field_off: usize, s: &str, table: &mut Vec<u8>, strings: &mut Vec<u8>| {
            if s.is_empty() {
                return;
            }
            let str_pos = table_pos + object_len + strings.len();
            let field_pos = table_pos + field_off;
            let rel = (str_pos - field_pos) as u32;
            table[field_off..field_off + 4].copy_from_slice(&rel.to_le_bytes());
            strings.extend_from_slice(&(s.len() as u32).to_le_bytes());
            strings.extend_from_slice(s.as_bytes());
        };
        put_str(o_text, text, &mut table, &mut strings);
        put_str(o_ident, identity, &mut table, &mut strings);
        put_str(o_prof, profile, &mut table, &mut strings);
        // Vtable: length, object length, then one offset per VT field.
        let mut vtable = Vec::new();
        vtable.extend_from_slice(&(vtable_len as u16).to_le_bytes());
        vtable.extend_from_slice(&(object_len as u16).to_le_bytes());
        for off in [
            o_version, o_kind, o_seq, 0, 0, 0, o_text, o_ident, o_prof, o_out, o_stop, o_err,
        ] {
            vtable.extend_from_slice(&(off as u16).to_le_bytes());
        }
        let mut body = Vec::new();
        body.extend_from_slice(&0u32.to_le_bytes()); // root offset placeholder
        body.extend_from_slice(b"AMST");
        body.extend_from_slice(&vtable);
        body.extend_from_slice(&table);
        body.extend_from_slice(&strings);
        // Root offset: table_pos relative to the buffer start after size.
        let root_offset = (table_pos - 4) as u32;
        body[0..4].copy_from_slice(&root_offset.to_le_bytes());
        let mut out = Vec::new();
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    #[test]
    fn parser_reads_metadata_and_token_frames() {
        let ident = "a".repeat(64);
        let meta = build_frame(0, 0, "", &ident, "width-4", 0, 0, 0);
        let f = parse_frame(&meta).expect("metadata parses");
        assert_eq!(f.kind, 0);
        assert_eq!(f.model_identity, ident);
        assert_eq!(f.profile, "width-4");
        assert_eq!(
            parse_info_body(&meta).unwrap(),
            (ident.clone(), "width-4".into())
        );
        let tok = build_frame(1, 1, "hello", "", "", 0, 0, 0);
        let f = parse_frame(&tok).expect("token parses");
        assert_eq!(f.kind, 1);
        assert_eq!(f.sequence, 1);
        assert_eq!(f.text, "hello");
        let mut future_vocab = f.clone();
        future_vocab.token_id = 300_000;
        assert!(validate_frame(&future_vocab).is_ok());
        assert!(parse_info_body(&tok).is_err());
        let fin = build_frame(2, 2, "", "", "", 1, 1, 0);
        let f = parse_frame(&fin).expect("finish parses");
        assert_eq!(f.kind, 2);
        assert_eq!(f.output_tokens, 1);
    }

    #[test]
    fn parser_rejects_bad_identifier_and_size() {
        let ident = "a".repeat(64);
        let mut bad = build_frame(0, 0, "", &ident, "width-4", 0, 0, 0);
        bad[8..12].copy_from_slice(b"XXXX");
        assert!(parse_frame(&bad).is_err());
        let mut bad2 = build_frame(0, 0, "", &ident, "width-4", 0, 0, 0);
        bad2[0] = 99;
        assert!(parse_frame(&bad2).is_err());
    }

    #[test]
    fn stream_parses_in_order() {
        let ident = "b".repeat(64);
        let mut body = Vec::new();
        body.extend_from_slice(&build_frame(0, 0, "", &ident, "width-4", 0, 0, 0));
        body.extend_from_slice(&build_frame(1, 1, "a", "", "", 0, 0, 0));
        body.extend_from_slice(&build_frame(2, 2, "", "", "", 1, 1, 0));
        let frames = parse_stream(&body).expect("stream parses");
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[1].text, "a");
    }
}
