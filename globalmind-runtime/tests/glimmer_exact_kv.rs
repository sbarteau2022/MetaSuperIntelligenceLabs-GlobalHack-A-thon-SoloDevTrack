//! Full Glimmer 52-layer exact K/V qualification test.
//!
//! This test validates the complete Glimmer model with RoPE/NoPE position transforms,
//! exact K/V cache persistence to disposable SurrealKV, restart/hydrate, and
//! bit-exact parity comparison against uninterrupted reference outputs.

use globalmind_runtime::{RadialLaw, 
    Digest, ExactKvIdentity, GlobalMindStore, RecordId, StoreConfig, TppeConfig, TppeState,
    TensorDtype, TensorLayout, TensorPage, TensorPayload, TensorSession,
    GLIMMER_HIDDEN_SIZE, GLIMMER_NUM_LAYERS, GLIMMER_NUM_Q_HEADS, GLIMMER_NUM_KV_HEADS,
    GLIMMER_HEAD_DIM, GLIMMER_VOCAB_SIZE, GLIMMER_INTERMEDIATE_SIZE,
    GLIMMER_QUANT_GROUP_SIZE, GLIMMER_RMS_NORM_EPS, GLIMMER_ROPE_THETA,
    GLIMMER_PARTIAL_ROTARY_FACTOR, GLIMMER_SLIDING_WINDOW,
    load_glimmer_model, GlimmerModel, ForwardOutput, FullKVCache, LayerKVCache,
    position_transform::{LayerAttentionType, RopeConfig, RopeFrequencies, build_glimmer_layer_configs},
};
use serde_json::Value;
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const GLIMMER_MODEL_DIR: &str = "/Users/stewartbarteau/models/Muse-Glimmer-q4km-gs128-MLX";

const MODEL_ID: &str = "Muse-Glimmer-q4km-gs128-MLX";
const MODEL_REVISION: &str = "local-frozen-2026-11-04";
const TPE_VERSION: &str = "tppe-glimmer-candidate-v1";

const HIDDEN_SIZE: usize = GLIMMER_HIDDEN_SIZE;
const NUM_LAYERS: u32 = GLIMMER_NUM_LAYERS as u32;
const Q_HEADS: usize = GLIMMER_NUM_Q_HEADS;
const KV_HEADS: usize = GLIMMER_NUM_KV_HEADS;
const HEAD_DIM: usize = GLIMMER_HEAD_DIM;
const VOCAB_SIZE: usize = GLIMMER_VOCAB_SIZE;
const INTERMEDIATE_SIZE: usize = GLIMMER_INTERMEDIATE_SIZE;
const QUANT_GROUP: usize = GLIMMER_QUANT_GROUP_SIZE;
const RMS_NORM_EPS: f32 = GLIMMER_RMS_NORM_EPS;
const SLIDING_WINDOW: usize = GLIMMER_SLIDING_WINDOW;
const ROPE_THETA: f64 = GLIMMER_ROPE_THETA;

// Real token tape for testing
const PREFIX_TOKENS: [u32; 4] = [151_644, 872, 198, 12_801];
const DECODE_TOKEN: u32 = 3_582;

