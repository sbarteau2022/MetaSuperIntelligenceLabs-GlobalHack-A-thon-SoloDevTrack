//! AMST frame encoder, hand-built FlatBuffers, no FlatBuffers crate.
//!
//! Layout matches the backend parser in native_mtp.rs (stream_wire::Frame):
//! size-prefixed frame, 4-byte root offset, "AMST" identifier at bytes 8-12
//! of the frame body, vtable, table, then strings.
//! Table offsets: version 4, kind 6, sequence 8, text 16, identity 20,
//! profile 24, output_tokens 28, stop 32, error_code 36, object length 40.
//! Index, token_id, and elapsed_us are omitted (vtable 0), so the parser
//! reads them as 0. Token frames carry text; the backend generate path
//! concatenates token-frame text, so text is the payload.
//! This is a reconstruction, 2026-11-03, after the original amst.rs was
//! lost with the live core_lm source. It is parser-compatible, not claimed
//! byte-identical to the lost original.

pub fn frame(
    kind: u8,
    sequence: u64,
    text: &str,
    identity: &str,
    profile: &str,
    output_tokens: u32,
    stop: u8,
    error_code: u32,
) -> Vec<u8> {
    let (o_version, o_kind, o_seq, o_text, o_ident, o_prof, o_out, o_stop, o_err) =
        (4usize, 6, 8, 16, 20, 24, 28, 32, 36);
    let object_len = 40usize;
    let vtable_len = 28usize;
    let vtable_pos = 12usize;
    let table_pos = vtable_pos + vtable_len;
    let mut table = vec![0u8; object_len];
    table[0..4].copy_from_slice(&(vtable_len as i32).to_le_bytes());
    table[o_version..o_version + 2].copy_from_slice(&1u16.to_le_bytes());
    table[o_kind] = kind;
    table[o_seq..o_seq + 8].copy_from_slice(&sequence.to_le_bytes());
    table[o_out..o_out + 4].copy_from_slice(&output_tokens.to_le_bytes());
    table[o_stop] = stop;
    table[o_err..o_err + 4].copy_from_slice(&error_code.to_le_bytes());
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
    let mut vtable = Vec::new();
    vtable.extend_from_slice(&(vtable_len as u16).to_le_bytes());
    vtable.extend_from_slice(&(object_len as u16).to_le_bytes());
    for off in [
        o_version, o_kind, o_seq, 0, 0, 0, o_text, o_ident, o_prof, o_out, o_stop, o_err,
    ] {
        vtable.extend_from_slice(&(off as u16).to_le_bytes());
    }
    let mut body = Vec::new();
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(b"AMST");
    body.extend_from_slice(&vtable);
    body.extend_from_slice(&table);
    body.extend_from_slice(&strings);
    let root_offset = (table_pos - 4) as u32;
    body[0..4].copy_from_slice(&root_offset.to_le_bytes());
    let mut out = Vec::new();
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&body);
    out
}
