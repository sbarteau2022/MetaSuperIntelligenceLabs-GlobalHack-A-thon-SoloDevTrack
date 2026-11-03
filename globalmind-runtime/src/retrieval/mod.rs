//! Minimal semantic-retrieval read path (Queue 2, ablation 1 enabler).
//!
//! Reads persisted atom coordinates from an isolated SurrealDB, ranks an
//! H3xT2 neighborhood client-side, and emits a typed binary recall bundle
//! (`GMRC` v1) for the forward harness. Nothing here executes a model.
//!
//! Boundaries: the private HTTP adapter carries only coordinates, ids, and
//! digests (control plane, same category as the store adapter); the recall
//! bundle crossing toward model I/O is binary-only. Retrieval never writes.
//! Candidate budget overruns fail closed (no silent truncation). Scoring
//! ties break by atom id, so rankings are deterministic.
//!
//! Metrics: [`RetrievalMetric::Hyperbolic`] (M-1 cost form d_H3 + 0.25*d_T2^2,
//! delegated to the verified Whitehead primitives) and [`RetrievalMetric::Flat`] (Euclidean
//! control, kept so Queue 2 ablation 3 needs no new code).

use crate::whitehead::{hyperbolic_distance, torus_distance_sq};
use crate::{Digest, RecordId};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub mod scoped;

pub const RECALL_MAGIC: [u8; 4] = *b"GMRC";
pub const RECALL_VERSION: u16 = 1;
pub const DEFAULT_CANDIDATE_BUDGET: u32 = 4096;
/// Schedule-v1 T2 weight in the combined hyperbolic score (M-1 cost form).
pub const T2_WEIGHT: f64 = 0.25;
/// Lorentz on-sheet tolerance: |<x,x>_L + 1| must be within this.
pub const ONSHEET_TOLERANCE: f64 = 1e-6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalMetric {
    Hyperbolic,
    Flat,
}

impl RetrievalMetric {
    pub fn code(self) -> u8 {
        match self {
            RetrievalMetric::Hyperbolic => 1,
            RetrievalMetric::Flat => 2,
        }
    }