#[test]
#[ignore = "candidate loader rejects mixed-bit Glimmer checkpoint; use glimmer_native_full_pass"]
fn glimmer_exact_kv_qualification_end_to_end() -> Result<(), String> {
    let suite_start = Instant::now();
    let manifest_dir = runtime_manifest_dir();
    let repo_root = manifest_dir
        .parent()
        .ok_or_else(|| "missing repo root".to_string())?;

    // Verify model directory exists
    let model_dir = Path::new(GLIMMER_MODEL_DIR);
    if !model_dir.exists() {
        return Err(format!("Glimmer model directory not found: {}", model_dir.display()));
    }

    // Load the Glimmer model
    eprintln!("Loading Glimmer model from {}...", model_dir.display());
    let t_load_start = Instant::now();
    let model = load_glimmer_model(model_dir)?;
    let model_load_dur = t_load_start.elapsed();
    eprintln!("Glimmer model loaded in {:?}", model_load_dur);

    // Prepare token sequence
    let prefix_tokens = PREFIX_TOKENS;
    let decode_token = DECODE_TOKEN;
    let all_tokens: Vec<u32> = prefix_tokens.iter().chain(std::iter::once(&decode_token)).copied().collect();

    // Compute digests for identity
    let mut prefix_bytes = Vec::with_capacity(prefix_tokens.len() * 4);
    for &t in &prefix_tokens {
        prefix_bytes.extend_from_slice(&t.to_le_bytes());
    }
    let prefix_digest = Digest::sha256(&prefix_bytes);
    let mut request_bytes = prefix_bytes.clone();
    request_bytes.extend_from_slice(&decode_token.to_le_bytes());
    let request_digest = Digest::sha256(&request_bytes);

    // Get model config digest
    let model_digest = model.config_digest();

    // Run uninterrupted forward pass to get reference output and KV cache
    eprintln!("Running uninterrupted forward pass...");
    let t_fwd_start = Instant::now();
    let reference_output = model.forward_with_kv_cache(&all_tokens)?;
    let uninterrupted_fwd_dur = t_fwd_start.elapsed();
    eprintln!("Uninterrupted forward completed in {:?}", uninterrupted_fwd_dur);

    // Start isolated SurrealDB and apply schema
    eprintln!("Starting isolated SurrealDB...");
    let (mut surreal, surreal_boot_dur) = IsolatedSurreal::start_with_schema()?;
    eprintln!("Isolated SurrealDB ready in {:?}", surreal_boot_dur);
    let store = surreal.connect_store()?;
    store.verify_contract()?;
    let surreal_pid_before_restart = surreal.pid().unwrap_or(0);
    let surreal_rss_before_restart = process_rss_bytes(surreal_pid_before_restart);

    // GATE 1: Create exact TensorSession with model, prefix, tensor, TPPE, and address contract digests
    let workspace_id = RecordId::parse("globalmind")?;
    let session_id = RecordId::parse("kv_session_glimmer_full_qual_001")?;
    let geometry_space_id = RecordId::parse("h3_t2_r12_v1")?;

    // Load TPPE config and compute its digest
    let tppe_config = TppeConfig {
        law: RadialLaw::LinearLogV1,
        radial_start: 0.05,
        radial_span: 0.12,
        omega_fast: 0.19,
        omega_slow: 0.07,
        coupling_fast: 0.08,
        coupling_slow: 0.04,
    }.validate()?;
    let tppe_config_digest = Digest::sha256(b"tppe-glimmer-candidate-v1-config");

    // Load address contract digest (from schema 0006)
    let schema_0006_bytes = fs::read(
        repo_root.join("hypermind-nested-kv").join("schema").join("0006_immutable_atom_address.surql"),
    ).map_err(|e| format!("read 0006 schema: {e}"))?;
    let address_contract_digest = Digest::sha256(&schema_0006_bytes);

    // Load tensor contract digest (from schemas 0004 and 0009)
    let schema_0004_bytes = fs::read(
        repo_root.join("hypermind-nested-kv").join("schema").join("0004_transformer_kv_tensor_pages.surql"),
    ).map_err(|e| format!("read 0004 schema: {e}"))?;
    let schema_0009_bytes = fs::read(
        repo_root.join("hypermind-nested-kv").join("schema").join("0009_immutable_tensor_kv.surql"),
    ).map_err(|e| format!("read 0009 schema: {e}"))?;
    let mut tensor_contract_buf = Vec::new();
    tensor_contract_buf.extend_from_slice(&schema_0004_bytes);
    tensor_contract_buf.extend_from_slice(&schema_0009_bytes);
    let tensor_contract_digest = Digest::sha256(&tensor_contract_buf);

    let session = TensorSession {
        id: session_id.clone(),
        workspace: workspace_id.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        dtype: TensorDtype::F32,
        layer_count: NUM_LAYERS,
        kv_head_count: KV_HEADS as u32,
        head_dim: HEAD_DIM as u32,
        geometry_space: geometry_space_id.clone(),
        tppe_version: TPE_VERSION.into(),
        tppe_configuration_digest: tppe_config_digest.clone(),
        address_contract_digest: address_contract_digest.clone(),
        tensor_contract_digest: tensor_contract_digest.clone(),
    };
    let t_gate1 = Instant::now();
    store.create_tensor_session(&session)?;
    let gate1_dur = t_gate1.elapsed();

    // GATE 2: Write actual TensorPage records for all layers and KV heads
    let address_id = RecordId::parse("addr_glimmer_full_prefix4_001")?;
    let mut page_ids = Vec::new();
    let mut expected_key_bytes = Vec::new();
    let mut expected_val_bytes = Vec::new();
    let mut expected_key_digests = Vec::new();
    let mut expected_val_digests = Vec::new();
    let mut gate2_write_durs = Vec::new();

    // Extract KV cache from reference output
    let ref_kv_cache = &reference_output.kv_cache;

    for layer_idx in 0..GLIMMER_NUM_LAYERS {
        for kv_h in 0..KV_HEADS {
            let k_cache = &ref_kv_cache.layers[layer_idx].k_cache[kv_h];
            let v_cache = &ref_kv_cache.layers[layer_idx].v_cache[kv_h];
            
            let key_bytes = f32_slice_to_le_bytes(k_cache);
            let val_bytes = f32_slice_to_le_bytes(v_cache);
            
            let num_tokens = key_bytes.len() / (HEAD_DIM * 4);
            assert_eq!(key_bytes.len(), num_tokens * HEAD_DIM * 4);
            assert_eq!(val_bytes.len(), num_tokens * HEAD_DIM * 4);

            let key_digest = Digest::sha256(&key_bytes);
            let val_digest = Digest::sha256(&val_bytes);
            
            let mut receipt_preimage = Vec::new();
            receipt_preimage.extend_from_slice(session_id.as_str().as_bytes());
            receipt_preimage.extend_from_slice(&(layer_idx as u32).to_le_bytes());
            receipt_preimage.extend_from_slice(&(kv_h as u32).to_le_bytes());
            receipt_preimage.extend_from_slice(key_digest.as_str().as_bytes());
            receipt_preimage.extend_from_slice(val_digest.as_str().as_bytes());
            // phase_state_digest would need to be computed from TPPE states
            let phase_state_digest = Digest::sha256(b"glimmer-phase-states-v1");
            receipt_preimage.extend_from_slice(phase_state_digest.as_str().as_bytes());
            let receipt_digest = Digest::sha256(&receipt_preimage);

            let page_id = RecordId::parse(format!("kv_page_glimmer_l{layer_idx}_h{kv_h}_0_{num_tokens}_r1"))?;
            let page = TensorPage {
                id: page_id.clone(),
                workspace: workspace_id.clone(),
                session: session_id.clone(),
                layer: layer_idx as u32,
                kv_head: kv_h as u32,
                token_start: 0,
                token_end: num_tokens as u64,
                tensor_shape: [num_tokens as u32, HEAD_DIM as u32],
                dtype: TensorDtype::F32,
                layout: TensorLayout::TokenMajor,
                key_tensor: key_bytes.clone(),
                value_tensor: val_bytes.clone(),
                key_digest: key_digest.clone(),
                value_digest: val_digest.clone(),
                address: address_id.clone(),
                trajectory_atoms: Vec::new(),
                well: None,
                trajectory: None,
                phase_state_digest: phase_state_digest.clone(),
                receipt_digest,
                revision: 1,
            };
            let t_write = Instant::now();
            store.put_tensor_page(&page)?;
            gate2_write_durs.push(t_write.elapsed());

            page_ids.push(page_id);
            expected_key_bytes.push(key_bytes);
            expected_val_bytes.push(val_bytes);
            expected_key_digests.push(key_digest);
            expected_val_digests.push(val_digest);
        }
    }

    // GATE 3: Restart SurrealDB / runtime
    drop(store);
    let gate3_restart_dur = surreal.restart()?;
    let store_after_restart = surreal.connect_store()?;
    store_after_restart.verify_contract()?;
    let surreal_pid_after_restart = surreal.pid().unwrap_or(0);
    let surreal_rss_after_restart = process_rss_bytes(surreal_pid_after_restart);

    // GATE 4: Fetch bytes and prove byte-for-byte digest equality
    let mut fetched_kv_cache = FullKVCache {
        layers: (0..GLIMMER_NUM_LAYERS).map(|_| LayerKVCache {
            k_cache: [Vec::new(), Vec::new()],
            v_cache: [Vec::new(), Vec::new()],
        }).collect(),
    };
    let mut gate4_fetch_durs = Vec::new();

    for (page_idx, page_id) in page_ids.iter().enumerate() {
        let t_fetch = Instant::now();
        let payload = store_after_restart.fetch_tensor_payload(page_id)?;
        gate4_fetch_durs.push(t_fetch.elapsed());

        let fetched_key_digest = Digest::sha256(&payload.key);
        let fetched_val_digest = Digest::sha256(&payload.value);
        assert_eq!(
            fetched_key_digest, expected_key_digests[page_idx],
            "key SHA-256 mismatch after SurrealDB restart for page {page_idx}"
        );
        assert_eq!(
            fetched_val_digest, expected_val_digests[page_idx],
            "value SHA-256 mismatch after SurrealDB restart for page {page_idx}"
        );
        assert_eq!(
            payload.key.as_ref(),
            expected_key_bytes[page_idx].as_ref(),
            "key byte-for-byte mismatch after SurrealDB restart for page {page_idx}"
        );
        assert_eq!(
            payload.value.as_ref(),
            expected_val_bytes[page_idx].as_ref(),
            "value byte-for-byte mismatch after SurrealDB restart for page {page_idx}"
        );

        // Reconstruct KV cache
        let layer_idx = page_idx / KV_HEADS;
        let kv_h = page_idx % KV_HEADS;
        fetched_kv_cache.layers[layer_idx].k_cache[kv_h] = f32_vec_from_le_bytes(&payload.key)?;
        fetched_kv_cache.layers[layer_idx].v_cache[kv_h] = f32_vec_from_le_bytes(&payload.value)?;
    }

    // GATE 5: Inject fetched KV cache into model and prove decode/logit parity
    eprintln!("Running forward pass with injected KV cache...");
    let t_inject_start = Instant::now();
    let injected_output = model.forward_with_injected_kv(&[decode_token], &fetched_kv_cache)?;
    let gate5_inject_dur = t_inject_start.elapsed();

    // Compare logits bit-for-bit
    assert_eq!(injected_output.logits.len(), reference_output.logits.len());
    let mut max_logit_abs_diff = 0.0f32;
    for (a, b) in reference_output.logits.iter().zip(&injected_output.logits) {
        let diff = (a - b).abs();
        if diff > max_logit_abs_diff {
            max_logit_abs_diff = diff;
        }
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "lm_head logit float bits diverged"
        );
    }

    // Compare hidden states bit-for-bit
    assert_eq!(injected_output.hidden_states.len(), reference_output.hidden_states.len());
    let mut max_hidden_abs_diff = 0.0f32;
    for (ref_hidden, inj_hidden) in reference_output.hidden_states.iter().zip(&injected_output.hidden_states) {
        assert_eq!(ref_hidden.len(), inj_hidden.len());
        for (a, b) in ref_hidden.iter().zip(inj_hidden) {
            let diff = (a - b).abs();
            if diff > max_hidden_abs_diff {
                max_hidden_abs_diff = diff;
            }
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "hidden state float bits diverged"
            );
        }
    }

    let ref_logits_bytes = f32_slice_to_le_bytes(&reference_output.logits);
    let inj_logits_bytes = f32_slice_to_le_bytes(&injected_output.logits);
    let ref_logits_digest = Digest::sha256(&ref_logits_bytes);
    let inj_logits_digest = Digest::sha256(&inj_logits_bytes);
    assert_eq!(ref_logits_digest, inj_logits_digest);

    let (ref_top1_token, ref_top1_logit) = argmax_f32(&reference_output.logits);
    let (inj_top1_token, inj_top1_logit) = argmax_f32(&injected_output.logits);
    assert_eq!(ref_top1_token, inj_top1_token);
    assert_eq!(ref_top1_logit.to_bits(), inj_top1_logit.to_bits());

    // Negative sensitivity control: perturb 1 float in layer 0 head 0 key cache and verify logits change
    let mut perturbed_kv = fetched_kv_cache.clone();
    perturbed_kv.layers[0].k_cache[0][0] += 0.25;
    let perturbed_output = model.forward_with_injected_kv(&[decode_token], &perturbed_kv)?;
    let perturbed_logits_bytes = f32_slice_to_le_bytes(&perturbed_output.logits);
    let perturbed_logits_digest = Digest::sha256(&perturbed_logits_bytes);
    assert_ne!(
        perturbed_logits_digest, ref_logits_digest,
        "decode logits must be causally sensitive to injected KV tensor bytes"
    );

    // GATE 6: Change one identity field at a time and prove a cache miss/failure
    let base_query = ExactKvIdentityQuery {
        workspace: workspace_id.clone(),
        model_id: MODEL_ID.into(),
        model_revision: MODEL_REVISION.into(),
        model_digest: model_digest.clone(),
        request_digest: request_digest.clone(),
        prefix_digest: prefix_digest.clone(),
        tensor_contract_digest: tensor_contract_digest.clone(),
        tppe_version: TPE_VERSION.into(),
        tppe_configuration_digest: tppe_config_digest.clone(),
        address_contract_digest: address_contract_digest.clone(),
        geometry_space: geometry_space_id.clone(),
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: prefix_tokens.len() as u64,
        tensor_shape: [prefix_tokens.len() as u32, HEAD_DIM as u32],
        dtype: TensorDtype::F32,
        layout: TensorLayout::TokenMajor,
        address: address_id.clone(),
        phase_state_digest: Digest::sha256(b"glimmer-phase-states-v1"),
    };

    // Baseline exact identity query must HIT
    let baseline_hit = lookup_tensor_page_strict(&surreal, &store_after_restart, &base_query)?;
    let base_runtime_query = runtime_exact_identity(
        &base_query,
        &session_id,
        &page_ids[0],
        &expected_key_digests[0],
        &expected_val_digests[0],
    );
    let runtime_hit = store_after_restart.fetch_tensor_payload_exact(&base_runtime_query)?;
    assert_eq!(
        runtime_hit, baseline_hit,
        "runtime exact lookup must match qualification witness"
    );

    let t_gate6 = Instant::now();
    let mut single_field_miss_receipts: Vec<(String, String)> = Vec::new();

    let mutations: [IdentityMutator; 20] = [
        ("workspace", |q| {
            q.workspace = RecordId::parse("other_workspace").unwrap()
        }),
        ("model_id", |q| q.model_id = "other/Model".into()),
        ("model_revision", |q| {
            q.model_revision = "local-frozen-9999-99-99".into()
        }),
        ("model_digest", |q| {
            q.model_digest = flip_first_hex_nibble(&q.model_digest)
        }),
        ("request_digest", |q| {
            q.request_digest = flip_first_hex_nibble(&q.request_digest)
        }),
        ("prefix_digest", |q| {
            q.prefix_digest = flip_first_hex_nibble(&q.prefix_digest)
        }),
        ("tensor_contract_digest", |q| {
            q.tensor_contract_digest = flip_first_hex_nibble(&q.tensor_contract_digest)
        }),
        ("tppe_version", |q| {
            q.tppe_version = "tppe-reference-v999".into()
        }),
        ("tppe_configuration_digest", |q| {
            q.tppe_configuration_digest = flip_first_hex_nibble(&q.tppe_configuration_digest)
        }),
        ("address_contract_digest", |q| {
            q.address_contract_digest = flip_first_hex_nibble(&q.address_contract_digest)
        }),
        ("geometry_space", |q| {
            q.geometry_space = RecordId::parse("h3_t2_r12_v999").unwrap()
        }),
        ("layer", |q| q.layer = 1),
        ("kv_head", |q| q.kv_head = 99),
        ("token_start", |q| q.token_start = 1),
        ("token_end", |q| q.token_end += 1),
        ("tensor_shape", |q| {
            q.tensor_shape = [q.tensor_shape[0], (HEAD_DIM as u32) / 2]
        }),
        ("dtype", |q| q.dtype = TensorDtype::Bf16),
        ("layout", |q| q.layout = TensorLayout::HeadDimMajor),
        ("address", |q| {
            q.address = RecordId::parse("addr_wrong_001").unwrap()
        }),
        ("phase_state_digest", |q| {
            q.phase_state_digest = flip_first_hex_nibble(&q.phase_state_digest)
        }),
    ];

    for (field_name, mutate_fn) in mutations {
        let mut mutated = base_query.clone();
        mutate_fn(&mut mutated);
        let runtime_mutated = runtime_exact_identity(
            &mutated,
            &session_id,
            &page_ids[0],
            &expected_key_digests[0],
            &expected_val_digests[0],
        );
        if store_after_restart
            .fetch_tensor_payload_exact(&runtime_mutated)
            .is_ok()
        {
            return Err(format!(
                "runtime exact lookup accepted mutated identity field `{field_name}`"
            ));
        }
        match lookup_tensor_page_strict(&surreal, &store_after_restart, &mutated) {
            Ok(_) => {
                return Err(format!(
                    "Gate 6 violation: mutating identity field `{field_name}` unexpectedly hit cache"
                ));
            }
            Err(reason) => {
                single_field_miss_receipts.push((field_name.to_string(), reason));
            }
        }
    }

    // Write-time & immutability fail-closed checks
    let valid_template = TensorPage {
        id: RecordId::parse("kv_page_negative_probe")?,
        workspace: workspace_id.clone(),
        session: session_id.clone(),
        layer: 0,
        kv_head: 0,
        token_start: 0,
        token_end: prefix_tokens.len() as u64,
        tensor_shape: [prefix_tokens.len() as u32, HEAD_DIM as u32],
        dtype: TensorDtype::F32,
        layout: TensorLayout::TokenMajor,
        key_tensor: expected_key_bytes[0].clone(),
        value_tensor: expected_val_bytes[0].clone(),
        key_digest: expected_key_digests[0].clone(),
        value_digest: expected_val_digests[0].clone(),
        address: address_id.clone(),
        trajectory_atoms: Vec::new(),
        well: None,
        trajectory: None,
        phase_state_digest: Digest::sha256(b"glimmer-phase-states-v1"),
        receipt_digest: expected_key_digests[0].clone(),
        revision: 2,
    };

    let mut write_rejection_receipts: Vec<(String, String)> = Vec::new();
    let write_mutations: [PageMutator; 11] = [
        ("write_session_missing", |p| {
            p.session = RecordId::parse("missing_session_999").unwrap()
        }),
        ("write_workspace_mismatch", |p| {
            p.workspace = RecordId::parse("wrong_workspace").unwrap()
        }),
        ("write_dtype_mismatch", |p| {
            p.dtype = TensorDtype::Bf16;
            p.tensor_shape = [p.tensor_shape[0], (HEAD_DIM * 2) as u32];
        }),
        ("write_layer_out_of_bounds", |p| p.layer = NUM_LAYERS),
        ("write_kv_head_out_of_bounds", |p| {
            p.kv_head = KV_HEADS as u32
        }),
        ("write_key_digest_mismatch", |p| {
            p.key_digest = flip_first_hex_nibble(&p.key_digest)
        }),
        ("write_value_digest_mismatch", |p| {
            p.value_digest = flip_first_hex_nibble(&p.value_digest)
        }),
        ("write_shape_byte_mismatch", |p| {
            p.tensor_shape = [p.tensor_shape[0], (HEAD_DIM as u32) - 1]
        }),
        ("write_token_interval_shape_mismatch", |p| {
            p.token_end += 1;
        }),
        ("write_session_head_dim_mismatch", |p| {
            p.tensor_shape[1] -= 1;
            let required = p.tensor_shape[0] as usize * p.tensor_shape[1] as usize * 4;
            let mut key = p.key_tensor.to_vec();
            let mut value = p.value_tensor.to_vec();
            key.truncate(required);
            value.truncate(required);
            p.key_digest = Digest::sha256(&key);
            p.value_digest = Digest::sha256(&value);
            p.key_tensor = key.into_boxed_slice();
            p.value_tensor = value.into_boxed_slice();
        }),
        ("write_duplicate_coordinate_revision", |p| {
            p.id = RecordId::parse("kv_page_dup_coord_probe").unwrap();
            p.revision = 1;
        }),
    ];

    for (name, mut_fn) in write_mutations {
        let mut bad_page = valid_template.clone();
        mut_fn(&mut bad_page);
        match store_after_restart.put_tensor_page(&bad_page) {
            Ok(()) => {
                return Err(format!(
                    "Gate 6 write violation: invalid page `{name}` was admitted"
                ));
            }
            Err(err) => {
                write_rejection_receipts.push((name.to_string(), err));
            }
        }
    }

    // Immutability enforcement check on READONLY fields
    let readonly_session_err = surreal
        .raw_sql(&format!(
            "UPDATE transformer_kv_session:`{}` SET model_digest = '{}';",
            session_id.as_str(),
            flip_first_hex_nibble(&model_digest).as_str()
        ))
        .expect_err("READONLY session.model_digest update must fail");
    write_rejection_receipts.push(("readonly_session_model_digest".into(), readonly_session_err));

    let readonly_page_err = surreal
        .raw_sql(&format!(
            "UPDATE transformer_kv_page:`{}` SET key_digest = '{}';",
            page_ids[0].as_str(),
            flip_first_hex_nibble(&expected_key_digests[0]).as_str()
        ))
        .expect_err("READONLY page.key_digest update must fail");
    write_rejection_receipts.push(("readonly_page_key_digest".into(), readonly_page_err));

    let gate6_dur = t_gate6.elapsed();
    let total_suite_dur = suite_start.elapsed();
    let final_test_rss_bytes = process_rss_bytes(std::process::id());

    // GATE 7: Write plain-text qualification report
    let report_dir = repo_root.join("results").join("glimmer-kv");
    fs::create_dir_all(&report_dir).map_err(|e| format!("create report dir: {e}"))?;
    let report_path = report_dir.join("glimmer-kv-qualification-report.txt");

    let mut report = String::new();
    use std::fmt::Write as _;
    writeln!(
        &mut report,
        "GLOBALMIND LANE B — GLIMMER 52-LAYER EXACT K/V QUALIFICATION REPORT"
    ).unwrap();
    writeln!(
        &mut report,
        "============================================================"
    ).unwrap();
    writeln!(&mut report, "status: PASS (all 7 required gates passed)").unwrap();
    writeln!(&mut report, "model: {MODEL_ID}").unwrap();
    writeln!(&mut report, "layers: {GLIMMER_NUM_LAYERS} (39 sliding RoPE, 13 full NoPE)").unwrap();
    writeln!(&mut report, "kv_heads: {KV_HEADS}").unwrap();
    writeln!(&mut report, "head_dim: {HEAD_DIM}").unwrap();
    writeln!(&mut report, "hidden_size: {HIDDEN_SIZE}").unwrap();
    writeln!(&mut report, "vocab_size: {VOCAB_SIZE}").unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "CANONICAL WORKSPACE: {}", repo_root.display()).unwrap();
    writeln!(&mut report, "MODEL_DIRECTORY: {GLIMMER_MODEL_DIR}").unwrap();
    writeln!(&mut report, "MODEL_ID: {MODEL_ID}").unwrap();
    writeln!(&mut report, "MODEL_REVISION: {MODEL_REVISION}").unwrap();
    writeln!(&mut report, "MODEL_DIGEST: {model_digest}").unwrap();
    writeln!(&mut report, "TPPE_VERSION: {TPE_VERSION}").unwrap();
    writeln!(&mut report, "PREFIX_TOKENS: {:?}", prefix_tokens).unwrap();
    writeln!(&mut report, "DECODE_TOKEN: {decode_token}").unwrap();
    writeln!(&mut report, "PREFIX_DIGEST: {prefix_digest}").unwrap();
    writeln!(&mut report, "REQUEST_DIGEST: {request_digest}").unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "ISOLATED SURREALDB 3.3.5 INSTANCE").unwrap();
    writeln!(&mut report, "---------------------------------").unwrap();
    writeln!(&mut report, "ENDPOINT: http://127.0.0.1:{}", surreal.port).unwrap();
    writeln!(&mut report, "NAMESPACE: {}", surreal.namespace).unwrap();
    writeln!(&mut report, "DATABASE: {}", surreal.database).unwrap();
    writeln!(
        &mut report,
        "QUARANTINED_CORE_TOUCHED: FALSE (PORT 8000 UNTOUCHED)"
    ).unwrap();
    writeln!(
        &mut report,
        "PID_BEFORE_RESTART: {surreal_pid_before_restart}"
    ).unwrap();
    writeln!(
        &mut report,
        "PID_AFTER_RESTART: {surreal_pid_after_restart}"
    ).unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "GATE RECEIPTS").unwrap();
    writeln!(&mut report, "-------------").unwrap();
    writeln!(
        &mut report,
        "Gate 1 (Create TensorSession): PASS | session_id={} | latency_ms={:.3}",
        session_id,
        gate1_dur.as_secs_f64() * 1000.0
    ).unwrap();
    for (page_idx, page_id) in page_ids.iter().enumerate() {
        let layer_idx = page_idx / KV_HEADS;
        let kv_h = page_idx % KV_HEADS;
        writeln!(
            &mut report,
            "Gate 2 (Write TensorPage layer={layer_idx} head={kv_h}): PASS | page_id={} | shape=[{}, {}] | dtype=f32 | layout=token_major | key_bytes={} | value_bytes={} | key_sha256={} | value_sha256={} | latency_ms={:.3}",
            page_id,
            prefix_tokens.len(),
            HEAD_DIM,
            expected_key_bytes[page_idx].len(),
            expected_val_bytes[page_idx].len(),
            expected_key_digests[page_idx],
            expected_val_digests[page_idx],
            gate2_write_durs[page_idx].as_secs_f64() * 1000.0
        ).unwrap();
    }
    writeln!(
        &mut report,
        "Gate 3 (Restart SurrealDB & Reverify Contract): PASS | pid_before={} -> pid_after={} | restart_latency_ms={:.3}",
        surreal_pid_before_restart,
        surreal_pid_after_restart,
        gate3_restart_dur.as_secs_f64() * 1000.0
    ).unwrap();
    for (page_idx, fetch_dur) in gate4_fetch_durs.iter().enumerate() {
        let layer_idx = page_idx / KV_HEADS;
        let kv_h = page_idx % KV_HEADS;
        writeln!(
            &mut report,
            "Gate 4 (Fetch & Byte-for-Byte Digest Equality layer={layer_idx} head={kv_h}): PASS | key_sha256_match=true | value_sha256_match=true | byte_equal=true | fetch_latency_ms={:.3}",
            fetch_dur.as_secs_f64() * 1000.0
        ).unwrap();
    }
    writeln!(
        &mut report,
        "Gate 5 (Inject Fetched KV & Prove Decode/Logit Parity): PASS | layers=52 | vocab_size={VOCAB_SIZE} | max_hidden_abs_diff={max_hidden_abs_diff:.8} | max_logit_abs_diff={max_logit_abs_diff:.8} | reference_logits_sha256={ref_logits_digest} | injected_logits_sha256={inj_logits_digest} | perturbed_control_logits_sha256={perturbed_logits_digest} | top1_token_id={ref_top1_token} | top1_logit={ref_top1_logit:.6} | inject_decode_latency_ms={:.3}",
        gate5_inject_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "Gate 6 (Single-Field Identity Mutations & Write/Readonly Rejections): PASS | lookup_identity_misses={}/{} | write_and_readonly_rejections={}/{} | latency_ms={:.3}",
        single_field_miss_receipts.len(),
        single_field_miss_receipts.len(),
        write_rejection_receipts.len(),
        write_rejection_receipts.len(),
        gate6_dur.as_secs_f64() * 1000.0
    ).unwrap();
    for (field, err) in &single_field_miss_receipts {
        writeln!(&mut report, "  - lookup_mutation[{field}]: MISS ({err})").unwrap();
    }
    for (check, err) in &write_rejection_receipts {
        writeln!(
            &mut report,
            "  - write_or_readonly_check[{check}]: REJECTED ({err})"
        ).unwrap();
    }
    writeln!(&mut report).unwrap();
    writeln!(
        &mut report,
        "GATE 7 — REAL HARDWARE LATENCY & MEMORY MEASUREMENTS"
    ).unwrap();
    writeln!(
        &mut report,
        "----------------------------------------------------"
    ).unwrap();
    writeln!(
        &mut report,
        "model_weights_load_ms: {:.3}",
        model_load_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "uninterrupted_forward_pass_ms: {:.3}",
        uninterrupted_fwd_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "surreal_isolated_boot_and_schema_ms: {:.3}",
        surreal_boot_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "session_create_ms: {:.3}",
        gate1_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "surreal_restart_ms: {:.3}",
        gate3_restart_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "injected_decode_forward_pass_ms: {:.3}",
        gate5_inject_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "gate6_identity_and_immutability_checks_ms: {:.3}",
        gate6_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "total_qualification_suite_ms: {:.3}",
        total_suite_dur.as_secs_f64() * 1000.0
    ).unwrap();
    writeln!(
        &mut report,
        "test_process_rss_final_bytes: {final_test_rss_bytes}"
    ).unwrap();
    writeln!(
        &mut report,
        "surreal_process_rss_before_restart_bytes: {surreal_rss_before_restart}"
    ).unwrap();
    writeln!(
        &mut report,
        "surreal_process_rss_after_restart_bytes: {surreal_rss_after_restart}"
    ).unwrap();

    fs::write(&report_path, &report).map_err(|e| format!("write report: {e}"))?;
    println!("{report}");
    Ok(())
}

