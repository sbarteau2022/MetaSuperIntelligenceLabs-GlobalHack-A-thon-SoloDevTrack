//! Hardware gate for the executor's exact-K/V path: call 1 misses (prefill ->
//! persist to disposable SurrealKV -> continue), Surreal restarts, call 2 hits
//! (hydrate by exact identity -> continue). Both must equal a plain forward-exact
//! executor bit for bit. Needs ARES_GLIMMER_NATIVE_URL (isolated server),
//! ARES_GLIMMER_WEIGHTS_SHA256, ARES_MCP_MIND_BIN, GLOBALMIND_SCHEMA_DIR.
//! Run: cargo test --test native_glimmer_kv_live -- --ignored --nocapture

use ares_backend::harness::mcp::McpAuthority;
use ares_backend::harness::mind_recall_executor::MindRecallExecutor;
use ares_backend::harness::mission::{
    EnergyBudget, Judge, JudgeKind, JudgeRef, Packet, PacketStatus, Plan, ReceiptLog, RetryPolicy,
    ToolCall,
};
use ares_backend::harness::native_glimmer_executor::{
    decode_forward_response, decode_token_response, encode_forward_request, encode_token_request,
    verified_recall_text, KvKey, KvPageStore, NativeGlimmerExecutor,
};
use ares_backend::harness::native_pages_client::{
    decode_prefill_response, decode_session_reply, decode_session_response,
    encode_continue_request, encode_prefill_request, encode_session_turn, Pages, Shape,
};
use ares_backend::harness::output_receipt::BoundCallOutcome;
use ares_backend::harness::runtime::{CapabilityGate, CapabilityNode, Engine, Executor};
use ares_backend::harness::telemetry::{HyperdxConfig, HyperdxTelemetry};
use globalmind_runtime::{
    Digest, ExactKvIdentity, GlobalMindStore, RadialLaw, RecordId, StoreConfig, TensorDtype,
    TensorLayout, TensorPage, TensorSession, TppeConfig, TppeState,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
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
        let dir =
            std::env::temp_dir().join(format!("glimmer-exec-kv-{}-{nonce}", std::process::id()));
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

const NS: &str = "globalmind_glimmer_iso";
const DB: &str = "glimmer_kv_qual";
const ATOM_TEXT: &str = "The capital of France is";

type Roster = Arc<Mutex<HashMap<(Vec<u8>, Vec<u32>), Vec<ExactKvIdentity>>>>;

/// Real globalmind store adapter. The page-digest roster (what an ExactKvIdentity
/// needs) is carried by the caller across restarts, as a receipt store would.
struct StoreKv {
    store: GlobalMindStore,
    roster: Roster,
    schema_dir: PathBuf,
    tppe_variant: &'static str,
}

fn digest_ids(ids: &[u32]) -> Digest {
    Digest::sha256(&ids.iter().flat_map(|i| i.to_le_bytes()).collect::<Vec<_>>())
}

impl StoreKv {
    fn contracts(&self, prefix_len: usize) -> Result<(Digest, Digest, Digest, Digest), String> {
        let cfg = TppeConfig {
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
        ch.update(self.tppe_variant.as_bytes());
        ch.update(cfg.law.as_str().as_bytes());
        for v in [
            cfg.radial_start,
            cfg.radial_span,
            cfg.omega_fast,
            cfg.omega_slow,
            cfg.coupling_fast,
            cfg.coupling_slow,
        ] {
            ch.update(v.to_le_bytes());
        }
        let tppe = Digest::parse(format!("{:x}", ch.finalize()))?;
        let mut phase = TppeState::origin(cfg)?;
        for position in 0..prefix_len {
            let step = (position + 1) as f64;
            phase = phase.step(cfg, step * 0.017, step * 0.009)?;
        }
        phase.validate()?;
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
        let addr = Digest::sha256(
            &fs::read(self.schema_dir.join("0006_immutable_atom_address.surql"))
                .map_err(|e| e.to_string())?,
        );
        let mut tc = fs::read(
            self.schema_dir
                .join("0004_transformer_kv_tensor_pages.surql"),
        )
        .map_err(|e| e.to_string())?;
        tc.extend(
            fs::read(self.schema_dir.join("0009_immutable_tensor_kv.surql"))
                .map_err(|e| e.to_string())?,
        );
        Ok((tppe, phase_digest, addr, Digest::sha256(&tc)))
    }

    fn identity(
        &self,
        key: &KvKey,
        shape: &Shape,
        layer: usize,
        head: usize,
        digests: (Digest, Digest),
    ) -> Result<ExactKvIdentity, String> {
        let (tppe, phase, addr, tensor) = self.contracts(key.prefix_ids.len())?;
        let model = Digest::parse(
            key.artifact_composite
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
        )?;
        let prefix = digest_ids(&key.prefix_ids);
        let tag = &prefix.as_str()[..16];
        Ok(ExactKvIdentity {
            workspace: RecordId::parse("globalmind")?,
            session: RecordId::parse(format!("exec_{tag}"))?,
            page: RecordId::parse(format!("exec_{tag}_l{layer}_h{head}"))?,
            model_id: MODEL_ID.into(),
            model_revision: MODEL_REVISION.into(),
            model_digest: model,
            request_digest: prefix.clone(),
            prefix_digest: prefix,
            tppe_version: TPPE_VERSION.into(),
            tppe_configuration_digest: tppe,
            address_contract_digest: addr,
            tensor_contract_digest: tensor,
            geometry_space: RecordId::parse("h3_t2_r12_v1")?,
            dtype: TensorDtype::F32,
            layer_count: shape.layers as u32,
            kv_head_count: shape.kv_heads as u32,
            head_dim: shape.head_dim as u32,
            layer: layer as u32,
            kv_head: head as u32,
            token_start: 0,
            token_end: key.prefix_ids.len() as u64,
            tensor_shape: [key.prefix_ids.len() as u32, shape.head_dim as u32],
            layout: TensorLayout::TokenMajor,
            address: RecordId::parse("smoke_addr")?,
            phase_state_digest: phase,
            revision: 1,
            key_digest: digests.0,
            value_digest: digests.1,
        })
    }

    fn roster_key(key: &KvKey) -> (Vec<u8>, Vec<u32>) {
        (key.artifact_composite.to_vec(), key.prefix_ids.clone())
    }
}

impl KvPageStore for StoreKv {
    fn fetch(&mut self, key: &KvKey) -> Result<Option<(Shape, Pages)>, String> {
        let Some(known) = self
            .roster
            .lock()
            .unwrap()
            .get(&Self::roster_key(key))
            .cloned()
        else {
            return Ok(None);
        };
        // Rebuild every identity under THIS adapter's contracts; a changed
        // model/shape/TPPE/address contract or page digest must error.
        let first = &known[0];
        let shape = Shape {
            layers: first.layer_count as usize,
            kv_heads: first.kv_head_count as usize,
            head_dim: first.head_dim as usize,
            tokens: key.prefix_ids.len(),
        };
        let mut identities = Vec::new();
        for old in &known {
            identities.push(self.identity(
                key,
                &shape,
                old.layer as usize,
                old.kv_head as usize,
                (old.key_digest.clone(), old.value_digest.clone()),
            )?);
        }
        let mut pages = Vec::new();
        for layer in 0..shape.layers {
            let start = layer * shape.kv_heads;
            for payload in self.store.fetch_complete_tensor_layer_exact(
                &identities[start..start + shape.kv_heads],
                layer as u32,
                0,
                key.prefix_ids.len() as u64,
            )? {
                pages.push((decode_f32(&payload.key)?, decode_f32(&payload.value)?));
            }
        }
        Ok(Some((shape, pages)))
    }

    fn store(&mut self, key: &KvKey, shape: &Shape, pages: &Pages) -> Result<(), String> {
        let zero = Digest::sha256(b"");
        let proto = self.identity(key, shape, 0, 0, (zero.clone(), zero))?;
        let (tppe, _, addr, tensor) = self.contracts(key.prefix_ids.len())?;
        let model = proto.model_digest.clone();
        self.store.create_tensor_session(&TensorSession {
            id: proto.session.clone(),
            workspace: proto.workspace.clone(),
            model_id: MODEL_ID.into(),
            model_revision: MODEL_REVISION.into(),
            model_digest: model,
            request_digest: proto.request_digest.clone(),
            prefix_digest: proto.prefix_digest.clone(),
            dtype: TensorDtype::F32,
            layer_count: shape.layers as u32,
            kv_head_count: shape.kv_heads as u32,
            head_dim: shape.head_dim as u32,
            geometry_space: proto.geometry_space.clone(),
            tppe_version: TPPE_VERSION.into(),
            tppe_configuration_digest: tppe,
            address_contract_digest: addr,
            tensor_contract_digest: tensor,
        })?;
        let mut identities = Vec::new();
        for (index, (keys, values)) in pages.iter().enumerate() {
            let (layer, head) = (index / shape.kv_heads, index % shape.kv_heads);
            let (kb, vb) = (f32_bytes(keys), f32_bytes(values));
            let (kd, vd) = (Digest::sha256(&kb), Digest::sha256(&vb));
            let id = self.identity(key, shape, layer, head, (kd.clone(), vd.clone()))?;
            let receipt_digest =
                Digest::sha256(format!("{}:{}:{}:{}", id.session, id.page, kd, vd).as_bytes());
            self.store.put_tensor_page(&TensorPage {
                id: id.page.clone(),
                workspace: id.workspace.clone(),
                session: id.session.clone(),
                layer: layer as u32,
                kv_head: head as u32,
                token_start: 0,
                token_end: id.token_end,
                tensor_shape: id.tensor_shape,
                dtype: TensorDtype::F32,
                layout: TensorLayout::TokenMajor,
                key_tensor: kb,
                value_tensor: vb,
                key_digest: kd,
                value_digest: vd,
                address: id.address.clone(),
                trajectory_atoms: vec![RecordId::parse("smoke_atom")?],
                well: None,
                trajectory: None,
                phase_state_digest: id.phase_state_digest.clone(),
                receipt_digest,
                revision: 1,
            })?;
            identities.push(id);
        }
        self.roster
            .lock()
            .unwrap()
            .insert(Self::roster_key(key), identities);
        Ok(())
    }
}

fn executor(db: &IsolatedSurreal, kv: Option<StoreKv>) -> Result<NativeGlimmerExecutor, String> {
    let authority = McpAuthority {
        base: db.endpoint(),
        ns: NS.into(),
        db: DB.into(),
    };
    let mind = MindRecallExecutor::from_state(authority, ("root".into(), db.pass.clone()))?;
    let exec = NativeGlimmerExecutor::from_env(mind)?;
    Ok(match kv {
        Some(kv) => exec.with_kv_store(Box::new(kv)),
        None => exec,
    })
}

#[test]
#[ignore = "isolated frozen-Glimmer server, real MCP child, disposable SurrealKV restart"]
fn executor_kv_miss_then_restart_hit_equals_plain_forward() -> Result<(), String> {
    let schema_dir = PathBuf::from(
        std::env::var("GLOBALMIND_SCHEMA_DIR").map_err(|_| "GLOBALMIND_SCHEMA_DIR required")?,
    );
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let mut db = IsolatedSurreal::start()?;
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let digest = format!("{:x}", Sha256::digest(ATOM_TEXT.as_bytes()));
    let (t, x) = (0.5f64.cosh(), 0.5f64.sinh());
    db.sql(&format!(
        "CREATE atom:`smoke_atom` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'smoke_atom', content_kind: 'text', content_text: '{ATOM_TEXT}', content_uri: NONE, content_digest: '{digest}', domain: 'forward-smoke', rank: 0, salience: 1.0, provenance: 'smoke-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`smoke_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`smoke_atom`, h3_lorentz: [{t:.17}, {x:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
    ))?;
    let args = serde_json::json!({"workspace":"globalmind","query_h3":[t,x,0.0,0.0],
        "query_t2":[0.1,0.2],"query_provenance":"smoke-query","k":1})
    .to_string();
    let new_kv = |db: &IsolatedSurreal, roster: &Roster, variant| -> Result<StoreKv, String> {
        Ok(StoreKv {
            store: db.store()?,
            roster: roster.clone(),
            schema_dir: schema_dir.clone(),
            tppe_variant: variant,
        })
    };
    let roster: Roster = Arc::new(Mutex::new(HashMap::new()));
    let names = |r: &ares_backend::harness::output_receipt::BoundOutputReceipt| {
        r.tool_results
            .iter()
            .map(|t| t.tool.clone())
            .collect::<Vec<_>>()
    };

    let plain = executor(&db, None)?
        .call_bound("mind.recall", &args)?
        .bound_receipt;
    let miss = executor(&db, Some(new_kv(&db, &roster, "v1")?))?
        .call_bound("mind.recall", &args)?
        .bound_receipt;
    assert!(
        names(&miss).contains(&"native.kv-prefill".to_string()),
        "{:?}",
        names(&miss)
    );

    db.restart()?;
    let hit = executor(&db, Some(new_kv(&db, &roster, "v1")?))?
        .call_bound("mind.recall", &args)?
        .bound_receipt;
    let hit_names = names(&hit);
    assert!(
        hit_names.contains(&"native.kv-hit".to_string()),
        "{hit_names:?}"
    );
    assert!(!hit_names.contains(&"native.kv-prefill".to_string()));
    for r in [&miss, &hit] {
        assert_eq!(r.generated_token_ids, plain.generated_token_ids);
        assert_eq!(r.logits_digest, plain.logits_digest);
        assert_eq!(r.kv_digest, plain.kv_digest);
        assert_eq!(r.model_artifact, plain.model_artifact);
        r.validate()?;
    }

    // A changed TPPE contract must fail closed, not fall back to a recompute.
    let wrong =
        executor(&db, Some(new_kv(&db, &roster, "v2-changed")?))?.call_bound("mind.recall", &args);
    assert!(wrong.is_err(), "changed TPPE contract must refuse");
    println!(
        "PASS executor kv: ids={:?} logits={} kv={} miss={:?} hit={:?}",
        hit.generated_token_ids,
        hit.logits_digest,
        hit.kv_digest,
        names(&miss),
        hit_names
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Governed 30-step run (handoff item 6)
// ---------------------------------------------------------------------------

const RUN_ATOMS: [&str; 6] = [
    "The capital of France is",
    "Water boils at one hundred degrees Celsius",
    "The speed of light in vacuum is",
    "Mount Everest is the highest mountain on Earth",
    "The chemical symbol for gold is",
    "Shakespeare wrote the play Hamlet",
];

#[derive(Clone)]
struct Recorded {
    packet_args: String,
    result: Result<BoundCallOutcome, String>,
    wall: Duration,
}

/// Records every executor call/outcome for the post-run audit and restarts the
/// disposable Surreal once, mid-run, so persistence is exercised under load.
struct Recording {
    inner: NativeGlimmerExecutor,
    log: Arc<Mutex<Vec<Recorded>>>,
    db: Arc<Mutex<IsolatedSurreal>>,
    restart_at: usize,
}

impl Executor for Recording {
    fn call_bound(&mut self, tool: &str, args_json: &str) -> Result<BoundCallOutcome, String> {
        if self.log.lock().unwrap().len() == self.restart_at {
            self.db.lock().unwrap().restart()?;
        }
        let started = std::time::Instant::now();
        let result = self.inner.call_bound(tool, args_json);
        self.log.lock().unwrap().push(Recorded {
            packet_args: args_json.to_string(),
            result: result.clone(),
            wall: started.elapsed(),
        });
        result
    }
}

/// Local OTLP capture endpoint. HyperDX itself is NOT exercised: this sink only
/// acknowledges and counts the harness's telemetry writes.
fn start_otlp_sink() -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let seen = bodies.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let seen = seen.clone();
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 8192];
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                while let Ok(n) = stream.read(&mut chunk) {
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&buf).to_string();
                    if let Some(h) = text.find("\r\n\r\n") {
                        let len = text[..h]
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if buf.len() >= h + 4 + len {
                            let line = text[..h].lines().next().unwrap_or("").to_string();
                            seen.lock()
                                .unwrap()
                                .push(format!("{line} {}", &text[h + 4..]));
                            let _ = stream.write_all(
                                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                            );
                            break;
                        }
                    }
                }
            });
        }
    });
    (port, bodies)
}

