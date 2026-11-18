//! Real Glimmer exact K/V restart plus MCP-selected prefix-atom component gate.
//! This does not claim that the prefix witness is distilled knowledge.

use core_lm::exact_kv::ExactKvCache;
use core_lm::glimmer_forward::GlimmerModel;
use core_lm::tokenizer::BpeTokenizer;
use globalmind_runtime::retrieval::{RetrievalConfig, RetrievalMetric};
use globalmind_runtime::{
    Digest, ExactKvIdentity, GlobalMindStore, RadialLaw, RecordId, StoreConfig, TensorDtype,
    TensorLayout, TensorPage, TensorSession, TppeConfig, TppeState,
};
use rgah::globalmind_bridge::McpSurrealTarget;
use rgah::mcp_client::McpChild;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MODEL_DIR: &str = "/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX";
const MODEL_ID: &str = "Muse-Glimmer-q4km-gs128-MLX";
const MODEL_REVISION: &str = "local-frozen-2026-11-04";
const TPPE_VERSION: &str = "tppe-glimmer-memory-address-candidate-v1";
const SCHEMAS: &[&str] = &[
    "0001_hypermind_graph.surql",
    "0003_nested_hyperbolic_kv.surql",
    "0004_transformer_kv_tensor_pages.surql",
    "0005_universal_atom_lineage.surql",
    "0006_immutable_atom_address.surql",
    "0007_globalmind_distillation_trajectories.surql",
    "0008_superposition_holding.surql",
    "0009_immutable_tensor_kv.surql",
    "0010_native_distillation_ingress.surql",
    "0011_atomic_trajectory_cells.surql",
];

struct IsolatedSurreal {
    child: Option<Child>,
    port: u16,
    dir: PathBuf,
    pass: String,
}

impl IsolatedSurreal {
    fn start() -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        drop(listener);
        if port == 8000 {
            return Err("live Surreal port selected; refuse".into());
        }
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("glimmer-native-kv-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut db = Self {
            child: None,
            port,
            dir,
            pass: format!("ephemeral_{nonce}"),
        };
        db.spawn()?;
        Ok(db)
    }

    fn spawn(&mut self) -> Result<(), String> {
        let bind = format!("127.0.0.1:{}", self.port);
        let uri = format!("surrealkv://{}", self.dir.display());
        self.child = Some(
            Command::new("/opt/homebrew/bin/surreal")
                .args([
                    "start",
                    "--no-banner",
                    "--log",
                    "none",
                    "--bind",
                    &bind,
                    "--user",
                    "root",
                    "--pass",
                    &self.pass,
                    &uri,
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?,
        );
        for _ in 0..300 {
            if TcpStream::connect(&bind).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err("isolated Surreal did not open".into())
    }

    fn restart(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.child.take() {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().map_err(|e| e.to_string())?;
        }
        if TcpStream::connect(format!("127.0.0.1:{}", self.port)).is_ok() {
            return Err("isolated Surreal did not stop".into());
        }
        self.spawn()
    }

    fn endpoint(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    fn config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: self.endpoint(),
            namespace: "globalmind_glimmer_iso".into(),
            database: "glimmer_kv_qual".into(),
            username: Some("root".into()),
            password: Some(self.pass.clone()),
            allow_quarantined_core: false,
        }
    }
    fn store(&self) -> Result<GlobalMindStore, String> {
        GlobalMindStore::connect(self.config())
    }

    fn sql(&self, sql: &str) -> Result<(), String> {
        let signin = serde_json::json!({"user": "root", "pass": self.pass});
        let signin_resp = http_post(
            self.port,
            "/signin",
            "application/json",
            &signin.to_string(),
            None,
        )?;
        let signin_json: serde_json::Value =
            serde_json::from_str(&signin_resp).map_err(|e| e.to_string())?;
        let token = signin_json["token"]
            .as_str()
            .ok_or("missing Surreal signin token")?;
        let full_sql = format!("USE NS globalmind_glimmer_iso DB glimmer_kv_qual; {sql}");
        let response = http_post(self.port, "/sql", "text/plain", &full_sql, Some(token))?;
        let statements: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| e.to_string())?;
        let rows = statements
            .as_array()
            .ok_or("Surreal SQL result is not an array")?;
        for (index, row) in rows.iter().enumerate().skip(1) {
            if row["status"].as_str() == Some("ERR") {
                return Err(format!(
                    "isolated Surreal statement {index} failed: {}",
                    row["result"]
                ));
            }
        }
        Ok(())
    }
}