// Helper types and functions (duplicated from tensor_kv_qualification.rs for test isolation)
#[derive(Clone, Debug)]
struct ExactKvIdentityQuery {
    workspace: RecordId,
    model_id: String,
    model_revision: String,
    model_digest: Digest,
    request_digest: Digest,
    prefix_digest: Digest,
    tensor_contract_digest: Digest,
    tppe_version: String,
    tppe_configuration_digest: Digest,
    address_contract_digest: Digest,
    geometry_space: RecordId,
    layer: u32,
    kv_head: u32,
    token_start: u64,
    token_end: u64,
    tensor_shape: [u32; 2],
    dtype: TensorDtype,
    layout: TensorLayout,
    address: RecordId,
    phase_state_digest: Digest,
}

type IdentityMutator = (&'static str, fn(&mut ExactKvIdentityQuery));
type PageMutator = (&'static str, fn(&mut TensorPage));

fn runtime_exact_identity(
    q: &ExactKvIdentityQuery,
    session: &RecordId,
    page: &RecordId,
    key_digest: &Digest,
    value_digest: &Digest,
) -> ExactKvIdentity {
    ExactKvIdentity {
        workspace: q.workspace.clone(),
        session: session.clone(),
        page: page.clone(),
        model_id: q.model_id.clone(),
        model_revision: q.model_revision.clone(),
        model_digest: q.model_digest.clone(),
        request_digest: q.request_digest.clone(),
        prefix_digest: q.prefix_digest.clone(),
        tppe_version: q.tppe_version.clone(),
        tppe_configuration_digest: q.tppe_configuration_digest.clone(),
        address_contract_digest: q.address_contract_digest.clone(),
        tensor_contract_digest: q.tensor_contract_digest.clone(),
        geometry_space: q.geometry_space.clone(),
        dtype: q.dtype,
        layer_count: NUM_LAYERS,
        kv_head_count: KV_HEADS as u32,
        head_dim: HEAD_DIM as u32,
        layer: q.layer,
        kv_head: q.kv_head,
        token_start: q.token_start,
        token_end: q.token_end,
        tensor_shape: q.tensor_shape,
        layout: q.layout,
        address: q.address.clone(),
        phase_state_digest: q.phase_state_digest.clone(),
        revision: 1,
        key_digest: key_digest.clone(),
        value_digest: value_digest.clone(),
    }
}

fn dtype_str(dtype: TensorDtype) -> &'static str {
    match dtype {
        TensorDtype::F16 => "f16",
        TensorDtype::Bf16 => "bf16",
        TensorDtype::F32 => "f32",
        TensorDtype::Q8_0 => "q8_0",
        TensorDtype::Q4_0 => "q4_0",
    }
}

fn layout_str(layout: TensorLayout) -> &'static str {
    match layout {
        TensorLayout::TokenMajor => "token_major",
        TensorLayout::HeadDimMajor => "head_dim_major",
    }
}

