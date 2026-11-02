//! Agent Integration — live distillation pipeline test.
//!
//! Envelopes -> extract -> judge -> replay -> walk artifact -> ingress ->
//! packets -> atoms -> trajectory -> readback -> restart -> re-verify from
//! persisted rows -> completed event -> count reconciliation -> report.
//!
//! Honesty: envelopes are constructed in process and the manifest artifact
//! is a temp stub (same level as the lane-5 walk). Real: artifact rehash,
//! extract, judge, replay verification with real TPPE stepping, atomic
//! store transactions, process restart, and exact reconciliation. No real
//! model forward pass exists yet (no trace producer), so this is
//! integration evidence, not distillation certification.

use globalmind_runtime::judge::JudgeVerdict;
use globalmind_runtime::replay::artifact::{
    EnvelopeDescriptor, decode as decode_walk, encode as encode_walk,
};
use globalmind_runtime::replay::{
    ReplayContext, ReplayInput, default_address_contract_digest, default_judge_policy,
    default_tppe_numeric, extract_and_judge, replay, upgrade_for_crystallization, verify_bindings,
};
use globalmind_runtime::trace::{
    TraceDtype, TraceEnvelope, TraceKind, decode_frame, encode as encode_trace,
};
use globalmind_runtime::{
    CellKind, Digest, DistillationPhase, DistillationRunEvent, GlobalMindStore, RadialLaw,
    RecordId, StoreConfig, TrajectoryState,
};
use std::collections::BTreeMap;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SCHEMA_FILES: [&str; 10] = [
    "0001_hypermind_graph",
    "0003_nested_hyperbolic_kv",
    "0004_transformer_kv_tensor_pages",
    "0005_universal_atom_lineage",
    "0006_immutable_atom_address",
    "0007_globalmind_distillation_trajectories",
    "0008_superposition_holding",
    "0009_immutable_tensor_kv",
    "0010_native_distillation_ingress",
    "0011_atomic_trajectory_cells",
];

struct IsolatedDb {
    child: Child,
    port: u16,
    storage: std::path::PathBuf,
}

impl IsolatedDb {
    fn start() -> Self {
        let storage = std::env::temp_dir().join(format!("gm-distill-db-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&storage);
        Self::start_at(storage)
    }

    fn start_at(storage: std::path::PathBuf) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        assert_ne!(port, 8000, "never touch the live port");
        let child = Command::new("surreal")
            .args([
                "start",
                "--no-banner",
                "--log",
                "error",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--user",
                "root",
                "--pass",
                "distillpass123",
                &format!("surrealkv://{}", storage.display()),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn isolated surrealdb (binary must be installed)");
        let db = Self {
            child,
            port,
            storage,
        };
        db.wait_ready();
        db
    }

    fn restart(self) -> Self {
        let storage = self.storage.clone();
        self.stop();
        Self::start_at(storage)
    }

    fn wait_ready(&self) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            if std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port)).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("isolated SurrealDB did not start on port {}", self.port);
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: format!("http://127.0.0.1:{}", self.port),
            namespace: "gmdistill".into(),
            database: "distilldev".into(),
            username: Some("root".into()),
            password: Some("distillpass123".into()),
            allow_quarantined_core: false,
        }
    }

    fn sql(&self, query: &str) -> String {
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port)).unwrap();
        let body = format!("USE NS gmdistill DB distilldev; {query}");
        let request = format!(
            "POST /sql HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.port,
            base64("root:distillpass123"),
            body.len(),
            body
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = Vec::new();
        std::io::Read::read_to_end(&mut stream, &mut response).unwrap();
        String::from_utf8(response).unwrap()
    }
}

fn base64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    use sha2::{Digest as _, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").unwrap();
    }
    Digest::parse(value).unwrap()
}

