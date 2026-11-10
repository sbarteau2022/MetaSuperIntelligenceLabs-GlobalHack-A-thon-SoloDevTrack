//! Isolated target discovery over a committed source snapshot.
//! No model, checkpoint, live store, or host checkout is opened here.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::Instant;

const MAX_GATE_OUTPUT: usize = 8 * 1024 * 1024;
const EXCERPT_BYTES: usize = 2048;

struct Target {
    id: &'static str,
    manifest: &'static str,
    gate_args: &'static [&'static str],
}

const TARGETS: [Target; 4] = [
    Target {
        id: "ares-backend",
        manifest: "Ares WebSite : App/backend/Cargo.toml",
        gate_args: &["check", "--tests", "--offline", "--message-format=short"],
    },
    Target {
        id: "ares-backend-tests",
        manifest: "Ares WebSite : App/backend/Cargo.toml",
        gate_args: &["test", "--lib", "--offline", "--", "--test-threads=1"],
    },
    Target {
        id: "core-lm",
        manifest: "core_lm/Cargo.toml",
        gate_args: &[], // Metal/CoreGraphics cannot be qualified in Linux Docker.
    },
    Target {
        id: "globalmind-runtime",
        manifest: "globalmind-runtime/Cargo.toml",
        gate_args: &["check", "--lib", "--offline", "--message-format=short"],
    },
];

fn sha256(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("sha256sum")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("sha256sum spawn: {e}"))?;
    child
        .stdin
        .take()
        .ok_or("sha256sum stdin unavailable")?
        .write_all(bytes)
        .map_err(|e| format!("sha256sum write: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("sha256sum wait: {e}"))?;
    if !output.status.success() {
        return Err("sha256sum failed".into());
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|e| e.to_string())?;
    let digest = text
        .split_whitespace()
        .next()
        .ok_or("sha256sum omitted digest")?;
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("sha256sum returned invalid digest".into());
    }
    Ok(digest.to_string())
}

fn hex_tail(bytes: &[u8]) -> String {
    let alphabet = b"0123456789abcdef";
    let tail = &bytes[bytes.len().saturating_sub(EXCERPT_BYTES)..];
    let mut result = String::with_capacity(tail.len() * 2);
    for byte in tail {
        result.push(alphabet[(byte >> 4) as usize] as char);
        result.push(alphabet[(byte & 0x0f) as usize] as char);
    }
    result
}

fn run_bounded_gate(mut command: Command) -> Result<(ExitStatus, Vec<u8>), String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gate spawn: {e}"))?;
    let stdout = child.stdout.take().ok_or("gate stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("gate stderr unavailable")?;
    let readers: [Box<dyn Read + Send>; 2] = [Box::new(stdout), Box::new(stderr)];
    let (tx, rx) = mpsc::channel();
    for (index, mut reader) in readers.into_iter().enumerate() {
        let sender = tx.clone();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = reader
                .by_ref()
                .take(MAX_GATE_OUTPUT as u64 + 1)
                .read_to_end(&mut bytes);
            let _ = sender.send((index, result, bytes));
        });
    }
    drop(tx);
    let mut streams = [Vec::new(), Vec::new()];
    let mut exceeded = false;
    for _ in 0..2 {
        let (index, result, bytes) = rx.recv().map_err(|e| format!("gate reader: {e}"))?;
        if let Err(e) = result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("gate read: {e}"));
        }
        if bytes.len() > MAX_GATE_OUTPUT {
            exceeded = true;
            let _ = child.kill();
        }
        streams[index] = bytes;
    }
    let status = child.wait().map_err(|e| format!("gate wait: {e}"))?;
    if exceeded || streams[0].len() + streams[1].len() > MAX_GATE_OUTPUT {
        return Err("gate output exceeded bound".into());
    }
    let mut evidence = std::mem::take(&mut streams[0]);
    evidence.extend_from_slice(&streams[1]);
    Ok((status, evidence))
}