fn http_post(
    port: u16,
    path: &str,
    content_type: &str,
    body: &str,
    token: Option<&str>,
) -> Result<String, String> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(&addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let auth = token
        .map(|value| format!("\r\nAuthorization: Bearer {value}"))
        .unwrap_or_default();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: {content_type}\r\nAccept: application/json\r\nContent-Length: {}{auth}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8(response).map_err(|e| e.to_string())?;
    if !text.starts_with("HTTP/1.1 200") {
        return Err(format!(
            "Surreal HTTP error: {}",
            text.lines().next().unwrap_or("unknown")
        ));
    }
    let split = text.find("\r\n\r\n").ok_or("missing HTTP body separator")?;
    Ok(text[split + 4..].to_string())
}

impl Drop for IsolatedSurreal {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn file_digest(path: &Path) -> Result<Digest, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buf[..n]);
    }
    Digest::parse(format!("{:x}", hash.finalize()))
}

fn model_roster_digest(dir: &Path) -> Result<Digest, String> {
    let mut hash = Sha256::new();
    hash.update(b"glimmer-frozen-roster-v1\0");
    for name in [
        "config.json",
        "tokenizer.json",
        "model.safetensors.index.json",
        "model-00001-of-00004.safetensors",
        "model-00002-of-00004.safetensors",
        "model-00003-of-00004.safetensors",
        "model-00004-of-00004.safetensors",
    ] {
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update(file_digest(&dir.join(name))?.as_str().as_bytes());
    }
    Digest::parse(format!("{:x}", hash.finalize()))
}