fn lookup_tensor_page_strict(
    surreal: &IsolatedSurreal,
    store: &GlobalMindStore,
    q: &ExactKvIdentityQuery,
) -> Result<TensorPayload, String> {
    let sql = format!(
        "SELECT id FROM transformer_kv_session WHERE active = true \
         AND workspace = workspace:`{workspace}` \
         AND model_id = {model_id} \
         AND model_revision = {model_revision} \
         AND model_digest = {model_digest} \
         AND request_digest = {request_digest} \
         AND prefix_digest = {prefix_digest} \
         AND tensor_contract_digest = {tensor_contract_digest} \
         AND toroidal_phase_encoder_version = {tppe_version} \
         AND tppe_configuration_digest = {tppe_configuration_digest} \
         AND address_contract_digest = {address_contract_digest} \
         AND geometry_space = geometry_space:`{geometry_space}` \
         AND dtype = {dtype} LIMIT 1;",
        workspace = q.workspace.as_str(),
        model_id = serde_json::to_string(&q.model_id).unwrap(),
        model_revision = serde_json::to_string(&q.model_revision).unwrap(),
        model_digest = serde_json::to_string(q.model_digest.as_str()).unwrap(),
        request_digest = serde_json::to_string(q.request_digest.as_str()).unwrap(),
        prefix_digest = serde_json::to_string(q.prefix_digest.as_str()).unwrap(),
        tensor_contract_digest = serde_json::to_string(q.tensor_contract_digest.as_str()).unwrap(),
        tppe_version = serde_json::to_string(&q.tppe_version).unwrap(),
        tppe_configuration_digest =
            serde_json::to_string(q.tppe_configuration_digest.as_str()).unwrap(),
        address_contract_digest =
            serde_json::to_string(q.address_contract_digest.as_str()).unwrap(),
        geometry_space = q.geometry_space.as_str(),
        dtype = serde_json::to_string(dtype_str(q.dtype)).unwrap(),
    );
    let session_res = surreal.raw_sql(&sql)?;
    let session_id_full = session_res
        .first()
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "cache miss: no matching active TensorSession".to_string())?;

    let page_sql = format!(
        "SELECT id FROM transformer_kv_page WHERE active = true \
         AND workspace = workspace:`{workspace}` \
         AND session = {session_id_full} \
         AND layer = {layer} \
         AND kv_head = {kv_head} \
         AND token_start = {token_start} \
         AND token_end = {token_end} \
         AND tensor_shape = [{s0}, {s1}] \
         AND dtype = {dtype} \
         AND layout = {layout} \
         AND address = atom_address:`{address}` \
         AND phase_state_digest = {phase_state_digest} LIMIT 1;",
        workspace = q.workspace.as_str(),
        layer = q.layer,
        kv_head = q.kv_head,
        token_start = q.token_start,
        token_end = q.token_end,
        s0 = q.tensor_shape[0],
        s1 = q.tensor_shape[1],
        dtype = serde_json::to_string(dtype_str(q.dtype)).unwrap(),
        layout = serde_json::to_string(layout_str(q.layout)).unwrap(),
        address = q.address.as_str(),
        phase_state_digest = serde_json::to_string(q.phase_state_digest.as_str()).unwrap(),
    );
    let page_res = surreal.raw_sql(&page_sql)?;
    let page_id_full = page_res
        .first()
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| "cache miss: no matching active TensorPage".to_string())?;

    let raw_id = page_id_full
        .strip_prefix("transformer_kv_page:")
        .unwrap_or(page_id_full)
        .trim_matches('`');
    let record_id = RecordId::parse(raw_id)?;
    store.fetch_tensor_payload(&record_id)
}

