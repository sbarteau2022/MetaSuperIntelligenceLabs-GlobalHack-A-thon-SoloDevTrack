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

        judge_joined: false,
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

// GAP-1 (C-RUN-04 follow-up, hermetic): sink records every call of a
// multi-same-tool packet as its own row. Two record_call rows with indexed
// keys (packet:attempt:tool#0 / #1) both persist and the chain verifies —
// the (packet,attempt,tool) collapse that cost C-RUN-03 its 11th row.
#[test]
fn sink_records_indexed_same_tool_calls() {
    let db_path = std::env::temp_dir().join(format!(
        "rgah-engine-n3-test-{}-gap1.sqlite3",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&db_path);
    let mut sink = TelemetrySink::open(&db_path).expect("sink opens");
    let s0 = sink
        .record_call(
            "seed:1:map.ingest#0",
            "map.ingest",
            "{\"text\":\"a\"}",
            5,
            true,
            Some("r0"),
        )
        .expect("row 0");
    let s1 = sink
        .record_call(
            "seed:1:map.ingest#1",
            "map.ingest",
            "{\"text\":\"b\"}",
            6,
            true,
            Some("r1"),
        )
        .expect("row 1");
    assert_ne!(s0, s1, "indexed keys must yield distinct rows");
    // The OLD unindexed key still collapses (idempotent-hit = tested sink
    // behavior, preserved — callers must index; the sink must not guess).
    let s_dup = sink
        .record_call(
            "seed:1:map.ingest#0",
            "map.ingest",
            "{\"text\":\"a\"}",
            5,
            true,
            Some("r0"),
        )
        .expect("idempotent re-record");
    assert_eq!(s_dup, s0, "same key returns original seq");
    assert!(sink.verify().is_ok());
    assert_eq!(sink.len().expect("len"), 2);
    let _ = std::fs::remove_file(&db_path);
}

// GAP-2 (C-RUN-04 follow-up, hermetic contract): judge_joined=false (the
// default) preserves per-call conjunctive judging. The `packet()` helper
// builds per-call packets; the joined mode is proved live below (merged
// gate+collect packet). Serde compat (`#[serde(default)]` on the field —
// old plan JSON without the key deserializes to false by serde's documented
// contract) is code-verified; serde_json is not a dependency of this crate
// (doctrine) so no JSON round-trip test is written here.
#[test]
fn per_call_judging_stays_default() {
    let p = packet("m", vec![]);
    assert!(!p.judge_joined, "helper builds per-call packets");
}
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

// C-RUN-04 (supersedes v1/v2/v3 before execution): daemon DAG (ignored unless HYPERMIND_MCP_BIN is set). ONE held-open
// child for a 4-packet fan-out/fan-in DAG through run_parallel_n3 + live sink
// read-back. Closes the three FULL-SMOKE NOT-VERIFIEDs: (1) daemon mode —
// many sequential calls, one stdin/stdout pair, state accumulates; (2) live
// write-through sink — verify() + len() + replay read-back after the run;
// (3) multi-packet DAG beyond 2-packet chains — seed → {qa,qb} → join.
#[test]
#[ignore]
fn live_daemon_dag_c_run01() {
    let bin = std::env::var("HYPERMIND_MCP_BIN").expect("HYPERMIND_MCP_BIN required");
    let root = std::env::temp_dir().join(format!("rgah-daemon-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&root);
    let mut eng = Engine::new();
    eng.register_judge(judge_pass("jr", "receipt_sha256"));
    eng.register_judge(judge_pass("ja", "bravo seven"));
    eng.register_judge(judge_pass("jb", "charlie nine"));
    eng.register_judge(judge_pass("jg", "retrieved_ids"));
    eng.register_judge(judge_pass("jj", "daemon"));
    let manifest = r#"[{"source_root":"r","source_path":"vault/map.ingest","category":"03-AgenticRuntime","dest_path":"d/map.ingest"},{"source_root":"r","source_path":"vault/map.query","category":"03-AgenticRuntime","dest_path":"d/map.query"},{"source_root":"r","source_path":"vault/map.attend","category":"03-AgenticRuntime","dest_path":"d/map.attend"}]"#;
    let gate = CapabilityGate::from_manifest(manifest).expect("manifest loads");
    let mut plan = Plan {
        id: "daemon".into(),
        objective: "daemon-dag".into(),
        packets: vec![
            Packet {
                depends_on: vec![],
                judges: vec![JudgeRef {
                    id: "jr".into(),
                    kind: "contains".into(),
                }],
                ..packet(
                    "seed",
                    vec![
                        map_call(
                            "map.ingest",
                            "{\"text\":\"daemon packet alpha bravo seven\"}",
                        ),
                        map_call(
                            "map.ingest",
                            "{\"text\":\"daemon packet alpha charlie nine\"}",
                        ),
                    ],
                )
            },
            Packet {
                depends_on: vec!["seed".to_string()],
                judges: vec![JudgeRef {
                    id: "ja".into(),
                    kind: "contains".into(),
                }],
                ..packet(
                    "qa",
                    vec![map_call("map.query", "{\"qtext\":\"alpha bravo\",\"k\":2}")],
                )
            },
            Packet {
                depends_on: vec!["seed".to_string()],
                judges: vec![JudgeRef {
                    id: "jb".into(),
                    kind: "contains".into(),
                }],
                ..packet(
                    "qb",
                    vec![map_call(
                        "map.query",
                        "{\"qtext\":\"alpha charlie\",\"k\":2}",
                    )],
                )
            },
            Packet {
                depends_on: vec!["qa".to_string(), "qb".to_string()],
                judges: vec![JudgeRef {
                    id: "jg".into(),
                    kind: "contains".into(),
                }],
                // gate runs the depth gate over the fan-in (contract: IDs +
                // gate metadata with receipt). Judged on its own contract key.
                ..packet(
                    "gate",
                    vec![map_call(
                        "map.attend",
                        "{\"qtext\":\"daemon alpha\",\"k\":3}",
                    )],
                )
            },
            Packet {
                depends_on: vec!["gate".to_string()],
                judges: vec![JudgeRef {
                    id: "jj".into(),
                    kind: "contains".into(),
                }],
                // collect returns the joined branch content as text.
                ..packet(
                    "collect",
                    vec![map_call(
                        "map.query",
                        "{\"qtext\":\"daemon alpha\",\"k\":3}",
                    )],
                )
            },
        ],
    };
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(100.0);
    let db_path = root.join("daemon.sqlite3");
    let mut sink = TelemetrySink::open(&db_path).expect("sink opens");
    // ONE child for the whole DAG — the daemon under test. No respawn.
    let mut mcp = McpChild::spawn(&bin, root.to_str().unwrap()).expect("daemon spawns");
    let report = eng.run_parallel_n3(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut mcp,
        &mut sink,
        &gate,
        "swarm-daemon",
    );
    mcp.close();
    // (3) DAG: all four pass — fan-out level + fan-in join, judged live.
    assert_eq!(
        report.passed.len(),
        5,
        "all 5 DAG packets must pass, got {:?}",
        report
    );
    // (1) daemon: every packet ran through the single child — receipts carry
    // the shared swarm id and the plan advanced past the 2-stage join.
    assert!(receipts.entries.iter().all(|r| r.outcome == "PASS"));
    assert!(
        plan.packets
            .iter()
            .all(|p| p.status == PacketStatus::Passed)
    );
    // (2) live sink (C-RUN-04): hash chain verifies, 10 rows = 5 distinct
    // call-keys (seed's two ingests collapse on the (packet,attempt,tool)
    // idempotency key — tested sink behavior) + 5 verdicts. Replay reads back.
    assert!(sink.verify().is_ok(), "sink chain must verify");
    let n = sink.len().expect("sink len");
    assert!(
        n >= 10,
        "sink must hold >=10 events (5 distinct call-rows + 5 verdicts, C-RUN-04), got {}",
        n
    );
    eprintln!(
        "C-RUN-04: 5/5 DAG pass, sink events={} chain=ok (db={:?})",
        n, db_path
    );
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

// GAP-2 live (ignored unless HYPERMIND_MCP_BIN is set): merged gate+collect
// packet with judge_joined=true — the exact v2-join shape that FAILED under
// per-call conjunctive judging (attend output can never satisfy a text
// needle). Under joined judging the packet passes: attend runs the gate,
// query returns the text, judges run once over the aggregate.
#[test]
#[ignore]
fn live_joined_judging_merged_join() {
    let bin = std::env::var("HYPERMIND_MCP_BIN").expect("HYPERMIND_MCP_BIN required");
    let root = std::env::temp_dir().join(format!("rgah-joined-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&root);
    let mut eng = Engine::new();
    eng.register_judge(judge_pass("jr", "receipt_sha256"));
    // GAP-2 needle matches the corpus text ("joined ... seven"): the query
    // call returns it; the attend call carries IDs. Joined output contains
    // "seven" iff the query arm retrieved the seed text.
    eng.register_judge(judge_pass("jj", "seven"));
    let manifest = r#"[{"source_root":"r","source_path":"vault/map.ingest","category":"03-AgenticRuntime","dest_path":"d/map.ingest"},{"source_root":"r","source_path":"vault/map.query","category":"03-AgenticRuntime","dest_path":"d/map.query"},{"source_root":"r","source_path":"vault/map.attend","category":"03-AgenticRuntime","dest_path":"d/map.attend"}]"#;
    let gate = CapabilityGate::from_manifest(manifest).expect("manifest loads");
    let mut seed = packet(
        "seed",
        vec![map_call(
            "map.ingest",
            "{\"text\":\"joined packet alpha bravo seven\"}",
        )],
    );
    seed.judges = vec![JudgeRef {
        id: "jr".into(),
        kind: "contains".into(),
    }];
    let mut join = packet(
        "join",
        vec![
            map_call("map.attend", "{\"qtext\":\"joined alpha\",\"k\":3}"),
            map_call("map.query", "{\"qtext\":\"joined alpha\",\"k\":3}"),
        ],
    );
    // THE flag: joined-output judging over the aggregate.
    join.judge_joined = true;
    join.judges = vec![JudgeRef {
        id: "jj".into(),
        kind: "contains".into(),
    }];
    join.depends_on = vec!["seed".to_string()];
    let mut plan = Plan {
        id: "joined".into(),
        objective: "joined".into(),
        packets: vec![seed, join],
    };
    let mut receipts = ReceiptLog::default();
    let mut budget = EnergyBudget::new(100.0);
    let db_path = root.join("joined.sqlite3");
    let mut sink = TelemetrySink::open(&db_path).expect("sink opens");
    let mut mcp = McpChild::spawn(&bin, root.to_str().unwrap()).expect("binary spawns");
    let report = eng.run_sequential_n3(
        &mut plan,
        &mut receipts,
        &mut budget,
        &mut mcp,
        &mut sink,
        &gate,
        "swarm-joined",
    );
    mcp.close();
    assert_eq!(
        report.passed,
        vec!["seed".to_string(), "join".to_string()],
        "joined packet must pass under joined judging, got {:?}",
        report
    );
    assert!(sink.verify().is_ok());
    eprintln!("GAP-2 live: merged join passes under judge_joined=true");
}
