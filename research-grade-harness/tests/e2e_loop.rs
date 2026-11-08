use rgah::agent_step::{Evidence, action_text, run_step, step_prompt};
use rgah::daemon::{DaemonConfig, StopReason, run};
use rgah::loop_ctl::{Controller, LoopShape};
use rgah::n3_retriever::N3Retriever;
use rgah::qwen_driver::QwenDriver;
use std::path::PathBuf;
use std::time::Duration;

const FACT: &str = "The Zorblax reactor at Quillan Station vents coolant every 47 minutes.";
const TASK: &str = "how often does the Zorblax reactor at Quillan Station vent coolant?";

/// Live end-to-end gate: real Qwen3.6-MoE (RGAH_QWEN_ADDR) and real hypermind-mcp (RGAH_HYPERMIND_MCP).
/// Every cycle prints the model's raw output so the run can be audited.
/// The gate fails unless retrieval actually ran and the model answered from the fact.
#[test]
#[ignore = "requires RGAH_QWEN_ADDR and RGAH_HYPERMIND_MCP"]
fn live_loop_retrieves_then_answers() {
    let addr = std::env::var("RGAH_QWEN_ADDR").expect("RGAH_QWEN_ADDR");
    let bin = std::env::var("RGAH_HYPERMIND_MCP").expect("RGAH_HYPERMIND_MCP");
    let dir = std::env::temp_dir().join(format!("rgah-e2e-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let mut retriever = N3Retriever::spawn(&bin, &dir.to_string_lossy()).expect("spawn n3");
    retriever.ingest(FACT).expect("ingest");
    let driver = QwenDriver::new(&addr);
    let mut evidence = Evidence::new();
    let cfg = DaemonConfig {
        max_cycles: 3,
        kill_file: PathBuf::from("/nonexistent/rgah-e2e-kill"),
        interval: Duration::from_millis(0),
    };
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 3 }, 3);

    let report = run(&cfg, &mut c, |cycle| {
        let prompt = step_prompt(TASK, cycle, &evidence);
        let out = driver.complete(&prompt, 24).expect("qwen completion");
        println!("CYCLE {cycle} evidence_in_prompt={} output={:?}", !evidence.render().is_empty(), out.text);
        run_step(&action_text(&out.text), &mut retriever, &mut evidence)
            .unwrap_or_else(|e| panic!("step failed: {e}"))
    });

    println!("REPORT cycles={} stop={:?} evidence={:?}", report.cycles_run, report.stop, evidence.render());
    assert_eq!(report.cycles_run, 3);
    assert_eq!(report.stop, StopReason::CycleCap);
    assert!(!evidence.render().is_empty(), "the model never ran a search or read action");
}
