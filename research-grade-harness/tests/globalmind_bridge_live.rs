//! Explicit live bridge read test against a disposable SurrealKV database.

use globalmind_runtime::retrieval::{RetrievalConfig, RetrievalMetric, decode_bundle};
use globalmind_runtime::{
    Digest, ExactKvIdentity, GlobalMindStore, RecordId, StoreConfig, TensorDtype, TensorLayout,
    TensorPage, TensorSession,
};
use rgah::GlobalMindBridge;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SCHEMAS: [&str; 10] = [
    "0001_hypermind_graph",
    "0003_nested_hyperbolic_kv",
    "0004_transformer_kv_tensor_pages",
    "0005_universal_atom_lineage",
    "0006_immutable_atom_address",
    "0007_globalmind_distillation_trajectories",
    "0008_superposition_holding",
    "0009_immutable_tensor_kv",
    "0010_native_distillation_ingress",
    "0011_atomic_trajectory_cells",
];

struct IsolatedDb {
    child: Child,
    port: u16,
    storage: std::path::PathBuf,
}

impl Drop for IsolatedDb {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.storage);
    }
}

impl IsolatedDb {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let storage = std::env::temp_dir().join(format!(
            "rgah-globalmind-bridge-{}-{port}",
            std::process::id()
        ));
        let child = Command::new("surreal")
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
                "testpass123",
                &format!("surrealkv://{}", storage.display()),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let db = Self {
            child,
            port,
            storage,
        };
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return db;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("isolated SurrealDB failed to start");
    }

    fn sql(&self, query: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        let body = format!("USE NS rgah_bridge DB bridge_live; {query}");
        let auth = basic64("root:testpass123");
        let request = format!(
            "POST /sql HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.port,
            auth,
            body.len(),
            body
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        String::from_utf8(response).unwrap()
    }
}

