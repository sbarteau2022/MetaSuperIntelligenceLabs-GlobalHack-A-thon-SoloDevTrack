//! Native binary tensor transport for nested hyperbolic KV.
//!
//! K and V pages move as raw `f32` or `bf16` bytes. This module does not
//! put those bytes on the MCP JSON tool path. Score, rank, `acosh`, and
//! `shell_project` stay `f64` in the map; this file does not touch them.
//!
//! Durable authority, when `HYPERMIND_SURREAL=on`, is SurrealDB namespace
//! `hypermind` database `core` through the existing HTTP `/sql` client.
//! Tables are the ones already named by the design:
//! `transformer_kv_session`, `transformer_kv_page`
//! (`key_tensor` / `value_tensor` are `bytes`), and `transformer_kv_next`.
//! Shadow mode is rejected. There is no file snapshot and no second store.
//!
//! `r_12` stays opaque: `geometry_space = h3_t2_r12_v1`,
//! `quotient_status = opaque`, `contract_digest = UNSPECIFIED-R12-ACTION`.
//! That is an engineering address label, not a certified `H³×T²/r_12`.
//! RoPE and Euclidean positional encodings are inadmissible.
//!
//! The in-process [`TensorHotPath`] is the inference working set used to
//! prove write / fetch / reuse without the live desktop database. It is
//! not durable. [`commit_page`] / [`fetch_authoritative`] are the Surreal
//! path and fail closed unless the authority is `on`.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const GEOMETRY_SPACE: &str = "h3_t2_r12_v1";
pub const QUOTIENT_STATUS: &str = "opaque";
pub const R12_CONTRACT_DIGEST: &str = "UNSPECIFIED-R12-ACTION";
pub const GEOMETRY_VERSION: &str = "H3xT2-Lorentz-canonical-v2";
pub const ADMITTED_LAYOUT: &str = "row-major";
pub const ADMITTED_POSITIONAL: &str = "toroidal-phase";
pub const FRAME_MAGIC: &[u8; 4] = b"HMKV";
pub const FRAME_VERSION: u16 = 1;

const MAX_TENSOR_BYTES: usize = 16 * 1024 * 1024;
const MAX_DIMS: usize = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TensorDtype {
    F32,
    Bf16,
}

