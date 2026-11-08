//! Daemon: runs a cycle step under the loop controller until a stop condition.
//! Stop conditions: cycle cap, kill-file present, or the controller says stop.

use crate::loop_ctl::{Controller, CycleSignal, Decision, SwitchReceipt};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DaemonConfig {
    pub max_cycles: u32,
    pub kill_file: PathBuf,
    pub interval: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    CycleCap,
    KillFile,
    ControllerStop,
}

#[derive(Debug, Clone)]
pub struct DaemonReport {
    pub cycles_run: u32,
    pub stop: StopReason,
    pub receipts: Vec<SwitchReceipt>,
}

/// Run cycles until a stop condition. `step` receives the cycle index and returns its signal.
pub fn run<F>(cfg: &DaemonConfig, controller: &mut Controller, mut step: F) -> DaemonReport
where
    F: FnMut(u32) -> CycleSignal,
{
    let mut cycles_run = 0;
    let stop = loop {
        if cycles_run >= cfg.max_cycles {
            break StopReason::CycleCap;
        }
        if cfg.kill_file.exists() {
            break StopReason::KillFile;
        }
        let sig = step(cycles_run);
        cycles_run += 1;
        if let Decision::Stop(_) = controller.observe(&sig) {
            break StopReason::ControllerStop;
        }
        if !cfg.interval.is_zero() {
            std::thread::sleep(cfg.interval);
        }
    };
    DaemonReport { cycles_run, stop, receipts: controller.receipts().to_vec() }
}