    pub fn from_code(code: u8) -> Result<Self, String> {
        match code {
            1 => Ok(RetrievalMetric::Hyperbolic),
            2 => Ok(RetrievalMetric::Flat),
            _ => Err(format!("unknown retrieval metric code {code}")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            RetrievalMetric::Hyperbolic => "hyperbolic_h3xt2_v1",
            RetrievalMetric::Flat => "flat_euclidean_v1",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RetrievalConfig {
    pub endpoint: String,
    pub namespace: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub workspace: RecordId,
    pub candidate_budget: u32,
    pub metric: RetrievalMetric,
}

impl RetrievalConfig {
    pub fn from_env(workspace: RecordId, metric: RetrievalMetric) -> Result<Self, String> {
        let database =
            std::env::var("GLOBALMIND_SURREAL_DB").unwrap_or_else(|_| "globalmind_dev".into());
        let allow_core =
            std::env::var("GLOBALMIND_ALLOW_QUARANTINED_CORE").is_ok_and(|value| value == "1");
        if database == "core" && !allow_core {
            return Err(
                "hypermind/core is quarantined; use an isolated database or set the explicit override"
                    .into(),
            );
        }
        Ok(Self {
            endpoint: std::env::var("GLOBALMIND_SURREAL_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000".into()),
            namespace: std::env::var("GLOBALMIND_SURREAL_NS")
                .unwrap_or_else(|_| "hypermind".into()),
            database,
            username: std::env::var("GLOBALMIND_SURREAL_USER")
                .map_err(|_| "GLOBALMIND_SURREAL_USER is required".to_string())?,
            password: std::env::var("GLOBALMIND_SURREAL_PASS")
                .map_err(|_| "GLOBALMIND_SURREAL_PASS is required".to_string())?,
            workspace,
            candidate_budget: DEFAULT_CANDIDATE_BUDGET,
            metric,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.candidate_budget == 0 {
            return Err("candidate budget must be positive".into());
        }
        parse_endpoint(&self.endpoint).map(|_| ())
    }
}

/// One ranked neighborhood member with its provenance.
#[derive(Clone, Debug)]
pub struct ScoredAtom {
    pub atom: RecordId,
    pub address: RecordId,
    pub content_digest: Digest,
    pub salience: f64,
    pub domain: String,
    pub h3_lorentz: [f64; 4],
    pub t2_phase: [f64; 2],
    pub score: f64,
    pub rank: u32,
}

/// Receipted recall bundle: ranked atoms plus the query binding.
#[derive(Clone, Debug)]
pub struct RecallBundle {
    pub workspace: RecordId,
    pub query_digest: Digest,
    pub query_provenance: String,
    pub metric: RetrievalMetric,
    pub candidates_scanned: u32,
    pub members: Vec<ScoredAtom>,
    pub ranking_digest: Digest,
}

fn validate_h3(point: &[f64; 4], what: &str) -> Result<(), String> {
    if !point.iter().all(|v| v.is_finite()) || point[0] <= 0.0 {
        return Err(format!("{what} H3 point is non-finite or past-sheet"));
    }
    if (crate::whitehead::lorentz_inner(*point, *point) - 1.0).abs() > ONSHEET_TOLERANCE {
        return Err(format!("{what} H3 point is off the hyperboloid sheet"));
    }
    Ok(())
}

fn validate_t2(phase: &[f64; 2], what: &str) -> Result<(), String> {
    if !phase.iter().all(|v| v.is_finite()) {
        return Err(format!("{what} T2 phase is non-finite"));
    }
    Ok(())
}

/// Hyperbolic H3 distance, delegated to the verified Whitehead primitive
/// (cancellation-free near x == y) after fail-closed validation.
pub fn h3_distance(a: &[f64; 4], b: &[f64; 4]) -> Result<f64, String> {
    validate_h3(a, "query")?;
    validate_h3(b, "candidate")?;
    Ok(hyperbolic_distance(*a, *b))
}

/// Circular T2 distance with wraparound per component.
pub fn t2_distance(a: &[f64; 2], b: &[f64; 2]) -> Result<f64, String> {
    validate_t2(a, "query")?;
    validate_t2(b, "candidate")?;
    let mut sum = 0.0;
    for i in 0..2 {
        let wrapped = (a[i] - b[i]).sin().atan2((a[i] - b[i]).cos());
        sum += wrapped * wrapped;
    }
    Ok(sum.sqrt())
}

/// Schedule-v1 combined score, following the codebase M-1 cost-form
/// convention: d_H3 + T2_WEIGHT * d_T2^2 with T2_WEIGHT = 0.25.
pub fn hyperbolic_score(
    query_h3: &[f64; 4],
    query_t2: &[f64; 2],
    atom_h3: &[f64; 4],
    atom_t2: &[f64; 2],
) -> Result<f64, String> {
    validate_h3(query_h3, "query")?;
    validate_h3(atom_h3, "candidate")?;
    validate_t2(query_t2, "query")?;
    validate_t2(atom_t2, "candidate")?;
    Ok(hyperbolic_distance(*query_h3, *atom_h3)
        + T2_WEIGHT * torus_distance_sq(*query_t2, *atom_t2))
}

/// Flat Euclidean control over concatenated [H3(4), T2(2)] coordinates.
pub fn flat_score(
    query_h3: &[f64; 4],
    query_t2: &[f64; 2],
    atom_h3: &[f64; 4],
    atom_t2: &[f64; 2],
) -> Result<f64, String> {
    for (point, what) in [(query_h3, "query"), (atom_h3, "candidate")] {
        if !point.iter().all(|v| v.is_finite()) {
            return Err(format!("{what} coordinates are non-finite"));
        }
    }
    validate_t2(query_t2, "query")?;
    validate_t2(atom_t2, "candidate")?;
    let mut sum = 0.0;
    for i in 0..4 {
        sum += (query_h3[i] - atom_h3[i]).powi(2);
    }
    for i in 0..2 {
        sum += (query_t2[i] - atom_t2[i]).powi(2);
    }
    Ok(sum.sqrt())
}

pub fn score(
    metric: RetrievalMetric,
    query_h3: &[f64; 4],
    query_t2: &[f64; 2],
    atom_h3: &[f64; 4],
    atom_t2: &[f64; 2],
) -> Result<f64, String> {
    match metric {
        RetrievalMetric::Hyperbolic => hyperbolic_score(query_h3, query_t2, atom_h3, atom_t2),
        RetrievalMetric::Flat => flat_score(query_h3, query_t2, atom_h3, atom_t2),
    }
}

fn parse_endpoint(endpoint: &str) -> Result<(String, u16), String> {
    let bare = endpoint
        .strip_prefix("http://")
        .or_else(|| endpoint.strip_prefix("https://"))
        .ok_or_else(|| format!("endpoint must be http(s): {endpoint}"))?;
    let (host, port) = bare
        .split_once(':')
        .ok_or_else(|| format!("endpoint is missing a port: {endpoint}"))?;
    Ok((
        host.to_string(),
        port.parse::<u16>()
            .map_err(|_| format!("endpoint port is invalid: {endpoint}"))?,
    ))
}

fn base64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.as_bytes().chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Runs read-only SurrealQL and returns one result per statement (the USE
/// preamble excluded). Any ERR status is an error. Retrieval never writes.
fn control_sql(config: &RetrievalConfig, query: &str) -> Result<Vec<Value>, String> {
    let (host, port) = parse_endpoint(&config.endpoint)?;
    let address = format!("{host}:{port}");
    let mut stream =
        TcpStream::connect(&address).map_err(|error| format!("connect {address}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|error| error.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|error| error.to_string())?;
    let body = format!(
        "USE NS {} DB {}; {query}",
        config.namespace, config.database
    );
    let request = format!(
        "POST /sql HTTP/1.1\r\nHost: {address}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        base64(&format!("{}:{}", config.username, config.password)),
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("write retrieval query: {error}"))?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|error| format!("read retrieval response: {error}"))?;
    let text =
        String::from_utf8(response).map_err(|_| "retrieval response is not UTF-8".to_string())?;
    let body_text = text
        .split("\r\n\r\n")
        .nth(1)
        .ok_or_else(|| "retrieval response has no body".to_string())?;
    let parsed: Value = serde_json::from_str(body_text)
        .map_err(|error| format!("invalid retrieval response: {error}"))?;
    let statements = parsed
        .as_array()
        .ok_or_else(|| "retrieval response was not a statement array".to_string())?;
    let mut results = Vec::new();
    for (index, statement) in statements.iter().enumerate() {
        if index == 0 {
            continue;
        }
        if statement.get("status").and_then(Value::as_str) == Some("ERR") {
            let result = statement.get("result").cloned().unwrap_or(Value::Null);
            return Err(format!("retrieval statement {index} failed: {result}"));
        }
        results.push(statement.get("result").cloned().unwrap_or(Value::Null));
    }
    Ok(results)
}

fn record_id_of(row: &Value, field: &str) -> Result<RecordId, String> {
    let raw = row
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("retrieval row is missing {field}"))?;
    let id = raw.split_once(':').map(|(_, id)| id).unwrap_or(raw);
    RecordId::parse(id)
}

fn digest_of(row: &Value, field: &str) -> Result<Digest, String> {
    let raw = row
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("retrieval row is missing {field}"))?;
    Digest::parse(raw)
}

fn coords_of<const N: usize>(row: &Value, field: &str) -> Result<[f64; N], String> {
    let values = row
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("retrieval row is missing {field}"))?;
    if values.len() != N {
        return Err(format!("retrieval row {field} has wrong arity"));
    }
    let mut out = [0.0f64; N];
    for (index, value) in values.iter().enumerate() {
        out[index] = value
            .as_f64()
            .ok_or_else(|| format!("retrieval row {field} is not numeric"))?;
    }
    Ok(out)
}

struct AddressRow {
    address: RecordId,
    atom: RecordId,
    h3: [f64; 4],
    t2: [f64; 2],
}

struct AtomRow {
    content_digest: Digest,
    salience: f64,
    domain: String,
}

/// Retrieves the k nearest atoms to a query point. Fails closed when the
/// candidate budget overruns or coordinates are invalid; k == 0 is an
/// error; k beyond the scanned set returns what exists. `query_provenance`
/// records where the query came from (e.g. a forward tape digest); it is
/// bound into the receipt.
pub fn retrieve(
    config: &RetrievalConfig,
    query_h3: [f64; 4],
    query_t2: [f64; 2],
    query_provenance: &str,
    k: u32,
) -> Result<RecallBundle, String> {
    config.validate()?;
    validate_h3(&query_h3, "query")?;
    validate_t2(&query_t2, "query")?;
    if query_provenance.trim().is_empty() {
        return Err("query provenance must be non-empty".into());
    }
    if k == 0 {
        return Err("k must be positive".into());
    }
    let workspace_ref = format!("workspace:`{}`", config.workspace.as_str());
    let address_rows = control_sql(
        config,
        &format!(
            "SELECT id, atom, h3_lorentz, t2_phase FROM atom_address WHERE workspace = {workspace_ref} LIMIT {};",
            config.candidate_budget
        ),
    )?;
    let raw_addresses = address_rows
        .first()
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if raw_addresses.len() as u32 >= config.candidate_budget {
        return Err(
            "candidate budget overrun: widen the budget explicitly, results are not truncated"
                .into(),
        );
    }
    let mut addresses = Vec::with_capacity(raw_addresses.len());
    for row in &raw_addresses {
        addresses.push(AddressRow {
            address: record_id_of(row, "id")?,
            atom: record_id_of(row, "atom")?,
            h3: coords_of(row, "h3_lorentz")?,
            t2: coords_of(row, "t2_phase")?,
        });
    }
    let atom_rows = control_sql(
        config,
        &format!(
            "SELECT id, content_digest, salience, domain FROM atom WHERE workspace = {workspace_ref} LIMIT {};",
            config.candidate_budget
        ),
    )?;
    let raw_atoms = atom_rows
        .first()
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if raw_atoms.len() as u32 >= config.candidate_budget {
        return Err(
            "candidate budget overrun: widen the budget explicitly, results are not truncated"
                .into(),
        );
    }
    let mut atoms = BTreeMap::new();
    for row in &raw_atoms {
        let id = record_id_of(row, "id")?;
        atoms.insert(
            id.as_str().to_string(),
            AtomRow {
                content_digest: digest_of(row, "content_digest")?,
                salience: row
                    .get("salience")
                    .and_then(Value::as_f64)
                    .ok_or_else(|| "atom row is missing salience".to_string())?,
                domain: row
                    .get("domain")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
        );
    }
    let mut scored: Vec<(u64, String, ScoredAtom)> = Vec::new();
    for address in &addresses {
        let Some(atom) = atoms.get(address.atom.as_str()) else {
            continue;
        };
        let score_value = score(
            config.metric,
            &query_h3,
            &query_t2,
            &address.h3,
            &address.t2,
        )?;
        if !score_value.is_finite() {
            return Err("non-finite retrieval score".into());
        }
        scored.push((
            score_value.to_bits(),
            address.atom.as_str().to_string(),
            ScoredAtom {
                atom: address.atom.clone(),
                address: address.address.clone(),
                content_digest: atom.content_digest.clone(),
                salience: atom.salience,
                domain: atom.domain.clone(),
                h3_lorentz: address.h3,
                t2_phase: address.t2,
                score: score_value,
                rank: 0,
            },
        ));
    }
    scored.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    let mut members = Vec::new();
    for (rank, (_, _, mut member)) in scored.into_iter().take(k as usize).enumerate() {
        member.rank = rank as u32;
        members.push(member);
    }
    let query_digest = digest_fields(
        b"globalmind-retrieval-query-v1",
        &[
            config.workspace.as_str(),
            query_provenance,
            &format!("{query_h3:?}"),
            &format!("{query_t2:?}"),
            config.metric.as_str(),
        ],
    );
    let ranking_digest = digest_fields(
        b"globalmind-retrieval-ranking-v1",
        &std::iter::once(query_digest.as_str().to_string())
            .chain(members.iter().map(|member| {
                format!(
                    "{}:{:?}:{}",
                    member.atom.as_str(),
                    member.score,
                    member.content_digest.as_str()
                )
            }))
            .collect::<Vec<_>>()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    );
    Ok(RecallBundle {
        workspace: config.workspace.clone(),
        query_digest,
        query_provenance: query_provenance.into(),
        metric: config.metric,
        candidates_scanned: addresses.len() as u32,
        members,
        ranking_digest,
    })
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}

fn digest_to_bytes(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let mut out = [0u8; 32];
    for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn write_text(buf: &mut Vec<u8>, value: &str) {
    buf.extend_from_slice(&(value.len() as u32).to_le_bytes());
    buf.extend_from_slice(value.as_bytes());
}

/// Encodes a recall bundle as `GMRC` v1 binary: magic, version, workspace,
/// query digest + provenance, metric, scanned count, members (atom id,
/// address id, content digest, salience, domain, coords, score, rank),
/// ranking digest, trailing sha256 over all preceding bytes.
pub fn encode_bundle(bundle: &RecallBundle) -> Result<Vec<u8>, String> {
    if bundle.members.len() > u32::MAX as usize {
        return Err("recall bundle member count overflows u32".into());
    }
    let mut out = Vec::new();
    out.extend_from_slice(&RECALL_MAGIC);
    out.extend_from_slice(&RECALL_VERSION.to_le_bytes());
    write_text(&mut out, bundle.workspace.as_str());
    out.extend_from_slice(&digest_to_bytes(&bundle.query_digest));
    write_text(&mut out, &bundle.query_provenance);
    out.push(bundle.metric.code());
    out.extend_from_slice(&bundle.candidates_scanned.to_le_bytes());
    out.extend_from_slice(&(bundle.members.len() as u32).to_le_bytes());
    for member in &bundle.members {
        write_text(&mut out, member.atom.as_str());
        write_text(&mut out, member.address.as_str());
        out.extend_from_slice(&digest_to_bytes(&member.content_digest));
        out.extend_from_slice(&member.salience.to_le_bytes());
        write_text(&mut out, &member.domain);
        for value in member.h3_lorentz {
            out.extend_from_slice(&value.to_le_bytes());
        }
        for value in member.t2_phase {
            out.extend_from_slice(&value.to_le_bytes());
        }
        out.extend_from_slice(&member.score.to_le_bytes());
        out.extend_from_slice(&member.rank.to_le_bytes());
    }
    out.extend_from_slice(&digest_to_bytes(&bundle.ranking_digest));
    let file_digest = Sha256::digest(&out);
    out.extend_from_slice(&file_digest);
    Ok(out)
}

struct BundleReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BundleReader<'a> {
    fn take(&mut self, len: usize, what: &str) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| format!("recall bundle overflows reading {what}"))?;
        if end > self.bytes.len() {
            return Err(format!("recall bundle truncated reading {what}"));
        }
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u32_value(&mut self, what: &str) -> Result<u32, String> {
        let raw = self.take(4, what)?;
        Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    fn f64_value(&mut self, what: &str) -> Result<f64, String> {
        let raw = self.take(8, what)?;
        Ok(f64::from_le_bytes([
            raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
        ]))
    }

    fn text(&mut self, what: &str) -> Result<String, String> {
        let len = self.u32_value(what)? as usize;
        let raw = self.take(len, what)?;
        String::from_utf8(raw.to_vec()).map_err(|_| format!("recall bundle {what} is not UTF-8"))
    }

    fn digest(&mut self, what: &str) -> Result<Digest, String> {
        let raw = self.take(32, what)?;
        let mut hex = String::with_capacity(64);
        for byte in raw {
            use std::fmt::Write as _;
            write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
        }
        Digest::parse(hex)
    }
}

/// Decodes a `GMRC` v1 bundle, verifying magic, version, file digest, and
/// rank contiguity. Decoded scores are re-checkable via [`score`].
pub fn decode_bundle(bytes: &[u8]) -> Result<RecallBundle, String> {
    if bytes.len() < 4 + 2 + 32 {
        return Err("recall bundle is truncated".into());
    }
    if bytes[..4] != RECALL_MAGIC {
        return Err("recall bundle magic mismatch".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != RECALL_VERSION {
        return Err(format!("unsupported recall bundle version {version}"));
    }
    let body_end = bytes.len() - 32;
    if Sha256::digest(&bytes[..body_end])[..] != bytes[body_end..] {
        return Err("recall bundle file digest mismatch".into());
    }
    let mut reader = BundleReader {
        bytes: &bytes[..body_end],
        pos: 6,
    };
    let workspace = RecordId::parse(reader.text("workspace")?)?;
    let query_digest = reader.digest("query digest")?;
    let query_provenance = reader.text("provenance")?;
    if query_provenance.trim().is_empty() {
        return Err("recall bundle provenance must be non-empty".into());
    }
    let metric = RetrievalMetric::from_code(reader.take(1, "metric")?[0])?;
    let candidates_scanned = reader.u32_value("scanned")?;
    let member_count = reader.u32_value("member count")? as usize;
    let mut members = Vec::with_capacity(member_count);
    for rank in 0..member_count {
        let atom = RecordId::parse(reader.text("atom")?)?;
        let address = RecordId::parse(reader.text("address")?)?;
        let content_digest = reader.digest("content digest")?;
        let salience = reader.f64_value("salience")?;
        let domain = reader.text("domain")?;
        let h3_lorentz = [
            reader.f64_value("h3[0]")?,
            reader.f64_value("h3[1]")?,
            reader.f64_value("h3[2]")?,
            reader.f64_value("h3[3]")?,
        ];
        let t2_phase = [reader.f64_value("t2[0]")?, reader.f64_value("t2[1]")?];
        let score = reader.f64_value("score")?;
        let stored_rank = reader.u32_value("rank")?;
        if stored_rank as usize != rank {
            return Err("recall bundle ranks are not contiguous from zero".into());
        }
        if !salience.is_finite() || !score.is_finite() {
            return Err("recall bundle salience/score must be finite".into());
        }
        members.push(ScoredAtom {
            atom,
            address,
            content_digest,
            salience,
            domain,
            h3_lorentz,
            t2_phase,
            score,
            rank: stored_rank,
        });
    }
    let ranking_digest = reader.digest("ranking digest")?;
    if reader.pos != body_end {
        return Err("recall bundle has trailing bytes".into());
    }
    Ok(RecallBundle {
        workspace,
        query_digest,
        query_provenance,
        metric,
        candidates_scanned,
        members,
        ranking_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HUB: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    const PHASE0: [f64; 2] = [0.0, 0.0];

    fn radial(r: f64) -> [f64; 4] {
        [r.cosh(), r.sinh(), 0.0, 0.0]
    }

    #[test]
    fn distances_match_executed_reference_values() {
        // Identical points: zero distance both metrics.
        assert_eq!(h3_distance(&HUB, &HUB).unwrap(), 0.0);
        assert_eq!(t2_distance(&PHASE0, &PHASE0).unwrap(), 0.0);
        // Radial point at executed r: H3 distance recovers r.
        for r in [0.25, 0.5, 1.0, 2.0] {
            let got = h3_distance(&HUB, &radial(r)).unwrap();
            assert!((got - r).abs() < 1e-12, "r={r} got={got}");
        }
        // T2 wraparound: 2π - 0.1 is 0.1 away, not 2π - 0.1 away.
        let wrapped = t2_distance(&PHASE0, &[2.0 * std::f64::consts::PI - 0.1, 0.0]).unwrap();
        assert!((wrapped - 0.1).abs() < 1e-12, "wrapped={wrapped}");
        // Antipodal phase is π away.
        let anti = t2_distance(&PHASE0, &[std::f64::consts::PI, 0.0]).unwrap();
        assert!((anti - std::f64::consts::PI).abs() < 1e-12, "anti={anti}");
        // Off-sheet, past-sheet, and non-finite inputs fail closed.
        assert!(h3_distance(&HUB, &[1.0, 0.0, 0.0, 0.5]).is_err());
        assert!(h3_distance(&HUB, &[-1.0, 0.0, 0.0, 0.0]).is_err());
        assert!(h3_distance(&HUB, &[f64::NAN, 0.0, 0.0, 0.0]).is_err());
        assert!(t2_distance(&PHASE0, &[0.0, f64::INFINITY]).is_err());
        assert!(flat_score(&HUB, &PHASE0, &[f64::NAN, 0.0, 0.0, 0.0], &PHASE0).is_err());
        // Metric codes roundtrip; unknown codes fail.
        assert_eq!(
            RetrievalMetric::from_code(1).unwrap(),
            RetrievalMetric::Hyperbolic
        );
        assert_eq!(
            RetrievalMetric::from_code(2).unwrap(),
            RetrievalMetric::Flat
        );
        assert!(RetrievalMetric::from_code(0).is_err());
        assert!(RetrievalMetric::from_code(3).is_err());
    }

    #[test]
    fn t2_weight_probe_before_after_1p0_vs_0p25() {
        // Allocator probe (allocating-weights): fusion weight H3 vs T2^2.
        // Before 1.0 (hand-set) vs after 0.25 (M-1 cost form). Ranking order
        // on the fixed 5-point probe seed must be identical under both;
        // only the T2 term scale changes. A quality-gated sweep stays open
        // until ablation 1's forward harness exists.
        use crate::whitehead::{hyperbolic_distance, torus_distance_sq};
        let seed: [([f64; 4], [f64; 2], &str); 5] = [
            (HUB, PHASE0, "hub"),
            (radial(0.5), PHASE0, "near"),
            (radial(1.0), PHASE0, "mid"),
            (radial(2.0), PHASE0, "far"),
            (HUB, [2.0 * std::f64::consts::PI - 0.1, 0.0], "wrapped"),
        ];
        let order_under = |weight: f64| {
            let mut ranked: Vec<(u64, &str)> = seed
                .iter()
                .map(|(h3, t2, name)| {
                    let score =
                        hyperbolic_distance(HUB, *h3) + weight * torus_distance_sq(PHASE0, *t2);
                    (score.to_bits(), *name)
                })
                .collect();
            ranked.sort();
            ranked.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
        };
        let before = order_under(1.0);
        let after = order_under(T2_WEIGHT);
        assert_eq!(T2_WEIGHT, 0.25);
        assert_eq!(before, ["hub", "wrapped", "near", "mid", "far"]);
        assert_eq!(
            before, after,
            "weight change must not reorder the probe seed"
        );
    }

    #[test]
    fn bundle_roundtrips_and_tamper_fails() {
        let bundle = RecallBundle {
            workspace: RecordId::parse("ws_test").unwrap(),
            query_digest: Digest::sha256(b"query"),
            query_provenance: "unit-test".into(),
            metric: RetrievalMetric::Hyperbolic,
            candidates_scanned: 2,
            members: vec![
                ScoredAtom {
                    atom: RecordId::parse("atom_a").unwrap(),
                    address: RecordId::parse("addr_a").unwrap(),
                    content_digest: Digest::sha256(b"a"),
                    salience: 0.9,
                    domain: "test".into(),
                    h3_lorentz: HUB,
                    t2_phase: PHASE0,
                    score: 0.0,
                    rank: 0,
                },
                ScoredAtom {
                    atom: RecordId::parse("atom_b").unwrap(),
                    address: RecordId::parse("addr_b").unwrap(),
                    content_digest: Digest::sha256(b"b"),
                    salience: 0.5,
                    domain: "test".into(),
                    h3_lorentz: radial(1.0),
                    t2_phase: PHASE0,
                    score: 1.0,
                    rank: 1,
                },
            ],
            ranking_digest: Digest::sha256(b"ranking"),
        };
        let bytes = encode_bundle(&bundle).unwrap();
        let decoded = decode_bundle(&bytes).unwrap();
        assert_eq!(decoded.members.len(), 2);
        assert_eq!(decoded.members[0].atom.as_str(), "atom_a");
        assert_eq!(decoded.members[1].rank, 1);
        assert_eq!(decoded.ranking_digest, bundle.ranking_digest);
        assert_eq!(decoded.metric, RetrievalMetric::Hyperbolic);
        let mut tampered = bytes.clone();
        tampered[20] ^= 0xFF;
        assert!(decode_bundle(&tampered).is_err());
        assert!(decode_bundle(&bytes[..bytes.len() - 33]).is_err());
        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(decode_bundle(&bad_magic).is_err());
        let mut bad_version = bytes.clone();
        bad_version[4] = 9;
        assert!(decode_bundle(&bad_version).is_err());
    }

    struct IsolatedRetrievalDb {
        child: std::process::Child,
        port: u16,
        storage: std::path::PathBuf,
    }

    impl IsolatedRetrievalDb {
        fn start() -> Self {
            // Unique per test thread: parallel live tests share one process.
            let storage = std::env::temp_dir().join(format!(
                "gm-retrieval-db-{}-{:?}-{}",
                std::process::id(),
                std::thread::current().id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ));
            let _ = std::fs::remove_dir_all(&storage);
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
            let port = listener.local_addr().expect("port").port();
            drop(listener);
            let child = std::process::Command::new("surreal")
                .args([
                    "start",
                    "--no-banner",
                    "--log",
                    "error",
                    "--bind",
                    &format!("127.0.0.1:{port}"),
                    "--user",
                    "root",
                    "--pass",
                    "retrieval_pw_001",
                    &format!("surrealkv://{}", storage.display()),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("spawn isolated surrealdb (binary must be installed)");
            let db = Self {
                child,
                port,
                storage,
            };
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while std::time::Instant::now() < deadline {
                if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                    return db;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            panic!("isolated retrieval SurrealDB did not start");
        }

        fn sql(&self, body: &str) -> String {
            use std::io::{Read, Write};
            let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port))
                .expect("connect retrieval db");
            let full = format!("USE NS vs_retrieval DB recall_audit; {body}");
            let request = format!(
                "POST /sql HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                self.port,
                base64("root:retrieval_pw_001"),
                full.len(),
                full
            );
            stream
                .write_all(request.as_bytes())
                .expect("write retrieval sql");
            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .expect("read retrieval response");
            String::from_utf8(response).expect("retrieval response is UTF-8")
        }
    }

    impl Drop for IsolatedRetrievalDb {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.storage);
        }
    }

    fn seed_retrieval_db(db: &IsolatedRetrievalDb) {
        let schema =
            std::fs::read_to_string("../hypermind-nested-kv/schema/0001_hypermind_graph.surql")
                .expect("0001 schema exists");
        let applied = db.sql(&schema);
        assert!(
            !applied.contains("\"status\":\"ERR\""),
            "0001 applies: {applied:.500}"
        );
        // Five atoms: hub, r=0.5, r=1.0, r=2.0 on the x-axis, plus a
        // T2-wrapped atom at the hub (phase 2π-0.1, hyperbolically near).
        let digest = Digest::sha256(b"retrieval-seed").as_str().to_string();
        let mut setup = "CREATE workspace:recall SET slug = 'recall';".to_string();
        for (name, h3, t2) in [
            ("hub", HUB, PHASE0),
            ("near", radial(0.5), PHASE0),
            ("mid", radial(1.0), PHASE0),
            ("far", radial(2.0), PHASE0),
            ("wrapped", HUB, [2.0 * std::f64::consts::PI - 0.1, 0.0]),
        ] {
            setup.push_str(&format!(
                "CREATE atom:{name} SET workspace = workspace:recall, external_id = 'ext_{name}',
                 content_kind = 'code', content_digest = '{digest}', domain = 'recall-test',
                 rank = 0, salience = 0.9, provenance = 'retrieval-test', taint = 'clean';
                 CREATE atom_address:addr_{name} SET workspace = workspace:recall, atom = atom:{name},
                 h3_lorentz = [{}, {}, {}, {}], t2_phase = [{}, {}],
                 geometry_model = 'lorentz', geometry_version = 'v1', revision = 1;",
                h3[0], h3[1], h3[2], h3[3], t2[0], t2[1]
            ));
        }
        let seeded = db.sql(&setup);
        assert!(
            !seeded.contains("\"status\":\"ERR\""),
            "seed ok: {seeded:.800}"
        );
    }

    fn test_config(db: &IsolatedRetrievalDb, metric: RetrievalMetric) -> RetrievalConfig {
        RetrievalConfig {
            endpoint: format!("http://127.0.0.1:{}", db.port),
            namespace: "vs_retrieval".into(),
            database: "recall_audit".into(),
            username: "root".into(),
            password: "retrieval_pw_001".into(),
            workspace: RecordId::parse("recall").unwrap(),
            candidate_budget: DEFAULT_CANDIDATE_BUDGET,
            metric,
        }
    }

    #[test]
    fn live_scoped_well_excludes_hidden_atoms_and_cross_edges() {
        let db = IsolatedRetrievalDb::start();
        seed_retrieval_db(&db);
        let schema = std::fs::read_to_string(
            "../hypermind-nested-kv/schema/0007_globalmind_distillation_trajectories.surql",
        )
        .expect("well schema");
        let applied = db.sql(&schema);
        assert!(
            !applied.contains("\"status\":\"ERR\""),
            "well schema: {applied:.500}"
        );
        let setup = format!(
            "CREATE cognitive_well:scope_a SET workspace = workspace:recall, geometry_space = geometry_space:demo, ordinal = 0, label = 'scope_a', h3_center_lorentz = [1.0,0.0,0.0,0.0], t2_sector = 0, boundary_contract_digest = '{}', status = 'specified'; \
            CREATE address_well_membership:hub SET workspace = workspace:recall, address = atom_address:addr_hub, well = cognitive_well:scope_a, classifier_version = 'test', classifier_digest = '{}', boundary_receipt_digest = '{}'; \
            CREATE address_well_membership:near SET workspace = workspace:recall, address = atom_address:addr_near, well = cognitive_well:scope_a, classifier_version = 'test', classifier_digest = '{}', boundary_receipt_digest = '{}'; \
            RELATE atom:hub->atom_edge:inside->atom:near SET workspace = workspace:recall, kind = 'semantic', weight = 0.8, confidence = 0.9, evidence_digest = '{}', producer = 'test', revision = 1; \
            RELATE atom:hub->atom_edge:outside->atom:far SET workspace = workspace:recall, kind = 'semantic', weight = 0.8, confidence = 0.9, evidence_digest = '{}', producer = 'test', revision = 1;",
            Digest::sha256(b"well"),
            Digest::sha256(b"classifier"),
            Digest::sha256(b"boundary"),
            Digest::sha256(b"classifier"),
            Digest::sha256(b"boundary"),
            Digest::sha256(b"evidence"),
            Digest::sha256(b"evidence")
        );
        let seeded = db.sql(&setup);
        assert!(
            !seeded.contains("\"status\":\"ERR\""),
            "scope seed: {seeded:.800}"
        );
        let config = test_config(&db, RetrievalMetric::Hyperbolic);
        let well = RecordId::parse("scope_a").unwrap();
        let first = scoped::retrieve_scoped(
            &config,
            std::slice::from_ref(&well),
            HUB,
            PHASE0,
            "scope-test",
            5,
        )
        .unwrap();
        assert_eq!(
            first
                .bundle
                .members
                .iter()
                .map(|m| m.atom.as_str())
                .collect::<Vec<_>>(),
            ["hub", "near"]
        );
        assert_eq!(first.edges.len(), 1);
        assert_eq!(first.edges[0].id.as_str(), "inside");
        let second =
            scoped::retrieve_scoped(&config, &[well], HUB, PHASE0, "scope-test", 5).unwrap();
        assert_eq!(first.scope_digest, second.scope_digest);
        assert!(scoped::retrieve_scoped(&config, &[], HUB, PHASE0, "scope-test", 5).is_err());
        let narrow = RetrievalConfig {
            candidate_budget: 2,
            ..config
        };
        assert!(
            scoped::retrieve_scoped(
                &narrow,
                &[RecordId::parse("scope_a").unwrap()],
                HUB,
                PHASE0,
                "scope-test",
                5
            )
            .is_err()
        );
    }

    #[test]
    fn live_neighborhood_orders_hyperbolically_and_deterministically() {
        let db = IsolatedRetrievalDb::start();
        seed_retrieval_db(&db);
        let config = test_config(&db, RetrievalMetric::Hyperbolic);
        let first = retrieve(&config, HUB, PHASE0, "tape_unit_001", 5).unwrap();
        assert_eq!(first.candidates_scanned, 5);
        assert_eq!(first.members.len(), 5);
        let order: Vec<_> = first.members.iter().map(|m| m.atom.as_str()).collect();
        // Hub (0.0), wrapped (M-1 cost of the 0.1 phase wrap), then 0.5/1.0/2.0.
        assert_eq!(order, ["hub", "wrapped", "near", "mid", "far"]);
        assert_eq!(first.members[0].score, 0.0);
        for window in first.members.windows(2) {
            assert!(
                window[0].score < window[1].score,
                "strictly increasing scores"
            );
        }
        for (index, member) in first.members.iter().enumerate() {
            assert_eq!(member.rank as usize, index);
        }
        // Deterministic rerun: identical ids, scores, and ranking digest.
        let second = retrieve(&config, HUB, PHASE0, "tape_unit_001", 5).unwrap();
        assert_eq!(second.ranking_digest, first.ranking_digest);
        assert_eq!(second.query_digest, first.query_digest);
        // k truncation keeps the head of the same ranking.
        let top2 = retrieve(&config, HUB, PHASE0, "tape_unit_001", 2).unwrap();
        assert_eq!(top2.members.len(), 2);
        assert_eq!(top2.members[0].atom.as_str(), "hub");
        assert_eq!(top2.members[1].atom.as_str(), "wrapped");
        // Decoded bundle re-scores identically against the live store rows.
        let bytes = encode_bundle(&first).unwrap();
        let decoded = decode_bundle(&bytes).unwrap();
        for member in &decoded.members {
            let expected = score(
                RetrievalMetric::Hyperbolic,
                &HUB,
                &PHASE0,
                &member.h3_lorentz,
                &member.t2_phase,
            )
            .unwrap();
            assert_eq!(expected.to_bits(), member.score.to_bits());
        }
    }

    struct XorShift64(u64);

    impl XorShift64 {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn next_unit(&mut self) -> f64 {
            (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    #[test]
    fn scale_probe_200_atoms_overlap_latency_determinism() {
        // Queue 2 ablation-3 retrieval slice (no task quality claimed):
        // 200 deterministic on-sheet atoms, both metrics, top-10 overlap,
        // per-query latency, rerun determinism. Prints the measured receipt.
        let db = IsolatedRetrievalDb::start();
        let schema =
            std::fs::read_to_string("../hypermind-nested-kv/schema/0001_hypermind_graph.surql")
                .expect("0001 schema exists");
        assert!(!db.sql(&schema).contains("\"status\":\"ERR\""));
        let digest = Digest::sha256(b"retrieval-scale").as_str().to_string();
        let mut rng = XorShift64(0x243F6A8885A308D3);
        let mut setup = "CREATE workspace:scale SET slug = 'scale';".to_string();
        for index in 0..200u32 {
            let r = rng.next_unit() * 3.0;
            let (mut x, mut y, mut z) = (
                rng.next_unit() * 2.0 - 1.0,
                rng.next_unit() * 2.0 - 1.0,
                rng.next_unit() * 2.0 - 1.0,
            );
            let norm = (x * x + y * y + z * z).sqrt().max(1e-12);
            x /= norm;
            y /= norm;
            z /= norm;
            let (t, s) = (r.cosh(), r.sinh());
            let (p0, p1) = (
                rng.next_unit() * 2.0 * std::f64::consts::PI,
                rng.next_unit() * 2.0 * std::f64::consts::PI,
            );
            setup.push_str(&format!(
                "CREATE atom:a{index:03} SET workspace = workspace:scale, external_id = 's_{index:03}',
                 content_kind = 'code', content_digest = '{digest}', domain = 'scale',
                 rank = 0, salience = 0.5, provenance = 'scale-probe', taint = 'clean';
                 CREATE atom_address:d{index:03} SET workspace = workspace:scale, atom = atom:a{index:03},
                 h3_lorentz = [{t}, {}, {}, {}], t2_phase = [{p0}, {p1}],
                 geometry_model = 'lorentz', geometry_version = 'v1', revision = 1;",
                x * s,
                y * s,
                z * s
            ));
        }
        let seeded = db.sql(&setup);
        assert!(!seeded.contains("\"status\":\"ERR\""), "scale seed ok");
        let mut queries = Vec::new();
        for _ in 0..5 {
            let r = rng.next_unit() * 3.0;
            queries.push((
                radial(r),
                [
                    rng.next_unit() * std::f64::consts::TAU,
                    rng.next_unit() * std::f64::consts::TAU,
                ],
            ));
        }
        let mut tops: Vec<Vec<String>> = Vec::new();
        for metric in [RetrievalMetric::Hyperbolic, RetrievalMetric::Flat] {
            let config = RetrievalConfig {
                endpoint: format!("http://127.0.0.1:{}", db.port),
                namespace: "vs_retrieval".into(),
                database: "recall_audit".into(),
                username: "root".into(),
                password: "retrieval_pw_001".into(),
                workspace: RecordId::parse("scale").unwrap(),
                candidate_budget: DEFAULT_CANDIDATE_BUDGET,
                metric,
            };
            let start = std::time::Instant::now();
            let mut first_ids: Vec<String> = Vec::new();
            for (qi, (qh3, qt2)) in queries.iter().enumerate() {
                let bundle = retrieve(&config, *qh3, *qt2, "scale_probe", 10).unwrap();
                assert_eq!(bundle.members.len(), 10);
                assert_eq!(bundle.candidates_scanned, 200);
                let ids: Vec<String> = bundle
                    .members
                    .iter()
                    .map(|m| m.atom.as_str().to_string())
                    .collect();
                if qi == 0 {
                    first_ids = ids.clone();
                }
                let rerun = retrieve(&config, *qh3, *qt2, "scale_probe", 10).unwrap();
                assert_eq!(rerun.ranking_digest, bundle.ranking_digest);
            }
            let elapsed = start.elapsed();
            println!(
                "scale_probe metric={} queries=5 k=10 scanned=200 elapsed_ms={} top10_q0={}",
                metric.as_str(),
                elapsed.as_millis(),
                first_ids.join(",")
            );
            tops.push(first_ids);
        }
        let overlap = tops[0].iter().filter(|id| tops[1].contains(id)).count();
        println!("scale_probe top10_overlap_q0={overlap}/10");
    }

    #[test]
    fn flat_control_disagrees_on_wrapped_phase() {
        let db = IsolatedRetrievalDb::start();
        seed_retrieval_db(&db);
        let config = test_config(&db, RetrievalMetric::Flat);
        let bundle = retrieve(&config, HUB, PHASE0, "tape_unit_001", 5).unwrap();
        let order: Vec<_> = bundle.members.iter().map(|m| m.atom.as_str()).collect();
        // Euclidean has no wraparound: 2π-0.1 reads as far, so `wrapped`
        // sorts last while the radial atoms keep their order.
        assert_eq!(order, ["hub", "near", "mid", "far", "wrapped"]);
    }

    #[test]
    fn retrieval_fails_closed_on_budget_quarantine_and_inputs() {
        let db = IsolatedRetrievalDb::start();
        seed_retrieval_db(&db);
        // Budget overrun refuses instead of truncating.
        let mut tight = test_config(&db, RetrievalMetric::Hyperbolic);
        tight.candidate_budget = 5;
        assert!(retrieve(&tight, HUB, PHASE0, "tape_unit_001", 5).is_err());
        // Quarantined core refuses without the explicit override.
        let saved_db = std::env::var("GLOBALMIND_SURREAL_DB").ok();
        let saved_allow = std::env::var("GLOBALMIND_ALLOW_QUARANTINED_CORE").ok();
        // SAFETY: single-threaded test section; no other thread touches
        // process env (all other tests construct configs directly).
        unsafe {
            std::env::set_var("GLOBALMIND_SURREAL_DB", "core");
            std::env::remove_var("GLOBALMIND_ALLOW_QUARANTINED_CORE");
        }
        let refused = RetrievalConfig::from_env(
            RecordId::parse("recall").unwrap(),
            RetrievalMetric::Hyperbolic,
        )
        .is_err();
        unsafe {
            match saved_db {
                Some(value) => std::env::set_var("GLOBALMIND_SURREAL_DB", value),
                None => std::env::remove_var("GLOBALMIND_SURREAL_DB"),
            }
            match saved_allow {
                Some(value) => std::env::set_var("GLOBALMIND_ALLOW_QUARANTINED_CORE", value),
                None => std::env::remove_var("GLOBALMIND_ALLOW_QUARANTINED_CORE"),
            }
        }
        assert!(refused, "quarantined core must refuse");
        // Input validation.
        let config = test_config(&db, RetrievalMetric::Hyperbolic);
        assert!(retrieve(&config, HUB, PHASE0, "", 2).is_err());
        assert!(retrieve(&config, HUB, PHASE0, "tape_unit_001", 0).is_err());
        assert!(retrieve(&config, [1.0, 0.0, 0.0, 0.5], PHASE0, "tape_unit_001", 2).is_err());
        assert!(retrieve(&config, HUB, [0.0, f64::NAN], "tape_unit_001", 2).is_err());
    }
}