fn packet(
    id: &str,
    tool: &str,
    args: &str,
    declared: bool,
    judge: &str,
    deps: Vec<String>,
) -> Packet {
    Packet {
        id: id.into(),
        task: format!("governed step {id}"),
        depends_on: deps,
        declared_context_tokens: 100,
        tool_calls: vec![ToolCall {
            tool: tool.into(),
            args_json: args.into(),
            declared,
        }],
        judges: vec![JudgeRef {
            id: judge.into(),
            kind: "contains".into(),
        }],
        branch: None,
        retry_policy: RetryPolicy {
            max_rounds: 2,
            material_change_required: true,
        },
        status: PacketStatus::Pending,
        attempts: 0,
        judge_joined: false,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "30 governed steps on frozen Glimmer: isolated server, real MCP child, SurrealKV restart"]
async fn governed_run_30_steps_audit() -> Result<(), String> {
    let schema_dir = PathBuf::from(
        std::env::var("GLOBALMIND_SCHEMA_DIR").map_err(|_| "GLOBALMIND_SCHEMA_DIR required")?,
    );
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let out_path = std::env::var("GOVERNED_RUN_REPORT").unwrap_or_default();
    // Telemetry goes to a local sink; never read the operator's secrets.env.
    let (sink_port, sink) = start_otlp_sink();
    let scratch_home = std::env::temp_dir().join(format!("gov-run-home-{}", std::process::id()));
    fs::create_dir_all(&scratch_home).map_err(|e| e.to_string())?;
    let real_home = std::env::var("HOME").unwrap_or_default();
    std::env::set_var("HOME", &scratch_home);
    std::env::set_var("HYPERDX_API_KEY", "local-sink-key");
    std::env::set_var(
        "HYPERDX_OTLP_ENDPOINT",
        format!("http://127.0.0.1:{sink_port}"),
    );
    std::env::set_var("ARES_GLIMMER_MAX_TOKENS", "4");
    let telemetry = HyperdxTelemetry::new(HyperdxConfig::from_env())?;
    // The key is read once at config time; Surreal and the MCP child need the real HOME.
    std::env::set_var("HOME", real_home);

    let db = IsolatedSurreal::start()?;
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let mut digests = Vec::new();
    let mut queries = Vec::new();
    for (k, text) in RUN_ATOMS.iter().enumerate() {
        let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
        let a = 0.2 + 0.35 * k as f64;
        let (t, x) = (a.cosh(), a.sinh());
        db.sql(&format!(
            "CREATE atom:`run_atom_{k}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'run_atom_{k}', content_kind: 'text', content_text: '{text}', content_uri: NONE, content_digest: '{digest}', domain: 'governed-run', rank: 0, salience: 1.0, provenance: 'governed-run-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`smoke_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`run_atom_{k}`, h3_lorentz: [{t:.17}, {x:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
        ).replace("smoke_addr", &format!("run_addr_{k}")))?;
        digests.push(digest);
        queries.push(
            serde_json::json!({"workspace":"globalmind","query_h3":[t,x,0.0,0.0],
                "query_t2":[0.1,0.2],"query_provenance":"governed-run","k":1})
            .to_string(),
        );
    }

    // KV pages reference an address record; the adapter's fixed id must exist.
    db.sql(&format!(
        "CREATE atom_address:`smoke_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`run_atom_0`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: false }} RETURN NONE;",
        0.2f64.cosh(),
        0.2f64.sinh()
    ))
    .ok();

    // Plan: 30 real recalls (6 distinct prefixes then repeats) + 3 must-fail steps.
    let mut packets = Vec::new();
    let mut expected_atom = Vec::new();
    for i in 0..30usize {
        let atom = if i < 6 { i } else { (i * 5 + 1) % 6 };
        let deps = if i > 0 && i % 5 == 0 {
            vec![format!("step-{:02}", i - 1)]
        } else {
            vec![]
        };
        packets.push(packet(
            &format!("step-{i:02}"),
            "mind.recall",
            &queries[atom],
            true,
            &format!("judge-atom-{atom}"),
            deps,
        ));
        expected_atom.push(atom);
    }
    packets.push(packet(
        "neg-undeclared",
        "mind.recall",
        &queries[0],
        false,
        "judge-atom-0",
        vec![],
    ));
    packets.push(packet(
        "neg-unsupported",
        "shell.exec",
        "{}",
        true,
        "judge-atom-0",
        vec![],
    ));
    packets.push(packet(
        "neg-malformed",
        "mind.recall",
        "not json",
        true,
        "judge-atom-0",
        vec![],
    ));
    let mut plan = Plan {
        id: "governed-run-30-2026-11-04".into(),
        objective:
            "30 governed mind.recall steps through native tokenize, exact K/V and exact forward"
                .into(),
        packets,
    };
    plan.validate()?;

    let mut engine = Engine::new();
    for (k, text) in RUN_ATOMS.iter().enumerate() {
        engine.register_judge(Judge {
            id: format!("judge-atom-{k}"),
            kind: JudgeKind::Contains {
                needle: (*text).to_string(),
            },
        });
    }
    let mut gate = CapabilityGate::default();
    for tool in [
        "map.query",
        "map.kv_fetch",
        "map.ingest",
        "mind.recall",
        "mind.kv_exact",
    ] {
        gate.register(CapabilityNode {
            name: tool.into(),
            family: "map".into(),
            judged_atom: true,
            receipt_present: true,
        });
    }

    let roster: Roster = Arc::new(Mutex::new(HashMap::new()));
    let kv = StoreKv {
        store: db.store()?,
        roster,
        schema_dir: schema_dir.clone(),
        tppe_variant: "v1",
    };
    let inner = executor(&db, Some(kv))?;
    let db = Arc::new(Mutex::new(db));
    let log = Arc::new(Mutex::new(Vec::new()));
    let mut rec = Recording {
        inner,
        log: log.clone(),
        db: db.clone(),
        restart_at: 15,
    };
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(100_000.0);
    let started = std::time::Instant::now();
    let report = engine
        .run_sequential_live(
            &mut plan,
            &mut receipts,
            &mut budget,
            &mut rec,
            &telemetry,
            &gate,
        )
        .await;
    let elapsed = started.elapsed();

    // ---------------- audit: fail on any missing evidence ----------------
    let mut audit = Vec::new();
    let mut fail = |m: String| audit.push(m);
    let calls = log.lock().unwrap().clone();
    let total = plan.packets.len();
    let accounted =
        report.passed.len() + report.escalated.len() + report.failed.len() + report.skipped.len();
    if accounted != total {
        fail(format!(
            "accounting: {accounted} of {total} packets reported"
        ));
    }
    for id in ["neg-undeclared", "neg-unsupported", "neg-malformed"] {
        if report.passed.contains(&id.to_string()) {
            fail(format!("{id} must not pass"));
        }
    }
    let good_calls: Vec<&Recorded> = calls.iter().filter(|c| c.result.is_ok()).collect();
    if report.passed.len() != 30 || good_calls.len() != 30 {
        fail(format!(
            "expected 30 passed packets and 30 successful executor calls, got {} and {}",
            report.passed.len(),
            good_calls.len()
        ));
    }
    if calls
        .iter()
        .any(|c| c.packet_args == "not json" || c.packet_args == "{}")
    {
        fail("a declared-bad packet reached the executor".into());
    }
    if receipts.verify_chain().is_err() {
        fail("mission receipt chain does not verify".into());
    }
    let mut seen_prefix = std::collections::HashSet::new();
    let (mut hits, mut misses) = (0usize, 0usize);
    let mut reference: HashMap<usize, Vec<u32>> = HashMap::new();
    let mut composites = std::collections::HashSet::new();
    let mut chains = Vec::new();
    for (i, call) in good_calls.iter().enumerate() {
        let atom = expected_atom[i];
        let out = call.result.as_ref().unwrap();
        let r = &out.bound_receipt;
        if r.validate().is_err() {
            fail(format!("step {i}: bound receipt invalid"));
        }
        if r.input_bytes.bytes != RUN_ATOMS[atom].as_bytes() {
            fail(format!(
                "step {i}: model input is not atom {atom}'s exact text"
            ));
        }
        if !out.inner_text.contains(&digests[atom]) {
            fail(format!(
                "step {i}: recall evidence lacks atom {atom}'s content digest"
            ));
        }
        let names: Vec<&str> = r.tool_results.iter().map(|t| t.tool.as_str()).collect();
        let first = seen_prefix.insert(atom);
        let want_kv = if first {
            "native.kv-prefill"
        } else {
            "native.kv-hit"
        };
        if !names.contains(&want_kv) || names.last() != Some(&"native.continue-exact") {
            fail(format!(
                "step {i}: expected {want_kv} then continue-exact, got {names:?}"
            ));
        }
        if first {
            misses += 1
        } else {
            hits += 1
        }
        match reference.get(&atom) {
            Some(ids) if *ids != r.generated_token_ids => fail(format!(
                "step {i}: atom {atom} generation differs from its first step"
            )),
            None => {
                reference.insert(atom, r.generated_token_ids.clone());
            }
            _ => {}
        }
        composites.insert(r.model_artifact.composite_sha256.clone());
        chains.push(r.chain_hash.clone());
    }
    if composites.len() != 1 {
        fail("model artifact composite changed within the run".into());
    }
    // Every PASS receipt must carry the exact bound chain hashes recorded above.
    let pass_chains: Vec<String> = receipts
        .entries
        .iter()
        .filter(|e| e.outcome == "PASS")
        .flat_map(|e| e.n3_receipts.clone())
        .collect();
    if pass_chains != chains {
        fail("PASS receipts do not carry the executor's bound chain hashes in order".into());
    }
    let telemetry_events = sink.lock().unwrap().len();
    if telemetry_events < 30 {
        fail(format!(
            "only {telemetry_events} telemetry writes for 30 steps"
        ));
    }
    let mut lat: Vec<f64> = good_calls.iter().map(|c| c.wall.as_secs_f64()).collect();
    lat.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let gen_tokens: usize = good_calls
        .iter()
        .map(|c| {
            c.result
                .as_ref()
                .unwrap()
                .bound_receipt
                .generated_token_ids
                .len()
        })
        .sum();
    let summary = format!(
        "GOVERNED RUN {}\nsteps_passed={} escalated={:?} skipped={} failed={}\nkv_misses={misses} kv_hits={hits} (expected 6 and 24)\n\
         telemetry_writes={telemetry_events} (local OTLP sink, not HyperDX)\nreceipt_entries={} chain_verified={}\n\
         latency_s min={:.1} median={:.1} max={:.1} total_wall_s={:.0}\ngenerated_tokens={gen_tokens} energy_spent_j={:.1}\n\
         model_composite={}\naudit_failures={}\n{}",
        plan.id,
        report.passed.len(),
        report.escalated,
        report.skipped.len(),
        report.failed.len(),
        receipts.entries.len(),
        receipts.verify_chain().is_ok(),
        lat.first().unwrap_or(&0.0),
        lat.get(lat.len() / 2).unwrap_or(&0.0),
        lat.last().unwrap_or(&0.0),
        elapsed.as_secs_f64(),
        budget.spent_j,
        composites.iter().next().cloned().unwrap_or_default(),
        audit.len(),
        audit.join("\n"),
    );
    println!("{summary}");
    if !out_path.is_empty() {
        fs::write(&out_path, &summary).map_err(|e| e.to_string())?;
    }
    if !audit.is_empty() {
        return Err(format!("audit failed: {audit:?}"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Quality: the same question with, without, and with a mismatched recalled atom
// ---------------------------------------------------------------------------

/// Invented facts the frozen model cannot know, so a correct answer must come
/// from the recalled atom. (atom text, question, expected answer)
const FACTS: [(&str, &str, &str); 6] = [
    (
        "The Zorblax reactor at Quillan Station vents coolant every 47 minutes.",
        "How often does the Zorblax reactor at Quillan Station vent coolant?",
        " 47",
    ),
    (
        "Captain Elowen Marsh named her ship the Tern of Halvard.",
        "What did Captain Elowen Marsh name her ship?",
        " Tern",
    ),
    (
        "The village of Brindlemoor paints every door violet in the month of March.",
        "What color are the doors in Brindlemoor?",
        " violet",
    ),
    (
        "Professor Okonkwo-Reyes discovered the mineral talvorite in the Kesh Mountains.",
        "Where did Professor Okonkwo-Reyes discover talvorite?",
        " Kesh",
    ),
    (
        "The annual Gantry Lantern Festival is held on the ninth day of Harvest Moon.",
        "On which day is the Gantry Lantern Festival held?",
        " ninth",
    ),
    (
        "Vessel Q-17 carries exactly 312 crates of saffron to Port Alderon.",
        "How many crates of saffron does Vessel Q-17 carry?",
        " 312",
    ),
];

fn native_tokens(url: &str, text: &str, bos: bool) -> Result<Vec<u32>, String> {
    let req = encode_token_request(text, bos)?;
    let resp = reqwest::blocking::Client::new()
        .post(format!("{url}/v1/tokenize-native"))
        .body(req.clone())
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    Ok(decode_token_response(&req, &resp)?.token_ids)
}

fn native_generate(url: &str, prompt: &[u32], max: usize) -> Result<Vec<u32>, String> {
    let req = encode_forward_request(prompt, max)?;
    let resp = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| e.to_string())?
        .post(format!("{url}/v1/forward-exact"))
        .body(req.clone())
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    Ok(decode_forward_response(&req, &resp)?.output_ids)
}

/// Judge independent of the model: the planted answer's token sequence must
/// appear contiguously in the generated tokens.
fn answered(generated: &[u32], expected: &[u32]) -> bool {
    !expected.is_empty() && generated.windows(expected.len()).any(|w| w == expected)
}

#[test]
#[ignore = "isolated frozen-Glimmer server, real MCP recall over disposable Surreal"]
fn recalled_atoms_improve_answers_on_novel_facts() -> Result<(), String> {
    let schema_dir = PathBuf::from(
        std::env::var("GLOBALMIND_SCHEMA_DIR").map_err(|_| "GLOBALMIND_SCHEMA_DIR required")?,
    );
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let out_path = std::env::var("QUALITY_REPORT").unwrap_or_default();
    let db = IsolatedSurreal::start()?;
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let mut queries = Vec::new();
    for (k, (text, _, _)) in FACTS.iter().enumerate() {
        let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
        let a = 0.2 + 0.35 * k as f64;
        let (t, x) = (a.cosh(), a.sinh());
        db.sql(&format!(
            "CREATE atom:`fact_{k}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'fact_{k}', content_kind: 'text', content_text: '{text}', content_uri: NONE, content_digest: '{digest}', domain: 'novel-facts', rank: 0, salience: 1.0, provenance: 'novel-fact-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`fact_addr_{k}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`fact_{k}`, h3_lorentz: [{t:.17}, {x:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
        ))?;
        queries.push(
            serde_json::json!({"workspace":"globalmind","query_h3":[t,x,0.0,0.0],
                "query_t2":[0.1,0.2],"query_provenance":"quality-test","k":1})
            .to_string(),
        );
    }
    let authority = McpAuthority {
        base: db.endpoint(),
        ns: NS.into(),
        db: DB.into(),
    };
    let mut mind = MindRecallExecutor::from_state(authority, ("root".into(), db.pass.clone()))?;

    // Real recall through the MCP child; text is digest-verified before use.
    let mut recalled = Vec::new();
    for (k, q) in queries.iter().enumerate() {
        let out = mind.call("mind.recall", q)?;
        let text = verified_recall_text(&out.inner_text)?;
        if text != FACTS[k].0 {
            return Err(format!(
                "recall for fact {k} returned the wrong atom: {text:?}"
            ));
        }
        recalled.push(text);
    }

    let mut rows = Vec::new();
    let (mut with_ok, mut without_ok, mut mismatch_ok) = (0, 0, 0);
    for (k, (_, question, expected)) in FACTS.iter().enumerate() {
        let expected_ids = native_tokens(&url, expected, false)?;
        let arm = |context: Option<&str>| -> Result<(Vec<u32>, bool), String> {
            let prompt = match context {
                Some(c) => format!("{c}\nQuestion: {question}\nAnswer:"),
                None => format!("Question: {question}\nAnswer:"),
            };
            let ids = native_tokens(&url, &prompt, true)?;
            let generated = native_generate(&url, &ids, 6)?;
            let ok = answered(&generated, &expected_ids);
            Ok((generated, ok))
        };
        let (g_with, ok_with) = arm(Some(&recalled[k]))?;
        let (g_without, ok_without) = arm(None)?;
        let (g_mis, ok_mis) = arm(Some(&recalled[(k + 1) % FACTS.len()]))?;
        with_ok += ok_with as usize;
        without_ok += ok_without as usize;
        mismatch_ok += ok_mis as usize;
        rows.push(format!(
            "fact {k} expected_ids={expected_ids:?}\n  with_atom    ok={ok_with} generated={g_with:?}\n  without_atom ok={ok_without} generated={g_without:?}\n  wrong_atom   ok={ok_mis} generated={g_mis:?}"
        ));
    }
    let n = FACTS.len();
    let report = format!(
        "QUALITY: novel facts, same question with / without / mismatched recalled atom\n\
         answered with_atom={with_ok}/{n} without_atom={without_ok}/{n} wrong_atom={mismatch_ok}/{n}\n\
         judge: planted-answer token sequence must appear in the 6 generated tokens (model-independent)\n{}",
        rows.join("\n")
    );
    println!("{report}");
    if !out_path.is_empty() {
        fs::write(&out_path, &report).map_err(|e| e.to_string())?;
    }
    if with_ok <= without_ok {
        return Err(format!(
            "no gain from recalled atoms: with={with_ok} without={without_ok}"
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Proof 1: planned-prefill recall vs full recompute (same seat, same inputs)
// ---------------------------------------------------------------------------

fn timed_post(url: &str, path: &str, body: Vec<u8>) -> Result<(Vec<u8>, Duration), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(900))
        .build()
        .map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let resp = client
        .post(format!("{url}{path}"))
        .body(body)
        .send()
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("{path}: {}", resp.status()));
    }
    let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
    Ok((bytes, started.elapsed()))
}

/// Full-recompute reference: output ids, logits/kv digests, expected ids, last prompt token.
type FullRef = (Vec<u32>, [u8; 32], [u8; 32], Vec<u32>, u32);

#[derive(Default, Clone)]
struct Proof1Row {
    prompt_tokens: usize,
    full: Duration,
    prefill: Duration,
    store: Duration,
    fetch: Duration,
    cont: Duration,
    same_output: bool,
    full_ok: bool,
    recall_ok: bool,
}

#[test]
#[ignore = "isolated frozen-Glimmer server (all routes), disposable SurrealKV restart"]
fn proof1_planned_prefill_recall_vs_full_recompute() -> Result<(), String> {
    let schema_dir = PathBuf::from(
        std::env::var("GLOBALMIND_SCHEMA_DIR").map_err(|_| "GLOBALMIND_SCHEMA_DIR required")?,
    );
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let out_path = std::env::var("PROOF1_REPORT").unwrap_or_default();
    let mut db = IsolatedSurreal::start()?;
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    let digest = format!("{:x}", Sha256::digest(b"proof1"));
    db.sql(&format!(
        "CREATE atom:`smoke_atom` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'smoke_atom', content_kind: 'text', content_text: 'proof1', content_uri: NONE, content_digest: '{digest}', domain: 'proof1', rank: 0, salience: 1.0, provenance: 'proof1', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`smoke_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`smoke_atom`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;",
        0.5f64.cosh(),
        0.5f64.sinh()
    ))?;
    let roster: Roster = Arc::new(Mutex::new(HashMap::new()));
    let mut kv = StoreKv {
        store: db.store()?,
        roster: roster.clone(),
        schema_dir: schema_dir.clone(),
        tppe_variant: "v1",
    };
    const GEN: usize = 6;
    let mut rows: Vec<Proof1Row> = vec![Proof1Row::default(); FACTS.len()];
    let mut keys = Vec::new();
    let mut shapes = Vec::new();
    let mut refs: Vec<FullRef> = Vec::new();

    // Phase 1: full recompute (control), then the planned prefill + persist.
    for (k, (atom, question, expected)) in FACTS.iter().enumerate() {
        let prompt = format!("{atom}\nQuestion: {question}\nAnswer:");
        let ids = native_tokens(&url, &prompt, true)?;
        if ids.len() < 2 || ids.len() - 1 > 48 {
            return Err(format!(
                "fact {k}: prompt has {} tokens, outside 2..=49",
                ids.len()
            ));
        }
        let expected_ids = native_tokens(&url, expected, false)?;
        let (full_bytes, t_full) = timed_post(
            &url,
            "/v1/forward-exact",
            encode_forward_request(&ids, GEN)?,
        )?;
        let full = decode_forward_response(&encode_forward_request(&ids, GEN)?, &full_bytes)?;
        let (prefix, _rest) = ids.split_at(ids.len() - 1);
        let preq = encode_prefill_request(prefix)?;
        let (pbytes, t_prefill) = timed_post(&url, "/v1/prefill-exact", preq.clone())?;
        let pre = decode_prefill_response(&preq, &pbytes)?;
        let key = KvKey {
            artifact_composite: pre.composite,
            prefix_ids: prefix.to_vec(),
        };
        let started = std::time::Instant::now();
        kv.store(&key, &pre.shape, &pre.pages)?;
        let t_store = started.elapsed();
        rows[k] = Proof1Row {
            prompt_tokens: ids.len(),
            full: t_full,
            prefill: t_prefill,
            store: t_store,
            full_ok: answered(&full.output_ids, &expected_ids),
            ..Default::default()
        };
        keys.push(key);
        shapes.push(pre.shape.clone());
        refs.push((
            full.output_ids.clone(),
            full.logits,
            full.kv,
            expected_ids,
            ids[ids.len() - 1],
        ));
        eprintln!(
            "phase1 fact {k}: tokens={} full={t_full:?} prefill={t_prefill:?} store={t_store:?}",
            ids.len()
        );
    }

    // The planned work is done. Restart the store, then recall instead of re-prefilling.
    db.restart()?;
    let mut kv = StoreKv {
        store: db.store()?,
        roster,
        schema_dir,
        tppe_variant: "v1",
    };
    for (k, key) in keys.iter().enumerate() {
        let started = std::time::Instant::now();
        let (shape, pages) = kv
            .fetch(key)?
            .ok_or_else(|| format!("fact {k}: planned pages missing after restart"))?;
        let t_fetch = started.elapsed();
        if shape != shapes[k] {
            return Err(format!("fact {k}: shape changed across restart"));
        }
        let (full_ids, full_logits, full_kv, expected_ids, last) = &refs[k];
        let creq = encode_continue_request(&shape, &pages, &[*last], GEN)?;
        let (cbytes, t_cont) = timed_post(&url, "/v1/continue-exact", creq.clone())?;
        let cont = decode_forward_response(&creq, &cbytes)?;
        let same =
            cont.output_ids == *full_ids && cont.logits == *full_logits && cont.kv == *full_kv;
        rows[k].fetch = t_fetch;
        rows[k].cont = t_cont;
        rows[k].same_output = same;
        rows[k].recall_ok = answered(&cont.output_ids, expected_ids);
        eprintln!("phase2 fact {k}: fetch={t_fetch:?} continue={t_cont:?} identical={same}");
    }

    let secs = |d: Duration| d.as_secs_f64();
    let mut lines = vec![
        "PROOF 1: planned-prefill recall vs full recompute (frozen Muse Glimmer, same seat, same inputs)".to_string(),
        format!("{:<4} {:>6} {:>9} {:>9} {:>8} {:>8} {:>9} {:>9} {:>8} {:>5} {:>5} {:>5}",
            "fact", "tokens", "full_s", "prefill_s", "store_s", "fetch_s", "cont_s", "recall_s", "speedup", "same", "fOK", "rOK"),
    ];
    let (mut tf, mut tr, mut tp, mut ts) = (0.0, 0.0, 0.0, 0.0);
    for (k, r) in rows.iter().enumerate() {
        let recall = secs(r.fetch) + secs(r.cont);
        tf += secs(r.full);
        tr += recall;
        tp += secs(r.prefill);
        ts += secs(r.store);
        lines.push(format!(
            "{:<4} {:>6} {:>9.2} {:>9.2} {:>8.3} {:>8.3} {:>9.2} {:>9.2} {:>7.1}x {:>5} {:>5} {:>5}",
            k, r.prompt_tokens, secs(r.full), secs(r.prefill), secs(r.store), secs(r.fetch),
            secs(r.cont), recall, secs(r.full) / recall, r.same_output, r.full_ok, r.recall_ok
        ));
    }
    let saved = tf - tr;
    let reuses = if saved > 0.0 {
        (tp + ts) / saved
    } else {
        f64::INFINITY
    };
    lines.push(format!(
        "TOTAL full={tf:.1}s recall(fetch+continue)={tr:.1}s planned_prefill+persist={:.1}s\nrecall speedup per reuse = {:.2}x; \
         prefill+persist is repaid after {:.1} reuses (all reuses beyond that are net savings)\n\
         identical output+logits+kv digests in {}/{} cases; answered full={} recall={}",
        tp + ts,
        tf / tr,
        reuses,
        rows.iter().filter(|r| r.same_output).count(),
        rows.len(),
        rows.iter().filter(|r| r.full_ok).count(),
        rows.iter().filter(|r| r.recall_ok).count(),
    ));
    let report = lines.join("\n");
    println!("{report}");
    if !out_path.is_empty() {
        fs::write(&out_path, &report).map_err(|e| e.to_string())?;
    }
    if rows.iter().any(|r| !r.same_output) {
        return Err("recall diverged from full recompute".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Carved swarm demo: four scoped workers, one Glimmer server, one set of weights
// ---------------------------------------------------------------------------

fn detokenize(url: &str, ids: &[u32]) -> Result<String, String> {
    let mut req = b"GDT2".to_vec();
    req.extend_from_slice(&(ids.len() as u32).to_le_bytes());
    for id in ids {
        req.extend_from_slice(&id.to_le_bytes());
    }
    let resp = reqwest::blocking::Client::new()
        .post(format!("{url}/v1/detokenize-native"))
        .body(req.clone())
        .send()
        .map_err(|e| e.to_string())?
        .bytes()
        .map_err(|e| e.to_string())?;
    if resp.len() < 40 || &resp[..4] != b"GDR2" {
        return Err("detokenize response header invalid".into());
    }
    let len = u32::from_le_bytes(resp[4..8].try_into().unwrap()) as usize;
    if resp.len() != 40 + len {
        return Err("detokenize response length invalid".into());
    }
    let mut bound = req;
    bound.extend_from_slice(&resp[..resp.len() - 32]);
    if Sha256::digest(&bound)[..] != resp[resp.len() - 32..] {
        return Err("detokenize receipt mismatch".into());
    }
    Ok(String::from_utf8_lossy(&resp[8..8 + len]).to_string())
}

/// (worker name, well ordinal, fact indices carved into that worker's well)
const SWARM: [(&str, usize, &[usize]); 4] = [
    ("reactor-ops", 0, &[0, 5]),
    ("fleet-nav", 1, &[1, 4]),
    ("village-lore", 2, &[2]),
    ("geology", 3, &[3]),
];

#[test]
#[ignore = "isolated frozen-Glimmer server, real MCP recall_scoped over disposable Surreal"]
fn carved_swarm_four_workers_one_weight_set() -> Result<(), String> {
    let schema_dir = PathBuf::from(
        std::env::var("GLOBALMIND_SCHEMA_DIR").map_err(|_| "GLOBALMIND_SCHEMA_DIR required")?,
    );
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let out_path = std::env::var("SWARM_REPORT").unwrap_or_default();
    let db = IsolatedSurreal::start()?;
    for name in SCHEMAS {
        db.sql(&fs::read_to_string(schema_dir.join(name)).map_err(|e| e.to_string())?)?;
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;")?;
    db.sql("UPSERT geometry_space:`h3_t2_r12_v1` CONTENT { workspace: workspace:`globalmind` } RETURN NONE;")
        .ok();
    let well_digest = format!("{:x}", Sha256::digest(b"swarm-well-contract"));
    let addr_of = |k: usize| 0.2 + 0.35 * k as f64;
    // Carve: atoms, addresses, wells, memberships.
    for (k, (text, _, _)) in FACTS.iter().enumerate() {
        let digest = format!("{:x}", Sha256::digest(text.as_bytes()));
        let a = addr_of(k);
        db.sql(&format!(
            "CREATE atom:`swarm_atom_{k}` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'swarm_atom_{k}', content_kind: 'text', content_text: '{text}', content_uri: NONE, content_digest: '{digest}', domain: 'swarm', rank: 0, salience: 1.0, provenance: 'swarm-demo-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; CREATE atom_address:`swarm_addr_{k}` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`swarm_atom_{k}`, h3_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;",
            a.cosh(), a.sinh()
        ))?;
    }
    for (name, ordinal, facts) in SWARM {
        let a = 0.3 + 0.4 * ordinal as f64;
        db.sql(&format!(
            "CREATE cognitive_well:`swarm_well_{ordinal}` CONTENT {{ workspace: workspace:`globalmind`, geometry_space: geometry_space:`h3_t2_r12_v1`, ordinal: {ordinal}, label: '{name}', h3_center_lorentz: [{:.17}, {:.17}, 0.0, 0.0], t2_sector: {ordinal}, boundary_contract_digest: '{well_digest}', status: 'specified' }} RETURN NONE;",
            a.cosh(), a.sinh()
        ))?;
        for f in facts {
            let cd = format!(
                "{:x}",
                Sha256::digest(format!("cls-{ordinal}-{f}").as_bytes())
            );
            db.sql(&format!(
                "CREATE address_well_membership CONTENT {{ workspace: workspace:`globalmind`, address: atom_address:`swarm_addr_{f}`, well: cognitive_well:`swarm_well_{ordinal}`, classifier_version: 'swarm-demo-v1', classifier_digest: '{cd}', boundary_receipt_digest: '{well_digest}' }} RETURN NONE;"
            ))?;
        }
    }
    let authority = McpAuthority {
        base: db.endpoint(),
        ns: NS.into(),
        db: DB.into(),
    };
    let mut mind = MindRecallExecutor::from_state(authority, ("root".into(), db.pass.clone()))?;

    let mut lines = vec![
        "CARVED SWARM DEMO: 4 scoped workers + 1 lead, ONE frozen Muse Glimmer server (one weight set)".to_string(),
        "Each worker may recall only atoms in its own well (mind.recall_scoped); every answer is generated by the same weights.".to_string(),
    ];
    let mut composites = std::collections::HashSet::new();
    let mut score = (0usize, 0usize, 0usize, 0usize); // own ok, own n, cross-blocked ok, cross n
    let mut ask = |who: &str, wells: &[usize], target: usize, kind: &str| -> Result<bool, String> {
        let a = addr_of(target);
        let args = serde_json::json!({
            "workspace": "globalmind",
            "well_ids": wells.iter().map(|w| format!("swarm_well_{w}")).collect::<Vec<_>>(),
            "query_h3": [a.cosh(), a.sinh(), 0.0, 0.0], "query_t2": [0.1, 0.2],
            "query_provenance": "swarm-demo", "k": 1
        })
        .to_string();
        let out = mind.call("mind.recall_scoped", &args)?;
        let atom_text = verified_recall_text(&out.inner_text)?;
        let atom_id = serde_json::from_str::<serde_json::Value>(&out.inner_text)
            .ok()
            .and_then(|v| v["members"][0]["atom"].as_str().map(str::to_string))
            .unwrap_or_default();
        let (_, question, expected) = FACTS[target];
        let prompt = format!("{atom_text}\nQuestion: {question}\nAnswer:");
        let ids = native_tokens(&url, &prompt, true)?;
        let req = encode_forward_request(&ids, 5)?;
        let started = std::time::Instant::now();
        let resp = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(900))
            .build()
            .map_err(|e| e.to_string())?
            .post(format!("{url}/v1/forward-exact"))
            .body(req.clone())
            .send()
            .map_err(|e| e.to_string())?
            .bytes()
            .map_err(|e| e.to_string())?;
        let r = decode_forward_response(&req, &resp)?;
        let wall = started.elapsed();
        composites.insert(r.composite);
        let answer = detokenize(&url, &r.output_ids)?;
        let expected_ids = native_tokens(&url, expected, false)?;
        let ok = answered(&r.output_ids, &expected_ids);
        lines.push(format!(
            "[{who:<12}] {kind:<9} Q: {question}\n               recalled {atom_id} (scope wells {wells:?}): \"{atom_text}\"\n               answer: {:?}  -> {}   ({:.1}s)",
            answer.trim(),
            if ok { "CORRECT" } else { "not answerable from this view" },
            wall.as_secs_f64()
        ));
        Ok(ok)
    };
    for (i, (name, ordinal, facts)) in SWARM.iter().enumerate() {
        let own = facts[0];
        let other = SWARM[(i + 1) % SWARM.len()].2[0];
        score.1 += 1;
        score.0 += ask(name, &[*ordinal], own, "own-view")? as usize;
        score.3 += 1;
        score.2 += (!ask(name, &[*ordinal], other, "cross-view")?) as usize;
    }
    // The lead holds all four wells: the union view answers the cross questions.
    let mut lead_ok = 0;
    for (i, _) in SWARM.iter().enumerate() {
        let other = SWARM[(i + 1) % SWARM.len()].2[0];
        lead_ok += ask("lead(all)", &[0, 1, 2, 3], other, "union")? as usize;
    }
    lines.push(format!(
        "SUMMARY: workers answered own-view {}/{}; cross-view blocked (not answerable) {}/{}; lead (union of wells) answered {}/4.\n\
         ONE WEIGHT SET: {} distinct artifact composite across all {} generations ({}).",
        score.0, score.1, score.2, score.3, lead_ok,
        composites.len(), score.1 + score.3 + 4,
        composites.iter().next().map(|c| c.iter().map(|b| format!("{b:02x}")).collect::<String>()).unwrap_or_default()
    ));
    let report = lines.join("\n");
    println!("{report}");
    if !out_path.is_empty() {
        fs::write(&out_path, &report).map_err(|e| e.to_string())?;
    }
    if composites.len() != 1 {
        return Err("more than one weight identity in the swarm".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Throughput: server-resident sessions vs shipping pages every turn
// ---------------------------------------------------------------------------

#[test]
#[ignore = "isolated frozen-Glimmer server with session routes"]
fn resident_sessions_equal_page_shipping_and_cut_transfer() -> Result<(), String> {
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").map_err(|_| "ARES_GLIMMER_NATIVE_URL")?;
    if url.contains(":8000") || url.contains(":8011") {
        return Err("use an isolated Glimmer server".into());
    }
    let out_path = std::env::var("RESIDENT_REPORT").unwrap_or_default();
    let (atom, q1, q2) = (
        FACTS[0].0,
        "How often does the Zorblax reactor at Quillan Station vent coolant?",
        "What number was in your last answer?",
    );
    let prefix = native_tokens(&url, &format!("{atom}\n"), true)?;
    let t1 = native_tokens(&url, &format!("Question: {q1}\nAnswer:"), false)?;
    let t2 = native_tokens(&url, &format!("Question: {q2}\nAnswer:"), false)?;
    const GEN: usize = 10;
    let mut lines = vec![
        "RESIDENT SESSIONS vs SHIPPING PAGES EVERY TURN (frozen Muse Glimmer, 2 turns)".to_string(),
    ];

    // Arm A: ship pages both ways each turn.
    let preq = encode_prefill_request(&prefix)?;
    let (pb, _) = timed_post(&url, "/v1/prefill-exact", preq.clone())?;
    let pre = decode_prefill_response(&preq, &pb)?;
    let r1q = encode_continue_request(&pre.shape, &pre.pages, &t1, GEN)?;
    let (r1b, a1_t) = timed_post(&url, "/v1/continue-session", r1q.clone())?;
    let a1 = decode_session_response(&r1q, &r1b)?;
    let mut cont2 = vec![*a1.output_ids.last().unwrap()];
    cont2.extend(&t2);
    let r2q = encode_continue_request(&a1.shape, &a1.pages, &cont2, GEN)?;
    let (r2b, a2_t) = timed_post(&url, "/v1/continue-session", r2q.clone())?;
    let a2 = decode_session_response(&r2q, &r2b)?;
    let shipped = r1q.len() + r1b.len() + r2q.len() + r2b.len();

    // Arm B: resident session; only ids and new tokens cross the wire.
    let (sb, _) = timed_post(&url, "/v1/session-prefill", preq.clone())?;
    let sid = decode_session_reply(&preq, &sb)?.id;
    let b1q = encode_session_turn(sid, &t1, GEN)?;
    let (b1b, b1_t) = timed_post(&url, "/v1/session-turn", b1q.clone())?;
    let b1 = decode_session_reply(&b1q, &b1b)?;
    let mut cont2b = vec![*b1.output_ids.last().unwrap()];
    cont2b.extend(&t2);
    let b2q = encode_session_turn(sid, &cont2b, GEN)?;
    let (b2b, b2_t) = timed_post(&url, "/v1/session-turn", b2q.clone())?;
    let b2 = decode_session_reply(&b2q, &b2b)?;
    let resident = b1q.len() + b1b.len() + b2q.len() + b2b.len();

    let same = a1.output_ids == b1.output_ids
        && a2.output_ids == b2.output_ids
        && a1.kv_digest == b1.kv_digest
        && a2.kv_digest == b2.kv_digest
        && a2.logits_digest == b2.logits_digest;
    lines.push(format!(
        "turn1: shipping {:.2}s  resident {:.2}s | turn2: shipping {:.2}s  resident {:.2}s",
        a1_t.as_secs_f64(),
        b1_t.as_secs_f64(),
        a2_t.as_secs_f64(),
        b2_t.as_secs_f64()
    ));
    lines.push(format!(
        "bytes over the wire (2 turns): shipping {:.2} MB, resident {:.4} MB ({:.0}x less)",
        shipped as f64 / 1e6,
        resident as f64 / 1e6,
        shipped as f64 / resident as f64
    ));
    lines.push(format!(
        "identical output ids, K/V digests and logits digest across both arms: {same}"
    ));
    let report = lines.join("\n");
    println!("{report}");
    if !out_path.is_empty() {
        fs::write(&out_path, &report).map_err(|e| e.to_string())?;
    }
    if !same {
        return Err("resident session diverged from page shipping".into());
    }
    Ok(())
}
