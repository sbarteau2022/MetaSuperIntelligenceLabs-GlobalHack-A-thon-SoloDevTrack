use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeKind {
    ExitCodeZero,
    GoldenHash { expected: String },
    Contains { needle: String },
    NotContains { needle: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Judge {
    pub id: String,
    pub kind: JudgeKind,
}

#[derive(Debug, Clone)]
pub enum JudgeOutcome {
    Pass,
    Fail(String),
}

#[derive(Debug, Clone)]
pub struct JudgeResult {
    pub judge_id: String,
    pub outcome: JudgeOutcome,
    pub evidence: String,
}

impl Judge {
    pub fn run(&self, output: &str, exit_code: i32) -> JudgeResult {
        match &self.kind {
            JudgeKind::ExitCodeZero => {
                let ok = exit_code == 0;
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("exit_code={}", exit_code))
                    },
                    evidence: format!("exit_code={}", exit_code),
                }
            }
            JudgeKind::GoldenHash { expected } => {
                let mut h = Sha256::new();
                h.update(output.as_bytes());
                let got = hex::encode(h.finalize());
                let ok = got == *expected;
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!(
                            "hash mismatch: got {} expected {}",
                            got, expected
                        ))
                    },
                    evidence: format!("got={}", got),
                }
            }
            JudgeKind::Contains { needle } => {
                let ok = output.contains(needle);
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("missing '{}'", needle))
                    },
                    evidence: format!("contains={}", ok),
                }
            }
            JudgeKind::NotContains { needle } => {
                let ok = !output.contains(needle);
                JudgeResult {
                    judge_id: self.id.clone(),
                    outcome: if ok {
                        JudgeOutcome::Pass
                    } else {
                        JudgeOutcome::Fail(format!("unexpected '{}'", needle))
                    },
                    evidence: format!("not_contains={}", ok),
                }
            }
        }
    }
}

pub fn run_command(cmd: &str, args: &[&str]) -> (i32, String) {
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            let mut out = String::from_utf8_lossy(&o.stdout).to_string();
            out.push_str(&String::from_utf8_lossy(&o.stderr));
            (o.status.code().unwrap_or(-1), out)
        }
        Err(e) => (-1, format!("spawn error: {}", e)),
    }
}
