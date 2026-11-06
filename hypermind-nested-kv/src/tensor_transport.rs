//! Native binary tensor transport aligned to the live Surreal schema
//! `schema/0004_transformer_kv_tensor_pages.surql`.
//!
//! That file is already applied on `hypermind` / `core`. This module does
//! not apply it and does not invent a replacement migration.
//!
//! K and V bytes move on an `HMKV` frame (version 2). They are not MCP JSON.
//! Score, rank, `acosh`, and `shell_project` stay `f64` elsewhere.
//!
//! Durable writes use the existing HTTP `/sql` client. `geometry_space` is
//! the record `geometry_space:h3_t2_r12_v1`. `quotient_status = opaque` and
//! `contract_digest = UNSPECIFIED-R12-ACTION` are receipt labels: the live
//! page and session tables do not have those columns. `r_12` is not certified.

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const GEOMETRY_RECORD: &str = "geometry_space:h3_t2_r12_v1";
pub const QUOTIENT_STATUS: &str = "opaque";
pub const R12_CONTRACT_DIGEST: &str = "UNSPECIFIED-R12-ACTION";
pub const GEOMETRY_VERSION: &str = "H3xT2-Lorentz-canonical-v2";
pub const ADMITTED_POSITIONAL: &str = "toroidal-phase";
pub const FRAME_MAGIC: &[u8; 4] = b"HMKV";
pub const FRAME_VERSION: u16 = 2;

const MAX_TENSOR_BYTES: usize = 16 * 1024 * 1024;
const QK: usize = 32;
const Q4_0_BLOCK: usize = 18;
const Q8_0_BLOCK: usize = 34;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TensorDtype {
    F16,
    Bf16,
    F32,
    Q8_0,
    Q4_0,
}

