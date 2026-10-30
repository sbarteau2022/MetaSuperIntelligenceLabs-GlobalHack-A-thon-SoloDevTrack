//! Engine <-> N3 wiring tests. Hermetic by default; live-binary tests are
//! #[ignore]d unless HYPERMIND_MCP_BIN points at a hypermind-mcp binary.
use rgah::capability::CapabilityGate;
use rgah::energy::EnergyBudget;
use rgah::engine::Engine;
use rgah::judge::{Judge, JudgeKind};
use rgah::mcp_client::McpChild;
use rgah::receipt::ReceiptLog;
use rgah::runbook::{JudgeRef, Packet, PacketStatus, Plan, RetryPolicy, ToolCall};
use rgah::sqlite_sink::TelemetrySink;

fn judge_pass(id: &str, needle: &str) -> Judge {
    Judge {
        id: id.into(),
        kind: JudgeKind::Contains {
            needle: needle.into(),
        },
    }
}

fn packet(id: &str, tool_calls: Vec<ToolCall>) -> Packet {
    Packet {
        id: id.into(),
        task: format!("task {}", id),
        depends_on: vec![],
        declared_context_tokens: 10,
        tool_calls,
        judges: vec![JudgeRef {
            id: "j".into(),
            kind: "contains".into(),
        }],
        branch: None,
        retry_policy: RetryPolicy {
            max_rounds: 2,
            material_change_required: true,
        },
        status: PacketStatus::Pending,
        attempts: 0,
    }
}

fn map_call(tool: &str, args: &str) -> ToolCall {
    ToolCall {
        tool: tool.into(),
        args_json: args.into(),
        declared: true,
    }
}

// judge_output: pass, fail, missing judge, multi-judge, exit-code passthrough.
#[test]
fn judge_output_pass_fail_missing_multi() {
    let mut eng = Engine::new();
    eng.register_judge(judge_pass("j", "PASS"));
    eng.register_judge(judge_pass("k", "packet"));
    let base = packet("a", vec![]);
    // pass: both registered judges hit
    let p = Packet {
        judges: vec![
            JudgeRef {
                id: "j".into(),
                kind: "contains".into(),
            },
            JudgeRef {
                id: "k".into(),
                kind: "contains".into(),
            },
        ],
        ..base.clone()
    };
    let (ok, ids) = eng.judge_output(&p, "PASS packet=a attempt=1", 0);
    assert!(ok);
    assert_eq!(ids, vec!["j".to_string(), "k".to_string()]);
    // fail: output misses the needle
    let (ok2, _) = eng.judge_output(&p, "nothing here", 0);
    assert!(!ok2);
    // missing judge id -> fail, id still recorded
    let p2 = Packet {
        judges: vec![JudgeRef {
            id: "ghost".into(),
            kind: "contains".into(),
        }],
        ..base.clone()
    };
    let (ok3, ids3) = eng.judge_output(&p2, "PASS packet=a attempt=1", 0);
    assert!(!ok3);
    assert_eq!(ids3, vec!["ghost".to_string()]);
    // exit code is passed through (Contains ignores it, ExitCodeZero uses it)
    let mut eng2 = Engine::new();
    eng2.register_judge(Judge {
        id: "z".into(),
        kind: JudgeKind::ExitCodeZero,
    });
    let p3 = Packet {
        judges: vec![JudgeRef {
            id: "z".into(),
            kind: "exit".into(),
        }],
        ..base.clone()
    };
    assert!(eng2.judge_output(&p3, "anything", 0).0);
    assert!(!eng2.judge_output(&p3, "anything", 1).0);
}

// Transport failure: spawn of a nonexistent binary must Err (fail-explained,
// never silent). The engine escalates on transport errors; this pins the cause.
#[test]
fn spawn_missing_binary_fails_explained() {
    let r = McpChild::spawn("/nonexistent-binary-xyz", "/tmp");
    assert!(r.is_err());
    let e = match r {
        Err(e) => e,
        Ok(_) => panic!("spawn of missing binary must fail"),
    };
    assert!(
        e.contains("/nonexistent-binary-xyz"),
        "must name the binary: {}",
        e
    );
}

