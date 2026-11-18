//! Server-path exact K/V gate: a running isolated `serve_glimmer` exports exact
//! pages (GPF2/GPO2), they are persisted to disposable SurrealKV, the DB is
//! restarted, pages are hydrated by exact identity and sent back (GCX2); the
//! continuation must equal the uninterrupted GFX2 run bit for bit (digests).
//! Set GLIMMER_SERVER (isolated port, never 8000/8011). The atom is a witness.

use core_lm::native_exact_wire as exact;
use core_lm::native_pages_wire as pages_wire;
use core_lm::native_token_wire as tokwire;
use globalmind_runtime::{
    Digest, ExactKvIdentity, GlobalMindStore, RadialLaw, RecordId, StoreConfig, TensorDtype,
    TensorLayout, TensorPage, TensorSession, TppeConfig, TppeState,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    Ok((0..bytes.len() / 4)
        .map(|i| f32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap()))
        .collect())
}

fn post_bin(addr: &str, path: &str, ctype: &str, body: &[u8]) -> Result<Vec<u8>, String> {
    let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(300)))
        .map_err(|e| e.to_string())?;
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(head.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.write_all(body).map_err(|e| e.to_string())?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).map_err(|e| e.to_string())?;
    let split = resp
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("no HTTP separator")?;
    let status = String::from_utf8_lossy(&resp[..resp.len().min(40)]).to_string();
    if !resp.starts_with(b"HTTP/1.1 200") {
        return Err(format!(
            "{path}: {status} {}",
            String::from_utf8_lossy(&resp[split + 4..])
        ));
    }
    Ok(resp[split + 4..].to_vec())
}