impl TensorDtype {
    pub fn as_str(self) -> &'static str {
        match self {
            TensorDtype::F16 => "f16",
            TensorDtype::Bf16 => "bf16",
            TensorDtype::F32 => "f32",
            TensorDtype::Q8_0 => "q8_0",
            TensorDtype::Q4_0 => "q4_0",
        }
    }

    fn code(self) -> u8 {
        match self {
            TensorDtype::F16 => 1,
            TensorDtype::Bf16 => 2,
            TensorDtype::F32 => 3,
            TensorDtype::Q8_0 => 4,
            TensorDtype::Q4_0 => 5,
        }
    }

    fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(TensorDtype::F16),
            2 => Some(TensorDtype::Bf16),
            3 => Some(TensorDtype::F32),
            4 => Some(TensorDtype::Q8_0),
            5 => Some(TensorDtype::Q4_0),
            _ => None,
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "f16" => Some(TensorDtype::F16),
            "bf16" => Some(TensorDtype::Bf16),
            "f32" => Some(TensorDtype::F32),
            "q8_0" => Some(TensorDtype::Q8_0),
            "q4_0" => Some(TensorDtype::Q4_0),
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
    pub model_id: String,
    pub model_revision: String,
    pub model_digest: [u8; 32],
    pub request_digest: [u8; 32],
    pub layer_count: u32,
    pub kv_head_count: u32,
    pub head_dim: u32,
    pub tppe: String,
    pub address_id: u64,
    pub trajectory_atoms: Vec<u64>,
    pub layer: u32,
    pub kv_head: u32,
    pub token_start: u32,
    pub token_end: u32,
    pub revision: u32,
    pub dtype: TensorDtype,
    pub layout: String,
    pub positional_encoding: String,
    pub tensor_shape: [u32; 2],
    pub key_tensor: Vec<u8>,
    pub value_tensor: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct FetchRequest {
    pub model_digest: [u8; 32],
    pub request_digest: [u8; 32],
    pub tppe: String,
    pub address_id: u64,
    pub address_contract_digest: [u8; 32],
    pub layer: u32,
    pub kv_head: u32,
    pub token_start: u32,
    pub token_end: u32,
    pub revision: u32,
    pub dtype: TensorDtype,
    pub layout: String,
    pub positional_encoding: String,
    pub tensor_shape: [u32; 2],
    pub key_digest: [u8; 32],
    pub value_digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct AdmittedPage {
    pub model_id: String,
    pub model_revision: String,
    pub model_digest: [u8; 32],
    pub request_digest: [u8; 32],
    pub layer_count: u32,
    pub kv_head_count: u32,
    pub head_dim: u32,
    pub tppe: String,
    pub address_id: u64,
    pub trajectory_atoms: Vec<u64>,
    pub address_contract_digest: [u8; 32],
    pub layer: u32,
    pub kv_head: u32,
    pub token_start: u32,
    pub token_end: u32,
    pub revision: u32,
    pub dtype: TensorDtype,
    pub layout: String,
    pub tensor_shape: [u32; 2],
    pub key_digest: [u8; 32],
    pub value_digest: [u8; 32],
    pub key_tensor: Vec<u8>,
    pub value_tensor: Vec<u8>,
    pub session_id: String,
    pub page_id: String,
    pub receipt_digest: String,
}

#[derive(Clone, Debug)]
pub struct TensorReceipt {
    pub status: String,
    pub reason: String,
    pub field: String,
    pub prefill: String,
    pub reused: bool,
    pub durability: String,
    pub geometry_record: String,
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

#[derive(Clone, Debug, Default)]
pub struct TensorHotPath {
    pages: BTreeMap<u64, Vec<AdmittedPage>>,
    next_links: Vec<(String, String)>,
}

impl TensorHotPath {
    pub fn new() -> Self {
        Self::default()
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
            Ok(slot) => slot,
            Err(r) => return r,
        };
        let Some(candidates) = self.pages.get(&req.address_id) else {
            return absent_receipt(&wanted, "ephemeral-hot-path");
        };
        match_candidates(candidates, &wanted, "ephemeral-hot-path")
    }

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
            "RELATE transformer_kv_next {} -> {}; quotient_status={QUOTIENT_STATUS}; contract_digest={R12_CONTRACT_DIGEST}",
            from.page_id, to.page_id
        );
        rcpt.reused = false;
        rcpt.prefill = "refused".into();
        rcpt
    }

    fn insert_page(&mut self, page: AdmittedPage, durability: &str) -> TensorReceipt {
        let slot = self.pages.entry(page.address_id).or_default();
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
                "page identity already exists with a different payload; revision is part of the unique key",
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
    if req.model_id.is_empty() || req.model_revision.is_empty() || req.tppe.is_empty() {
        return Err(closed_miss(
            "model_id",
            "model_id, model_revision, and TPPE are required",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.address_id == 0 {
        return Err(closed_miss(
            "address",
            "page requires an atom_address id",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.layer_count == 0 || req.kv_head_count == 0 || req.head_dim == 0 {
        return Err(closed_miss(
            "layer_count",
            "session layer_count, kv_head_count, and head_dim must be > 0",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.revision == 0 {
        return Err(closed_miss(
            "revision",
            "page revision must be >= 1",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.token_end == 0 || req.token_end <= req.token_start {
        return Err(closed_miss(
            "token_end",
            "token interval must be non-empty (token_end > token_start, token_end > 0)",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    if req.layer >= req.layer_count || req.kv_head >= req.kv_head_count {
        return Err(closed_miss(
            "layer",
            "layer and kv_head must sit inside the session counts",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    let nbytes = expected_nbytes(req.tensor_shape, req.dtype).map_err(|reason| {
        closed_miss("tensor_shape", reason, &req.tppe, "refused", "none")
    })?;
    if req.key_tensor.len() != nbytes || req.value_tensor.len() != nbytes {
        return Err(closed_miss(
            "tensor_shape",
            "tensor byte length does not match tensor_shape and dtype",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    let address_contract_digest = address_contract_digest(&req.model_digest, &req.tppe);
    let key_digest = sha256(&req.key_tensor);
    let value_digest = sha256(&req.value_tensor);
    let session_id = session_id(&req.model_digest, &req.request_digest);
    let page_id = page_id(
        &session_id,
        req.layer,
        req.kv_head,
        req.token_start,
        req.token_end,
        req.revision,
    );
    let receipt_digest = receipt_digest(&page_id, &key_digest, &value_digest);
    Ok(AdmittedPage {
        model_id: req.model_id.clone(),
        model_revision: req.model_revision.clone(),
        model_digest: req.model_digest,
        request_digest: req.request_digest,
        layer_count: req.layer_count,
        kv_head_count: req.kv_head_count,
        head_dim: req.head_dim,
        tppe: req.tppe.clone(),
        address_id: req.address_id,
        trajectory_atoms: req.trajectory_atoms.clone(),
        address_contract_digest,
        layer: req.layer,
        kv_head: req.kv_head,
        token_start: req.token_start,
        token_end: req.token_end,
        revision: req.revision,
        dtype: req.dtype,
        layout: req.layout.clone(),
        tensor_shape: req.tensor_shape,
        key_digest,
        value_digest,
        key_tensor: req.key_tensor.clone(),
        value_tensor: req.value_tensor.clone(),
        session_id,
        page_id,
        receipt_digest,
    })
}

struct FetchSlot {
    model_digest: [u8; 32],
    tppe: String,
    address_id: u64,
    address_contract_digest: [u8; 32],
    layer: u32,
    kv_head: u32,
    token_start: u32,
    token_end: u32,
    revision: u32,
    dtype: TensorDtype,
    layout: String,
    tensor_shape: [u32; 2],
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
    if req.revision == 0 || req.token_end == 0 {
        return Err(closed_miss(
            "revision",
            "revision and token_end must satisfy the live schema",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    expected_nbytes(req.tensor_shape, req.dtype).map_err(|reason| {
        closed_miss("tensor_shape", reason, &req.tppe, "refused", "none")
    })?;
    let canonical = address_contract_digest(&req.model_digest, &req.tppe);
    if req.address_contract_digest != canonical {
        return Err(closed_miss(
            "address_contract_digest",
            "exact-match admission failed",
            &req.tppe,
            "refused",
            "none",
        ));
    }
    Ok(FetchSlot {
        model_digest: req.model_digest,
        tppe: req.tppe.clone(),
        address_id: req.address_id,
        address_contract_digest: req.address_contract_digest,
        layer: req.layer,
        kv_head: req.kv_head,
        token_start: req.token_start,
        token_end: req.token_end,
        revision: req.revision,
        dtype: req.dtype,
        layout: req.layout.clone(),
        tensor_shape: req.tensor_shape,
        key_digest: req.key_digest,
        value_digest: req.value_digest,
    })
}

fn match_candidates(candidates: &[AdmittedPage], wanted: &FetchSlot, durability: &str) -> TensorReceipt {
    if candidates.is_empty() {
        return absent_receipt(wanted, durability);
    }
    let mut first_miss = None;
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
        return closed_miss(field, "exact-match admission failed", &wanted.tppe, "refused", durability);
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
    if page.tensor_shape != wanted.tensor_shape {
        return Some("tensor_shape");
    }
    if page.tppe != wanted.tppe {
        return Some("toroidal_phase_encoder_version");
    }
    if page.address_contract_digest != wanted.address_contract_digest {
        return Some("address_contract_digest");
    }
    if page.address_id != wanted.address_id {
        return Some("address");
    }
    if page.layer != wanted.layer {
        return Some("layer");
    }
    if page.kv_head != wanted.kv_head {
        return Some("kv_head");
    }
    if page.token_start != wanted.token_start {
        return Some("token_start");
    }
    if page.token_end != wanted.token_end {
        return Some("token_end");
    }
    if page.revision != wanted.revision {
        return Some("revision");
    }
    if page.key_digest != wanted.key_digest {
        return Some("key_digest");
    }
    if page.value_digest != wanted.value_digest {
        return Some("value_digest");
    }
    None
}

pub fn commit_page(sink: &impl TensorSql, page: &AdmittedPage) -> TensorReceipt {
    if let Some(rcpt) = authority_block(sink.authority()) {
        return rcpt;
    }
    match sink.query(&insert_page_sql(page)) {
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
    match sink.query(&next_sql(from, to)) {
        Ok(_) => {
            let mut rcpt = admitted_receipt(from, "surreal");
            rcpt.status = "checkpoint".into();
            rcpt.reason = format!(
                "RELATE transformer_kv_next {} -> {}; quotient_status={QUOTIENT_STATUS}",
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
        Ok(slot) => slot,
        Err(r) => return r,
    };
    if let Some(rcpt) = authority_block(sink.authority()) {
        return rcpt;
    }
    let results = match sink.query(&select_pages_sql(req.address_id)) {
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
                Ok(page) if page.address_id == req.address_id => pages.push(page),
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
    let traj = if page.trajectory_atoms.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            page.trajectory_atoms
                .iter()
                .map(|id| format!("atom:{id}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!(
        "CREATE transformer_kv_session:`{session}` CONTENT {{ workspace: workspace:hypermind, model_id: {model_id}, model_revision: {model_rev}, model_digest: {model}, request_digest: {request}, dtype: {dtype}, layer_count: {layers}, kv_head_count: {heads}, head_dim: {head_dim}, geometry_space: geometry_space:`h3_t2_r12_v1`, toroidal_phase_encoder_version: {tppe}, address_contract_digest: {addr}, active: true, created_at: time::now(), closed_at: NONE }}; CREATE transformer_kv_page:`{page_id}` CONTENT {{ workspace: workspace:hypermind, session: transformer_kv_session:`{session}`, layer: {layer}, kv_head: {kv_head}, token_start: {token_start}, token_end: {token_end}, tensor_shape: [{s0}, {s1}], dtype: {dtype}, layout: {layout}, key_tensor: b\"{kbytes}\", value_tensor: b\"{vbytes}\", key_digest: {kd}, value_digest: {vd}, key_bytes: {klen}, value_bytes: {vlen}, address: atom_address:{address}, trajectory_atoms: {traj}, receipt_digest: {receipt}, revision: {revision}, hits: 0, active: true, created_at: time::now(), last_accessed_at: NONE }};",
        session = page.session_id,
        page_id = page.page_id,
        model_id = sql_quote(&page.model_id),
        model_rev = sql_quote(&page.model_revision),
        model = sql_quote(&hex_encode(&page.model_digest)),
        request = sql_quote(&hex_encode(&page.request_digest)),
        dtype = sql_quote(page.dtype.as_str()),
        layers = page.layer_count,
        heads = page.kv_head_count,
        head_dim = page.head_dim,
        tppe = sql_quote(&page.tppe),
        addr = sql_quote(&hex_encode(&page.address_contract_digest)),
        layer = page.layer,
        kv_head = page.kv_head,
        token_start = page.token_start,
        token_end = page.token_end,
        s0 = page.tensor_shape[0],
        s1 = page.tensor_shape[1],
        layout = sql_quote(&page.layout),
        kbytes = hex_encode(&page.key_tensor),
        vbytes = hex_encode(&page.value_tensor),
        kd = sql_quote(&hex_encode(&page.key_digest)),
        vd = sql_quote(&hex_encode(&page.value_digest)),
        klen = page.key_tensor.len(),
        vlen = page.value_tensor.len(),
        address = page.address_id,
        traj = traj,
        receipt = sql_quote(&page.receipt_digest),
        revision = page.revision,
    )
}

pub fn select_pages_sql(address_id: u64) -> String {
    format!(
        "SELECT layer, kv_head, token_start, token_end, revision, tensor_shape, dtype, layout, key_tensor, value_tensor, key_digest, value_digest, key_bytes, value_bytes, address, trajectory_atoms, receipt_digest, session.model_id AS model_id, session.model_revision AS model_revision, session.model_digest AS model_digest, session.request_digest AS request_digest, session.layer_count AS layer_count, session.kv_head_count AS kv_head_count, session.head_dim AS head_dim, session.toroidal_phase_encoder_version AS toroidal_phase_encoder_version, session.address_contract_digest AS address_contract_digest, session.geometry_space AS geometry_space FROM transformer_kv_page WHERE address = atom_address:{address_id} AND active = true;"
    )
}

pub fn next_sql(from: &AdmittedPage, to: &AdmittedPage) -> String {
    let receipt = receipt_digest(
        &format!("{}->{}", from.page_id, to.page_id),
        &from.key_digest,
        &to.value_digest,
    );
    format!(
        "RELATE transformer_kv_page:`{from}`->transformer_kv_next->transformer_kv_page:`{to}` CONTENT {{ workspace: workspace:hypermind, receipt_digest: {receipt}, created_at: time::now() }};",
        from = from.page_id,
        to = to.page_id,
        receipt = sql_quote(&receipt),
    )
}

pub fn page_from_row(row: &Value) -> Result<AdmittedPage, TensorReceipt> {
    if !geometry_matches(row.get("geometry_space")) {
        return Err(closed_miss(
            "geometry_space",
            "page is not linked to geometry_space:h3_t2_r12_v1",
            "",
            "refused",
            "surreal",
        ));
    }
    let dtype = TensorDtype::parse(row.get("dtype").and_then(|v| v.as_str()).unwrap_or("")).ok_or_else(|| {
        closed_miss("dtype", "stored dtype is not one of f16, bf16, f32, q8_0, q4_0", "", "refused", "surreal")
    })?;
    let layout = row.get("layout").and_then(|v| v.as_str()).unwrap_or("").to_string();
    reject_layout(&layout)?;
    let tppe = row
        .get("toroidal_phase_encoder_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let tensor_shape = shape2_from_value(row.get("tensor_shape"))?;
    let key_tensor = bytes_from_value(row.get("key_tensor"))?;
    let value_tensor = bytes_from_value(row.get("value_tensor"))?;
    let nbytes = expected_nbytes(tensor_shape, dtype).map_err(|reason| {
        closed_miss("tensor_shape", reason, &tppe, "refused", "surreal")
    })?;
    if key_tensor.len() != nbytes || value_tensor.len() != nbytes {
        return Err(closed_miss(
            "tensor_shape",
            "stored tensor length does not match tensor_shape and dtype",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let key_bytes = json_u32(row.get("key_bytes")).unwrap_or(key_tensor.len() as u32);
    let value_bytes = json_u32(row.get("value_bytes")).unwrap_or(value_tensor.len() as u32);
    if key_bytes as usize != key_tensor.len() || value_bytes as usize != value_tensor.len() {
        return Err(closed_miss(
            "key_bytes",
            "key_bytes or value_bytes does not match the tensor payload",
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
    let request_digest = parse_digest_field(row, "request_digest")?;
    let stored_contract = parse_digest_field(row, "address_contract_digest")?;
    let canonical = address_contract_digest(&model_digest, &tppe);
    if stored_contract != canonical {
        return Err(closed_miss(
            "address_contract_digest",
            "stored address contract is not the opaque h3_t2_r12_v1 digest",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let address_id = record_id(row.get("address"), "atom_address").unwrap_or(0);
    if address_id == 0 {
        return Err(closed_miss(
            "address",
            "tensor page is not linked to an atom_address",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let layer = json_u32(row.get("layer")).unwrap_or(u32::MAX);
    let kv_head = json_u32(row.get("kv_head")).unwrap_or(u32::MAX);
    let token_start = json_u32(row.get("token_start")).unwrap_or(u32::MAX);
    let token_end = json_u32(row.get("token_end")).unwrap_or(0);
    let revision = json_u32(row.get("revision")).unwrap_or(0);
    if revision == 0 || token_end == 0 {
        return Err(closed_miss(
            "revision",
            "stored revision or token_end violates the live schema",
            &tppe,
            "refused",
            "surreal",
        ));
    }
    let session_id = session_id(&model_digest, &request_digest);
    let page_id = page_id(&session_id, layer, kv_head, token_start, token_end, revision);
    let receipt_digest = row
        .get("receipt_digest")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    Ok(AdmittedPage {
        model_id: row.get("model_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        model_revision: row
            .get("model_revision")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        model_digest,
        request_digest,
        layer_count: json_u32(row.get("layer_count")).unwrap_or(1),
        kv_head_count: json_u32(row.get("kv_head_count")).unwrap_or(1),
        head_dim: json_u32(row.get("head_dim")).unwrap_or(1),
        tppe,
        address_id,
        trajectory_atoms: trajectory_from_value(row.get("trajectory_atoms")),
        address_contract_digest: stored_contract,
        layer,
        kv_head,
        token_start,
        token_end,
        revision,
        dtype,
        layout,
        tensor_shape,
        key_digest,
        value_digest,
        key_tensor,
        value_tensor,
        session_id,
        page_id,
        receipt_digest,
    })
}

pub fn row_from_page(page: &AdmittedPage) -> Value {
    serde_json::json!({
        "model_id": page.model_id,
        "model_revision": page.model_revision,
        "model_digest": hex_encode(&page.model_digest),
        "request_digest": hex_encode(&page.request_digest),
        "layer_count": page.layer_count,
        "kv_head_count": page.kv_head_count,
        "head_dim": page.head_dim,
        "toroidal_phase_encoder_version": page.tppe,
        "address_contract_digest": hex_encode(&page.address_contract_digest),
        "geometry_space": GEOMETRY_RECORD,
        "layer": page.layer,
        "kv_head": page.kv_head,
        "token_start": page.token_start,
        "token_end": page.token_end,
        "revision": page.revision,
        "tensor_shape": page.tensor_shape,
        "dtype": page.dtype.as_str(),
        "layout": page.layout,
        "key_tensor": format!("b\"{}\"", hex_encode(&page.key_tensor)),
        "value_tensor": format!("b\"{}\"", hex_encode(&page.value_tensor)),
        "key_digest": hex_encode(&page.key_digest),
        "value_digest": hex_encode(&page.value_digest),
        "key_bytes": page.key_tensor.len(),
        "value_bytes": page.value_tensor.len(),
        "address": format!("atom_address:{}", page.address_id),
        "trajectory_atoms": page.trajectory_atoms.iter().map(|id| format!("atom:{id}")).collect::<Vec<_>>(),
        "receipt_digest": page.receipt_digest,
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
        let _ = write_framed(&mut writer, &encode_receipt(&rcpt));
        return 2;
    }
    let mut exit = 0;
    loop {
        let len = match read_u32(&mut reader) {
            Ok(Some(n)) => n as usize,
            Ok(None) => break,
            Err(_) => return 2,
        };
        if len > MAX_TENSOR_BYTES * 2 + 8192 {
            let frame = encode_receipt(&closed_miss("frame", "tensor frame exceeds limit", "", "refused", "none"));
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

pub fn demo_request(address_id: u64) -> WriteRequest {
    WriteRequest {
        model_id: "demo-model".into(),
        model_revision: "demo-rev".into(),
        model_digest: [0x11; 32],
        request_digest: [0x22; 32],
        layer_count: 1,
        kv_head_count: 1,
        head_dim: 4,
        tppe: "toroidal-phase-finite-feature-v1".into(),
        address_id,
        trajectory_atoms: vec![address_id],
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: 2,
        revision: 1,
        dtype: TensorDtype::F32,
        layout: "token_major".into(),
        positional_encoding: ADMITTED_POSITIONAL.into(),
        tensor_shape: [2, 4],
        key_tensor: (0u8..32).collect(),
        value_tensor: (32u8..64).collect(),
    }
}

pub fn write_demo_frames(address_id: u64, dir: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let req = demo_request(address_id);
    let page = prepare_write(&req).map_err(|r| r.reason)?;
    let fetch = fetch_of(&page, &req);
    let mut miss = fetch.clone();
    miss.model_digest[0] ^= 0xff;
    miss.address_contract_digest = address_contract_digest(&miss.model_digest, &miss.tppe);
    write_one_frame(&dir.join("write.bin"), &encode_write(&req))?;
    write_one_frame(&dir.join("fetch-hit.bin"), &encode_fetch(&fetch))?;
    write_one_frame(&dir.join("fetch-miss.bin"), &encode_fetch(&miss))?;
    Ok(())
}

pub fn show_receipts<R: std::io::Read, W: std::io::Write>(mut reader: R, mut writer: W) -> std::io::Result<()> {
    loop {
        let len = match read_u32(&mut reader)? {
            Some(n) => n as usize,
            None => break,
        };
        let mut frame = vec![0u8; len];
        reader.read_exact(&mut frame)?;
        match decode_receipt(&frame) {
            Ok(rcpt) => writeln!(
                writer,
                "status={} prefill={} reused={} field={} reason={} quotient_status={} contract_digest={} geometry={} key_len={} value_len={}",
                rcpt.status,
                rcpt.prefill,
                rcpt.reused,
                rcpt.field,
                rcpt.reason,
                rcpt.quotient_status,
                rcpt.contract_digest,
                rcpt.geometry_record,
                rcpt.key_tensor.len(),
                rcpt.value_tensor.len()
            )?,
            Err(rcpt) => writeln!(
                writer,
                "status={} prefill={} field={} reason={}",
                rcpt.status, rcpt.prefill, rcpt.field, rcpt.reason
            )?,
        }
    }
    Ok(())
}

fn write_one_frame(path: &std::path::Path, frame: &[u8]) -> Result<(), String> {
    let mut out = Vec::new();
    write_framed(&mut out, frame).map_err(|e| e.to_string())?;
    std::fs::write(path, out).map_err(|e| e.to_string())
}

enum Decoded {
    Write(WriteRequest),
    Fetch(FetchRequest),
}

pub fn encode_write(req: &WriteRequest) -> Vec<u8> {
    let mut out = Vec::new();
    push_header(&mut out, 1);
    push_common(
        &mut out,
        req.dtype,
        &req.layout,
        &req.positional_encoding,
        &req.model_id,
        &req.model_revision,
        &req.tppe,
        &req.model_digest,
        &req.request_digest,
        req.layer_count,
        req.kv_head_count,
        req.head_dim,
        req.address_id,
        req.layer,
        req.kv_head,
        req.token_start,
        req.token_end,
        req.revision,
        req.tensor_shape,
        &req.trajectory_atoms,
    );
    push_bytes(&mut out, &req.key_tensor);
    push_bytes(&mut out, &req.value_tensor);
    out
}

pub fn encode_fetch(req: &FetchRequest) -> Vec<u8> {
    let mut out = Vec::new();
    push_header(&mut out, 2);
    push_common(
        &mut out,
        req.dtype,
        &req.layout,
        &req.positional_encoding,
        "",
        "",
        &req.tppe,
        &req.model_digest,
        &req.request_digest,
        1,
        1,
        1,
        req.address_id,
        req.layer,
        req.kv_head,
        req.token_start,
        req.token_end,
        req.revision,
        req.tensor_shape,
        &[],
    );
    out.extend_from_slice(&req.address_contract_digest);
    out.extend_from_slice(&req.key_digest);
    out.extend_from_slice(&req.value_digest);
    out
}

fn push_common(
    out: &mut Vec<u8>,
    dtype: TensorDtype,
    layout: &str,
    positional: &str,
    model_id: &str,
    model_revision: &str,
    tppe: &str,
    model_digest: &[u8; 32],
    request_digest: &[u8; 32],
    layer_count: u32,
    kv_head_count: u32,
    head_dim: u32,
    address_id: u64,
    layer: u32,
    kv_head: u32,
    token_start: u32,
    token_end: u32,
    revision: u32,
    tensor_shape: [u32; 2],
    trajectory: &[u64],
) {
    out.push(dtype.code());
    push_str(out, layout);
    push_str(out, positional);
    push_str(out, model_id);
    push_str(out, model_revision);
    push_str(out, tppe);
    out.extend_from_slice(model_digest);
    out.extend_from_slice(request_digest);
    out.extend_from_slice(&layer_count.to_le_bytes());
    out.extend_from_slice(&kv_head_count.to_le_bytes());
    out.extend_from_slice(&head_dim.to_le_bytes());
    out.extend_from_slice(&address_id.to_le_bytes());
    out.extend_from_slice(&layer.to_le_bytes());
    out.extend_from_slice(&kv_head.to_le_bytes());
    out.extend_from_slice(&token_start.to_le_bytes());
    out.extend_from_slice(&token_end.to_le_bytes());
    out.extend_from_slice(&revision.to_le_bytes());
    out.extend_from_slice(&tensor_shape[0].to_le_bytes());
    out.extend_from_slice(&tensor_shape[1].to_le_bytes());
    let n = u16::try_from(trajectory.len()).unwrap_or(u16::MAX);
    out.extend_from_slice(&n.to_le_bytes());
    for id in trajectory.iter().take(n as usize) {
        out.extend_from_slice(&id.to_le_bytes());
    }
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
        geometry_record: GEOMETRY_RECORD.into(),
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
        closed_miss("dtype", "dtype is not f16, bf16, f32, q8_0, or q4_0", "", "refused", "none")
    })?;
    let layout = rd.str()?;
    let positional_encoding = rd.str()?;
    let model_id = rd.str()?;
    let model_revision = rd.str()?;
    let tppe = rd.str()?;
    let model_digest = rd.arr32()?;
    let request_digest = rd.arr32()?;
    let layer_count = rd.u32()?;
    let kv_head_count = rd.u32()?;
    let head_dim = rd.u32()?;
    let address_id = rd.u64()?;
    let layer = rd.u32()?;
    let kv_head = rd.u32()?;
    let token_start = rd.u32()?;
    let token_end = rd.u32()?;
    let revision = rd.u32()?;
    let tensor_shape = [rd.u32()?, rd.u32()?];
    let n = rd.u16()? as usize;
    let mut trajectory_atoms = Vec::with_capacity(n);
    for _ in 0..n {
        trajectory_atoms.push(rd.u64()?);
    }
    match op {
        1 => {
            let key_tensor = rd.bytes()?;
            let value_tensor = rd.bytes()?;
            Ok(Decoded::Write(WriteRequest {
                model_id,
                model_revision,
                model_digest,
                request_digest,
                layer_count,
                kv_head_count,
                head_dim,
                tppe,
                address_id,
                trajectory_atoms,
                layer,
                kv_head,
                token_start,
                token_end,
                revision,
                dtype,
                layout,
                positional_encoding,
                tensor_shape,
                key_tensor,
                value_tensor,
            }))
        }
        2 => Ok(Decoded::Fetch(FetchRequest {
            model_digest,
            request_digest,
            tppe,
            address_id,
            address_contract_digest: rd.arr32()?,
            layer,
            kv_head,
            token_start,
            token_end,
            revision,
            dtype,
            layout,
            positional_encoding,
            tensor_shape,
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
            0 => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "truncated length",
                ))
            }
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
    if layout == "token_major" || layout == "head_dim_major" {
        Ok(())
    } else {
        Err(closed_miss(
            "layout",
            "layout must be token_major or head_dim_major",
            "",
            "refused",
            "none",
        ))
    }
}

fn expected_nbytes(shape: [u32; 2], dtype: TensorDtype) -> Result<usize, &'static str> {
    if shape[0] == 0 || shape[1] == 0 {
        return Err("tensor_shape dimension is zero");
    }
    let elems = (shape[0] as usize)
        .checked_mul(shape[1] as usize)
        .ok_or("tensor_shape overflow")?;
    match dtype {
        TensorDtype::F32 => elems.checked_mul(4).ok_or("tensor_shape overflow"),
        TensorDtype::F16 | TensorDtype::Bf16 => elems.checked_mul(2).ok_or("tensor_shape overflow"),
        TensorDtype::Q8_0 => quant_bytes(elems, Q8_0_BLOCK),
        TensorDtype::Q4_0 => quant_bytes(elems, Q4_0_BLOCK),
    }
}

fn quant_bytes(elems: usize, block: usize) -> Result<usize, &'static str> {
    if elems % QK != 0 {
        return Err("q8_0 and q4_0 element count must be a multiple of 32");
    }
    (elems / QK).checked_mul(block).ok_or("tensor_shape overflow")
}

fn address_contract_digest(model: &[u8; 32], tppe: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"ADDR");
    h.update(GEOMETRY_RECORD.as_bytes());
    h.update(QUOTIENT_STATUS.as_bytes());
    h.update(R12_CONTRACT_DIGEST.as_bytes());
    h.update(GEOMETRY_VERSION.as_bytes());
    h.update(model);
    h.update(tppe.as_bytes());
    h.finalize().into()
}

fn session_id(model: &[u8; 32], request: &[u8; 32]) -> String {
    let mut h = Sha256::new();
    h.update(b"SESS");
    h.update(model);
    h.update(request);
    hex_encode(&h.finalize())
}

fn page_id(session: &str, layer: u32, kv_head: u32, token_start: u32, token_end: u32, revision: u32) -> String {
    let mut h = Sha256::new();
    h.update(b"PAGE");
    h.update(session.as_bytes());
    h.update(layer.to_le_bytes());
    h.update(kv_head.to_le_bytes());
    h.update(token_start.to_le_bytes());
    h.update(token_end.to_le_bytes());
    h.update(revision.to_le_bytes());
    hex_encode(&h.finalize())
}

fn receipt_digest(page_id: &str, key: &[u8; 32], value: &[u8; 32]) -> String {
    let mut h = Sha256::new();
    h.update(b"RCPT");
    h.update(page_id.as_bytes());
    h.update(key);
    h.update(value);
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
    let bytes = s.as_bytes();
    let mut out = [0u8; 32];
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
    parse_hex32(s).ok_or_else(|| closed_miss(field, "digest is not 32-byte hex", "", "refused", "surreal"))
}

fn shape2_from_value(v: Option<&Value>) -> Result<[u32; 2], TensorReceipt> {
    let Some(arr) = v.and_then(|v| v.as_array()) else {
        return Err(closed_miss("tensor_shape", "tensor_shape is not an array", "", "refused", "surreal"));
    };
    if arr.len() != 2 {
        return Err(closed_miss(
            "tensor_shape",
            "tensor_shape must be array<int, 2>",
            "",
            "refused",
            "surreal",
        ));
    }
    Ok([json_u32(Some(&arr[0])).unwrap_or(0), json_u32(Some(&arr[1])).unwrap_or(0)])
}

fn json_u32(v: Option<&Value>) -> Option<u32> {
    let v = v?;
    let n = v.as_u64().or_else(|| v.as_i64().and_then(|i| u64::try_from(i).ok()))?;
    u32::try_from(n).ok()
}

fn bytes_from_value(v: Option<&Value>) -> Result<Vec<u8>, TensorReceipt> {
    let Some(v) = v else {
        return Err(closed_miss("key_tensor", "tensor bytes missing from surreal row", "", "refused", "surreal"));
    };
    if let Some(arr) = v.as_array() {
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let n = item.as_u64().ok_or_else(|| {
                closed_miss("key_tensor", "tensor byte is not an integer", "", "refused", "surreal")
            })?;
            if n > 255 {
                return Err(closed_miss("key_tensor", "tensor byte out of range", "", "refused", "surreal"));
            }
            out.push(n as u8);
        }
        return Ok(out);
    }
    if let Some(s) = v.as_str() {
        let hex = s.strip_prefix("b\"").and_then(|rest| rest.strip_suffix('"')).unwrap_or(s);
        return decode_hex(hex).ok_or_else(|| {
            closed_miss("key_tensor", "tensor-bytes-undecodable", "", "refused", "surreal")
        });
    }
    Err(closed_miss("key_tensor", "tensor-bytes-undecodable", "", "refused", "surreal"))
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

fn geometry_matches(v: Option<&Value>) -> bool {
    match v {
        Some(Value::String(s)) => s == GEOMETRY_RECORD || s == "h3_t2_r12_v1",
        Some(Value::Object(map)) => {
            map.get("id").and_then(|id| id.as_str()) == Some("h3_t2_r12_v1")
        }
        _ => false,
    }
}

fn record_id(v: Option<&Value>, prefix: &str) -> Option<u64> {
    let v = v?;
    if let Some(s) = v.as_str() {
        let rest = s.strip_prefix(&format!("{prefix}:"))?;
        return rest.parse::<u64>().ok();
    }
    let id = v.get("id")?;
    if let Some(n) = id.as_u64() {
        return Some(n);
    }
    id.as_str().and_then(|s| s.parse::<u64>().ok())
}

fn trajectory_from_value(v: Option<&Value>) -> Vec<u64> {
    v.and_then(|v| v.as_array())
        .map(|items| items.iter().filter_map(|item| record_id(Some(item), "atom")).collect())
        .unwrap_or_default()
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
        geometry_record: GEOMETRY_RECORD.into(),
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

fn absent_receipt(wanted: &FetchSlot, durability: &str) -> TensorReceipt {
    closed_miss(
        "transformer_kv_page",
        "no tensor page for this atom_address; closed miss, not a silent miss",
        &wanted.tppe,
        "required",
        durability,
    )
}

fn closed_miss(field: &str, reason: &str, tppe: &str, prefill: &str, durability: &str) -> TensorReceipt {
    let status = if field == "surreal" || field == "frame" { "fail-closed" } else { "closed-miss" };
    TensorReceipt {
        status: status.into(),
        reason: reason.into(),
        field: field.into(),
        prefill: prefill.into(),
        reused: false,
        durability: durability.into(),
        geometry_record: GEOMETRY_RECORD.into(),
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

fn fetch_of(page: &AdmittedPage, req: &WriteRequest) -> FetchRequest {
    FetchRequest {
        model_digest: req.model_digest,
        request_digest: req.request_digest,
        tppe: req.tppe.clone(),
        address_id: req.address_id,
        address_contract_digest: page.address_contract_digest,
        layer: req.layer,
        kv_head: req.kv_head,
        token_start: req.token_start,
        token_end: req.token_end,
        revision: req.revision,
        dtype: req.dtype,
        layout: req.layout.clone(),
        positional_encoding: req.positional_encoding.clone(),
        tensor_shape: req.tensor_shape,
        key_digest: page.key_digest,
        value_digest: page.value_digest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn write_req() -> WriteRequest {
        demo_request(7)
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
        assert_eq!(admitted.geometry_record, GEOMETRY_RECORD);
        assert_ne!(admitted.quotient_status, "certified");
        let page = prepare_write(&req).unwrap();
        let got = path.fetch(&fetch_of(&page, &req));
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert!(got.reused);
        assert_eq!(got.key_tensor, req.key_tensor);
        assert_eq!(got.value_tensor, req.value_tensor);
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
        let got = decode_receipt(&dispatch_hot(&mut path, &encode_fetch(&fetch_of(&page, &req)))).unwrap();
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert_eq!(got.key_tensor, req.key_tensor);
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
        model_mismatch.address_contract_digest = address_contract_digest(&model_mismatch.model_digest, &model_mismatch.tppe);
        expect_closed(&path.fetch(&model_mismatch), "model_digest");

        let mut dtype_mismatch = base.clone();
        dtype_mismatch.dtype = TensorDtype::Bf16;
        expect_closed(&path.fetch(&dtype_mismatch), "dtype");

        let mut layout_mismatch = base.clone();
        layout_mismatch.layout = "head_dim_major".into();
        expect_closed(&path.fetch(&layout_mismatch), "layout");

        let mut shape_mismatch = base.clone();
        shape_mismatch.tensor_shape = [4, 2];
        expect_closed(&path.fetch(&shape_mismatch), "tensor_shape");

        let mut tppe_mismatch = base.clone();
        tppe_mismatch.tppe = "other-tppe".into();
        tppe_mismatch.address_contract_digest = address_contract_digest(&tppe_mismatch.model_digest, &tppe_mismatch.tppe);
        expect_closed(&path.fetch(&tppe_mismatch), "toroidal_phase_encoder_version");

        let mut address_mismatch = base.clone();
        address_mismatch.address_contract_digest[0] ^= 0xff;
        expect_closed(&path.fetch(&address_mismatch), "address_contract_digest");

        let mut key_mismatch = base.clone();
        key_mismatch.key_digest[0] ^= 0xff;
        expect_closed(&path.fetch(&key_mismatch), "key_digest");

        let mut value_mismatch = base;
        value_mismatch.value_digest[1] ^= 0xff;
        expect_closed(&path.fetch(&value_mismatch), "value_digest");
    }

    fn expect_closed(rcpt: &TensorReceipt, field: &str) {
        assert_eq!(rcpt.status, "closed-miss", "{} {}", field, rcpt.reason);
        assert_eq!(rcpt.field, field);
        assert_eq!(rcpt.prefill, "refused");
        assert!(!rcpt.reused);
        assert!(rcpt.key_tensor.is_empty());
        assert_eq!(rcpt.quotient_status, "opaque");
    }

    #[test]
    fn rope_and_euclidean_and_row_major_are_inadmissible() {
        let mut path = TensorHotPath::new();
        for enc in ["rope", "RoPE", "euclidean", "Euclidean"] {
            let mut req = write_req();
            req.positional_encoding = enc.into();
            let rcpt = path.admit(req);
            assert_eq!(rcpt.status, "closed-miss");
            assert_eq!(rcpt.field, "positional_encoding");
        }
        let mut row_major = write_req();
        row_major.layout = "row-major".into();
        let rcpt = path.admit(row_major);
        assert_eq!(rcpt.field, "layout");
        assert!(path.pages.is_empty());
    }

    #[test]
    fn quant_widths_and_identity_conflict() {
        let mut q4 = write_req();
        q4.dtype = TensorDtype::Q4_0;
        q4.tensor_shape = [32, 1];
        q4.token_end = 32;
        q4.key_tensor = vec![1u8; 18];
        q4.value_tensor = vec![2u8; 18];
        assert_eq!(prepare_write(&q4).unwrap().dtype, TensorDtype::Q4_0);
        q4.key_tensor = vec![1u8; 17];
        assert_eq!(prepare_write(&q4).unwrap_err().field, "tensor_shape");

        let mut q8 = write_req();
        q8.dtype = TensorDtype::Q8_0;
        q8.tensor_shape = [32, 1];
        q8.token_end = 32;
        q8.key_tensor = vec![3u8; 34];
        q8.value_tensor = vec![4u8; 34];
        assert_eq!(prepare_write(&q8).unwrap().key_tensor.len(), 34);

        let mut req = write_req();
        req.dtype = TensorDtype::Bf16;
        req.key_tensor = vec![1u8; 16];
        req.value_tensor = vec![2u8; 16];
        let mut path = TensorHotPath::new();
        assert_eq!(path.admit(req.clone()).status, "admitted");
        req.key_tensor = vec![9u8; 16];
        let conflict = path.admit(req);
        assert_eq!(conflict.status, "closed-miss");
        assert_eq!(conflict.field, "key_digest");
    }

    #[test]
    fn surreal_sql_matches_live_0004_columns() {
        let page = prepare_write(&write_req()).unwrap();
        let sql = insert_page_sql(&page);
        assert!(sql.contains("transformer_kv_session"));
        assert!(sql.contains("transformer_kv_page"));
        assert!(sql.contains("tensor_shape: [2, 4]"));
        assert!(sql.contains("layout: \"token_major\""));
        assert!(sql.contains("key_tensor: b\""));
        assert!(sql.contains("value_tensor: b\""));
        assert!(sql.contains("geometry_space: geometry_space:`h3_t2_r12_v1`"));
        assert!(sql.contains("address: atom_address:7"));
        assert!(sql.contains("address_contract_digest:"));
        assert!(sql.contains("toroidal_phase_encoder_version:"));
        assert!(sql.contains("model_digest:"));
        assert!(sql.contains("key_digest:"));
        assert!(sql.contains("value_digest:"));
        assert!(!sql.contains("row-major"));
        assert!(!sql.contains("quotient_status"));
        assert!(!sql.contains(" atom:"));
        let next = next_sql(&page, &page);
        assert!(next.contains("RELATE transformer_kv_page:"));
        assert!(next.contains("->transformer_kv_next->"));
        let select = select_pages_sql(7);
        assert!(select.contains("WHERE address = atom_address:7"));
        assert!(!select.contains("FIELDS atom"));
    }

    #[test]
    fn shadow_does_not_query() {
        let page = prepare_write(&write_req()).unwrap();
        let shadow = Spy {
            mode: SurrealAuthority::Shadow,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(Vec::new()),
        };
        assert_eq!(commit_page(&shadow, &page).status, "fail-closed");
        assert!(shadow.calls.borrow().is_empty());
    }

    #[test]
    fn authoritative_fetch_reuses_and_mismatch_fails_closed() {
        let req = write_req();
        let page = prepare_write(&req).unwrap();
        let sink = Spy {
            mode: SurrealAuthority::On,
            calls: RefCell::new(Vec::new()),
            rows: RefCell::new(vec![row_from_page(&page)]),
        };
        assert_eq!(commit_page(&sink, &page).durability, "surreal");
        assert!(sink.calls.borrow()[0].contains("key_tensor: b\""));
        let got = fetch_authoritative(&sink, &fetch_of(&page, &req));
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        assert_eq!(got.key_tensor, req.key_tensor);
        let mut bad = fetch_of(&page, &req);
        bad.model_digest[3] ^= 0x5a;
        bad.address_contract_digest = address_contract_digest(&bad.model_digest, &bad.tppe);
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
        let code = run_tensor_session(input.as_slice(), &mut output, &sink);
        assert_eq!(code, 2);
        assert!(sink.calls.borrow().is_empty());
    }

    #[test]
    fn checkpoint_link_is_a_relation_and_stays_opaque() {
        let req = write_req();
        let mut other = write_req();
        other.layer_count = 2;
        other.layer = 1;
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
        assert!(!rcpt.reason.to_lowercase().contains("certified"));
    }

    #[test]
    fn demo_frames_hit_and_miss() {
        let dir = std::env::temp_dir().join(format!("hmkv-frames-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        write_demo_frames(7, &dir).unwrap();
        let mut path = TensorHotPath::new();
        let write = std::fs::read(dir.join("write.bin")).unwrap();
        let hit = std::fs::read(dir.join("fetch-hit.bin")).unwrap();
        let miss = std::fs::read(dir.join("fetch-miss.bin")).unwrap();
        let admitted = decode_receipt(&dispatch_hot(&mut path, &frame_body(&write))).unwrap();
        assert_eq!(admitted.status, "admitted");
        let got = decode_receipt(&dispatch_hot(&mut path, &frame_body(&hit))).unwrap();
        assert_eq!(got.status, "hit");
        assert_eq!(got.prefill, "skipped");
        let bad = decode_receipt(&dispatch_hot(&mut path, &frame_body(&miss))).unwrap();
        assert_eq!(bad.status, "closed-miss");
        assert_eq!(bad.field, "model_digest");
        assert_eq!(bad.prefill, "refused");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn frame_body(framed: &[u8]) -> Vec<u8> {
        let n = u32::from_le_bytes(framed[0..4].try_into().unwrap()) as usize;
        framed[4..4 + n].to_vec()
    }
}