// Gate denial: unknown map.* tool fails the N3 attempt with a named denial
// and the packet escalates (no silent execution, no identical retry loop).
#[test]
fn gate_denial_fails_packet_without_transport() {
    let mut eng = Engine::new();
    eng.register_judge(judge_pass("j", "ok"));
    // Empty manifest -> every tool unknown -> gate denies without spawning.
    let gate = CapabilityGate::from_manifest("[]").expect("empty manifest loads");
    let mut plan = Plan {
        id: "p".into(),
        objective: "o".into(),
        packets: vec![packet(
            "a",
            vec![map_call("map.query", "{\"qtext\":\"x\",\"k\":1}")],
        )],
    };
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(100.0);
    let db_path = std::env::temp_dir().join(format!(
        "rgah-engine-n3-test-{}-denial.sqlite3",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&db_path);
    let mut sink = TelemetrySink::open(&db_path).expect("sink opens");
    // Dead child stand-in: spawn fails, so use a live-but-wrong approach —
    // instead exercise execute_packet_n3's gate path with a child that will
    // never be reached (gate denies first). We still need a McpChild value:
    // spawn /usr/bin/true (gate denies before any call is made).
    let mut mcp = McpChild::spawn("/usr/bin/true", "/tmp").expect("true spawns");
    let report = eng.run_sequential_n3(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut mcp,
        &mut sink,
        &gate,
        "swarm-test",
    );
    mcp.close();
    // Gate denied map.query (not in empty DAG) -> attempt failed -> escalated
    // (max_rounds=2 but denial is deterministic: second attempt would be
    // identical, so the engine escalates after the retry round).
    assert!(
        report.escalated.contains(&"a".to_string()),
        "gate-denied packet must escalate, got {:?}",
        report
    );
    assert!(receipts.entries.iter().any(|r| r.outcome == "ESCALATE"));
    // Verdict recorded in the sink chain.
    assert!(sink.verify().is_ok());
    assert!(!sink.is_empty().unwrap());
    let _ = std::fs::remove_file(&db_path);
}

// Live round-trip (ignored unless HYPERMIND_MCP_BIN is set): real binary,
// real map.query judged on returned text, receipt + sink chain verified.
#[test]
#[ignore]
fn live_n3_round_trip() {
    let bin = std::env::var("HYPERMIND_MCP_BIN").expect("HYPERMIND_MCP_BIN required");
    let root = std::env::temp_dir().join(format!("rgah-live-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&root);
    let mut eng = Engine::new();
    eng.register_judge(judge_pass("j", "kuramoto"));
    let manifest = r#"[{"source_root":"r","source_path":"vault/map.query","category":"03-AgenticRuntime","dest_path":"d/map.query"},{"source_root":"r","source_path":"vault/map.ingest","category":"03-AgenticRuntime","dest_path":"d/map.ingest"}]"#;
    let gate = CapabilityGate::from_manifest(manifest).expect("manifest loads");
    // Seed then query: the fresh HYPERMIND_ROOT starts empty, so packet
    // "seed" ingests the probe text, then "q" queries it back. Separate
    // judges per packet: the seed judge looks for the ingest receipt
    // (receipt_sha256 present on every N3 response); the query judge looks
    // for the retrieved text. Judging ingest output for query text would
    // conflate stages — each stage judges its own contract.
    eng.register_judge(judge_pass("jr", "receipt_sha256"));
    let mut plan = Plan {
        id: "live".into(),
        objective: "live".into(),
        packets: vec![
            Packet {
                depends_on: vec![],
                judges: vec![JudgeRef {
                    id: "jr".into(),
                    kind: "contains".into(),
                }],
                ..packet(
                    "seed",
                    vec![map_call(
                        "map.ingest",
                        "{\"text\":\"kuramoto phase coupling test\"}",
                    )],
                )
            },
            Packet {
                depends_on: vec!["seed".to_string()],
                judges: vec![JudgeRef {
                    id: "j".into(),
                    kind: "contains".into(),
                }],
                ..packet(
                    "q",
                    vec![map_call("map.query", "{\"qtext\":\"kuramoto\",\"k\":1}")],
                )
            },
        ],
    };
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(100.0);
    let db_path = root.join("live.sqlite3");
    let mut sink = TelemetrySink::open(&db_path).expect("sink opens");
    let mut mcp = McpChild::spawn(&bin, root.to_str().unwrap()).expect("live binary spawns");
    let report = eng.run_sequential_n3(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut mcp,
        &mut sink,
        &gate,
        "swarm-live",
    );
    mcp.close();
    // Both stages pass with their own contracts: seed on its receipt, query
    // on the retrieved text.
    assert_eq!(report.passed, vec!["seed".to_string(), "q".to_string()]);
    assert!(sink.verify().is_ok());
    assert!(!sink.is_empty().unwrap());
}