fn hex32(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn pages_from_store(
    store: &GlobalMindStore,
    identities: &[ExactKvIdentity],
    layers: u32,
    kv_heads: usize,
    tokens: u64,
) -> Result<pages_wire::Pages, String> {
    let mut pages = Vec::new();
    for layer in 0..layers {
        let start = layer as usize * kv_heads;
        for payload in store.fetch_complete_tensor_layer_exact(
            &identities[start..start + kv_heads],
            layer,
            0,
            tokens,
        )? {
            pages.push((decode_f32(&payload.key)?, decode_f32(&payload.value)?));
        }
    }
    Ok(pages)
}

#[test]
#[ignore = "running isolated frozen-Glimmer server (GLIMMER_SERVER) plus disposable SurrealKV restart"]
fn server_exact_pages_survive_restart_and_continue_bit_exact() -> Result<(), String> {
    let addr = std::env::var("GLIMMER_SERVER").map_err(|_| "GLIMMER_SERVER required")?;
    if addr.ends_with(":8000") || addr.ends_with(":8011") {
        return Err("use an isolated Glimmer server, not a live port".into());
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("missing repo root")?
        .to_path_buf();

    // Real tokenization through the model-owned binary route.
    let treq = tokwire::encode_request(&tokwire::TokenizeRequest {
        text: "The capital of France is".into(),
        add_bos: true,
    })?;
    let tokens = tokwire::decode_response(
        &treq,
        &post_bin(
            &addr,
            "/v1/tokenize-native",
            "application/vnd.glimmer.tokenize-v1",
            &treq,
        )?,
    )?
    .token_ids;
    let prefix_len = tokens.len() - 1;
    let ids_digest =
        |ids: &[u32]| Digest::sha256(&ids.iter().flat_map(|i| i.to_le_bytes()).collect::<Vec<_>>());
    let (prefix_digest, request_digest) = (ids_digest(&tokens[..prefix_len]), ids_digest(&tokens));

    // Uninterrupted run, then the prefix export, from the same server.
    let ureq = exact::encode_request(&exact::ExactForwardRequest {
        input_ids: tokens.clone(),
        max_tokens: 4,
    })?;
    let uninterrupted = exact::decode_response(
        &ureq,
        &post_bin(
            &addr,
            "/v1/forward-exact",
            "application/vnd.glimmer.exact-v1",
            &ureq,
        )?,
    )?;
    let preq = pages_wire::encode_prefill_request(&tokens[..prefix_len])?;
    let prefill = pages_wire::decode_prefill_response(
        &preq,
        &post_bin(
            &addr,
            "/v1/prefill-exact",
            "application/vnd.glimmer.prefill-v1",
            &preq,
        )?,
    )?;
    if prefill.artifact != uninterrupted.artifact {
        return Err("prefill and forward report different artifacts".into());
    }
    let shape = prefill.shape.clone();
    let model_digest = Digest::parse(hex32(&prefill.artifact.composite))?;

    let mut db = IsolatedSurreal::start()?;
    let schema_dir = root.join("hypermind-nested-kv/schema");
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let store = db.store()?;
    store.verify_recall_kv_contract()?;

    let workspace = RecordId::parse("globalmind")?;
    let atom = RecordId::parse("server_prefix_witness")?;
    let address = RecordId::parse("server_prefix_address")?;
    let session_id = RecordId::parse("server_exact_001")?;
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
    let mut ch = Sha256::new();
    ch.update(b"tppe-glimmer-memory-address-candidate-v1\0");
    ch.update(tppe_config.law.as_str().as_bytes());
    for v in [
        tppe_config.radial_start,
        tppe_config.radial_span,
        tppe_config.omega_fast,
        tppe_config.omega_slow,
        tppe_config.coupling_fast,
        tppe_config.coupling_slow,
    ] {
        ch.update(v.to_le_bytes());
    }
    let tppe_config_digest = Digest::parse(format!("{:x}", ch.finalize()))?;
    let mut phase = TppeState::origin(tppe_config)?;
    for position in 0..prefix_len {
        let step = (position + 1) as f64;
        phase = phase.step(tppe_config, step * 0.017, step * 0.009)?;
    }
    phase.validate()?;
    let t2 = phase.phase_t2();
    let mut ph = Sha256::new();
    ph.update(b"tppe-glimmer-prefix-state-v1\0");
    for v in [
        phase.theta_fast_unwrapped,
        phase.theta_slow_unwrapped,
        phase.cumulative_revolutions,
        phase.hyperbolic_radius,
        phase.poincare_radius,
        phase.boundary_margin,
        phase.hyperboloid_time,
        phase.hyperboloid_spatial_norm,
    ] {
        ph.update(v.to_le_bytes());
    }
    let phase_digest = Digest::parse(format!("{:x}", ph.finalize()))?;
    let address_contract = Digest::sha256(
        &fs::read(schema_dir.join("0006_immutable_atom_address.surql"))
            .map_err(|e| e.to_string())?,
    );
    let mut tc = fs::read(schema_dir.join("0004_transformer_kv_tensor_pages.surql"))
        .map_err(|e| e.to_string())?;
    tc.extend(
        fs::read(schema_dir.join("0009_immutable_tensor_kv.surql")).map_err(|e| e.to_string())?,
    );
    let tensor_contract = Digest::sha256(&tc);
    db.sql(&format!("CREATE atom:`{}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'server_prefix_witness', content_kind: 'text', content_text: 'The capital of France is', content_uri: NONE, content_digest: '{}', domain: 'forward-smoke', rank: 0, salience: 1.0, provenance: 'server-prefix-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`{}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`{}`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [{:.17}, {:.17}], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;", atom, content_digest, address, atom, phase.hyperboloid_time, phase.hyperboloid_spatial_norm, t2[0], t2[1]))?;

    let session = TensorSession {
        id: session_id.clone(),
        workspace: workspace.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        dtype: TensorDtype::F32,
        layer_count: shape.layers as u32,
        kv_head_count: shape.kv_heads as u32,
        head_dim: shape.head_dim as u32,
        geometry_space: geometry.clone(),
        tppe_version: TPPE_VERSION.into(),
        tppe_configuration_digest: tppe_config_digest.clone(),
        address_contract_digest: address_contract.clone(),
        tensor_contract_digest: tensor_contract.clone(),
    };
    store.create_tensor_session(&session)?;
    let mut identities = Vec::new();
    for (index, (keys, values)) in prefill.pages.iter().enumerate() {
        let (layer, head) = (index / shape.kv_heads, index % shape.kv_heads);
        let (key_bytes, value_bytes) = (f32_bytes(keys), f32_bytes(values));
        let (key_digest, value_digest) = (Digest::sha256(&key_bytes), Digest::sha256(&value_bytes));
        let page_id = RecordId::parse(format!("server_l{layer}_h{head}_prefix"))?;
        let receipt_digest = Digest::sha256(
            format!("{}:{}:{}:{}", session_id, page_id, key_digest, value_digest).as_bytes(),
        );
        let tensor_shape = [prefix_len as u32, shape.head_dim as u32];
        store.put_tensor_page(&TensorPage {
            id: page_id.clone(),
            workspace: workspace.clone(),
            session: session_id.clone(),
            layer: layer as u32,
            kv_head: head as u32,
            token_start: 0,
            token_end: prefix_len as u64,
            tensor_shape,
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
        })?;
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
            tensor_shape,
            layout: TensorLayout::TokenMajor,
            address: address.clone(),
            phase_state_digest: phase_digest.clone(),
            revision: 1,
            key_digest,
            value_digest,
        });
    }
    drop(store);
    db.restart()?;
    let store = db.store()?;
    store.verify_recall_kv_contract()?;

    // Rejections before any bytes are handed to the server.
    let mut changed_model = identities[0].clone();
    changed_model.model_digest = Digest::sha256(b"wrong frozen model");
    if store.fetch_tensor_payload_exact(&changed_model).is_ok() {
        return Err("changed model identity reached K/V bytes".into());
    }
    let mut changed_shape = identities[0].clone();
    changed_shape.head_dim += 1;
    if store.fetch_tensor_payload_exact(&changed_shape).is_ok() {
        return Err("changed shape reached K/V bytes".into());
    }
    let mut changed_tppe = identities[0].clone();
    changed_tppe.tppe_configuration_digest = Digest::sha256(b"other tppe contract");
    if store.fetch_tensor_payload_exact(&changed_tppe).is_ok() {
        return Err("changed TPPE contract reached K/V bytes".into());
    }
    let mut changed_addr = identities[0].clone();
    changed_addr.address_contract_digest = Digest::sha256(b"other address contract");
    if store.fetch_tensor_payload_exact(&changed_addr).is_ok() {
        return Err("changed address contract reached K/V bytes".into());
    }
    let mut changed_page = identities[0].clone();
    changed_page.key_digest = Digest::sha256(b"wrong K/V bytes");
    if store.fetch_tensor_payload_exact(&changed_page).is_ok() {
        return Err("changed page digest reached K/V bytes".into());
    }
    if store
        .fetch_complete_tensor_layer_exact(&identities[..1], 0, 0, prefix_len as u64)
        .is_ok()
    {
        return Err("missing K/V head passed the complete-layer gate".into());
    }

    // Hydrate from the restarted store and continue on the server.
    let hydrated = pages_from_store(
        &store,
        &identities,
        session.layer_count,
        shape.kv_heads,
        prefix_len as u64,
    )?;
    if hydrated != prefill.pages {
        return Err("hydrated pages differ from exported pages".into());
    }
    let resume = |pages: pages_wire::Pages| -> Result<exact::ExactForwardResponse, String> {
        let creq = pages_wire::encode_continue_request(&pages_wire::ContinueRequest {
            shape: shape.clone(),
            pages,
            continuation_ids: vec![tokens[prefix_len]],
            max_tokens: 4,
        })?;
        exact::decode_response(
            &creq,
            &post_bin(
                &addr,
                "/v1/continue-exact",
                "application/vnd.glimmer.continue-v1",
                &creq,
            )?,
        )
    };
    let resumed = resume(hydrated.clone())?;
    if resumed.output_ids != uninterrupted.output_ids
        || resumed.logits_digest != uninterrupted.logits_digest
        || resumed.kv_digest != uninterrupted.kv_digest
        || resumed.artifact != uninterrupted.artifact
    {
        return Err("restart continuation differs from uninterrupted run".into());
    }

    // Controls: a perturbed value must change the result; a wrong shape must be refused.
    let mut perturbed = hydrated;
    perturbed[0].1[0] += 1.0;
    let changed = resume(perturbed)?;
    if changed.logits_digest == uninterrupted.logits_digest
        && changed.kv_digest == uninterrupted.kv_digest
    {
        return Err("perturbed K/V did not change the continuation".into());
    }
    let wrong_shape = pages_wire::Shape {
        head_dim: shape.head_dim - 1,
        ..shape.clone()
    };
    let bad = pages_wire::encode_continue_request(&pages_wire::ContinueRequest {
        pages: vec![
            (
                vec![0.0; wrong_shape.tokens * wrong_shape.head_dim],
                vec![0.0; wrong_shape.tokens * wrong_shape.head_dim]
            );
            wrong_shape.layers * wrong_shape.kv_heads
        ],
        shape: wrong_shape,
        continuation_ids: vec![tokens[prefix_len]],
        max_tokens: 1,
    })?;
    if post_bin(
        &addr,
        "/v1/continue-exact",
        "application/vnd.glimmer.continue-v1",
        &bad,
    )
    .is_ok()
    {
        return Err("server accepted pages of the wrong shape".into());
    }
    println!(
        "PASS server exact pages: tokens={:?} pages={} shape={}x{}x{} out={:?} logits={} kv={} artifact={}",
        tokens,
        identities.len(),
        shape.layers,
        shape.kv_heads,
        shape.head_dim,
        resumed.output_ids,
        hex32(&resumed.logits_digest),
        hex32(&resumed.kv_digest),
        hex32(&resumed.artifact.composite)
    );
    Ok(())
}
