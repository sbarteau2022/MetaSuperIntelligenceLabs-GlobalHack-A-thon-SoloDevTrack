//! Model-neutral idle supervisor. It launches only an already-built Docker
//! worker image and streams a committed source snapshot without host mounts.

use crate::governor::AppState;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

const IDLE_GRACE: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_secs(60);
const JOB_LIMIT: Duration = Duration::from_secs(1800);
const MIN_COLIMA_AVAILABLE_KIB: u64 = 786_432;
const MAX_RECEIPT_BYTES: usize = 65_536;

pub fn status_label(code: u8) -> &'static str {
    match code {
        0 => "disabled",
        1 => "waiting_for_idle",
        2 => "unavailable",
        3 => "running",
        4 => "receipt_recorded",
        5 => "failed",
        _ => "invalid_status",
    }
}

fn valid_image(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 200
        && s.bytes().all(|b| {
            b.is_ascii_alphanumeric() || matches!(b, b'/' | b'.' | b'_' | b':' | b'-' | b'@')
        })
}

fn docker_command(gateway: &str) -> Command {
    let mut command = if gateway == "colima" {
        let mut c = Command::new("colima");
        c.args(["ssh", "--", "docker"]);
        c
    } else {
        Command::new("docker")
    };
    command.kill_on_drop(true);
    command
}

