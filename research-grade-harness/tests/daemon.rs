use rgah::daemon::{DaemonConfig, StopReason, run};
use rgah::loop_ctl::{Controller, CycleSignal, LoopShape};
use std::path::PathBuf;
use std::time::Duration;

fn fail() -> CycleSignal {
    CycleSignal { verifier_pass: false, stalled: false, budget_used_frac: 0.1, error_class: Some("verifier".into()) }
}

fn pass() -> CycleSignal {
    CycleSignal { verifier_pass: true, stalled: false, budget_used_frac: 0.1, error_class: None }
}

fn cfg(max_cycles: u32, kill_file: PathBuf) -> DaemonConfig {
    DaemonConfig { max_cycles, kill_file, interval: Duration::from_millis(0) }
}

fn no_kill() -> PathBuf {
    PathBuf::from("/nonexistent/rgah-kill-switch-never-created")
}

#[test]
fn stops_at_cycle_cap() {
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 9 }, 3);
    let report = run(&cfg(4, no_kill()), &mut c, |_| pass());
    assert_eq!(report.cycles_run, 4);
    assert_eq!(report.stop, StopReason::CycleCap);
}

#[test]
fn kill_file_stops_before_next_cycle() {
    let dir = std::env::temp_dir().join(format!("rgah-daemon-kill-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let kill = dir.join("KILL");
    let _ = std::fs::remove_file(&kill);
    let mut c = Controller::new(LoopShape::BoundedRetry { max_rounds: 9 }, 3);
    let kill2 = kill.clone();
    let report = run(&cfg(100, kill.clone()), &mut c, move |cycle| {
        if cycle == 2 {
            std::fs::write(&kill2, b"stop").unwrap();
        }
        pass()
    });
    assert_eq!(report.stop, StopReason::KillFile);
    assert_eq!(report.cycles_run, 3);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reselection_is_recorded_in_report() {
    let mut c = Controller::new(LoopShape::OneShot, 2);
    // Ladder: OneShot -> BoundedRetry -> JudgeGated, then no stronger shape exists.
    let report = run(&cfg(10, no_kill()), &mut c, |_| fail());
    assert_eq!(report.receipts.len(), 2);
    assert_eq!(report.stop, StopReason::ControllerStop);
}
