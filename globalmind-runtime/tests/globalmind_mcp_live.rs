//! Live stdio test for globalmind-mcp. Loopback dev only.
//!
//! Credentials are discovered from the running SurrealDB process arguments
//! (the same mechanism the backend uses in `surreal.rs`), never printed.
//! The test uses an isolated database `globalmind_mcp_test` and unique
//! session/page ids per run, so it never writes to hypermind/core.
//! It is skipped (not failed) when no SurrealDB is listening on 8000.
use globalmind_runtime::{
    Digest, GlobalMindStore, RecordId, StoreConfig, TensorDtype, TensorLayout, TensorPage,
    TensorSession,
};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn discover_credentials() -> Option<(String, String)> {
    let out = std::process::Command::new("ps").args(["-eo", "command"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        if line.contains("surreal") && line.contains("--bind 127.0.0.1:8000") {
            let toks: Vec<&str> = line.split_whitespace().collect();
            let mut user = "root".to_string();
            let mut pass: Option<String> = None;
            let mut i = 0;
            while i < toks.len() {
                if toks[i] == "--user" && i + 1 < toks.len() {
                    user = toks[i + 1].to_string();
                }
                if toks[i] == "--pass" && i + 1 < toks.len() {
                    pass = Some(toks[i + 1].to_string());
                }
                i += 1;
            }
            if let Some(p) = pass {
                return Some((user, p));
            }
        }
    }
    None
}

fn spawn_mcp(env: &[(&str, &str)]) -> (Child, ChildStdin, BufReader<std::process::ChildStdout>) {
    let bin = env!("CARGO_BIN_EXE_globalmind-mcp");
    let mut cmd = Command::new(bin);
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("spawn globalmind-mcp");
    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    (child, stdin, stdout)
}

fn call(stdin: &mut ChildStdin, stdout: &mut BufReader<std::process::ChildStdout>, _id: u64, req: Value) -> Value {
    let mut line = req.to_string();
    line.push('\n');
    stdin.write_all(line.as_bytes()).unwrap();
    stdin.flush().unwrap();
    let mut out = String::new();
    stdout.read_line(&mut out).unwrap();
    serde_json::from_str(&out).expect("mcp response json")
}

fn inner_text(resp: &Value) -> Value {
    let text = resp["result"]["content"][0]["text"].as_str().expect("text content");
    serde_json::from_str(text).expect("inner json")
}

#[test]
fn globalmind_mcp_live() {
    let Some((user, pass)) = discover_credentials() else {
        eprintln!("SKIP: no SurrealDB process found on 8000");
        return;
    };
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let session_id = format!("mcp_test_session_{}", stamp);
    let page_id = format!("mcp_test_page_{}", stamp);
    let key: Vec<u8> = (0u8..16).collect();
    let value: Vec<u8> = (100u8..116).collect();
    let d = |s: &str| Digest::parse(s).unwrap();
    let zero = d(&"0".repeat(64));
    let key_digest = Digest::sha256(&key);
    let value_digest = Digest::sha256(&value);
    let cfg = StoreConfig {
        endpoint: "http://127.0.0.1:8000".into(),
        namespace: "hypermind".into(),
        database: "globalmind_mcp_test".into(),
        username: Some(user.clone()),
        password: Some(pass.clone()),
        allow_quarantined_core: false,
    };
    let store = GlobalMindStore::connect(cfg).expect("store connect");
    let session = TensorSession {
        id: RecordId::parse(session_id.clone()).unwrap(),
        workspace: RecordId::parse("hypermind").unwrap(),
        model_id: "mcp-test-model".into(),
        model_revision: "rev1".into(),
        model_digest: zero.clone(),
        request_digest: zero.clone(),
        prefix_digest: zero.clone(),
        dtype: TensorDtype::F16,
        layer_count: 2,
        kv_head_count: 2,
        head_dim: 4,
        geometry_space: RecordId::parse("geom1").unwrap(),
        tppe_version: "tppe-v1".into(),
        tppe_configuration_digest: zero.clone(),
        address_contract_digest: zero.clone(),
        tensor_contract_digest: zero.clone(),
    };
    store.create_tensor_session(&session).expect("create session");
    let page = TensorPage {
        id: RecordId::parse(page_id.clone()).unwrap(),
        workspace: RecordId::parse("hypermind").unwrap(),
        session: RecordId::parse(session_id.clone()).unwrap(),
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: 2,
        tensor_shape: [2, 4],
        dtype: TensorDtype::F16,
        layout: TensorLayout::TokenMajor,
        key_tensor: key.clone().into_boxed_slice(),
        value_tensor: value.clone().into_boxed_slice(),
        key_digest: key_digest.clone(),
        value_digest: value_digest.clone(),
        address: RecordId::parse("1").unwrap(),
        trajectory_atoms: vec![],
        well: None,
        trajectory: None,
        phase_state_digest: zero.clone(),
        receipt_digest: zero.clone(),
        revision: 1,
    };
    store.put_tensor_page(&page).expect("put page");

    let env: Vec<(&str, &str)> = vec![
        ("GLOBALMIND_SURREAL_USER", &user),
        ("GLOBALMIND_SURREAL_PASS", &pass),
        ("GLOBALMIND_SURREAL_NS", "hypermind"),
        ("GLOBALMIND_SURREAL_DB", "globalmind_mcp_test"),
    ];
    let (mut child, mut stdin, mut stdout) = spawn_mcp(&env);
    let init = call(&mut stdin, &mut stdout, 1, json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}));
    assert_eq!(init["result"]["serverInfo"]["name"], "globalmind-mcp");
    let list = call(&mut stdin, &mut stdout, 2, json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    let names: Vec<&str> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"mind.recall"));
    assert!(names.contains(&"mind.kv_exact"));
    assert!(names.contains(&"mind.db_status"));
    assert!(names.contains(&"mind.automaton_decide"));

    let kv_args = json!({
        "workspace": "hypermind", "session": session_id, "page": page_id,
        "model_id": "mcp-test-model", "model_revision": "rev1",
        "model_digest": zero.as_str(), "request_digest": zero.as_str(), "prefix_digest": zero.as_str(),
        "tppe_version": "tppe-v1", "tppe_configuration_digest": zero.as_str(),
        "address_contract_digest": zero.as_str(), "tensor_contract_digest": zero.as_str(),
        "geometry_space": "geom1", "dtype": "f16",
        "layer_count": 2, "kv_head_count": 2, "head_dim": 4,
        "layer": 0, "kv_head": 0, "token_start": 0, "token_end": 2,
        "tensor_shape": [2, 4], "layout": "token_major", "address": "1",
        "phase_state_digest": zero.as_str(), "revision": 1,
        "key_digest": key_digest.as_str(), "value_digest": value_digest.as_str(),
    });
    let kv = call(&mut stdin, &mut stdout, 3, json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "mind.kv_exact", "arguments": kv_args}}));
    let kv_inner = inner_text(&kv);
    assert_eq!(kv_inner["key_bytes"], 16);
    assert_eq!(kv_inner["value_bytes"], 16);
    assert_eq!(kv_inner["key_digest"], key_digest.as_str());
    // Base64 of 0..16 and 100..116, small payload so bytes are returned.
    assert!(kv_inner["key_base64"].as_str().unwrap().starts_with("AAECAw"));
    drop(stdin);
    let _ = child.wait();
}