async fn docker_ready(gateway: &str, image: &str) -> Result<bool, String> {
    for args in [
        vec!["info", "--format", "{{.ServerVersion}}"],
        vec!["image", "inspect", image],
    ] {
        let status = tokio::time::timeout(
            Duration::from_secs(10),
            docker_command(gateway)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status(),
        )
        .await
        .map_err(|_| "frontier Docker probe timed out".to_string())?
        .map_err(|e| format!("frontier Docker probe: {e}"))?;
        if !status.success() {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn enough_colima_memory(gateway: &str) -> Result<bool, String> {
    if gateway != "colima" {
        return Ok(true);
    }
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        Command::new("colima")
            .args(["ssh", "--", "cat", "/proc/meminfo"])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| "Colima memory probe timed out".to_string())?
    .map_err(|e| format!("Colima memory probe: {e}"))?;
    if !output.status.success() {
        return Err("Colima memory probe failed".into());
    }
    let text = std::str::from_utf8(&output.stdout).map_err(|e| e.to_string())?;
    let available = text
        .lines()
        .find_map(|line| line.strip_prefix("MemAvailable:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or("Colima memory probe omitted MemAvailable")?;
    Ok(available >= MIN_COLIMA_AVAILABLE_KIB)
}

fn persist_receipt(repo: &str, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_RECEIPT_BYTES {
        return Err("frontier receipt size invalid".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    if text
        .lines()
        .filter(|line| line.starts_with("decision\t"))
        .count()
        != 1
    {
        return Err("frontier receipt lacks one target decision".into());
    }
    let dir = std::path::Path::new(repo).join("results/frontier-loop");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = dir.join(format!(
        "docker-discovery-{}-{timestamp}.receipt.tsv",
        std::process::id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}

/// Start at most one isolated frontier worker after an idle grace interval.
/// Missing configuration disables the supervisor; no host fallback exists.
pub fn spawn_if_configured(state: Arc<AppState>) -> Result<bool, String> {
    let image = std::env::var("ARES_FRONTIER_IMAGE");
    let repo = std::env::var("ARES_FRONTIER_REPO");
    let (image, repo) = match (image, repo) {
        (Err(std::env::VarError::NotPresent), Err(std::env::VarError::NotPresent)) => {
            return Ok(false)
        }
        (Ok(image), Ok(repo)) => (image, repo),
        _ => return Err("ARES_FRONTIER_IMAGE and ARES_FRONTIER_REPO must be set together".into()),
    };
    if !valid_image(&image) {
        return Err("frontier Docker image configuration invalid".into());
    }
    if !std::path::Path::new(&repo).is_absolute() {
        return Err("frontier repository path must be absolute".into());
    }
    let gateway =
        std::env::var("ARES_FRONTIER_DOCKER_GATEWAY").unwrap_or_else(|_| "docker".to_string());
    if gateway != "docker" && gateway != "colima" {
        return Err("frontier Docker gateway must be docker or colima".into());
    }
    state.frontier_status.store(1, Ordering::Release);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(POLL);
        let mut idle_since = Instant::now();
        loop {
            interval.tick().await;
            if state.active_missions() != 0 {
                idle_since = Instant::now();
                continue;
            }
            if idle_since.elapsed() < IDLE_GRACE
                || state
                    .frontier_running
                    .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                    .is_err()
            {
                continue;
            }
            let readiness = match docker_ready(&gateway, &image).await {
                Ok(true) => enough_colima_memory(&gateway).await,
                other => other,
            };
            match readiness {
                Ok(true) => {
                    state.frontier_status.store(3, Ordering::Release);
                    let name = format!("ares-frontier-{}", std::process::id());
                    let script = r#"set -o pipefail
cd "$1" || exit 1
commit="$(git rev-parse HEAD)" || exit 2
if [ "$4" = colima ]; then docker_cmd=(colima ssh -- docker); else docker_cmd=(docker); fi
git archive "$commit" | "${docker_cmd[@]}" run --rm -i --name "$3" --network none --read-only --mount type=volume,destination=/work --memory 768m --cpus 2 --pids-limit 256 --cap-drop ALL --security-opt no-new-privileges --env HOME=/work --env TMPDIR=/work/tmp --env CARGO_HOME=/work/cargo --env CARGO_TARGET_DIR=/work/target --env CARGO_BUILD_JOBS=1 --env ARES_FRONTIER_SOURCE_COMMIT="$commit" "$2" /bin/sh -c 'mkdir -p /work/src /work/cargo /work/tmp && ln -s /usr/local/cargo/registry /work/cargo/registry && tar -xf - -C /work/src && exec /usr/local/bin/frontier-worker --once /work/src'
"#;
                    let mut command = Command::new("bash");
                    command
                        .args(["-c", script, "frontier", &repo, &image, &name, &gateway])
                        .kill_on_drop(true)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    match tokio::time::timeout(JOB_LIMIT, command.output()).await {
                        Ok(Ok(output)) => {
                            let stdout_digest = format!("{:x}", Sha256::digest(&output.stdout));
                            let stderr_digest = format!("{:x}", Sha256::digest(&output.stderr));
                            eprintln!(
                                "frontier Docker job status={} stdout_sha256={stdout_digest} stderr_sha256={stderr_digest}",
                                output.status
                            );
                            if output.status.success() && output.stdout.len() <= MAX_RECEIPT_BYTES {
                                match persist_receipt(&repo, &output.stdout) {
                                    Ok(()) => state.frontier_status.store(4, Ordering::Release),
                                    Err(e) => {
                                        state.frontier_status.store(5, Ordering::Release);
                                        eprintln!("frontier receipt persistence: {e}");
                                    }
                                }
                            } else {
                                state.frontier_status.store(5, Ordering::Release);
                            }
                        }
                        Ok(Err(e)) => {
                            state.frontier_status.store(5, Ordering::Release);
                            eprintln!("frontier Docker job transport: {e}");
                        }
                        Err(_) => {
                            state.frontier_status.store(5, Ordering::Release);
                            let _ = docker_command(&gateway)
                                .args(["rm", "-f", &name])
                                .stdout(Stdio::null())
                                .stderr(Stdio::null())
                                .status()
                                .await;
                            eprintln!("frontier Docker job exceeded time limit");
                        }
                    }
                }
                Ok(false) => {
                    state.frontier_status.store(2, Ordering::Release);
                    eprintln!("frontier Docker daemon, image, or memory unavailable");
                }
                Err(e) => {
                    state.frontier_status.store(2, Ordering::Release);
                    eprintln!("frontier Docker probe failed: {e}");
                }
            }
            state.frontier_running.store(false, Ordering::Release);
            idle_since = Instant::now();
        }
    });
    Ok(true)
}