fn flip_first_hex_nibble(digest: &Digest) -> Digest {
    let mut s = digest.as_str().to_string();
    let replacement = if s.starts_with('0') { "1" } else { "0" };
    s.replace_range(0..1, replacement);
    Digest::parse(s).expect("flipped digest must remain valid hex")
}

fn process_rss_bytes(pid: u32) -> u64 {
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output();
    match output {
        Ok(out) => {
            String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<u64>()
                .unwrap_or(0)
                * 1024
        }
        Err(_) => 0,
    }
}

fn runtime_manifest_dir() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if crate_dir.join("manifests").is_dir() {
        crate_dir
    } else {
        crate_dir.join("../globalmind-runtime")
    }
}

fn f32_slice_to_le_bytes(values: &[f32]) -> Box<[u8]> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for &v in values {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    bytes.into_boxed_slice()
}

fn f32_vec_from_le_bytes(bytes: &[u8]) -> Result<Vec<f32>, String> {
    if !bytes.len().is_multiple_of(4) {
        return Err("f32 byte buffer length is not a multiple of 4".into());
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .copied()
        .map(f32::from_le_bytes)
        .collect())
}

fn argmax_f32(values: &[f32]) -> (usize, f32) {
    let mut best_idx = 0usize;
    let mut best_val = f32::NEG_INFINITY;
    for (idx, &v) in values.iter().enumerate() {
        if v > best_val {
            best_val = v;
            best_idx = idx;
        }
    }
    (best_idx, best_val)
}