fn apply_schema(db: &IsolatedDb) {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../hypermind-nested-kv/schema");
    for file in SCHEMA_FILES {
        let text = std::fs::read_to_string(root.join(format!("{file}.surql")))
            .unwrap_or_else(|e| panic!("read schema {file}: {e}"));
        let response = db.sql(&text);
        let errors = response.matches("\"status\":\"ERR\"").count();
        let allowed = if file.starts_with("0001") { 1 } else { 0 };
        assert!(errors <= allowed, "schema {file} errors: {response:.200}");
    }
    let seed = format!(
        "CREATE workspace:`distillws` CONTENT {{ slug: 'distillws' }} RETURN NONE; \
         CREATE geometry_space:`h3t2` CONTENT {{ name: 'h3t2', manifold: 'H3xT2', quotient_symbol: 'Gamma12', quotient_status: 'specified', coordinate_model: 'lorentz-t2', contract_version: 'v1', contract_digest: '{}' }} RETURN NONE; \
         CREATE cognitive_hub:`hub` CONTENT {{ workspace: workspace:`distillws`, geometry_space: geometry_space:`h3t2`, label: 'hub', h3_center_lorentz: [1.0, 0.0, 0.0, 0.0], t2_phase: [0.0, 0.0], reference_contract_digest: '{}', status: 'specified' }} RETURN NONE;",
        "e".repeat(64),
        "f".repeat(64)
    );
    let wells: String = (0..12)
        .map(|i| {
            format!(
                "CREATE cognitive_well:`well{i}` CONTENT {{ workspace: workspace:`distillws`, geometry_space: geometry_space:`h3t2`, ordinal: {i}, label: 'well{i}', h3_center_lorentz: [1.0, 0.0, 0.0, 0.0], t2_sector: {i}, boundary_contract_digest: '{}', status: 'specified' }} RETURN NONE; ",
                "9".repeat(64)
            )
        })
        .collect();
    let response = db.sql(&format!("{seed} {wells}"));
    assert!(
        !response.contains("\"status\":\"ERR\""),
        "seed failed: {response:.300}"
    );
}

fn envelope(token: u64, values: &[f32]) -> TraceEnvelope {
    let payload: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    TraceEnvelope {
        model_digest: Digest::parse("a".repeat(64)).unwrap(),
        config_digest: Digest::parse("b".repeat(64)).unwrap(),
        tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
        kind: TraceKind::Activation,
        dtype: TraceDtype::F32,
        shape: vec![values.len() as u32],
        layer: 3,
        head: 1,
        token_position: token,
        observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
        payload_digest: Digest::sha256(&payload),
        payload: payload.into_boxed_slice(),
    }
}