fn f32_bytes(values: &[f32]) -> Box<[u8]> {
    values
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn decode_f32(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if !bytes.len().is_multiple_of(4) {
        return Err("K/V byte length is not f32 aligned".into());
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect())
}

#[test]
#[ignore = "real frozen Glimmer, disposable SurrealKV restart, and MCP child hardware gate"]
fn glimmer_native_mcp_kv_full_pass() -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repo root")?
        .to_path_buf();
    let model_dir = Path::new(MODEL_DIR);
    let model_digest = model_roster_digest(model_dir)?;
    let tokenizer = BpeTokenizer::from_json_file(&model_dir.join("tokenizer.json"))
        .map_err(|e| e.to_string())?;
    let mut model = GlimmerModel::load(model_dir).map_err(|e| e.to_string())?;
    model.weights.retain_cache = false;
    let mut tokens = tokenizer
        .encode("The capital of France is")
        .map_err(|e| e.to_string())?;
    tokens.insert(0, model.config.bos_token_id);
    let continuation = *tokens.last().ok_or("no continuation token")?;
    let prefix_len = tokens.len() - 1;
    let digest_tokens = |ids: &[u32]| -> Digest {
        let bytes = ids
            .iter()
            .flat_map(|id| id.to_le_bytes())
            .collect::<Vec<_>>();
        Digest::sha256(&bytes)
    };
    let prefix_digest = digest_tokens(&tokens[..prefix_len]);
    let request_digest = digest_tokens(&tokens);
    let mut cache = ExactKvCache::new(
        model.config.num_hidden_layers,
        model.config.num_attention_heads,
        model.config.num_key_value_heads,
        model.config.head_dim,
    )?;
    for (position, &token) in tokens[..prefix_len].iter().enumerate() {
        model
            .forward_with_backend(token, position, &mut cache)
            .map_err(|e| e.to_string())?;
    }
    cache.verify_complete_prefix(prefix_len)?;
    let uninterrupted = model
        .forward_with_backend(continuation, prefix_len, &mut cache)
        .map_err(|e| e.to_string())?;

    let mut db = IsolatedSurreal::start()?;
    let schema_dir = root.join("hypermind-nested-kv/schema");
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let store = db.store()?;
    store.verify_recall_kv_contract()?;

    let workspace = RecordId::parse("globalmind")?;
    let atom = RecordId::parse("glimmer_prefix_witness")?;
    let address = RecordId::parse("glimmer_prefix_address")?;
    let session_id = RecordId::parse("glimmer_exact_full_001")?;
    let geometry = RecordId::parse("h3_t2_r12_v1")?;
    let content_digest = Digest::sha256(b"The capital of France is");
    let tppe_config = TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.05,
        radial_span: 0.12,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }
    .validate()?;
    let mut config_hash = Sha256::new();
    config_hash.update(b"tppe-glimmer-memory-address-candidate-v1\0");
    config_hash.update(tppe_config.law.as_str().as_bytes());
    for value in [
        tppe_config.radial_start,
        tppe_config.radial_span,
        tppe_config.omega_fast,
        tppe_config.omega_slow,
        tppe_config.coupling_fast,
        tppe_config.coupling_slow,
    ] {
        config_hash.update(value.to_le_bytes());
    }
    let tppe_config_digest = Digest::parse(format!("{:x}", config_hash.finalize()))?;
    let mut phase = TppeState::origin(tppe_config)?;
    for position in 0..prefix_len {
        let step = (position + 1) as f64;
        phase = phase.step(tppe_config, step * 0.017, step * 0.009)?;
    }
    phase.validate()?;
    let t2 = phase.phase_t2();
    let mut phase_hash = Sha256::new();
    phase_hash.update(b"tppe-glimmer-prefix-state-v1\0");
    for value in [
        phase.theta_fast_unwrapped,
        phase.theta_slow_unwrapped,
        phase.cumulative_revolutions,
        phase.hyperbolic_radius,
        phase.poincare_radius,
        phase.boundary_margin,
        phase.hyperboloid_time,
        phase.hyperboloid_spatial_norm,
    ] {
        phase_hash.update(value.to_le_bytes());
    }
    let phase_digest = Digest::parse(format!("{:x}", phase_hash.finalize()))?;
    let address_contract = Digest::sha256(
        &fs::read(schema_dir.join("0006_immutable_atom_address.surql"))
            .map_err(|e| e.to_string())?,
    );
    let mut tensor_contract_bytes =
        fs::read(schema_dir.join("0004_transformer_kv_tensor_pages.surql"))
            .map_err(|e| e.to_string())?;
    tensor_contract_bytes.extend(
        fs::read(schema_dir.join("0009_immutable_tensor_kv.surql")).map_err(|e| e.to_string())?,
    );
    let tensor_contract = Digest::sha256(&tensor_contract_bytes);
    db.sql(&format!("CREATE atom:`{}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'glimmer_prefix_witness', content_kind: 'text', content_text: 'The capital of France is', content_uri: NONE, content_digest: '{}', domain: 'forward-smoke', rank: 0, salience: 1.0, provenance: 'real-glimmer-prefix-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`{}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`{}`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [{:.17}, {:.17}], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;", atom, content_digest, address, atom, phase.hyperboloid_time, phase.hyperboloid_spatial_norm, t2[0], t2[1]))?;

    let session = TensorSession {
        id: session_id.clone(),
        workspace: workspace.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        dtype: TensorDtype::F32,
        layer_count: model.config.num_hidden_layers as u32,
        kv_head_count: model.config.num_key_value_heads as u32,
        head_dim: model.config.head_dim as u32,
        geometry_space: geometry.clone(),
        tppe_version: TPPE_VERSION.into(),
        tppe_configuration_digest: tppe_config_digest.clone(),
        address_contract_digest: address_contract.clone(),
        tensor_contract_digest: tensor_contract.clone(),
    };
    store.create_tensor_session(&session)?;
    let mut identities = Vec::new();
    for layer in 0..model.config.num_hidden_layers {
        for head in 0..model.config.num_key_value_heads {
            let (keys, values) = cache
                .head_page(layer, head)
                .ok_or("missing native K/V head")?;
            // The uninterrupted continuation has one extra token. Persist only the prefix.
            let keys = &keys[..prefix_len * model.config.head_dim];
            let values = &values[..prefix_len * model.config.head_dim];
            let key_bytes = f32_bytes(keys);
            let value_bytes = f32_bytes(values);
            let key_digest = Digest::sha256(&key_bytes);
            let value_digest = Digest::sha256(&value_bytes);
            let page_id = RecordId::parse(format!("glimmer_l{layer}_h{head}_prefix"))?;
            let receipt_digest = Digest::sha256(
                format!("{}:{}:{}:{}", session_id, page_id, key_digest, value_digest).as_bytes(),
            );
            let page = TensorPage {
                id: page_id.clone(),
                workspace: workspace.clone(),
                session: session_id.clone(),
                layer: layer as u32,
                kv_head: head as u32,
                token_start: 0,
                token_end: prefix_len as u64,
                tensor_shape: [prefix_len as u32, model.config.head_dim as u32],
                dtype: TensorDtype::F32,
                layout: TensorLayout::TokenMajor,
                key_tensor: key_bytes,
                value_tensor: value_bytes,
                key_digest: key_digest.clone(),
                value_digest: value_digest.clone(),
                address: address.clone(),
                trajectory_atoms: vec![atom.clone()],
                well: None,
                trajectory: None,
                phase_state_digest: phase_digest.clone(),
                receipt_digest,
                revision: 1,
            };
            store.put_tensor_page(&page)?;
            identities.push(ExactKvIdentity {
                workspace: workspace.clone(),
                session: session_id.clone(),
                page: page_id,
                model_id: MODEL_ID.into(),
                model_revision: MODEL_REVISION.into(),
                model_digest: model_digest.clone(),
                request_digest: request_digest.clone(),
                prefix_digest: prefix_digest.clone(),
                tppe_version: TPPE_VERSION.into(),
                tppe_configuration_digest: tppe_config_digest.clone(),
                address_contract_digest: address_contract.clone(),
                tensor_contract_digest: tensor_contract.clone(),
                geometry_space: geometry.clone(),
                dtype: TensorDtype::F32,
                layer_count: session.layer_count,
                kv_head_count: session.kv_head_count,
                head_dim: session.head_dim,
                layer: layer as u32,
                kv_head: head as u32,
                token_start: 0,
                token_end: prefix_len as u64,
                tensor_shape: [prefix_len as u32, model.config.head_dim as u32],
                layout: TensorLayout::TokenMajor,
                address: address.clone(),
                phase_state_digest: phase_digest.clone(),
                revision: 1,
                key_digest,
                value_digest,
            });
        }
    }
    drop(store);
    db.restart()?;
    let store = db.store()?;
    store.verify_recall_kv_contract()?;

    let mcp_binary = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/globalmind-mcp");
    if !mcp_binary.is_file() {
        return Err("build globalmind-mcp before the live gate".into());
    }
    let mcp_root = db.dir.join("mcp-child-root");
    fs::create_dir_all(&mcp_root).map_err(|e| e.to_string())?;
    let endpoint = db.endpoint();
    let mut child = McpChild::spawn_with_env(
        mcp_binary.to_str().ok_or("invalid MCP binary path")?,
        mcp_root.to_str().ok_or("invalid MCP root path")?,
        &[
            ("GLOBALMIND_SURREAL_URL", endpoint.as_str()),
            ("GLOBALMIND_SURREAL_NS", "globalmind_glimmer_iso"),
            ("GLOBALMIND_SURREAL_DB", "glimmer_kv_qual"),
            ("GLOBALMIND_SURREAL_USER", "root"),
            ("GLOBALMIND_SURREAL_PASS", db.pass.as_str()),
        ],
    )?;
    let retrieval = RetrievalConfig {
        endpoint: endpoint.clone(),
        namespace: "globalmind_glimmer_iso".into(),
        database: "glimmer_kv_qual".into(),
        username: "root".into(),
        password: db.pass.clone(),
        workspace: workspace.clone(),
        candidate_budget: 8,
        metric: RetrievalMetric::Hyperbolic,
    };
    let authority = McpSurrealTarget {
        mode: "on".into(),
        endpoint: endpoint.clone(),
        namespace: "globalmind_glimmer_iso".into(),
        database: "glimmer_kv_qual".into(),
    };
    let authority_receipt = authority.verify_live_child(&retrieval, &mut child)?;
    let recall_json = serde_json::json!({"workspace":"globalmind",
        "query_h3":[phase.hyperboloid_time, phase.hyperboloid_spatial_norm, 0.0, 0.0],
        "query_t2":t2, "query_provenance":"glimmer-native-real-prefix", "k":1});
    let recall_args = rgah::json::parse(&recall_json.to_string()).map_err(|e| e.to_string())?;
    let recall = child.call("mind.recall", &recall_args)?;
    let members = recall
        .inner
        .get("members")
        .ok_or("MCP recall missing members")?;
    if !matches!(members, rgah::json::Json::Arr(rows) if rows.iter().any(|row|
        row.get("atom").and_then(rgah::json::Json::as_str) == Some(atom.as_str())
        && row.get("address").and_then(rgah::json::Json::as_str) == Some(address.as_str())
        && row.get("content_digest").and_then(rgah::json::Json::as_str) == Some(content_digest.as_str())
        && row.get("content_text").and_then(rgah::json::Json::as_str) == Some("The capital of France is")))
    {
        return Err(
            "MCP did not return the exact prefix atom/address/content; K/V hydration denied".into(),
        );
    }

    let first = &identities[0];
    let kv_args = serde_json::json!({ "workspace": first.workspace.as_str(), "session": first.session.as_str(),
        "page": first.page.as_str(), "model_id": first.model_id, "model_revision": first.model_revision,
        "model_digest": first.model_digest.as_str(), "request_digest": first.request_digest.as_str(),
        "prefix_digest": first.prefix_digest.as_str(), "tppe_version": first.tppe_version,
        "tppe_configuration_digest": first.tppe_configuration_digest.as_str(),
        "address_contract_digest": first.address_contract_digest.as_str(), "tensor_contract_digest": first.tensor_contract_digest.as_str(),
        "geometry_space": first.geometry_space.as_str(), "dtype": "f32", "layer_count": first.layer_count,
        "kv_head_count": first.kv_head_count, "head_dim": first.head_dim, "layer": first.layer,
        "kv_head": first.kv_head, "token_start": first.token_start, "token_end": first.token_end,
        "tensor_shape": first.tensor_shape, "layout": "token_major", "address": first.address.as_str(),
        "phase_state_digest": first.phase_state_digest.as_str(), "revision": first.revision,
        "key_digest": first.key_digest.as_str(), "value_digest": first.value_digest.as_str() });
    let kv_result = child.call(
        "mind.kv_exact",
        &rgah::json::parse(&kv_args.to_string()).map_err(|e| e.to_string())?,
    )?;
    if kv_result
        .inner
        .get("key_digest")
        .and_then(rgah::json::Json::as_str)
        != Some(first.key_digest.as_str())
    {
        return Err("MCP exact K/V digest mismatch".into());
    }

    let mut changed_model = identities[0].clone();
    changed_model.model_digest = Digest::sha256(b"wrong frozen model");
    if store.fetch_tensor_payload_exact(&changed_model).is_ok() {
        return Err("changed model identity reached K/V bytes".into());
    }
    if store
        .fetch_complete_tensor_layer_exact(&identities[..1], 0, 0, prefix_len as u64)
        .is_ok()
    {
        return Err("missing K/V head passed complete-layer gate".into());
    }
    let mut changed_payload_digest = identities[0].clone();
    changed_payload_digest.key_digest = Digest::sha256(b"wrong K/V bytes");
    if store
        .fetch_tensor_payload_exact(&changed_payload_digest)
        .is_ok()
    {
        return Err("changed K/V digest reached tensor bytes".into());
    }

    let mut pages = Vec::with_capacity(identities.len());
    for layer in 0..session.layer_count {
        let start = layer as usize * session.kv_head_count as usize;
        let end = start + session.kv_head_count as usize;
        let payloads = store.fetch_complete_tensor_layer_exact(
            &identities[start..end],
            layer,
            0,
            prefix_len as u64,
        )?;
        for payload in payloads {
            pages.push((decode_f32(&payload.key)?, decode_f32(&payload.value)?));
        }
    }
    let mut changed_pages = pages.clone();
    changed_pages[0].1[0] += 1.0;
    let mut changed_cache = ExactKvCache::from_head_pages(
        model.config.num_hidden_layers,
        model.config.num_attention_heads,
        model.config.num_key_value_heads,
        model.config.head_dim,
        prefix_len,
        changed_pages,
    )?;
    let changed_logits = model
        .forward_with_backend(continuation, prefix_len, &mut changed_cache)
        .map_err(|e| e.to_string())?;
    let mut restored = ExactKvCache::from_head_pages(
        model.config.num_hidden_layers,
        model.config.num_attention_heads,
        model.config.num_key_value_heads,
        model.config.head_dim,
        prefix_len,
        pages,
    )?;
    let resumed = model
        .forward_with_backend(continuation, prefix_len, &mut restored)
        .map_err(|e| e.to_string())?;
    if uninterrupted.len() != resumed.len()
        || !uninterrupted
            .iter()
            .zip(&resumed)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    {
        return Err("full-model K/V restart logits diverged".into());
    }
    if changed_logits
        .iter()
        .zip(&resumed)
        .all(|(a, b)| a.to_bits() == b.to_bits())
    {
        return Err("perturbed K/V bytes did not affect full-model logits".into());
    }
    let top = resumed
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .ok_or("empty logits")?;
    let corrupt_digest = Digest::sha256(b"expected inline content");
    db.sql(&format!("CREATE atom:`corrupt_recall_control` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'corrupt-control', content_kind: 'text', content_text: 'different inline content', content_uri: NONE, content_digest: '{}', domain: 'adversarial-control', rank: 0, salience: 1.0, provenance: 'negative-control', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`corrupt_recall_address` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`corrupt_recall_control`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [{:.17}, {:.17}], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;", corrupt_digest, phase.hyperboloid_time, phase.hyperboloid_spatial_norm, t2[0], t2[1]))?;
    let mut corrupt_query = recall_json.clone();
    corrupt_query["k"] = serde_json::json!(2);
    let corrupt_args = rgah::json::parse(&corrupt_query.to_string()).map_err(|e| e.to_string())?;
    if child.call("mind.recall", &corrupt_args).is_ok() {
        return Err("digest-mismatched inline atom passed MCP recall".into());
    }
    child.close();
    println!(
        "PASS Glimmer MCP prefix atom + disposable SurrealKV restart + exact 52-layer K/V continuation: pages={} logits={} argmax={} authority={} recall={} kv={}",
        identities.len(),
        resumed.len(),
        top.0,
        authority_receipt,
        recall.receipt_sha256,
        kv_result.receipt_sha256
    );
    Ok(())
}
