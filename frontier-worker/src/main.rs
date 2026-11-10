//! Isolated target discovery over a committed source snapshot.
//! No model, checkpoint, live store, or host checkout is opened here.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

const MAX_GATE_OUTPUT: usize = 8 * 1024 * 1024;

struct Target {
    id: &'static str,
    manifest: &'static str,
    gate_args: &'static [&'static str],
}

const TARGETS: [Target; 3] = [
    Target {
        id: "ares-backend",
        manifest: "Ares WebSite : App/backend/Cargo.toml",
        gate_args: &["check", "--tests", "--offline", "--message-format=short"],
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
        let output = Command::new("cargo")
            .args(target.gate_args)
            .arg("--manifest-path")
            .arg(&manifest)
            .current_dir(snapshot)
            .output()
            .map_err(|e| format!("cargo check {}: {e}", target.id))?;
        let mut evidence = output.stdout;
        evidence.extend_from_slice(&output.stderr);
        if evidence.len() > MAX_GATE_OUTPUT {
            return Err(format!("gate output exceeded bound for {}", target.id));
        }
        let digest = sha256(&evidence)?;
        let exit = output.status.code().unwrap_or(255);
        println!(
            "gate\tsource={}\ttarget={}\texit={}\toutput_sha256={}\tduration_ms={}",
            commit,
            target.id,
            exit,
            digest,
            started.elapsed().as_millis()
        );
        if !output.status.success() {
            let reason = if evidence
                .windows(b"failed to download".len())
                .any(|w| w == b"failed to download")
                || evidence
                    .windows(b"no matching package named".len())
                    .any(|w| w == b"no matching package named")
            {
                "dependency-unavailable"
            } else {
                "gate-failure"
            };
            failures.push((target.id, reason, digest));
        }
    }
    failures.sort_by(|a, b| a.0.cmp(b.0));
    let environment_unavailable = failures.iter().any(|f| f.1 == "dependency-unavailable");
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
