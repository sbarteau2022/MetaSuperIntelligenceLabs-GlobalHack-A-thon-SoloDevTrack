//! Model-neutral idle supervisor. It launches only an already-built Docker
//! worker image and streams a committed source snapshot without host mounts.

use crate::governor::AppState;
use std::process::Stdio;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;

const IDLE_GRACE: Duration = Duration::from_secs(300);
const POLL: Duration = Duration::from_secs(60);
const JOB_LIMIT: Duration = Duration::from_secs(1800);

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

/// Start at most one isolated frontier worker after an idle grace interval.
/// Missing configuration disables the supervisor; no host fallback exists.
pub fn spawn_if_configured(state: Arc<AppState>) -> Result<bool, String> {
    let image = match std::env::var("ARES_FRONTIER_IMAGE") {
        Ok(image) => image,
        Err(std::env::VarError::NotPresent) => return Ok(false),
        Err(e) => return Err(format!("frontier image configuration: {e}")),
    };
    if !valid_image(&image) {
        return Err("frontier Docker image configuration invalid".into());
    }
    let repo = std::env::var("ARES_FRONTIER_REPO")
        .map_err(|_| "ARES_FRONTIER_REPO required with ARES_FRONTIER_IMAGE".to_string())?;
    if !std::path::Path::new(&repo).is_absolute() {
        return Err("frontier repository path must be absolute".into());
    }
    let gateway =
        std::env::var("ARES_FRONTIER_DOCKER_GATEWAY").unwrap_or_else(|_| "docker".to_string());
    if gateway != "docker" && gateway != "colima" {
        return Err("frontier Docker gateway must be docker or colima".into());
    }
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
            match docker_ready(&gateway, &image).await {
                Ok(true) => {
                    let name = format!("ares-frontier-{}", std::process::id());
                    let script = r#"set -o pipefail
cd "$1" || exit 1
test -z "$(git status --porcelain)" || exit 2
if [ "$4" = colima ]; then docker_cmd=(colima ssh -- docker); else docker_cmd=(docker); fi
git archive HEAD | "${docker_cmd[@]}" run --rm -i --name "$3" --network none --read-only --user 65532:65532 --tmpfs /work:rw,exec,size=2g,uid=65532,gid=65532 --memory 8g --cpus 4 --pids-limit 256 --cap-drop ALL --security-opt no-new-privileges --env HOME=/work --env CARGO_HOME=/work/cargo --env CARGO_TARGET_DIR=/work/target "$2" /bin/sh -c 'mkdir -p /work/src /work/tmp && tar -xf - -C /work/src && exec /usr/local/bin/frontier-worker --once /work/src'
"#;
                    let mut command = Command::new("bash");
                    command
                        .args(["-c", script, "frontier", &repo, &image, &name, &gateway])
                        .kill_on_drop(true)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null());
                    match tokio::time::timeout(JOB_LIMIT, command.status()).await {
                        Ok(Ok(status)) => eprintln!("frontier Docker job status={status}"),
                        Ok(Err(e)) => eprintln!("frontier Docker job transport: {e}"),
                        Err(_) => {
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
                Ok(false) => eprintln!("frontier Docker daemon or image unavailable"),
                Err(e) => eprintln!("frontier Docker probe failed: {e}"),
            }
            state.frontier_running.store(false, Ordering::Release);
            idle_since = Instant::now();
        }
    });
    Ok(true)
}
