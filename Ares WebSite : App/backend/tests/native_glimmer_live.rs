//! Hardware gate for `NativeGlimmerExecutor`: real `globalmind-mcp` child over a
//! disposable in-memory Surreal (never port 8000), then real binary tokenize and
//! exact forward against a running frozen-Glimmer `serve_glimmer`.
//!
//! Requires env: ARES_GLIMMER_NATIVE_URL (isolated server, not 8011),
//! ARES_GLIMMER_WEIGHTS_SHA256, ARES_MCP_MIND_BIN (built `globalmind-mcp`),
//! GLOBALMIND_SCHEMA_DIR (`hypermind-nested-kv/schema`).
//! Run: cargo test --test native_glimmer_live -- --ignored --nocapture

use ares_backend::harness::mcp::McpAuthority;
use ares_backend::harness::mind_recall_executor::MindRecallExecutor;
use ares_backend::harness::native_glimmer_executor::NativeGlimmerExecutor;
use ares_backend::harness::runtime::Executor;
use sha2::{Digest, Sha256};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const NS: &str = "globalmind_glimmer_iso";
const DB: &str = "native_exec_smoke";
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
const ATOM_TEXT: &str = "The capital of France is";

struct Surreal {
    child: Child,
    port: u16,
    pass: String,
}

impl Surreal {
    fn start() -> Self {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        drop(l);
        assert_ne!(port, 8000, "refuse live Surreal port");
        let pass = format!("ephemeral_{}", std::process::id());
        let child = Command::new("/opt/homebrew/bin/surreal")
            .args([
                "start",
                "--no-banner",
                "--log",
                "none",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--user",
                "root",
                "--pass",
                &pass,
                "memory",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn surreal");
        let me = Self { child, port, pass };
        for _ in 0..300 {
            if TcpStream::connect(("127.0.0.1", port)).is_ok() {
                return me;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("isolated Surreal did not open");
    }

    fn sql(&self, sql: &str) -> serde_json::Value {
        let c = reqwest::blocking::Client::new();
        let base = format!("http://127.0.0.1:{}", self.port);
        let signin: serde_json::Value = c
            .post(format!("{base}/signin"))
            .header("accept", "application/json")
            .json(&serde_json::json!({"user":"root","pass":self.pass}))
            .send()
            .unwrap()
            .json()
            .unwrap();
        let token = signin["token"].as_str().expect("token");
        let rows: serde_json::Value = c
            .post(format!("{base}/sql"))
            .header("accept", "application/json")
            .bearer_auth(token)
            .body(format!("USE NS {NS} DB {DB}; {sql}"))
            .send()
            .unwrap()
            .json()
            .unwrap();
        for (i, row) in rows.as_array().unwrap().iter().enumerate().skip(1) {
            assert_ne!(
                row["status"].as_str(),
                Some("ERR"),
                "stmt {i}: {}",
                row["result"]
            );
        }
        rows
    }
}

impl Drop for Surreal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn executor(db: &Surreal) -> NativeGlimmerExecutor {
    let authority = McpAuthority {
        base: format!("http://127.0.0.1:{}", db.port),
        ns: NS.into(),
        db: DB.into(),
    };
    let mind = MindRecallExecutor::from_state(authority, ("root".into(), db.pass.clone()))
        .expect("mind executor");
    NativeGlimmerExecutor::from_env(mind).expect("native executor")
}

#[test]
#[ignore = "real globalmind-mcp child, disposable Surreal, frozen-Glimmer binary server"]
fn native_executor_binds_recall_tokenize_forward() {
    let url = std::env::var("ARES_GLIMMER_NATIVE_URL").expect("ARES_GLIMMER_NATIVE_URL");
    assert!(
        !url.contains(":8000") && !url.contains(":8011"),
        "use an isolated server"
    );
    let schema_dir = std::env::var("GLOBALMIND_SCHEMA_DIR").expect("GLOBALMIND_SCHEMA_DIR");
    let db = Surreal::start();
    for name in SCHEMAS {
        db.sql(&std::fs::read_to_string(format!("{schema_dir}/{name}")).unwrap());
    }
    db.sql("UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;");
    let digest = format!("{:x}", Sha256::digest(ATOM_TEXT.as_bytes()));
    let (t, x) = (0.5f64.cosh(), 0.5f64.sinh());
    db.sql(&format!(
        "CREATE atom:`smoke_atom` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'smoke_atom', content_kind: 'text', content_text: '{ATOM_TEXT}', content_uri: NONE, content_digest: '{digest}', domain: 'forward-smoke', rank: 0, salience: 1.0, provenance: 'smoke-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
         CREATE atom_address:`smoke_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`smoke_atom`, h3_lorentz: [{t:.17}, {x:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
    ));

    let args = serde_json::json!({
        "workspace": "globalmind",
        "query_h3": [t, x, 0.0, 0.0],
        "query_t2": [0.1, 0.2],
        "query_provenance": "smoke-query",
        "k": 1
    })
    .to_string();

    let mut exec = executor(&db);
    let out = exec.call_bound("mind.recall", &args).expect("bound call");
    let r = &out.bound_receipt;
    r.validate().expect("receipt validates");
    assert_eq!(r.input_bytes.bytes, ATOM_TEXT.as_bytes());
    assert_eq!(r.tool_results.len(), 3);
    assert!(!r.generated_token_ids.is_empty());
    println!("generated_token_ids={:?}", r.generated_token_ids);
    println!("logits={} kv={}", r.logits_digest, r.kv_digest);
    println!("artifact_composite={}", r.model_artifact.composite_sha256);
    for t in &r.tool_results {
        println!("tool={} receipt={}", t.tool, t.receipt_sha256);
    }
    println!("chain_hash={}", r.chain_hash);

    // Negative control 1: wrong pinned weights digest -> refuse.
    let good_pin = std::env::var("ARES_GLIMMER_WEIGHTS_SHA256").unwrap();
    std::env::set_var("ARES_GLIMMER_WEIGHTS_SHA256", "0".repeat(64));
    let mut exec = executor(&db);
    assert!(
        exec.call_bound("mind.recall", &args).is_err(),
        "wrong weights pin must be rejected"
    );
    std::env::set_var("ARES_GLIMMER_WEIGHTS_SHA256", good_pin);
    println!("negative control: wrong weights pin refused");

    // Negative control 2: an in-range atom whose text does not match its
    // stored digest must stop the recall before any text reaches the model.
    let wrong = format!("{:x}", Sha256::digest(b"different content"));
    db.sql(&format!(
        "CREATE atom:`bad_atom` CONTENT {{ workspace: workspace:`globalmind`, external_id: 'bad_atom', content_kind: 'text', content_text: '{ATOM_TEXT}', content_uri: NONE, content_digest: '{wrong}', domain: 'forward-smoke', rank: 1, salience: 1.0, provenance: 'smoke-witness', taint: 'none', pinned: false, current_address: NONE }} RETURN NONE; \
         CREATE atom_address:`bad_addr` CONTENT {{ workspace: workspace:`globalmind`, atom: atom:`bad_atom`, h3_lorentz: [{t:.17}, {x:.17}, 0.0, 0.0], t2_phase: [0.1, 0.2], geometry_model: 'lorentz-t2', geometry_version: 'v1', revision: 1, active: true }} RETURN NONE;"
    ));
    let wide = args.replace("\"k\":1", "\"k\":2");
    let mut exec = executor(&db);
    assert!(
        exec.call_bound("mind.recall", &wide).is_err(),
        "digest-mismatched atom must not reach the model"
    );
    println!("negative control: digest-mismatched atom refused");
}