impl TensorDtype {
    pub fn as_str(self) -> &'static str {
        match self {
            TensorDtype::F32 => "f32",
            TensorDtype::Bf16 => "bf16",
        }
    }

    pub fn width(self) -> usize {
        match self {
            TensorDtype::F32 => 4,
            TensorDtype::Bf16 => 2,
        }
    }

    fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(TensorDtype::F32),
            2 => Some(TensorDtype::Bf16),
            _ => None,
        }
    }

    fn code(self) -> u8 {
        match self {
            TensorDtype::F32 => 1,
            TensorDtype::Bf16 => 2,
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "f32" => Some(TensorDtype::F32),
            "bf16" => Some(TensorDtype::Bf16),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SurrealAuthority {
    Off,
    Shadow,
    On,
}

pub trait TensorSql {
    fn authority(&self) -> SurrealAuthority;
    fn query(&self, surrealql: &str) -> Result<Vec<Value>, String>;
}

#[derive(Clone, Debug)]
pub struct WriteRequest {
    pub atom_id: u64,
    pub address_revision: u64,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub dtype: TensorDtype,
    pub layout: String,
    pub positional_encoding: String,
    pub shape: Vec<u32>,
    pub tppe: String,
    pub model_digest: [u8; 32],
    pub key_tensor: Vec<u8>,
    pub value_tensor: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct FetchRequest {
    pub atom_id: u64,
    pub address_revision: u64,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub dtype: TensorDtype,
    pub layout: String,
    pub positional_encoding: String,
    pub shape: Vec<u32>,
    pub tppe: String,
    pub model_digest: [u8; 32],
    pub key_digest: [u8; 32],
    pub value_digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct AdmittedPage {
    pub atom_id: u64,
    pub address_revision: u64,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub dtype: TensorDtype,
    pub layout: String,
    pub positional_encoding: String,
    pub shape: Vec<u32>,
    pub tppe: String,
    pub model_digest: [u8; 32],
    pub address_contract_digest: [u8; 32],
    pub key_digest: [u8; 32],
    pub value_digest: [u8; 32],
    pub key_tensor: Vec<u8>,
    pub value_tensor: Vec<u8>,
    pub session_id: String,
    pub page_id: String,
}

#[derive(Clone, Debug)]
pub struct TensorReceipt {
    pub status: String,
    pub reason: String,
    pub field: String,
    pub prefill: String,
    pub reused: bool,
    pub durability: String,
    pub geometry_space: String,
    pub quotient_status: String,
    pub contract_digest: String,
    pub model_digest_hex: String,
    pub key_digest_hex: String,
    pub value_digest_hex: String,
    pub address_contract_digest_hex: String,
    pub tppe: String,
    pub key_tensor: Vec<u8>,
    pub value_tensor: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct TensorHotPath {
    pages: BTreeMap<u64, Vec<AdmittedPage>>,
    next_links: Vec<(String, String)>,
}

impl Default for TensorHotPath {
    fn default() -> Self {
        Self::new()
    }
}

impl TensorHotPath {
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
            next_links: Vec::new(),
        }
    }

    pub fn admit(&mut self, req: WriteRequest) -> TensorReceipt {
        let page = match prepare_write(&req) {
            Ok(p) => p,
            Err(r) => return r,
        };
        self.insert_page(page, "ephemeral-hot-path")
    }

    pub fn fetch(&self, req: &FetchRequest) -> TensorReceipt {
        let wanted = match prepare_fetch(req) {
            Ok(p) => p,
            Err(r) => return r,
        };
        let Some(candidates) = self.pages.get(&req.atom_id) else {
            return absent_receipt(&wanted, "ephemeral-hot-path");
        };
        match_candidates(candidates, &wanted, "ephemeral-hot-path")
    }

    /// Record a `transformer_kv_next` link between two already-admitted pages.
    /// This does not execute an `r_12` quotient.
    pub fn record_next(&mut self, from: &AdmittedPage, to: &AdmittedPage) -> TensorReceipt {
        if from.session_id != to.session_id {
            return closed_miss(
                "session",
                "transformer_kv_next requires both pages in one session",
                "",
                "refused",
                "ephemeral-hot-path",
            );
        }
        if !self.contains_page(&from.page_id) || !self.contains_page(&to.page_id) {
            return closed_miss(
                "transformer_kv_next",
                "both pages must be admitted before a next link",
                "",
                "refused",
                "ephemeral-hot-path",
            );
        }
        self.next_links
            .push((from.page_id.clone(), to.page_id.clone()));
        let mut rcpt = admitted_receipt(from, "ephemeral-hot-path");
        rcpt.status = "checkpoint".into();
        rcpt.reason = format!(
            "transformer_kv_next {} -> {}; quotient_status={QUOTIENT_STATUS}; contract_digest={R12_CONTRACT_DIGEST}",
            from.page_id, to.page_id
        );
        rcpt.reused = false;
        rcpt.prefill = "refused".into();
        rcpt
    }

    fn insert_page(&mut self, page: AdmittedPage, durability: &str) -> TensorReceipt {
        let slot = self.pages.entry(page.atom_id).or_default();
        if let Some(existing) = slot.iter().find(|p| p.page_id == page.page_id) {
            if existing.key_digest == page.key_digest && existing.value_digest == page.value_digest
            {
                let mut rcpt = admitted_receipt(existing, durability);
                rcpt.reason = "already-admitted".into();
                return rcpt;
            }
            let field = if existing.key_digest != page.key_digest {
                "key_digest"
            } else {
                "value_digest"
            };
            return closed_miss(
                field,
                "immutable tensor page (schema 0009 READONLY); payload differs",
                &page.tppe,
                "refused",
                durability,
            );
        }
        let rcpt = admitted_receipt(&page, durability);
        slot.push(page);
        rcpt
    }

    fn contains_page(&self, page_id: &str) -> bool {
        self.pages
            .values()
            .any(|pages| pages.iter().any(|p| p.page_id == page_id))
    }
}

pub fn prepare_write(req: &WriteRequest) -> Result<AdmittedPage, TensorReceipt> {
    reject_positional(&req.positional_encoding)?;
    reject_layout(&req.layout)?;
    if req.tppe.is_empty() {
        return Err(closed_miss(
            "toroidal_phase_encoder_version",
            "TPPE version is empty",
            "",
            "refused",
            "none",
        ));
    }
    let nbytes = expected_nbytes(&req.shape, req.dtype).map_err(|reason| {
        closed_miss("shape", reason, &req.tppe, "refused", "none")
    })?;
    if req.key_tensor.len() != nbytes || req.value_tensor.len() != nbytes {
        return Err(closed_miss(
            "shape",
            "tensor byte length does not match shape, layout, and dtype",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.key_tensor.is_empty() || req.value_tensor.is_empty() {
        return Err(closed_miss(
            "key_tensor",
            "empty K or V page",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    let address_contract_digest = address_digest(req);
    let key_digest = sha256(&req.key_tensor);
    let value_digest = sha256(&req.value_tensor);
    let session_id = session_id(&req.model_digest, &req.tppe);
    let page_id = page_id(
        &session_id,
        req.atom_id,
        req.address_revision,
        &address_contract_digest,
        req.dtype,
        &req.layout,
        &req.shape,
    );
    Ok(AdmittedPage {
        atom_id: req.atom_id,
        address_revision: req.address_revision,
        h3_lorentz: req.h3_lorentz,
        t2_phase: req.t2_phase,
        dtype: req.dtype,
        layout: req.layout.clone(),
        positional_encoding: ADMITTED_POSITIONAL.into(),
        shape: req.shape.clone(),
        tppe: req.tppe.clone(),
        model_digest: req.model_digest,
        address_contract_digest,
        key_digest,
        value_digest,
        key_tensor: req.key_tensor.clone(),
        value_tensor: req.value_tensor.clone(),
        session_id,
        page_id,
    })
}

struct FetchSlot {
    dtype: TensorDtype,
    layout: String,
    shape: Vec<u32>,
    tppe: String,
    model_digest: [u8; 32],
    address_contract_digest: [u8; 32],
    key_digest: [u8; 32],
    value_digest: [u8; 32],
}

fn prepare_fetch(req: &FetchRequest) -> Result<FetchSlot, TensorReceipt> {
    reject_positional(&req.positional_encoding)?;
    reject_layout(&req.layout)?;
    if req.tppe.is_empty() {
        return Err(closed_miss(
            "toroidal_phase_encoder_version",
            "TPPE version is empty",
            "",
            "refused",
            "none",
        ));
    }
    expected_nbytes(&req.shape, req.dtype).map_err(|reason| {
        closed_miss("shape", reason, &req.tppe, "refused", "none")
    })?;
    let address_contract_digest = address_digest_parts(
        req.atom_id,
        req.address_revision,
        &req.h3_lorentz,
        &req.t2_phase,
    );
    Ok(FetchSlot {
        dtype: req.dtype,
        layout: req.layout.clone(),
        shape: req.shape.clone(),
        tppe: req.tppe.clone(),
        model_digest: req.model_digest,
        address_contract_digest,
        key_digest: req.key_digest,
        value_digest: req.value_digest,
    })
}

fn match_candidates(candidates: &[AdmittedPage], wanted: &FetchSlot, durability: &str) -> TensorReceipt {
    if candidates.is_empty() {
        return absent_receipt(wanted, durability);
    }
    let mut first_miss: Option<TensorReceipt> = None;
    for page in candidates {
        let rcpt = exact_match(page, wanted, durability);
        if rcpt.status == "hit" {
            return rcpt;
        }
        if first_miss.is_none() {
            first_miss = Some(rcpt);
        }
    }
    first_miss.unwrap_or_else(|| absent_receipt(wanted, durability))
}

fn exact_match(page: &AdmittedPage, wanted: &FetchSlot, durability: &str) -> TensorReceipt {
    if let Some(field) = first_mismatch(page, wanted) {
        return closed_miss(
            field,
            "exact-match admission failed",
            &wanted.tppe,
            "refused",
            durability,
        );
    }
    let mut rcpt = admitted_receipt(page, durability);
    rcpt.status = "hit".into();
    rcpt.reason = "exact contract; reuse tensor page without re-prefill".into();
    rcpt.prefill = "skipped".into();
    rcpt.reused = true;
    rcpt.key_tensor = page.key_tensor.clone();
    rcpt.value_tensor = page.value_tensor.clone();
    rcpt
}

fn first_mismatch(page: &AdmittedPage, wanted: &FetchSlot) -> Option<&'static str> {
    if page.model_digest != wanted.model_digest {
        return Some("model_digest");
    }
    if page.dtype != wanted.dtype {
        return Some("dtype");
    }
    if page.layout != wanted.layout {
        return Some("layout");
    }
    if page.shape != wanted.shape {
        return Some("shape");
    }
    if page.tppe != wanted.tppe {
        return Some("toroidal_phase_encoder_version");
    }
    if page.address_contract_digest != wanted.address_contract_digest {
        return Some("address_contract_digest");
    }
    if page.key_digest != wanted.key_digest {
        return Some("key_digest");
    }
    if page.value_digest != wanted.value_digest {
        return Some("value_digest");
    }
    None
}

fn absent_receipt(wanted: &FetchSlot, durability: &str) -> TensorReceipt {
    closed_miss(
        "transformer_kv_page",
        "no tensor page for this atom; closed miss, not a silent miss",
        &wanted.tppe,
        "required",
        durability,
    )
}

pub fn commit_page(sink: &impl TensorSql, page: &AdmittedPage) -> TensorReceipt {
    if let Some(rcpt) = authority_block(sink.authority()) {
        return rcpt;
    }
    let statement = insert_page_sql(page);
    match sink.query(&statement) {
        Ok(_) => admitted_receipt(page, "surreal"),
        Err(e) => closed_miss(
            "surreal",
            &format!("surreal transport failed: {e}"),
            &page.tppe,
            "refused",
            "none",
        ),
    }
}

pub fn commit_next(sink: &impl TensorSql, from: &AdmittedPage, to: &AdmittedPage) -> TensorReceipt {
    if let Some(rcpt) = authority_block(sink.authority()) {
        return rcpt;
    }
    if from.session_id != to.session_id {
        return closed_miss(
            "session",
            "transformer_kv_next requires both pages in one session",
            &from.tppe,
            "refused",
            "none",
        );
    }
    let statement = next_sql(from, to);
    match sink.query(&statement) {
        Ok(_) => {
            let mut rcpt = admitted_receipt(from, "surreal");
            rcpt.status = "checkpoint".into();
            rcpt.reason = format!(
                "transformer_kv_next {} -> {}; quotient_status={QUOTIENT_STATUS}",
                from.page_id, to.page_id
            );
            rcpt
        }
        Err(e) => closed_miss(
            "transformer_kv_next",
            &format!("surreal transport failed: {e}"),
            &from.tppe,
            "refused",
            "none",
        ),
    }
}

pub fn fetch_authoritative(sink: &impl TensorSql, req: &FetchRequest) -> TensorReceipt {
    let wanted = match prepare_fetch(req) {
        Ok(p) => p,
        Err(r) => return r,
    };
    if let Some(rcpt) = authority_block(sink.authority()) {
        return rcpt;
    }
    let statement = select_pages_sql(req.atom_id);
    let results = match sink.query(&statement) {
        Ok(v) => v,
        Err(e) => {
            return closed_miss(
                "surreal",
                &format!("surreal transport failed: {e}"),
                &req.tppe,
                "refused",
                "none",
            );
        }
    };
    let mut pages = Vec::new();
    for result in &results {
        for row in rows_of(result) {
            match page_from_row(row) {
                Ok(page) if page.atom_id == req.atom_id => pages.push(page),
                Ok(_) => {}
                Err(rcpt) => return rcpt,
            }
        }
    }
    match_candidates(&pages, &wanted, "surreal")
}

fn authority_block(mode: SurrealAuthority) -> Option<TensorReceipt> {
    match mode {
        SurrealAuthority::On => None,
        SurrealAuthority::Shadow => Some(closed_miss(
            "surreal",
            "shadow mode is rejected; tensor pages require HYPERMIND_SURREAL=on",
            "",
            "refused",
            "none",
        )),
        SurrealAuthority::Off => Some(closed_miss(
            "surreal",
            "transport disabled (HYPERMIND_SURREAL=off); tensor pages fail closed",
            "",
            "refused",
            "none",
        )),
    }
}

pub fn insert_page_sql(page: &AdmittedPage) -> String {
    let shape = page
        .shape
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "CREATE transformer_kv_session:`{session}` CONTENT {{ workspace: workspace:hypermind, geometry_space: {geo}, quotient_status: {quot}, contract_digest: {r12}, model_digest: {model}, toroidal_phase_encoder_version: {tppe}, created_at: time::now() }}; CREATE transformer_kv_page:`{page_id}` CONTENT {{ workspace: workspace:hypermind, session: transformer_kv_session:`{session}`, atom: atom:{atom}, address: atom_address:{atom}, model_digest: {model}, dtype: {dtype}, layout: {layout}, shape: [{shape}], toroidal_phase_encoder_version: {tppe}, address_contract_digest: {addr}, key_digest: {kd}, value_digest: {vd}, geometry_space: {geo}, quotient_status: {quot}, contract_digest: {r12}, key_tensor: b\"{kbytes}\", value_tensor: b\"{vbytes}\", created_at: time::now() }};",
        session = page.session_id,
        page_id = page.page_id,
        atom = page.atom_id,
        geo = sql_quote(GEOMETRY_SPACE),
        quot = sql_quote(QUOTIENT_STATUS),
        r12 = sql_quote(R12_CONTRACT_DIGEST),
        model = sql_quote(&hex_encode(&page.model_digest)),
        tppe = sql_quote(&page.tppe),
        dtype = sql_quote(page.dtype.as_str()),
        layout = sql_quote(&page.layout),
        shape = shape,
        addr = sql_quote(&hex_encode(&page.address_contract_digest)),
        kd = sql_quote(&hex_encode(&page.key_digest)),
        vd = sql_quote(&hex_encode(&page.value_digest)),
        kbytes = hex_encode(&page.key_tensor),
        vbytes = hex_encode(&page.value_tensor),
    )
}

pub fn select_pages_sql(atom_id: u64) -> String {
    format!(
        "SELECT atom, model_digest, dtype, layout, shape, toroidal_phase_encoder_version, address_contract_digest, key_digest, value_digest, key_tensor, value_tensor, geometry_space, quotient_status, contract_digest FROM transformer_kv_page WHERE atom = atom:{atom_id};"
    )
}

pub fn next_sql(from: &AdmittedPage, to: &AdmittedPage) -> String {
    format!(
        "CREATE transformer_kv_next CONTENT {{ workspace: workspace:hypermind, session: transformer_kv_session:`{session}`, from_page: transformer_kv_page:`{from}`, to_page: transformer_kv_page:`{to}`, created_at: time::now() }};",
        session = from.session_id,
        from = from.page_id,
        to = to.page_id,
    )
}

pub fn page_from_row(row: &Value) -> Result<AdmittedPage, TensorReceipt> {
    let geometry = row.get("geometry_space").and_then(|v| v.as_str()).unwrap_or("");
    let quotient = row.get("quotient_status").and_then(|v| v.as_str()).unwrap_or("");
    let contract = row.get("contract_digest").and_then(|v| v.as_str()).unwrap_or("");
    if geometry != GEOMETRY_SPACE || quotient != QUOTIENT_STATUS || contract != R12_CONTRACT_DIGEST
    {
        return Err(closed_miss(
            "quotient_status",
            "page geometry is not the opaque h3_t2_r12_v1 contract",
            "",
            "refused",
            "surreal",
        ));
    }
    let dtype = TensorDtype::parse(row.get("dtype").and_then(|v| v.as_str()).unwrap_or("")).ok_or_else(|| {
        closed_miss("dtype", "stored dtype is not f32 or bf16", "", "refused", "surreal")
    })?;
    let layout = row.get("layout").and_then(|v| v.as_str()).unwrap_or("").to_string();
    reject_layout(&layout)?;
    let tppe = row
        .get("toroidal_phase_encoder_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let shape = shape_from_value(row.get("shape"))?;
    let key_tensor = bytes_from_value(row.get("key_tensor"))?;
    let value_tensor = bytes_from_value(row.get("value_tensor"))?;
    let nbytes = expected_nbytes(&shape, dtype).map_err(|reason| {
        closed_miss("shape", reason, &tppe, "refused", "surreal")
    })?;
    if key_tensor.len() != nbytes || value_tensor.len() != nbytes {
        return Err(closed_miss(
            "shape",
            "stored tensor length does not match shape and dtype",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let key_digest = parse_digest_field(row, "key_digest")?;
    let value_digest = parse_digest_field(row, "value_digest")?;
    if sha256(&key_tensor) != key_digest || sha256(&value_tensor) != value_digest {
        return Err(closed_miss(
            "key_digest",
            "stored bytes do not match stored digest",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let model_digest = parse_digest_field(row, "model_digest")?;
    let address_contract_digest = parse_digest_field(row, "address_contract_digest")?;
    let atom_id = record_atom_id(row.get("atom")).unwrap_or(0);
    if atom_id == 0 {
        return Err(closed_miss(
            "atom",
            "tensor page is not linked to an atom",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let session_id = session_id(&model_digest, &tppe);
    let page_id = page_id(
        &session_id,
        atom_id,
        0,
        &address_contract_digest,
        dtype,
        &layout,
        &shape,
    );
    Ok(AdmittedPage {
        atom_id,
        address_revision: 0,
        h3_lorentz: [0.0; 4],
        t2_phase: [0.0; 2],
        dtype,
        layout,
        positional_encoding: ADMITTED_POSITIONAL.into(),
        shape,
        tppe,
        model_digest,
        address_contract_digest,
        key_digest,
        value_digest,
        key_tensor,
        value_tensor,
        session_id,
        page_id,
    })
}

/// JSON view of a page the way SurrealDB 3 renders `bytes` in an `/sql`
/// `application/json` body: `b"HEX"`. Used to test the decoder. The hot
/// path never builds this.
pub fn row_from_page(page: &AdmittedPage) -> Value {
    serde_json::json!({
        "atom": format!("atom:{}", page.atom_id),
        "model_digest": hex_encode(&page.model_digest),
        "dtype": page.dtype.as_str(),
        "layout": page.layout,
        "shape": page.shape,
        "toroidal_phase_encoder_version": page.tppe,
        "address_contract_digest": hex_encode(&page.address_contract_digest),
        "key_digest": hex_encode(&page.key_digest),
        "value_digest": hex_encode(&page.value_digest),
        "key_tensor": format!("b\"{}\"", hex_encode(&page.key_tensor)),
        "value_tensor": format!("b\"{}\"", hex_encode(&page.value_tensor)),
        "geometry_space": GEOMETRY_SPACE,
        "quotient_status": QUOTIENT_STATUS,
        "contract_digest": R12_CONTRACT_DIGEST,
    })
}

pub fn dispatch_hot(path: &mut TensorHotPath, frame: &[u8]) -> Vec<u8> {
    encode_receipt(&match decode_request(frame) {
        Ok(Decoded::Write(req)) => path.admit(req),
        Ok(Decoded::Fetch(req)) => path.fetch(&req),
        Err(rcpt) => rcpt,
    })
}

pub fn dispatch_authoritative(sink: &impl TensorSql, frame: &[u8]) -> Vec<u8> {
    let rcpt = match decode_request(frame) {
        Ok(Decoded::Write(req)) => match prepare_write(&req) {
            Ok(page) => commit_page(sink, &page),
            Err(rcpt) => rcpt,
        },
        Ok(Decoded::Fetch(req)) => fetch_authoritative(sink, &req),
        Err(rcpt) => rcpt,
    };
    encode_receipt(&rcpt)
}

pub fn run_tensor_session<R: std::io::Read, W: std::io::Write>(
    mut reader: R,
    mut writer: W,
    sink: &impl TensorSql,
) -> i32 {
    if sink.authority() != SurrealAuthority::On {
        let rcpt = authority_block(sink.authority()).unwrap_or_else(|| {
            closed_miss("surreal", "tensor session requires HYPERMIND_SURREAL=on", "", "refused", "none")
        });
        let frame = encode_receipt(&rcpt);
        let _ = write_framed(&mut writer, &frame);
        return 2;
    }
    let mut exit = 0;
    loop {
        let len = match read_u32(&mut reader) {
            Ok(Some(n)) => n as usize,
            Ok(None) => break,
            Err(_) => return 2,
        };
        if len > MAX_TENSOR_BYTES * 2 + 4096 {
            let frame = encode_receipt(&closed_miss(
                "frame",
                "tensor frame exceeds limit",
                "",
                "refused",
                "none",
            ));
            let _ = write_framed(&mut writer, &frame);
            return 2;
        }
        let mut frame = vec![0u8; len];
        if reader.read_exact(&mut frame).is_err() {
            return 2;
        }
        let out = dispatch_authoritative(sink, &frame);
        if write_framed(&mut writer, &out).is_err() {
            return 2;
        }
        if let Ok(rcpt) = decode_receipt(&out) {
            if rcpt.status == "fail-closed" || rcpt.field == "surreal" {
                exit = 2;
            } else if rcpt.status == "closed-miss" && exit == 0 {
                exit = 1;
            }
        }
    }
    exit
}

enum Decoded {
    Write(WriteRequest),
    Fetch(FetchRequest),
}

pub fn encode_write(req: &WriteRequest) -> Vec<u8> {
    let mut out = Vec::new();
    push_header(&mut out, 1);
    out.push(req.dtype.code());
    push_str(&mut out, &req.positional_encoding);
    push_str(&mut out, &req.layout);
    push_str(&mut out, &req.tppe);
    out.extend_from_slice(&req.model_digest);
    out.extend_from_slice(&req.atom_id.to_le_bytes());
    out.extend_from_slice(&req.address_revision.to_le_bytes());
    for x in req.h3_lorentz {
        out.extend_from_slice(&x.to_le_bytes());
    }
    for x in req.t2_phase {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out.push(req.shape.len() as u8);
    for d in &req.shape {
        out.extend_from_slice(&d.to_le_bytes());
    }
    push_bytes(&mut out, &req.key_tensor);
    push_bytes(&mut out, &req.value_tensor);
    out
}

pub fn encode_fetch(req: &FetchRequest) -> Vec<u8> {
    let mut out = Vec::new();
    push_header(&mut out, 2);
    out.push(req.dtype.code());
    push_str(&mut out, &req.positional_encoding);
    push_str(&mut out, &req.layout);
    push_str(&mut out, &req.tppe);
    out.extend_from_slice(&req.model_digest);
    out.extend_from_slice(&req.atom_id.to_le_bytes());
    out.extend_from_slice(&req.address_revision.to_le_bytes());
    for x in req.h3_lorentz {
        out.extend_from_slice(&x.to_le_bytes());
    }
    for x in req.t2_phase {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out.push(req.shape.len() as u8);
    for d in &req.shape {
        out.extend_from_slice(&d.to_le_bytes());
    }
    out.extend_from_slice(&req.key_digest);
    out.extend_from_slice(&req.value_digest);
    out
}

pub fn encode_receipt(rcpt: &TensorReceipt) -> Vec<u8> {
    let mut out = Vec::new();
    push_header(&mut out, 3);
    out.push(status_code(&rcpt.status));
    out.push(prefill_code(&rcpt.prefill));
    out.push(u8::from(rcpt.reused));
    push_str(&mut out, &rcpt.field);
    push_str(&mut out, &rcpt.reason);
    push_str(&mut out, &rcpt.durability);
    push_str(&mut out, &rcpt.tppe);
    out.extend_from_slice(&parse_hex32(&rcpt.model_digest_hex).unwrap_or([0u8; 32]));
    out.extend_from_slice(&parse_hex32(&rcpt.key_digest_hex).unwrap_or([0u8; 32]));
    out.extend_from_slice(&parse_hex32(&rcpt.value_digest_hex).unwrap_or([0u8; 32]));
    out.extend_from_slice(&parse_hex32(&rcpt.address_contract_digest_hex).unwrap_or([0u8; 32]));
    if rcpt.reused && !rcpt.key_tensor.is_empty() {
        out.push(1);
        push_bytes(&mut out, &rcpt.key_tensor);
        push_bytes(&mut out, &rcpt.value_tensor);
    } else {
        out.push(0);
    }
    out
}

pub fn decode_receipt(frame: &[u8]) -> Result<TensorReceipt, TensorReceipt> {
    let mut rd = Rd::new(frame);
    rd.expect_header(3)?;
    let status = status_name(rd.u8()?);
    let prefill = prefill_name(rd.u8()?);
    let reused = rd.u8()? == 1;
    let field = rd.str()?;
    let reason = rd.str()?;
    let durability = rd.str()?;
    let tppe = rd.str()?;
    let model = rd.arr32()?;
    let key_d = rd.arr32()?;
    let value_d = rd.arr32()?;
    let addr = rd.arr32()?;
    let has = rd.u8()?;
    let (key_tensor, value_tensor) = if has == 1 {
        (rd.bytes()?, rd.bytes()?)
    } else {
        (Vec::new(), Vec::new())
    };
    Ok(TensorReceipt {
        status,
        reason,
        field,
        prefill,
        reused,
        durability,
        geometry_space: GEOMETRY_SPACE.into(),
        quotient_status: QUOTIENT_STATUS.into(),
        contract_digest: R12_CONTRACT_DIGEST.into(),
        model_digest_hex: hex_encode(&model),
        key_digest_hex: hex_encode(&key_d),
        value_digest_hex: hex_encode(&value_d),
        address_contract_digest_hex: hex_encode(&addr),
        tppe,
        key_tensor,
        value_tensor,
    })
}

fn decode_request(frame: &[u8]) -> Result<Decoded, TensorReceipt> {
    let mut rd = Rd::new(frame);
    let op = rd.header_op()?;
    let dtype = TensorDtype::from_code(rd.u8()?).ok_or_else(|| {
        closed_miss("dtype", "dtype is not f32 or bf16", "", "refused", "none")
    })?;
    let positional_encoding = rd.str()?;
    let layout = rd.str()?;
    let tppe = rd.str()?;
    let model_digest = rd.arr32()?;
    let atom_id = rd.u64()?;
    let address_revision = rd.u64()?;
    let h3_lorentz = [rd.f64()?, rd.f64()?, rd.f64()?, rd.f64()?];
    let t2_phase = [rd.f64()?, rd.f64()?];
    let rank = rd.u8()? as usize;
    if rank == 0 || rank > MAX_DIMS {
        return Err(closed_miss("shape", "rank out of range", &tppe, "refused", "none"));
    }
    let mut shape = Vec::with_capacity(rank);
    for _ in 0..rank {
        shape.push(rd.u32()?);
    }
    match op {
        1 => {
            let key_tensor = rd.bytes()?;
            let value_tensor = rd.bytes()?;
            Ok(Decoded::Write(WriteRequest {
                atom_id,
                address_revision,
                h3_lorentz,
                t2_phase,
                dtype,
                layout,
                positional_encoding,
                shape,
                tppe,
                model_digest,
                key_tensor,
                value_tensor,
            }))
        }
        2 => Ok(Decoded::Fetch(FetchRequest {
            atom_id,
            address_revision,
            h3_lorentz,
            t2_phase,
            dtype,
            layout,
            positional_encoding,
            shape,
            tppe,
            model_digest,
            key_digest: rd.arr32()?,
            value_digest: rd.arr32()?,
        })),
        _ => Err(closed_miss("frame", "unknown tensor op", &tppe, "refused", "none")),
    }
}

struct Rd<'a> {
    buf: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, i: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], TensorReceipt> {
        let end = self.i.checked_add(n).ok_or_else(|| bad_frame("length overflow"))?;
        if end > self.buf.len() {
            return Err(bad_frame("truncated frame"));
        }
        let s = &self.buf[self.i..end];
        self.i = end;
        Ok(s)
    }

    fn u8(&mut self) -> Result<u8, TensorReceipt> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, TensorReceipt> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&mut self) -> Result<u32, TensorReceipt> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&mut self) -> Result<u64, TensorReceipt> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
    }

    fn f64(&mut self) -> Result<f64, TensorReceipt> {
        let b = self.take(8)?;
        Ok(f64::from_le_bytes(b.try_into().unwrap_or([0; 8])))
    }

    fn arr32(&mut self) -> Result<[u8; 32], TensorReceipt> {
        let b = self.take(32)?;
        let mut a = [0u8; 32];
        a.copy_from_slice(b);
        Ok(a)
    }

    fn str(&mut self) -> Result<String, TensorReceipt> {
        let n = self.u16()? as usize;
        let b = self.take(n)?;
        String::from_utf8(b.to_vec()).map_err(|_| bad_frame("utf8"))
    }

    fn bytes(&mut self) -> Result<Vec<u8>, TensorReceipt> {
        let n = self.u32()? as usize;
        if n > MAX_TENSOR_BYTES {
            return Err(closed_miss("frame", "tensor exceeds limit", "", "refused", "none"));
        }
        Ok(self.take(n)?.to_vec())
    }

    fn header_op(&mut self) -> Result<u8, TensorReceipt> {
        let magic = self.take(4)?;
        if magic != FRAME_MAGIC {
            return Err(bad_frame("magic"));
        }
        let ver = self.u16()?;
        if ver != FRAME_VERSION {
            return Err(bad_frame("version"));
        }
        self.u8()
    }

    fn expect_header(&mut self, op: u8) -> Result<(), TensorReceipt> {
        let got = self.header_op()?;
        if got != op {
            return Err(bad_frame("op"));
        }
        Ok(())
    }
}

fn push_header(out: &mut Vec<u8>, op: u8) {
    out.extend_from_slice(FRAME_MAGIC);
    out.extend_from_slice(&FRAME_VERSION.to_le_bytes());
    out.push(op);
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    let n = u16::try_from(b.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&n.to_le_bytes());
    out.extend_from_slice(&b[..n as usize]);
}

fn push_bytes(out: &mut Vec<u8>, b: &[u8]) {
    let n = u32::try_from(b.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&n.to_le_bytes());
    out.extend_from_slice(&b[..n as usize]);
}

fn write_framed<W: std::io::Write>(writer: &mut W, frame: &[u8]) -> std::io::Result<()> {
    let n = u32::try_from(frame.len()).unwrap_or(u32::MAX);
    writer.write_all(&n.to_le_bytes())?;
    writer.write_all(&frame[..n as usize])?;
    writer.flush()
}

fn read_u32<R: std::io::Read>(reader: &mut R) -> std::io::Result<Option<u32>> {
    let mut buf = [0u8; 4];
    let mut filled = 0;
    while filled < 4 {
        match reader.read(&mut buf[filled..])? {
            0 if filled == 0 => return Ok(None),
            0 => return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "truncated length")),
            n => filled += n,
        }
    }
    Ok(Some(u32::from_le_bytes(buf)))
}

fn status_code(status: &str) -> u8 {
    match status {
        "admitted" => 1,
        "hit" => 2,
        "closed-miss" => 3,
        "checkpoint" => 5,
        _ => 4,
    }
}

fn status_name(code: u8) -> String {
    match code {
        1 => "admitted",
        2 => "hit",
        3 => "closed-miss",
        5 => "checkpoint",
        _ => "fail-closed",
    }
    .into()
}

fn prefill_code(prefill: &str) -> u8 {
    match prefill {
        "skipped" => 0,
        "required" => 2,
        _ => 1,
    }
}

fn prefill_name(code: u8) -> String {
    match code {
        0 => "skipped",
        2 => "required",
        _ => "refused",
    }
    .into()
}

fn bad_frame(why: &str) -> TensorReceipt {
    closed_miss("frame", &format!("bad tensor frame: {why}"), "", "refused", "none")
}

fn reject_positional(enc: &str) -> Result<(), TensorReceipt> {
    let lower = enc.to_ascii_lowercase();
    if lower == ADMITTED_POSITIONAL {
        return Ok(());
    }
    let why = if lower.contains("rope") || lower.contains("euclid") {
        "RoPE and Euclidean positional encodings are inadmissible"
    } else {
        "positional encoding is not toroidal-phase"
    };
    Err(closed_miss("positional_encoding", why, "", "refused", "none"))
}

fn reject_layout(layout: &str) -> Result<(), TensorReceipt> {
    if layout == ADMITTED_LAYOUT {
        Ok(())
    } else {
        Err(closed_miss(
            "layout",
            "layout is not row-major",
            "",
            "refused",
            "none",
        ))
    }
}

fn expected_nbytes(shape: &[u32], dtype: TensorDtype) -> Result<usize, &'static str> {
    if shape.is_empty() || shape.len() > MAX_DIMS {
        return Err("shape rank out of range");
    }
    let mut n: usize = 1;
    for d in shape {
        if *d == 0 {
            return Err("shape dimension is zero");
        }
        n = n.checked_mul(*d as usize).ok_or("shape overflow")?;
    }
    n.checked_mul(dtype.width()).ok_or("shape overflow")
}

fn address_digest(req: &WriteRequest) -> [u8; 32] {
    address_digest_parts(req.atom_id, req.address_revision, &req.h3_lorentz, &req.t2_phase)
}

fn address_digest_parts(
    atom_id: u64,
    revision: u64,
    h3: &[f64; 4],
    phase: &[f64; 2],
) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ADDR");
    h.update(GEOMETRY_SPACE.as_bytes());
    h.update(GEOMETRY_VERSION.as_bytes());
    h.update(QUOTIENT_STATUS.as_bytes());
    h.update(R12_CONTRACT_DIGEST.as_bytes());
    h.update(atom_id.to_le_bytes());
    h.update(revision.to_le_bytes());
    for x in h3 {
        h.update(x.to_le_bytes());
    }
    for x in phase {
        h.update(x.to_le_bytes());
    }
    h.finalize().into()
}

fn session_id(model: &[u8; 32], tppe: &str) -> String {
    let mut h = Sha256::new();
    h.update(b"SESS");
    h.update(model);
    h.update(tppe.as_bytes());
    h.update(GEOMETRY_SPACE.as_bytes());
    h.update(QUOTIENT_STATUS.as_bytes());
    h.update(R12_CONTRACT_DIGEST.as_bytes());
    hex_encode(&h.finalize())
}

fn page_id(
    session: &str,
    atom_id: u64,
    revision: u64,
    address: &[u8; 32],
    dtype: TensorDtype,
    layout: &str,
    shape: &[u32],
) -> String {
    let mut h = Sha256::new();
    h.update(b"PAGE");
    h.update(session.as_bytes());
    h.update(atom_id.to_le_bytes());
    h.update(revision.to_le_bytes());
    h.update(address);
    h.update([dtype.code()]);
    h.update(layout.as_bytes());
    for d in shape {
        h.update(d.to_le_bytes());
    }
    hex_encode(&h.finalize())
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn parse_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    let bytes = s.as_bytes();
    for i in 0..32 {
        let hi = hex_val(bytes[i * 2])?;
        let lo = hex_val(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn sql_quote(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string())
}

fn parse_digest_field(row: &Value, field: &str) -> Result<[u8; 32], TensorReceipt> {
    let s = row.get(field).and_then(|v| v.as_str()).unwrap_or("");
    parse_hex32(s).ok_or_else(|| {
        closed_miss(field, "digest is not 32-byte hex", "", "refused", "surreal")
    })
}

fn shape_from_value(v: Option<&Value>) -> Result<Vec<u32>, TensorReceipt> {
    let Some(arr) = v.and_then(|v| v.as_array()) else {
        return Err(closed_miss("shape", "shape is not an array", "", "refused", "surreal"));
    };
    let mut shape = Vec::with_capacity(arr.len());
    for item in arr {
        let n = item.as_u64().or_else(|| item.as_i64().map(|i| i as u64)).unwrap_or(0);
        if n == 0 || n > u64::from(u32::MAX) {
            return Err(closed_miss("shape", "shape dimension is invalid", "", "refused", "surreal"));
        }
        shape.push(n as u32);
    }
    Ok(shape)
}

fn bytes_from_value(v: Option<&Value>) -> Result<Vec<u8>, TensorReceipt> {
    let Some(v) = v else {
        return Err(closed_miss(
            "key_tensor",
            "tensor bytes missing from surreal row",
            "",
            "refused",
            "surreal",
        ));
    };
    if let Some(arr) = v.as_array() {
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let n = item.as_u64().ok_or_else(|| {
                closed_miss("key_tensor", "tensor byte is not an integer", "", "refused", "surreal")
            })?;
            if n > 255 {
                return Err(closed_miss(
                    "key_tensor",
                    "tensor byte out of range",
                    "",
                    "refused",
                    "surreal",
                ));
            }
            out.push(n as u8);
        }
        return Ok(out);
    }
    if let Some(s) = v.as_str() {
        let hex = s
            .strip_prefix("b\"")
            .and_then(|rest| rest.strip_suffix('"'))
            .unwrap_or(s);
        return decode_hex(hex).ok_or_else(|| {
            closed_miss(
                "key_tensor",
                "tensor-bytes-undecodable",
                "",
                "refused",
                "surreal",
            )
        });
    }
    Err(closed_miss(
        "key_tensor",
        "tensor-bytes-undecodable",
        "",
        "refused",
        "surreal",
    ))
}

fn decode_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Some(out)
}

fn record_atom_id(v: Option<&Value>) -> Option<u64> {
    let v = v?;
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    let s = v.as_str()?;
    let rest = s.strip_prefix("atom:").unwrap_or(s);
    rest.parse::<u64>().ok()
}

fn rows_of(result: &Value) -> Vec<&Value> {
    match result {
        Value::Array(items) => items.iter().collect(),
        Value::Null => Vec::new(),
        other => vec![other],
    }
}

fn admitted_receipt(page: &AdmittedPage, durability: &str) -> TensorReceipt {
    TensorReceipt {
        status: "admitted".into(),
        reason: "tensor page admitted".into(),
        field: String::new(),
        prefill: "refused".into(),
        reused: false,
        durability: durability.into(),
        geometry_space: GEOMETRY_SPACE.into(),
        quotient_status: QUOTIENT_STATUS.into(),
        contract_digest: R12_CONTRACT_DIGEST.into(),
        model_digest_hex: hex_encode(&page.model_digest),
        key_digest_hex: hex_encode(&page.key_digest),
        value_digest_hex: hex_encode(&page.value_digest),
        address_contract_digest_hex: hex_encode(&page.address_contract_digest),
        tppe: page.tppe.clone(),
        key_tensor: Vec::new(),
        value_tensor: Vec::new(),
    }
}

fn closed_miss(field: &str, reason: &str, tppe: &str, prefill: &str, durability: &str) -> TensorReceipt {
    let status = if field == "surreal" || field == "frame" {
        "fail-closed"
    } else {
        "closed-miss"
    };
    TensorReceipt {
        status: status.into(),
        reason: reason.into(),
        field: field.into(),
        prefill: prefill.into(),
        reused: false,
        durability: durability.into(),
        geometry_space: GEOMETRY_SPACE.into(),
        quotient_status: QUOTIENT_STATUS.into(),
        contract_digest: R12_CONTRACT_DIGEST.into(),
        model_digest_hex: String::new(),
        key_digest_hex: String::new(),
        value_digest_hex: String::new(),
        address_contract_digest_hex: String::new(),
        tppe: tppe.into(),
        key_tensor: Vec::new(),
        value_tensor: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn model() -> [u8; 32] {
        let mut m = [0u8; 32];
        m[0] = 0x11;
        m[31] = 0x22;
        m
    }

    fn write_req() -> WriteRequest {
        WriteRequest {
            atom_id: 7,
            address_revision: 1,
            h3_lorentz: [1.0, 0.0, 0.0, 0.0],
            t2_phase: [0.25, -0.5],
            dtype: TensorDtype::F32,
            layout: ADMITTED_LAYOUT.into(),
            positional_encoding: ADMITTED_POSITIONAL.into(),
            shape: vec![2, 4],
            tppe: "toroidal-phase-finite-feature-v1".into(),
            model_digest: model(),
            key_tensor: (0u8..32).collect(),
            value_tensor: (32u8..64).collect(),
        }
    }

    fn fetch_of(page: &AdmittedPage, req: &WriteRequest) -> FetchRequest {
        FetchRequest {
            atom_id: req.atom_id,
            address_revision: req.address_revision,
            h3_lorentz: req.h3_lorentz,
            t2_phase: req.t2_phase,
            dtype: req.dtype,
            layout: req.layout.clone(),
            positional_encoding: req.positional_encoding.clone(),
            shape: req.shape.clone(),
            tppe: req.tppe.clone(),
            model_digest: req.model_digest,
            key_digest: page.key_digest,
            value_digest: page.value_digest,
        }
    }

    struct Spy {
        mode: SurrealAuthority,
        calls: RefCell<Vec<String>>,
        rows: RefCell<Vec<Value>>,
    }

    impl TensorSql for Spy {
        fn authority(&self) -> SurrealAuthority {
            self.mode
        }
        fn query(&self, surrealql: &str) -> Result<Vec<Value>, String> {
            self.calls.borrow_mut().push(surrealql.to_string());
            if surrealql.contains("SELECT") {
                Ok(vec![Value::Array(self.rows.borrow().clone())])
            } else {
                Ok(vec![Value::Null])
            }
        }
    }

    #[test]
    fn write_fetch_reuses_without_reprefill() {
        let req = write_req();
        let mut path = TensorHotPath::new();
        let admitted = path.admit(req.clone());
        assert_eq!(admitted.status, "admitted");
        assert_eq!(admitted.quotient_status, QUOTIENT_STATUS);
        assert_eq!(admitted.contract_digest, R12_CONTRACT_DIGEST);
        assert_eq!(admitted.geometry_space, GEOMETRY_SPACE);
        assert_ne!(admitted.quotient_status, "certified");
        let page = prepare_write(&req).unwrap();
        let got = path.fetch(&fetch_of(&page, &req));
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert!(got.reused);
        assert_eq!(got.key_tensor, req.key_tensor);
        assert_eq!(got.value_tensor, req.value_tensor);
        assert_eq!(got.durability, "ephemeral-hot-path");
    }

    #[test]
    fn binary_frame_roundtrip_reuses() {
        let req = write_req();
        let mut path = TensorHotPath::new();
        let write_frame = encode_write(&req);
        assert_eq!(&write_frame[0..4], b"HMKV");
        let ack = decode_receipt(&dispatch_hot(&mut path, &write_frame)).unwrap();
        assert_eq!(ack.status, "admitted");
        let page = prepare_write(&req).unwrap();
        let fetch = fetch_of(&page, &req);
        let got = decode_receipt(&dispatch_hot(&mut path, &encode_fetch(&fetch))).unwrap();
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert!(got.reused);
        assert_eq!(got.key_tensor, req.key_tensor);
        assert_eq!(got.value_tensor, req.value_tensor);
        assert_eq!(got.quotient_status, "opaque");
        assert_eq!(got.contract_digest, "UNSPECIFIED-R12-ACTION");
    }

    #[test]
    fn each_gate_field_fails_closed() {
        let req = write_req();
        let mut path = TensorHotPath::new();
        path.admit(req.clone());
        let page = prepare_write(&req).unwrap();
        let base = fetch_of(&page, &req);

        let mut model_mismatch = base.clone();
        model_mismatch.model_digest[0] ^= 0xff;
        expect_closed(&path.fetch(&model_mismatch), "model_digest");

        let mut dtype_mismatch = base.clone();
        dtype_mismatch.dtype = TensorDtype::Bf16;
        expect_closed(&path.fetch(&dtype_mismatch), "dtype");

        let mut layout_mismatch = base.clone();
        layout_mismatch.layout = "column-major".into();
        expect_closed(&path.fetch(&layout_mismatch), "layout");

        let mut shape_mismatch = base.clone();
        shape_mismatch.shape = vec![4, 2];
        expect_closed(&path.fetch(&shape_mismatch), "shape");

        let mut tppe_mismatch = base.clone();
        tppe_mismatch.tppe = "other-tppe".into();
        expect_closed(&path.fetch(&tppe_mismatch), "toroidal_phase_encoder_version");

        let mut address_mismatch = base.clone();
        address_mismatch.t2_phase[0] += 0.1;
        expect_closed(&path.fetch(&address_mismatch), "address_contract_digest");

        let mut key_mismatch = base.clone();
        key_mismatch.key_digest[0] ^= 0xff;
        expect_closed(&path.fetch(&key_mismatch), "key_digest");

        let mut value_mismatch = base.clone();
        value_mismatch.value_digest[1] ^= 0xff;
        expect_closed(&path.fetch(&value_mismatch), "value_digest");

        let missing = path.fetch(&{
            let mut q = base.clone();
            q.atom_id = 99;
            q
        });
        assert_eq!(missing.status, "closed-miss");
        assert!(missing.key_tensor.is_empty());
        assert!(!missing.reused);
        assert_eq!(missing.prefill, "required");
    }

    fn expect_closed(rcpt: &TensorReceipt, field: &str) {
        assert_eq!(rcpt.status, "closed-miss", "{}", rcpt.reason);
        assert_eq!(rcpt.field, field);
        assert_eq!(rcpt.prefill, "refused");
        assert!(!rcpt.reused);
        assert!(rcpt.key_tensor.is_empty());
        assert!(rcpt.value_tensor.is_empty());
        assert_eq!(rcpt.quotient_status, "opaque");
    }

    #[test]
    fn rope_and_euclidean_are_inadmissible() {
        let mut path = TensorHotPath::new();
        for enc in ["rope", "RoPE", "euclidean", "Euclidean"] {
            let mut req = write_req();
            req.positional_encoding = enc.into();
            let rcpt = path.admit(req);
            assert_eq!(rcpt.status, "closed-miss");
            assert_eq!(rcpt.field, "positional_encoding");
            assert_eq!(rcpt.prefill, "refused");
        }
        assert!(path.pages.is_empty());
    }

    #[test]
    fn bf16_width_and_immutable_conflict() {
        let mut req = write_req();
        req.dtype = TensorDtype::Bf16;
        req.shape = vec![2, 4];
        req.key_tensor = vec![1u8; 16];
        req.value_tensor = vec![2u8; 16];
        let mut path = TensorHotPath::new();
        assert_eq!(path.admit(req.clone()).status, "admitted");
        req.key_tensor = vec![9u8; 16];
        let conflict = path.admit(req);
        assert_eq!(conflict.status, "closed-miss");
        assert_eq!(conflict.field, "key_digest");
        assert_eq!(conflict.prefill, "refused");
    }

    #[test]
    fn surreal_sql_is_bytes_not_json_and_shadow_does_not_query() {
        let page = prepare_write(&write_req()).unwrap();
        let sql = insert_page_sql(&page);
        assert!(sql.contains("transformer_kv_session"));
        assert!(sql.contains("transformer_kv_page"));
        assert!(sql.contains("key_tensor: b\""));
        assert!(sql.contains("value_tensor: b\""));
        assert!(sql.contains("h3_t2_r12_v1"));
        assert!(sql.contains("UNSPECIFIED-R12-ACTION"));
        assert!(sql.contains("opaque"));
        assert!(!sql.contains("[0,1,2,3"));
        let next = next_sql(&page, &page);
        assert!(next.contains("transformer_kv_next"));
        let shadow = Spy {
            mode: SurrealAuthority::Shadow,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(Vec::new()),
        };
        let rcpt = commit_page(&shadow, &page);
        assert_eq!(rcpt.status, "fail-closed");
        assert_eq!(rcpt.field, "surreal");
        assert!(shadow.calls.borrow().is_empty());
        let off = Spy {
            mode: SurrealAuthority::Off,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(Vec::new()),
        };
        assert_eq!(commit_page(&off, &page).status, "fail-closed");
        assert!(off.calls.borrow().is_empty());
    }

    #[test]
    fn authoritative_fetch_reuses_and_mismatch_fails_closed() {
        let req = write_req();
        let page = prepare_write(&req).unwrap();
        let row = row_from_page(&page);
        let sink = Spy {
            mode: SurrealAuthority::On,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(vec![row]),
        };
        let committed = commit_page(&sink, &page);
        assert_eq!(committed.status, "admitted");
        assert_eq!(committed.durability, "surreal");
        assert!(sink.calls.borrow()[0].contains("key_tensor: b\""));
        let got = fetch_authoritative(&sink, &fetch_of(&page, &req));
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert!(got.reused);
        assert_eq!(got.key_tensor, req.key_tensor);
        assert_eq!(got.value_tensor, req.value_tensor);
        assert_eq!(got.durability, "surreal");
        let mut bad = fetch_of(&page, &req);
        bad.model_digest[3] ^= 0x5a;
        let miss = fetch_authoritative(&sink, &bad);
        assert_eq!(miss.status, "closed-miss");
        assert_eq!(miss.field, "model_digest");
        assert!(miss.key_tensor.is_empty());
        assert_eq!(miss.prefill, "refused");
    }

    #[test]
    fn stored_digest_mismatch_fails_closed() {
        let page = prepare_write(&write_req()).unwrap();
        let mut row = row_from_page(&page);
        row["key_digest"] = Value::String("00".repeat(32));
        let err = page_from_row(&row).unwrap_err();
        assert_eq!(err.status, "closed-miss");
        assert_eq!(err.field, "key_digest");
        assert_eq!(err.prefill, "refused");
        assert!(err.key_tensor.is_empty());
    }

    #[test]
    fn session_fail_closed_when_surreal_is_not_on() {
        let sink = Spy {
            mode: SurrealAuthority::Shadow,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(Vec::new()),
        };
        let input = Vec::new();
        let mut output = Vec::new();
        let code = run_tensor_session(&mut input.as_slice(), &mut output, &sink);
        assert_eq!(code, 2);
        assert!(sink.calls.borrow().is_empty());
        assert!(!output.is_empty());
    }

    #[test]
    fn checkpoint_link_stays_opaque() {
        let req = write_req();
        let mut other = write_req();
        other.atom_id = 8;
        other.key_tensor = vec![7u8; 32];
        other.value_tensor = vec![8u8; 32];
        let mut path = TensorHotPath::new();
        path.admit(req.clone());
        path.admit(other.clone());
        let a = prepare_write(&req).unwrap();
        let b = prepare_write(&other).unwrap();
        let rcpt = path.record_next(&a, &b);
        assert_eq!(rcpt.status, "checkpoint");
        assert!(rcpt.reason.contains("transformer_kv_next"));
        assert_eq!(rcpt.quotient_status, "opaque");
        assert_eq!(rcpt.contract_digest, "UNSPECIFIED-R12-ACTION");
        assert!(!rcpt.reason.to_lowercase().contains("certified"));
    }
}
