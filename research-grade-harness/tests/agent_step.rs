use rgah::agent_step::{signal_from_output, step_prompt};
use rgah::glimmer_driver::GlimmerDriver;
use rgah::daemon::{DaemonConfig, StopReason, run};
use rgah::loop_ctl::{Controller, LoopShape};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn done_action_passes_the_verifier() {
    let s = signal_from_output("ACTION done :: 47 minutes");
    assert!(s.verifier_pass);
    assert!(s.error_class.is_none());
}

#[test]
fn search_and_read_do_not_pass_yet() {
    assert!(!signal_from_output("ACTION search :: reactor").verifier_pass);
    assert!(!signal_from_output("ACTION read :: doc-1").verifier_pass);
}

#[test]
fn missing_action_line_is_a_grammar_error_not_a_pass() {
    let s = signal_from_output("I think it is 47 minutes");
    assert!(!s.verifier_pass);
    assert_eq!(s.error_class.as_deref(), Some("grammar"));
}

#[test]
fn prompt_carries_task_and_grammar() {
    let p = step_prompt("how often does the reactor vent?", 2, &rgah::agent_step::Evidence::new());
    assert!(p.contains("how often does the reactor vent?"));
    assert!(p.contains("ACTION <search|read|done> :: <argument>"));
    assert!(p.contains("step 2"));
}

/// Live gate: the daemon drives real Glimmer completions for a bounded number of cycles.
#[test]
#[ignore = "requires a running Muse Glimmer server (RGAH_GLIMMER_ADDR)"]
fn live_daemon_runs_bounded_glimmer_cycles() {
    let addr = std::env::var("RGAH_GLIMMER_ADDR").expect("RGAH_GLIMMER_ADDR");
    let driver = GlimmerDriver::new(&addr);
    let cfg = DaemonConfig {
        max_cycles: 2,
        kill_file: PathBuf::from("/nonexistent/rgah-kill-never"),
        interval: Duration::from_millis(0),
    };
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 2 }, 2);
    let report = run(&cfg, &mut c, |cycle| {
        let prompt = step_prompt("how often does the reactor vent coolant?", cycle, &rgah::agent_step::Evidence::new());
        match driver.complete(&prompt, 40) {
            Ok(out) => signal_from_output(&out.text),
            Err(e) => panic!("live completion failed: {e}"),
        }
    });
    assert_eq!(report.cycles_run, 2);
    assert_eq!(report.stop, StopReason::CycleCap);
}

#[test]
fn search_result_is_fed_into_the_next_prompt() {
    use rgah::agent_step::{Evidence, Retriever, run_step};
    struct Fixed;
    impl Retriever for Fixed {
        fn lookup(&mut self, _query: &str) -> Result<String, String> {
            Ok("The Zorblax reactor vents coolant every 47 minutes.".into())
        }
    }
    let mut ev = Evidence::new();
    let first = run_step("ACTION search :: reactor vent rate", &mut Fixed, &mut ev).unwrap();
    assert!(!first.verifier_pass);
    assert!(ev.render().contains("every 47 minutes"));
    // The next prompt must carry the gathered evidence.
    let p = step_prompt("how often does the reactor vent?", 1, &ev);
    assert!(p.contains("every 47 minutes"));
}

#[test]
fn continuation_becomes_a_single_action_line() {
    use rgah::agent_step::action_text;
    assert_eq!(action_text("search :: reactor\nI'll search the web too"), "ACTION search :: reactor");
    assert_eq!(action_text("done :: 47 minutes"), "ACTION done :: 47 minutes");
}

#[test]
fn prompt_ends_with_action_prefix() {
    assert!(step_prompt("q", 0, &rgah::agent_step::Evidence::new()).ends_with("ACTION "));
}

#[test]
fn prompt_uses_qwen_chatml_with_thinking_disabled() {
    let p = step_prompt("q", 0, &rgah::agent_step::Evidence::new());
    assert!(p.starts_with("<|im_start|>user\n"), "got: {p:.120}");
    assert!(
        p.ends_with("<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\nACTION "),
        "got: {p:.300}"
    );
}
