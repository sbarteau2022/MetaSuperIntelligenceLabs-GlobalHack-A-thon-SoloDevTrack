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
use tokio::io::{AsyncRead, AsyncReadExt};
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
        6 => "cancelled_for_mission",
        7 => "awaiting_source_change",
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

async fn read_bounded<R: AsyncRead + Unpin>(reader: R) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_RECEIPT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| format!("frontier pipe read: {e}"))?;
    if bytes.len() > MAX_RECEIPT_BYTES {
        return Err("frontier pipe exceeded receipt bound".into());
    }
    Ok(bytes)
}

async fn bounded_command_output(mut command: Command) -> Result<std::process::Output, String> {
    let mut child = command
        .spawn()
        .map_err(|e| format!("frontier command spawn: {e}"))?;
    let stdout = child.stdout.take().ok_or("frontier stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("frontier stderr unavailable")?;
    let (status, stdout, stderr) = tokio::try_join!(
        async {
            child
                .wait()
                .await
                .map_err(|e| format!("frontier wait: {e}"))
        },
        read_bounded(stdout),
        read_bounded(stderr)
    )?;
    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
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

fn validate_receipt(bytes: &[u8], expected_commit: &str) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() > MAX_RECEIPT_BYTES {
        return Err("frontier receipt size invalid".into());
    }
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let mut gates = std::collections::HashSet::new();
    let mut decision = None;
    for line in text.lines() {
        let mut fields = line.split('\t');
        let kind = fields.next().ok_or("frontier receipt line empty")?;
        if kind != "gate" && kind != "decision" {
            return Err("frontier receipt has unknown line".into());
        }
        let mut source = None;
        let mut target = None;
        for field in fields {
            let (key, value) = field
                .split_once('=')
                .ok_or("frontier receipt field malformed")?;
            match key {
                "source" if source.replace(value).is_some() => {
                    return Err("frontier receipt repeats source".into());
                }
                "target" if target.replace(value).is_some() => {
                    return Err("frontier receipt repeats target".into());
                }
                "source" | "target" => {}
                _ => {}
            }
        }
        if source != Some(expected_commit) {
            return Err("frontier receipt source differs from pinned snapshot".into());
        }
        let target = target.ok_or("frontier receipt lacks target")?;
        if kind == "gate" {
            if !gates.insert(target) {
                return Err("frontier receipt repeats gate".into());
            }
        } else if decision.replace(target).is_some() {
            return Err("frontier receipt repeats decision".into());
        }
    }
    let selected = decision.ok_or("frontier receipt lacks decision")?;
    if selected != "none" && !gates.contains(selected) {
        return Err("frontier decision lacks matching gate".into());
    }
    if gates.is_empty() {
        return Err("frontier receipt has no gates".into());
    }
    Ok(())
}

fn persist_receipt(repo: &str, bytes: &[u8], expected_commit: &str) -> Result<(), String> {
    validate_receipt(bytes, expected_commit)?;
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

async fn pinned_commit(repo: &str) -> Result<String, String> {
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        Command::new("git")
            .args(["-C", repo, "rev-parse", "HEAD"])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| "frontier source pin timed out".to_string())?
    .map_err(|e| format!("frontier source pin: {e}"))?;
    if !output.status.success() {
        return Err("frontier source pin failed".into());
    }
    let commit = std::str::from_utf8(&output.stdout)
        .map_err(|e| e.to_string())?
        .trim()
        .to_string();
    if commit.len() != 40 || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("frontier source pin invalid".into());
    }
    Ok(commit)
}