fn run(snapshot: &Path) -> Result<(), String> {
    if !snapshot.is_dir() {
        return Err("source snapshot directory missing".into());
    }
    let commit = std::env::var("ARES_FRONTIER_SOURCE_COMMIT")
        .map_err(|_| "source commit identity missing".to_string())?;
    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("source commit identity invalid".into());
    }
    let mut failures = Vec::new();
    let mut portable_gate_unavailable = false;
    for target in TARGETS {
        let manifest = snapshot.join(target.manifest);
        if !manifest.is_file() {
            return Err(format!("registered target {} is missing", target.id));
        }
        if target.gate_args.is_empty() {
            println!(
                "gate\tsource={}\ttarget={}\tstatus=unavailable\treason=apple-framework",
                commit, target.id
            );
            continue;
        }
        let started = Instant::now();
        let mut command = Command::new("cargo");
        command
            .args(target.gate_args)
            .arg("--manifest-path")
            .arg(&manifest)
            .current_dir(snapshot);
        let (status, evidence) = match run_bounded_gate(command) {
            Ok(result) => result,
            Err(e) => {
                portable_gate_unavailable = true;
                let reason = if e == "gate output exceeded bound" {
                    "output-bound"
                } else {
                    "gate-io"
                };
                eprintln!("cargo gate {} unavailable: {e}", target.id);
                println!(
                    "gate\tsource={}\ttarget={}\tstatus=unavailable\treason={reason}",
                    commit, target.id
                );
                continue;
            }
        };
        let digest = sha256(&evidence)?;
        let exit = status.code().unwrap_or(255);
        let reason = if status.success() {
            "pass"
        } else if status.code().is_none() || matches!(status.code(), Some(137 | 143)) {
            "resource-unavailable"
        } else if evidence
            .windows(b"failed to download".len())
            .any(|w| w == b"failed to download")
            || evidence
                .windows(b"no matching package named".len())
                .any(|w| w == b"no matching package named")
        {
            "dependency-unavailable"
        } else if evidence.windows(b"E0455".len()).any(|w| w == b"E0455") {
            "platform-unavailable"
        } else {
            "gate-failure"
        };
        let excerpt = if status.success() {
            String::new()
        } else {
            hex_tail(&evidence)
        };
        println!(
            "gate\tsource={}\ttarget={}\texit={}\treason={}\toutput_sha256={}\tduration_ms={}\texcerpt_hex={}",
            commit,
            target.id,
            exit,
            reason,
            digest,
            started.elapsed().as_millis(),
            excerpt,
        );
        if !status.success() {
            failures.push((target.id, reason, digest));
        }
    }
    failures.sort_by(|a, b| a.0.cmp(b.0));
    let environment_unavailable =
        portable_gate_unavailable || failures.iter().any(|f| f.1 != "gate-failure");
    if let Some((id, reason, digest)) = failures.into_iter().find(|f| f.1 == "gate-failure") {
        println!(
            "decision\tsource={commit}\ttarget={id}\treason={reason}\tevidence_sha256={digest}"
        );
    } else if environment_unavailable {
        println!("decision\tsource={commit}\tscope=portable\ttarget=none\treason=environment-unavailable");
    } else {
        println!("decision\tsource={commit}\tscope=portable\ttarget=none\treason=no-admitted-gate-failure");
    }
    Ok(())
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--once" {
        eprintln!("usage: frontier-worker --once SOURCE_SNAPSHOT");
        std::process::exit(2);
    }
    if let Err(e) = run(Path::new(&args[2])) {
        eprintln!("frontier worker refused: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{hex_tail, run_bounded_gate, EXCERPT_BYTES};
    use std::process::Command;

    #[test]
    fn bounded_gate_preserves_small_output() {
        let mut command = Command::new("sh");
        command.args(["-c", "printf 'ok' && printf 'err' >&2"]);
        let (status, evidence) = run_bounded_gate(command).expect("small output is admitted");
        assert!(status.success());
        assert_eq!(evidence, b"okerr");
    }

    #[test]
    fn bounded_gate_kills_output_over_limit() {
        let mut command = Command::new("sh");
        command.args(["-c", "head -c 8388609 /dev/zero"]);
        let result = run_bounded_gate(command);
        assert!(matches!(result, Err(reason) if reason == "gate output exceeded bound"));
    }

    #[test]
    fn failure_excerpt_is_byte_exact_and_bounded() {
        assert_eq!(hex_tail(&[0, 10, 255]), "000aff");
        let bytes = vec![0xab; EXCERPT_BYTES + 1];
        let excerpt = hex_tail(&bytes);
        assert_eq!(excerpt.len(), EXCERPT_BYTES * 2);
        assert!(excerpt.bytes().all(|byte| byte == b'a' || byte == b'b'));
    }
}