fn basic64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in input.as_bytes().chunks(3) {
        let a = chunk[0] as u32;
        let b = *chunk.get(1).unwrap_or(&0) as u32;
        let c = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[test]
#[ignore = "requires installed surreal binary and disposable local SurrealKV"]
fn bridge_reads_one_real_atom_as_binary_recall() {
    let db = IsolatedDb::start();
    let schema_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../hypermind-nested-kv/schema");
    for schema in SCHEMAS {
        let text = std::fs::read_to_string(schema_root.join(format!("{schema}.surql"))).unwrap();
        let response = db.sql(&text);
        let errors = response.matches("\"status\":\"ERR\"").count();
        let allowed = usize::from(schema.starts_with("0001"));
        assert!(errors <= allowed, "schema {schema} errors: {response:.200}");
    }
    let digest = "a".repeat(64);
    let seed = format!(
        "CREATE workspace:`globalmind` CONTENT {{ slug: 'globalmind' }} RETURN NONE; \
         CREATE geometry_space:`bridge_h3t2` CONTENT {{ name: 'bridge_h3t2', manifold: 'H3xT2', quotient_symbol: 'Gamma12', quotient_status: 'specified', coordinate_model: 'lorentz-t2', contract_version: 'v1', contract_digest: '{digest}' }} RETURN NONE; \
         CREATE atom:`bridge_sample` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'bridge_sample', content_kind: 'text', content_text: 'bridge live read', content_uri: NONE, content_digest: '{digest}', domain: 'bridge', rank: 0, salience: 0.5, provenance: 'live-test', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
         CREATE atom_address:`bridge_sample` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`bridge_sample`, h3_lorentz: [1.0, 0.0, 0.0, 0.0], t2_phase: [0.0, 0.0], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
    );
    let response = db.sql(&seed);
    assert!(
        !response.contains("\"status\":\"ERR\""),
        "seed failed: {response:.200}"
    );

    let config = RetrievalConfig {
        endpoint: format!("http://127.0.0.1:{}", db.port),
        namespace: "rgah_bridge".into(),
        database: "bridge_live".into(),
        username: "root".into(),
        password: "testpass123".into(),
        workspace: RecordId::parse("globalmind").unwrap(),
        candidate_budget: 8,
        metric: RetrievalMetric::Hyperbolic,
    };
    let bridge = GlobalMindBridge::connect(config).unwrap();
    let bytes = bridge
        .recall_binary([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], "bridge-live-test", 1)
        .unwrap();
    assert!(bytes.starts_with(b"GMRC"));
    let bundle = decode_bundle(&bytes).unwrap();
    assert_eq!(bundle.members.len(), 1);
    assert_eq!(bundle.members[0].atom.as_str(), "bridge_sample");

    let rid = |name: &str| RecordId::parse(name).unwrap();
    let hash = |name: &str| Digest::sha256(name.as_bytes());
    let workspace = rid("globalmind");
    let session_id = rid("bridge_tensor_session");
    let page_id = rid("bridge_tensor_page");
    let geometry = rid("bridge_h3t2");
    let address = rid("bridge_sample");
    let key: Vec<u8> = [1.0_f32, 2.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let value: Vec<u8> = [3.0_f32, 4.0]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let key_digest = Digest::sha256(&key);
    let value_digest = Digest::sha256(&value);
    let session = TensorSession {
        id: session_id.clone(),
        workspace: workspace.clone(),
        model_id: "bridge-test-model".into(),
        model_revision: "frozen-v1".into(),
        model_digest: hash("model"),
        request_digest: hash("request"),
        prefix_digest: hash("prefix"),
        dtype: TensorDtype::F32,
        layer_count: 1,
        kv_head_count: 1,
        head_dim: 2,
        geometry_space: geometry.clone(),
        tppe_version: "tppe-layer3-candidate-v2".into(),
        tppe_configuration_digest: hash("tppe"),
        address_contract_digest: hash("address"),
        tensor_contract_digest: hash("tensor"),
    };
    let page = TensorPage {
        id: page_id.clone(),
        workspace: workspace.clone(),
        session: session_id.clone(),
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: 1,
        tensor_shape: [1, 2],
        dtype: TensorDtype::F32,
        layout: TensorLayout::TokenMajor,
        key_tensor: key.clone().into_boxed_slice(),
        value_tensor: value.clone().into_boxed_slice(),
        key_digest: key_digest.clone(),
        value_digest: value_digest.clone(),
        address: address.clone(),
        trajectory_atoms: Vec::new(),
        well: None,
        trajectory: None,
        phase_state_digest: hash("phase"),
        receipt_digest: hash("receipt"),
        revision: 1,
    };
    let store = GlobalMindStore::connect(StoreConfig {
        endpoint: format!("http://127.0.0.1:{}", db.port),
        namespace: "rgah_bridge".into(),
        database: "bridge_live".into(),
        username: Some("root".into()),
        password: Some("testpass123".into()),
        allow_quarantined_core: false,
    })
    .unwrap();
    store.create_tensor_session(&session).unwrap();
    store.put_tensor_page(&page).unwrap();
    let exact = ExactKvIdentity {
        workspace,
        session: session_id,
        page: page_id,
        model_id: session.model_id.clone(),
        model_revision: session.model_revision.clone(),
        model_digest: session.model_digest.clone(),
        request_digest: session.request_digest.clone(),
        prefix_digest: session.prefix_digest.clone(),
        tppe_version: session.tppe_version.clone(),
        tppe_configuration_digest: session.tppe_configuration_digest.clone(),
        address_contract_digest: session.address_contract_digest.clone(),
        tensor_contract_digest: session.tensor_contract_digest.clone(),
        geometry_space: geometry,
        dtype: TensorDtype::F32,
        layer_count: 1,
        kv_head_count: 1,
        head_dim: 2,
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: 1,
        tensor_shape: [1, 2],
        layout: TensorLayout::TokenMajor,
        address,
        phase_state_digest: page.phase_state_digest,
        revision: 1,
        key_digest,
        value_digest,
    };
    let prepared = bridge
        .prepare_forward_recall(
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 0.0],
            "bridge-live-test",
            1,
            &exact,
        )
        .unwrap();
    assert_eq!(
        decode_bundle(&prepared.neighborhood_gmrc)
            .unwrap()
            .members
            .len(),
        1
    );
    let fetched = prepared.tensor;
    assert_eq!(fetched.key.as_ref(), key.as_slice());
    assert_eq!(fetched.value.as_ref(), value.as_slice());
    // Both reads would independently succeed: the K/V page belongs to
    // bridge_sample, while the nearest recalled atom is bridge_decoy.
    let decoy_seed = format!(
        "CREATE atom:`bridge_decoy` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'bridge_decoy', content_kind: 'text', content_text: 'decoy', content_uri: NONE, content_digest: '{digest}', domain: 'bridge', rank: 0, salience: 0.5, provenance: 'live-test', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
         CREATE atom_address:`bridge_decoy` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`bridge_decoy`, h3_lorentz: [{}, {}, 0.0, 0.0], t2_phase: [0.0, 0.0], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;",
        1.0_f64.cosh(),
        1.0_f64.sinh()
    );
    let decoy_response = db.sql(&decoy_seed);
    assert!(
        !decoy_response.contains("\"status\":\"ERR\""),
        "decoy seed failed"
    );
    let wrong_recall = bridge
        .prepare_forward_recall(
            [1.0_f64.cosh(), 1.0_f64.sinh(), 0.0, 0.0],
            [0.0, 0.0],
            "bridge-live-test",
            1,
            &exact,
        )
        .err()
        .expect("unrelated K/V page must be refused");
    assert!(
        wrong_recall.contains("absent from the recalled neighborhood"),
        "{wrong_recall}"
    );
    let mut drifted = exact.clone();
    drifted.model_digest = hash("different-model");
    assert!(bridge.fetch_tensor_exact(&drifted).is_err());
    assert!(
        bridge
            .prepare_forward_recall(
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0],
                "bridge-live-test",
                1,
                &drifted,
            )
            .is_err()
    );
}