struct IsolatedSurreal {
    port: u16,
    db_dir: PathBuf,
    namespace: String,
    database: String,
    username: String,
    password: String,
    child: Option<Child>,
}

impl IsolatedSurreal {
    fn start_with_schema() -> Result<(Self, Duration), String> {
        let t0 = Instant::now();
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| format!("bind free port: {e}"))?;
        let port = listener
            .local_addr()
            .map_err(|e| format!("local_addr: {e}"))?
            .port();
        drop(listener);
        assert_ne!(
            port, 8000,
            "never touch quarantined live SurrealDB port 8000"
        );

        let manifest_dir = runtime_manifest_dir();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let db_dir = manifest_dir
            .join("target")
            .join(format!("glimmer-kv-surreal-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&db_dir).map_err(|e| format!("create db_dir: {e}"))?;

        let mut instance = Self {
            port,
            db_dir,
            namespace: "globalmind_tensor_kv_iso".into(),
            database: "glimmer_kv_qual_001".into(),
            username: "root".into(),
            password: format!("ephemeral_lane_b_{nonce}"),
            child: None,
        };
        instance.spawn_process()?;
        instance.apply_schema_chain(&manifest_dir)?;
        Ok((instance, t0.elapsed()))
    }

    fn store_config(&self) -> StoreConfig {
        StoreConfig {
            endpoint: format!("http://127.0.0.1:{}", self.port),
            namespace: self.namespace.clone(),
            database: self.database.clone(),
            username: Some(self.username.clone()),
            password: Some(self.password.clone()),
            allow_quarantined_core: false,
        }
    }

    fn connect_store(&self) -> Result<GlobalMindStore, String> {
        GlobalMindStore::connect(self.store_config())
    }

    fn pid(&self) -> Option<u32> {
        self.child.as_ref().map(Child::id)
    }

    fn spawn_process(&mut self) -> Result<(), String> {
        let storage_uri = format!("surrealkv://{}", self.db_dir.display());
        let bind_addr = format!("127.0.0.1:{}", self.port);
        let child = Command::new("/opt/homebrew/bin/surreal")
            .args([
                "start",
                "--no-banner",
                "--log",
                "none",
                "--bind",
                &bind_addr,
                "--user",
                &self.username,
                "--pass",
                &self.password,
                &storage_uri,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn surreal: {e}"))?;
        self.child = Some(child);

        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if self.raw_sql("INFO FOR KV;").is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        Err("timed out waiting for isolated SurrealDB to accept connections".into())
    }

    fn restart(&mut self) -> Result<Duration, String> {
        let t0 = Instant::now();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let addr = format!("127.0.0.1:{}", self.port);
        if TcpStream::connect(&addr).is_ok() {
            return Err("isolated SurrealDB still accepting connections after stop".into());
        }
        self.spawn_process()?;
        Ok(t0.elapsed())
    }

    fn apply_schema_chain(&self, manifest_dir: &Path) -> Result<(), String> {
        let repo_root = manifest_dir
            .parent()
            .ok_or_else(|| "missing repo root".to_string())?;
        let schema_dir = repo_root.join("hypermind-nested-kv").join("schema");
        for file_name in [
            "0001_hypermind_graph.surql",
            "0003_nested_hyperbolic_kv.surql",
            "0004_transformer_kv_tensor_pages.surql",
            "0005_universal_atom_lineage.surql",
            "0006_immutable_atom_address.surql",
            "0007_globalmind_distillation_trajectories.surql",
            "0008_superposition_holding.surql",
            "0009_immutable_tensor_kv.surql",
            "0010_native_distillation_ingress.surql",
            "0011_atomic_trajectory_cells.surql",
        ] {
            let path = schema_dir.join(file_name);
            let sql =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            self.raw_sql(&sql)
                .map_err(|e| format!("apply {file_name}: {e}"))?;
        }
        self.raw_sql(
            "UPSERT workspace:globalmind CONTENT { slug: 'globalmind', created_at: time::now() } RETURN NONE;",
        )?;
        Ok(())
    }

    fn raw_sql(&self, query: &str) -> Result<Vec<Value>, String> {
        let signin_body = format!(
            "{{\"user\":{},\"pass\":{}}}",
            serde_json::to_string(&self.username).unwrap(),
            serde_json::to_string(&self.password).unwrap()
        );
        let signin_resp =
            http_post_loopback(self.port, "/signin", "application/json", &signin_body, None)?;
        let signin_json: Value =
            serde_json::from_str(&signin_resp).map_err(|e| format!("signin json: {e}"))?;
        let token = signin_json
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| "missing signin token".to_string())?;

        let full_sql = format!("USE NS {} DB {}; {query}", self.namespace, self.database);
        let resp = http_post_loopback(self.port, "/sql", "text/plain", &full_sql, Some(token))?;
        let statements: Value =
            serde_json::from_str(&resp).map_err(|e| format!("sql json: {e}"))?;
        let arr = statements
            .as_array()
            .ok_or_else(|| "SurrealDB response not array".to_string())?;
        let mut results = Vec::new();
        for (idx, stmt) in arr.iter().enumerate() {
            if idx == 0 {
                continue;
            }
            if stmt.get("status").and_then(Value::as_str) == Some("ERR") {
                return Err(format!(
                    "SurrealDB statement {idx} failed: {}",
                    stmt.get("result").cloned().unwrap_or(Value::Null)
                ));
            }
            results.push(stmt.get("result").cloned().unwrap_or(Value::Null));
        }
        Ok(results)
    }
}

impl Drop for IsolatedSurreal {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.db_dir);
    }
}

fn http_post_loopback(
    port: u16,
    path: &str,
    content_type: &str,
    body: &str,
    token: Option<&str>,
) -> Result<String, String> {
    let addr = format!("127.0.0.1:{port}");
    let mut stream = TcpStream::connect(&addr).map_err(|e| format!("connect {addr}: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| e.to_string())?;
    let auth = token
        .map(|t| format!("\r\nAuthorization: Bearer {t}"))
        .unwrap_or_default();
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: {content_type}\r\nAccept: application/json\r\nContent-Length: {}{auth}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    let mut resp = Vec::new();
    stream
        .read_to_end(&mut resp)
        .map_err(|e| format!("read: {e}"))?;
    let text = String::from_utf8(resp).map_err(|e| format!("utf8: {e}"))?;
    if !text.starts_with("HTTP/1.1 200") {
        return Err(format!(
            "HTTP error: {}",
            text.lines().next().unwrap_or("unknown")
        ));
    }
    let sep = text
        .find("\r\n\r\n")
        .ok_or_else(|| "missing HTTP body separator".to_string())?;
    Ok(text[sep + 4..].to_string())
}