async fn remove_frontier_container(gateway: &str, name: &str) {
    loop {
        let cleanup = tokio::time::timeout(
            Duration::from_secs(10),
            docker_command(gateway).args(["rm", "-f", name]).output(),
        )
        .await;
        if let Ok(Ok(output)) = cleanup {
            if output.status.success()
                || String::from_utf8_lossy(&output.stderr).contains("No such container")
            {
                break;
            }
        }
        eprintln!("frontier container cleanup pending; governed missions remain gated");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
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
        let mut last_completed_source: Option<String> = None;
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
            let request_epoch = state.frontier_request_epoch.load(Ordering::Acquire);
            let commit = pinned_commit(&repo).await;
            if matches!(&commit, Ok(source) if last_completed_source.as_ref() == Some(source)) {
                state.frontier_status.store(7, Ordering::Release);
                state.frontier_running.store(false, Ordering::Release);
                continue;
            }
            let readiness = match &commit {
                Ok(_) => match docker_ready(&gateway, &image).await {
                    Ok(true) => enough_colima_memory(&gateway).await,
                    other => other,
                },
                Err(e) => Err(e.clone()),
            };
            match readiness {
                Ok(true) => {
                    let Ok(_frontier_guard) = state.frontier_gate.try_write() else {
                        state.frontier_running.store(false, Ordering::Release);
                        idle_since = Instant::now();
                        continue;
                    };
                    if state.active_missions() != 0
                        || state.frontier_request_epoch.load(Ordering::Acquire) != request_epoch
                    {
                        state.frontier_running.store(false, Ordering::Release);
                        idle_since = Instant::now();
                        continue;
                    }
                    let commit = commit.expect("source pin checked before readiness");
                    state.frontier_status.store(3, Ordering::Release);
                    let name = format!("ares-frontier-{}", std::process::id());
                    let script = r#"set -o pipefail
cd "$1" || exit 1
commit="$5"
if [ "$4" = colima ]; then docker_cmd=(colima ssh -- docker); else docker_cmd=(docker); fi
git archive "$commit" | "${docker_cmd[@]}" run --rm -i --name "$3" --network none --read-only --mount type=volume,destination=/work --memory 768m --cpus 2 --pids-limit 256 --cap-drop ALL --security-opt no-new-privileges --env HOME=/work --env TMPDIR=/work/tmp --env CARGO_HOME=/work/cargo --env CARGO_TARGET_DIR=/work/target --env CARGO_BUILD_JOBS=1 --env ARES_FRONTIER_SOURCE_COMMIT="$commit" "$2" /bin/sh -c 'mkdir -p /work/src /work/cargo /work/tmp && ln -s /usr/local/cargo/registry /work/cargo/registry && tar -xf - -C /work/src && exec /usr/local/bin/frontier-worker --once /work/src'
"#;
                    let mut command = Command::new("bash");
                    command
                        .args([
                            "-c", script, "frontier", &repo, &image, &name, &gateway, &commit,
                        ])
                        .kill_on_drop(true)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());
                    let work = async {
                        let mut output = Box::pin(bounded_command_output(command));
                        loop {
                            tokio::select! {
                                result = &mut output => return Some(result),
                                _ = tokio::time::sleep(Duration::from_millis(250)) => {
                                    if state.frontier_request_epoch.load(Ordering::Acquire) != request_epoch {
                                        return None;
                                    }
                                }
                            }
                        }
                    };
                    match tokio::time::timeout(JOB_LIMIT, work).await {
                        Ok(Some(Ok(output))) => {
                            if state.frontier_request_epoch.load(Ordering::Acquire) != request_epoch
                            {
                                state.frontier_status.store(6, Ordering::Release);
                                remove_frontier_container(&gateway, &name).await;
                            } else {
                                let stdout_digest = format!("{:x}", Sha256::digest(&output.stdout));
                                let stderr_digest = format!("{:x}", Sha256::digest(&output.stderr));
                                eprintln!(
                                    "frontier Docker job status={} stdout_sha256={stdout_digest} stderr_sha256={stderr_digest}",
                                    output.status
                                );
                                if output.status.success()
                                    && output.stdout.len() <= MAX_RECEIPT_BYTES
                                {
                                    match persist_receipt(&repo, &output.stdout, &commit) {
                                        Ok(()) => {
                                            last_completed_source = Some(commit.clone());
                                            state.frontier_status.store(4, Ordering::Release);
                                        }
                                        Err(e) => {
                                            state.frontier_status.store(5, Ordering::Release);
                                            eprintln!("frontier receipt persistence: {e}");
                                        }
                                    }
                                } else {
                                    state.frontier_status.store(5, Ordering::Release);
                                }
                            }
                        }
                        Ok(Some(Err(e))) => {
                            state.frontier_status.store(5, Ordering::Release);
                            eprintln!("frontier Docker job transport: {e}");
                            remove_frontier_container(&gateway, &name).await;
                        }
                        outcome => {
                            let cancelled = matches!(outcome, Ok(None));
                            state
                                .frontier_status
                                .store(if cancelled { 6 } else { 5 }, Ordering::Release);
                            remove_frontier_container(&gateway, &name).await;
                            eprintln!(
                                "frontier Docker job stopped: {}",
                                if cancelled {
                                    "mission requested"
                                } else {
                                    "time limit"
                                }
                            );
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

#[cfg(test)]
mod tests {
    use super::{bounded_command_output, validate_receipt};
    use crate::governor::{default_base_weights, default_policy, AppState};
    use crate::store::CarvedSwarmMap;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::process::Command;

    const COMMIT: &str = "26dfa6ddf073f986fecb4037b1aa20d9f7b5633c";

    fn receipt(gate_source: &str, decision_source: &str, decision_target: &str) -> String {
        format!(
            "gate\tsource={gate_source}\ttarget=ares-backend\texit=0\toutput_sha256=abc\tduration_ms=1\n\
             decision\tsource={decision_source}\ttarget={decision_target}\treason=gate-failure\n"
        )
    }

    #[test]
    fn accepts_pinned_gate_and_decision() {
        let text = receipt(COMMIT, COMMIT, "ares-backend");
        assert!(validate_receipt(text.as_bytes(), COMMIT).is_ok());
    }

    #[test]
    fn refuses_mixed_or_unseen_source() {
        let other = "0000000000000000000000000000000000000000";
        assert!(
            validate_receipt(receipt(COMMIT, other, "ares-backend").as_bytes(), COMMIT).is_err()
        );
        assert!(
            validate_receipt(receipt(other, other, "ares-backend").as_bytes(), COMMIT).is_err()
        );
    }

    #[test]
    fn refuses_unmatched_or_duplicate_target() {
        let unseen = receipt(COMMIT, COMMIT, "unseen");
        assert!(validate_receipt(unseen.as_bytes(), COMMIT).is_err());
        let duplicated = format!(
            "{}{}",
            receipt(COMMIT, COMMIT, "ares-backend"),
            receipt(COMMIT, COMMIT, "ares-backend")
        );
        assert!(validate_receipt(duplicated.as_bytes(), COMMIT).is_err());
    }

    #[test]
    fn refuses_extra_source_field() {
        let text = receipt(COMMIT, COMMIT, "ares-backend").replacen(
            "target=ares-backend",
            "source=0000000000000000000000000000000000000000\ttarget=ares-backend",
            1,
        );
        assert!(validate_receipt(text.as_bytes(), COMMIT).is_err());
    }

    #[tokio::test]
    async fn mission_request_waits_for_frontier_barrier_and_changes_epoch() {
        let base = default_base_weights();
        let immune = crate::compliance::ImmuneSystem::new(default_policy(), base.fingerprint);
        let map = CarvedSwarmMap::new(base, crate::fixed::Q32_31::from_f64(1.22));
        let state = Arc::new(AppState::new(immune, map, None));
        let held = state.frontier_gate.write().await;
        let prior_epoch = state.frontier_request_epoch.load(Ordering::Acquire);
        let (entered_tx, mut entered_rx) = tokio::sync::oneshot::channel();
        let mission_state = state.clone();
        let mission = tokio::spawn(async move {
            let _guard = mission_state.enter_governed_run().await;
            entered_tx.send(()).expect("receiver remains open");
        });
        tokio::task::yield_now().await;
        assert!(state.frontier_request_epoch.load(Ordering::Acquire) > prior_epoch);
        assert!(matches!(
            entered_rx.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ));
        drop(held);
        entered_rx
            .await
            .expect("mission enters after frontier releases");
        mission.await.expect("mission task completes");
    }

    #[tokio::test]
    async fn docker_output_is_bounded_during_read() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "head -c 65537 /dev/zero"])
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let result = tokio::time::timeout(Duration::from_secs(5), bounded_command_output(command))
            .await
            .expect("bounded reader returns promptly");
        assert!(matches!(result, Err(reason) if reason == "frontier pipe exceeded receipt bound"));
    }
}