#[test]
fn distillation_pipeline_survives_restart_and_reconciles_exactly() {
    let db = IsolatedDb::start();
    apply_schema(&db);
    let store = GlobalMindStore::connect(db.config()).unwrap();
    store.verify_contract().unwrap();

    // Envelopes (constructed) -> .gmtrace file roundtrip -> extract -> judge.
    let envelopes = vec![
        envelope(0, &[1.0, 0.0]),
        envelope(1, &[0.0, 1.0]),
        envelope(2, &[1.0, 0.0]),
    ];
    let mut trace_bytes = Vec::new();
    for envelope in &envelopes {
        trace_bytes.extend_from_slice(&encode_trace(envelope).unwrap());
    }
    let mut decoded = Vec::new();
    let mut rest = trace_bytes.as_slice();
    while !rest.is_empty() {
        let (envelope, consumed) = decode_frame(rest).unwrap();
        decoded.push(envelope);
        rest = &rest[consumed..];
    }
    assert_eq!(decoded.len(), envelopes.len());
    let judged = extract_and_judge(&decoded, default_judge_policy()).unwrap();
    assert_eq!(judged.len(), 5, "2 rank-0 + 2 rank-1 + 1 rank-2");
    assert!(judged.iter().all(|(_, r)| r.verdict == JudgeVerdict::Pass));

    // Verified manifest from a real temp artifact (eligibility/judge receipt
    // digests are owner-pending placeholders, same as the lane-5 walk).
    let artifact = std::env::temp_dir().join("gm-distill-weights.bin");
    std::fs::write(&artifact, b"distill-real-artifact-bytes").unwrap();
    let artifact_digest = Digest::sha256(b"distill-real-artifact-bytes");
    let manifest_text = format!(
        "workspace = distillws\nmethod = fresh-globalmind-distillation-v1\nconfiguration_digest = {}\ntpe_version = tppe-reference-v1\ntpe_configuration_digest = {}\nmodel.0.provider = local\nmodel.0.name = DistillModel\nmodel.0.revision = rev-1\nmodel.0.modality = text\nmodel.0.eligibility_receipt_digest = {}\nmodel.0.judge_receipt_digest = {}\nmodel.0.artifact.0.role = weights\nmodel.0.artifact.0.path = {}\nmodel.0.artifact.0.sha256 = {}\n",
        "1".repeat(64),
        "2".repeat(64),
        "3".repeat(64),
        "4".repeat(64),
        artifact.display(),
        artifact_digest.as_str()
    );
    let manifest_path = std::env::temp_dir().join("gm-distill.manifest");
    std::fs::write(&manifest_path, manifest_text).unwrap();
    let manifest = globalmind_runtime::Manifest::load(manifest_path.to_str().unwrap()).unwrap();
    let verified = manifest.verify_artifacts().unwrap();
    let model = &verified.models[0];

    // Replay with real TPPE stepping, then the walk artifact roundtrip.
    let context = ReplayContext {
        workspace: verified.workspace.clone(),
        run: verified.run_id.clone(),
        admission: model.admission_id.clone(),
        model_roster_digest: model.roster_digest.clone(),
        configuration_digest: verified.configuration_digest.clone(),
        tpe_version: verified.tpe_version.clone(),
        tpe_configuration_digest: verified.tpe_configuration_digest.clone(),
        radial_law: RadialLaw::LogV1,
        tpe_numeric: default_tppe_numeric(RadialLaw::LogV1),
        address_contract_digest: default_address_contract_digest(),
        runtime_digest: ReplayContext::runtime_digest_for(),
    };
    let chart_receipts = BTreeMap::new();
    let walk = replay(&ReplayInput {
        context: &context,
        envelopes: &decoded,
        judged: &judged,
        chart_receipts: &chart_receipts,
    })
    .unwrap();
    assert_eq!(walk.atoms.len(), 2);
    assert_eq!(walk.trajectory.steps.len(), 2);
    assert_eq!(walk.trajectory.loops.len(), 1);
    assert!(walk.charts.is_empty());
    assert_eq!(walk.trajectory.state, TrajectoryState::ReplayPassed);
    verify_bindings(&context, &walk).unwrap();
    let descriptors: Vec<EnvelopeDescriptor> = decoded
        .iter()
        .map(|envelope| EnvelopeDescriptor {
            kind: envelope.kind,
            dtype: envelope.dtype,
            shape: envelope.shape.clone(),
            layer: envelope.layer,
            head: envelope.head,
            token_position: envelope.token_position,
            payload_digest: envelope.payload_digest.clone(),
        })
        .collect();
    let walk_bytes = encode_walk(
        &context,
        &judged,
        &descriptors,
        &chart_receipts,
        &walk.trajectory_digest,
        &walk.candidate_bundle_digest,
        &walk.verdict_bundle_digest,
        &walk.replay_receipt_digest,
    )
    .unwrap();
    let file = decode_walk(&walk_bytes).unwrap();
    let rerun = replay(&ReplayInput {
        context: &file.context,
        envelopes: &decoded,
        judged: &file.judged,
        chart_receipts: &file.chart_receipts,
    })
    .unwrap();
    assert_eq!(rerun.replay_receipt_digest, walk.replay_receipt_digest);
    assert_eq!(rerun.trajectory_digest, file.trajectory_digest);

    // Crystallize through the typed store: ingress, then ONE atomic
    // bundle (phase events, packets, atoms, trajectory, charts).
    store.open_distillation_ingress(&verified).unwrap();
    let event =
        |ordinal: u32, phase: DistillationPhase, output: Option<Digest>| DistillationRunEvent {
            id: RecordId::parse(format!(
                "run_event_{}",
                &digest_fields(
                    b"globalmind-distill-test-event-v1",
                    &[verified.run_id.as_str(), &ordinal.to_string()],
                )
                .as_str()[..32]
            ))
            .unwrap(),
            workspace: verified.workspace.clone(),
            run: verified.run_id.clone(),
            ordinal,
            phase,
            input_digest: verified.input_digest.clone(),
            output_digest: output,
            tpe_version: verified.tpe_version.clone(),
            tpe_configuration_digest: verified.tpe_configuration_digest.clone(),
            receipt_digest: digest_fields(
                b"globalmind-distill-test-event-receipt-v1",
                &[&ordinal.to_string(), verified.input_digest.as_str()],
            ),
        };
    let trajectory = upgrade_for_crystallization(&walk).unwrap();
    let events = vec![
        event(
            1,
            DistillationPhase::Extracting,
            Some(walk.candidate_bundle_digest.clone()),
        ),
        event(
            2,
            DistillationPhase::Judging,
            Some(walk.verdict_bundle_digest.clone()),
        ),
        event(
            3,
            DistillationPhase::Replaying,
            Some(walk.replay_receipt_digest.clone()),
        ),
    ];

    // Crash-atomicity: a bundle poisoned with a duplicate packet id
    // (the second CREATE must fail) persists nothing at all — no
    // events, no packets, no atoms, no trajectory.
    let body_of0 = |response: &str| response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    let mut poisoned_packets = walk.packets.clone();
    poisoned_packets.push(walk.packets[0].clone());
    assert!(
        store
            .admit_crystallization(
                &events,
                &poisoned_packets,
                &walk.atoms,
                &trajectory,
                &walk.charts,
            )
            .is_err(),
        "poisoned bundle must fail"
    );
    let event_count =
        body_of0(&db.sql("SELECT count() AS c FROM distillation_run_event GROUP ALL;"));
    assert!(
        event_count.contains("\"c\":1"),
        "only the opened event may exist after a failed bundle: {event_count}"
    );
    let packet_count = body_of0(&db.sql("SELECT count() AS c FROM distillation_packet GROUP ALL;"));
    assert!(
        packet_count.contains("\"c\":0"),
        "failed bundle must persist zero packets: {packet_count}"
    );

    store
        .admit_crystallization(
            &events,
            &walk.packets,
            &walk.atoms,
            &trajectory,
            &walk.charts,
        )
        .unwrap();

    // Failed-write atomicity: duplicate admission fails, counts unchanged.
    let body_of = |response: &str| response.split("\r\n\r\n").nth(1).unwrap_or("").to_string();
    let before = body_of(&db.sql("SELECT count() AS c FROM distilled_trajectory GROUP ALL;"));
    assert!(store.admit_trajectory(&trajectory).is_err());
    let after = body_of(&db.sql("SELECT count() AS c FROM distilled_trajectory GROUP ALL;"));
    assert!(
        before.contains("\"c\":1") && after.contains("\"c\":1"),
        "failed write must be atomic: before={before} after={after}"
    );

    // Restart on the same storage, then re-verify the walk from persisted rows.
    let storage = db.storage.clone();
    let db = db.restart();
    let store = GlobalMindStore::connect(db.config()).unwrap();
    store.verify_contract().unwrap();
    let header = db.sql(&format!(
        "SELECT state, trajectory_digest, replay_receipt_digest, step_count FROM distilled_trajectory:`{}`;",
        trajectory.id.as_str()
    ));
    assert!(
        header.contains("crystallized"),
        "state persists: {header:.300}"
    );
    assert!(
        header.contains(walk.trajectory_digest.as_str()),
        "trajectory digest persists"
    );
    assert!(
        header.contains(walk.replay_receipt_digest.as_str()),
        "replay receipt persists"
    );
    let steps = db.sql(&format!(
        "SELECT ordinal, from_atom, to_atom, transition_digest FROM trajectory_step WHERE trajectory = distilled_trajectory:`{}` ORDER BY ordinal;",
        trajectory.id.as_str()
    ));
    assert!(
        steps.contains("\"ordinal\":0") || steps.contains("\"ordinal\": 0"),
        "step 0 persists: {steps:.300}"
    );
    // Rebuild the atom order from persisted steps and re-prove closure.
    let mut order: Vec<String> = Vec::new();
    for (index, step) in walk.trajectory.steps.iter().enumerate() {
        assert!(
            steps.contains(&format!("atom:{}", step.from_atom.as_str())),
            "step {index} from-atom persists"
        );
        assert!(
            steps.contains(&format!("atom:{}", step.to_atom.as_str())),
            "step {index} to-atom persists"
        );
        assert!(
            steps.contains(step.transition_digest.as_str()),
            "step {index} transition digest persists"
        );
        if index == 0 {
            order.push(step.from_atom.as_str().to_string());
        }
        order.push(step.to_atom.as_str().to_string());
    }
    assert_eq!(order.len(), walk.trajectory.atoms.len());
    for (persisted, expected) in order.iter().zip(walk.trajectory.atoms.iter()) {
        assert_eq!(persisted, expected.as_str());
    }
    let loops = db.sql(&format!(
        "SELECT first_step, last_step FROM trajectory_loop WHERE trajectory = distilled_trajectory:`{}`;",
        trajectory.id.as_str()
    ));
    assert!(
        loops.contains("\"first_step\":0") || loops.contains("\"first_step\": 0"),
        "loop persists: {loops:.300}"
    );
    assert_eq!(
        order[walk.trajectory.loops[0].first_step as usize],
        order[walk.trajectory.loops[0].last_step as usize + 1],
        "persisted loop still closes"
    );

    // Exact reconciliation: source candidates, verdicts, written entities,
    // ordinals, events, receipts.
    let count = |table: &str, extra: &str| {
        body_of(&db.sql(&format!(
            "SELECT count() AS c FROM {table} WHERE {extra} GROUP ALL;"
        )))
    };
    let run_filter = format!("run = distillation_run:`{}`", verified.run_id.as_str());
    let packets = count("distillation_packet", &run_filter);
    assert!(packets.contains("\"c\":5"), "5 packets persist: {packets}");
    let atoms = count(
        "atom",
        &format!(
            "distillation_run = distillation_run:`{}`",
            verified.run_id.as_str()
        ),
    );
    assert!(atoms.contains("\"c\":2"), "2 atoms persist: {atoms}");
    let events = count("distillation_run_event", &run_filter);
    assert!(events.contains("\"c\":4"), "4 events persist: {events}");
    assert_eq!(walk.packets.len(), 5);
    assert_eq!(walk.rejected.len(), 0);

    // Completed terminal event with the audit output digest.
    let output_digest = digest_fields(
        b"globalmind-distill-audit-output-v1",
        &[
            walk.trajectory_digest.as_str(),
            &walk.atoms.len().to_string(),
            &walk.trajectory.steps.len().to_string(),
            &walk.trajectory.loops.len().to_string(),
            &walk.charts.len().to_string(),
            walk.replay_receipt_digest.as_str(),
        ],
    );
    store
        .append_distillation_run_event(&event(
            4,
            DistillationPhase::Completed,
            Some(output_digest.clone()),
        ))
        .unwrap();

    // Plain-text report under results/distillation/.
    let rank = |kind: CellKind| walk.packets.iter().filter(|p| p.cell_kind == kind).count();
    let report = format!(
        "distillation integration report\n\
         run_id={}\ninput_digest={}\nmodel=DistillModel rev-1 (temp stub artifact, real rehash)\n\
         radial_law=log_v1 tpe_schedule=replay-v1\n\
         candidates=5 passed=5 rejected=0\nrank0={} rank1={} rank2={} rank8={}\n\
         packets_written=5 atoms_written=2 steps_written=2 loops_written=1 charts_written=0\n\
         trajectory_digest={}\nreplay_receipt={}\noutput_digest={}\n\
         restart=replayed-from-persisted-rows-identical\nbalance=PASS\n",
        verified.run_id.as_str(),
        verified.input_digest.as_str(),
        rank(CellKind::Rank0Atom),
        rank(CellKind::Rank1Transition),
        rank(CellKind::Rank2Loop),
        rank(CellKind::Rank8Jacobian),
        walk.trajectory_digest.as_str(),
        walk.replay_receipt_digest.as_str(),
        output_digest.as_str(),
    );
    let report_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../results/distillation/distillation-integration-report.txt");
    std::fs::create_dir_all(report_path.parent().unwrap()).unwrap();
    std::fs::write(&report_path, &report).unwrap();

    db.stop();
    let _ = std::fs::remove_dir_all(&storage);
    let _ = std::fs::remove_file(&artifact);
    let _ = std::fs::remove_file(&manifest_path);
}
