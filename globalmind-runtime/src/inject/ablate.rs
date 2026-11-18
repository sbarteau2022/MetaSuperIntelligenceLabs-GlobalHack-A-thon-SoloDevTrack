//! Ablation harness: memory seeding from real traces, recall injection,
//! and dual-arm scoring for Queue 2 ablations 1-2.
//!
//! Heavy orchestration runs behind `#[ignore]`d tests (explicit invocation);
//! pure scoring/layout helpers are always-run unit tests.

use crate::{Manifest, VerifiedManifest};

pub const QWEN36_MANIFEST: &str = "manifests/qwen36-opus-candidate.manifest";

/// Loads + rehashes the frozen roster. One call per harness run; the
/// verified manifest is reused for every forward in the sweep.
pub fn load_frozen_roster() -> Result<VerifiedManifest, String> {
    Manifest::load(QWEN36_MANIFEST)?.verify_artifacts()
}

/// Stable next-token score from a full logit vector: (top1_hit, nll).
pub fn score_next_token(logits: &[f32], target: u32) -> Result<(bool, f64), String> {
    if logits.is_empty() {
        return Err("empty logit vector".into());
    }
    let target = target as usize;
    if target >= logits.len() {
        return Err("target token outside logit range".into());
    }
    let max_logit = logits.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b));
    if !max_logit.is_finite() {
        return Err("non-finite logits".into());
    }
    let mut sum_exp = 0.0f64;
    let mut argmax = 0usize;
    for (index, logit) in logits.iter().enumerate() {
        if *logit > logits[argmax] {
            argmax = index;
        }
        sum_exp += ((*logit - max_logit) as f64).exp();
    }
    let nll = -(((logits[target] - max_logit) as f64) - sum_exp.ln());
    if !nll.is_finite() {
        return Err("non-finite NLL".into());
    }
    Ok((argmax == target, nll))
}

/// Tape layout for the injection arm: recalled context tokens first, then
/// the prefix. Truncates oldest context first under budget; the receipt
/// records exactly what ran.
#[derive(Debug, Clone)]
pub struct InjectedTape {
    pub tokens: Vec<u32>,
    pub context_tokens: usize,
    pub prefix_tokens: usize,
    pub context_truncated: usize,
}

pub fn build_injected_tape(
    context_ids: &[u32],
    prefix_ids: &[u32],
    context_budget: usize,
) -> Result<InjectedTape, String> {
    if prefix_ids.is_empty() {
        return Err("prefix must be non-empty".into());
    }
    let take = context_ids.len().min(context_budget);
    let skipped = context_ids.len() - take;
    let mut tokens = Vec::with_capacity(take + prefix_ids.len());
    tokens.extend_from_slice(&context_ids[skipped..]);
    tokens.extend_from_slice(prefix_ids);
    Ok(InjectedTape {
        tokens,
        context_tokens: take,
        prefix_tokens: prefix_ids.len(),
        context_truncated: skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::extract;
    use crate::inject::tokenize::ByteLevelBpe;
    use crate::retrieval::{RetrievalConfig, RetrievalMetric, retrieve};
    use crate::trace::qwen36::{encode_token_tape, produce_real_qwen36_trace};
    use crate::trace::{TraceEnvelope, TraceKind};
    use crate::{CellKind, Digest, RecordId};
    use std::collections::BTreeMap;

    #[test]
    fn scoring_matches_hand_computed_values() {
        // Logits [1, 2, 3], target 2: top1 hit, NLL = 3 - logsumexp.
        let (hit, nll) = score_next_token(&[1.0, 2.0, 3.0], 2).unwrap();
        assert!(hit);
        let lse = ((1.0f64 - 3.0).exp() + (2.0f64 - 3.0).exp() + 1.0).ln() + 3.0;
        assert!((nll - (lse - 3.0)).abs() < 1e-9, "nll={nll}");
        let (miss, _) = score_next_token(&[1.0, 2.0, 3.0], 0).unwrap();
        assert!(!miss);
        assert!(score_next_token(&[], 0).is_err());
        assert!(score_next_token(&[1.0], 5).is_err());
        assert!(score_next_token(&[f32::NAN, 1.0], 1).is_err());
    }

    #[test]
    fn tape_layout_truncates_oldest_first_and_receipts() {
        let tape = build_injected_tape(&[10, 11, 12, 13], &[1, 2], 3).unwrap();
        assert_eq!(tape.tokens, vec![11, 12, 13, 1, 2]);
        assert_eq!(tape.context_tokens, 3);
        assert_eq!(tape.prefix_tokens, 2);
        assert_eq!(tape.context_truncated, 1);
        let tape = build_injected_tape(&[10, 11], &[1, 2], 8).unwrap();
        assert_eq!(tape.context_truncated, 0);
        assert!(build_injected_tape(&[10], &[], 8).is_err());
    }

    /// Line-window chunks: consecutive non-overlapping groups of lines.
    /// Deterministic; receipt records line ranges and token counts.
    #[derive(Debug, Clone)]
    pub struct TextChunk {
        pub index: usize,
        pub first_line: usize,
        pub last_line: usize,
        pub text: String,
        pub token_ids: Vec<u32>,
    }

    pub fn split_line_windows(
        bpe: &ByteLevelBpe,
        corpus: &str,
        lines_per_window: usize,
        min_tokens: usize,
        max_tokens: usize,
    ) -> Result<Vec<TextChunk>, String> {
        if lines_per_window == 0 || min_tokens == 0 || max_tokens < min_tokens {
            return Err("invalid chunking parameters".into());
        }
        let lines: Vec<&str> = corpus.lines().collect();
        let mut chunks = Vec::new();
        let mut index = 0usize;
        for window in lines.chunks(lines_per_window) {
            let text = window.join("\n");
            if text.trim().is_empty() {
                continue;
            }
            let token_ids = bpe.encode(&text)?;
            if token_ids.len() < min_tokens || token_ids.len() > max_tokens {
                continue;
            }
            let first_line = index * lines_per_window;
            chunks.push(TextChunk {
                index,
                first_line,
                last_line: first_line + window.len() - 1,
                text,
                token_ids,
            });
            index += 1;
        }
        Ok(chunks)
    }

    /// Decodes per-token F32 activation slices, ordered by token position.
    /// Fails closed unless exactly one slice per tape token exists.
    pub fn activation_slices(
        envelopes: &[crate::trace::TraceEnvelope],
        tape_len: usize,
    ) -> Result<Vec<Vec<f32>>, String> {
        use crate::trace::{TraceDtype, TraceKind};
        let mut slices: Vec<(u64, Vec<f32>)> = Vec::new();
        for envelope in envelopes {
            if envelope.kind != TraceKind::Activation || envelope.dtype != TraceDtype::F32 {
                continue;
            }
            if envelope.payload.len() % 4 != 0 {
                return Err("activation payload is not F32-aligned".into());
            }
            let values: Vec<f32> = envelope
                .payload
                .as_chunks::<4>()
                .0
                .iter()
                .map(|chunk| f32::from_le_bytes(*chunk))
                .collect();
            if values.is_empty() || !values.iter().all(|v| v.is_finite()) {
                return Err("activation slice is empty or non-finite".into());
            }
            slices.push((envelope.token_position, values));
        }
        slices.sort_by_key(|(pos, _)| *pos);
        if slices.len() != tape_len {
            return Err(format!(
                "expected one activation slice per token: {} slices for {} tokens",
                slices.len(),
                tape_len
            ));
        }
        for (index, (pos, _)) in slices.iter().enumerate() {
            if *pos as usize != index {
                return Err("activation slice positions are not contiguous".into());
            }
        }
        Ok(slices.into_iter().map(|(_, values)| values).collect())
    }

    /// Mean L2 + mean cosine distance over suffix-aligned slices: arm B
    /// positions [C..C+P) against arm A positions [0..P).
    pub fn slice_divergence(a: &[Vec<f32>], b: &[Vec<f32>]) -> Result<(f64, f64), String> {
        if b.len() < a.len() || a.is_empty() {
            return Err("divergence needs arm-B to cover arm-A positions".into());
        }
        let offset = b.len() - a.len();
        let mut l2_sum = 0.0f64;
        let mut cos_sum = 0.0f64;
        for (i, av) in a.iter().enumerate() {
            let bv = &b[offset + i];
            if av.len() != bv.len() || av.is_empty() {
                return Err("activation slice width mismatch".into());
            }
            let mut l2 = 0.0f64;
            let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
            for (x, y) in av.iter().zip(bv.iter()) {
                let (x, y) = (*x as f64, *y as f64);
                l2 += (x - y) * (x - y);
                dot += x * y;
                na += x * x;
                nb += y * y;
            }
            if na == 0.0 || nb == 0.0 {
                return Err("zero-norm activation slice".into());
            }
            l2_sum += l2.sqrt();
            cos_sum += 1.0 - dot / (na.sqrt() * nb.sqrt());
        }
        Ok((l2_sum / a.len() as f64, cos_sum / a.len() as f64))
    }

    #[test]
    fn divergence_matches_hand_computed_values() {
        let (l2, cos) = slice_divergence(
            &[vec![1.0, 0.0], vec![0.0, 1.0]],
            &[vec![1.0, 0.0], vec![0.0, 1.0]],
        )
        .unwrap();
        assert_eq!((l2, cos), (0.0, 0.0));
        let (l2, cos) =
            slice_divergence(&[vec![1.0, 0.0]], &[vec![9.0, 9.0], vec![0.0, 1.0]]).unwrap();
        assert!((l2 - std::f64::consts::SQRT_2).abs() < 1e-9, "{l2}");
        assert!((cos - 1.0).abs() < 1e-9, "{cos}");
        assert!(slice_divergence(&[vec![1.0]], &[vec![1.0, 2.0]]).is_err());
        assert!(slice_divergence(&[vec![1.0], vec![1.0]], &[vec![1.0]]).is_err());
        assert!(slice_divergence(&[], &[]).is_err());
    }

    #[test]
    fn slices_decode_order_and_reject_gaps() {
        use crate::trace::{TraceDtype, TraceEnvelope, TraceKind};
        fn envelope(pos: u64, values: &[f32]) -> TraceEnvelope {
            let payload: Vec<u8> = values.iter().flat_map(|v| v.to_le_bytes()).collect();
            TraceEnvelope {
                model_digest: Digest::parse("a".repeat(64)).unwrap(),
                config_digest: Digest::parse("b".repeat(64)).unwrap(),
                tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
                kind: TraceKind::Activation,
                dtype: TraceDtype::F32,
                shape: vec![values.len() as u32],
                layer: 3,
                head: 0,
                token_position: pos,
                observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
                payload_digest: Digest::sha256(&payload),
                payload: payload.into_boxed_slice(),
            }
        }
        let envelopes = vec![envelope(1, &[3.0, 4.0]), envelope(0, &[1.0, 2.0])];
        let slices = activation_slices(&envelopes, 2).unwrap();
        assert_eq!(slices, vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert!(activation_slices(&envelopes, 3).is_err());
        assert!(activation_slices(&[envelope(0, &[1.0]), envelope(2, &[2.0])], 2).is_err());
    }

    #[test]
    fn chunker_windows_are_deterministic_and_bounded() {
        let bpe = ByteLevelBpe::load(
            "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/tokenizer.json",
        )
        .unwrap();
        let corpus = "alpha beta\ngamma delta\nepsilon zeta\neta theta\n";
        let chunks = split_line_windows(&bpe, corpus, 2, 1, 1000).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!((chunks[0].first_line, chunks[0].last_line), (0, 1));
        assert_eq!((chunks[1].first_line, chunks[1].last_line), (2, 3));
        let none = split_line_windows(&bpe, corpus, 2, 10000, 20000).unwrap();
        assert!(none.is_empty());
        assert!(split_line_windows(&bpe, corpus, 0, 1, 10).is_err());
    }

    #[test]
    #[ignore]
    fn probe_forward_cost_and_envelope_layout() {
        // Sizing probe (explicit runs only): one rehash, then timed
        // sequential forwards + full envelope inventory.
        let start = std::time::Instant::now();
        let verified = load_frozen_roster().expect("roster rehash");
        println!("probe rehash_ms={}", start.elapsed().as_millis());
        let tape =
            crate::trace::qwen36::encode_token_tape(&[7, 23, 512, 96156, 2]).expect("tape encodes");
        let tape_digest = Digest::sha256(&tape);
        for round in 0..2 {
            let start = std::time::Instant::now();
            let trace = crate::trace::qwen36::produce_real_qwen36_trace(
                &verified,
                &[7, 23, 512, 96156, 2],
                &tape_digest,
            )
            .expect("forward runs");
            println!(
                "probe round={round} forward_ms={} envelopes={} stream_bytes={}",
                start.elapsed().as_millis(),
                trace.envelopes.len(),
                trace.stream_bytes.len()
            );
            if round == 0 {
                for envelope in &trace.envelopes {
                    println!(
                        "probe env kind={:?} dtype={:?} shape={:?} layer={} head={} pos={} payload_len={}",
                        envelope.kind,
                        envelope.dtype,
                        envelope.shape,
                        envelope.layer,
                        envelope.head,
                        envelope.token_position,
                        envelope.payload.len()
                    );
                }
                println!("probe trace_digest={}", trace.trace_digest.as_str());
            }
        }
    }

    // ---- A1 orchestration (heavy; run under --release -- --ignored) ----
    struct AblationDb {
        child: std::process::Child,
        port: u16,
        storage: std::path::PathBuf,
    }

    impl AblationDb {
        fn start() -> Self {
            let storage = std::env::temp_dir().join(format!(
                "gm-ab1-db-{}-{:?}-{}",
                std::process::id(),
                std::thread::current().id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos()
            ));
            let _ = std::fs::remove_dir_all(&storage);
            let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
            let port = listener.local_addr().expect("port").port();
            drop(listener);
            let child = std::process::Command::new("surreal")
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
                    "ab1_pw_007",
                    &format!("surrealkv://{}", storage.display()),
                ])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("spawn isolated surrealdb");
            let db = Self {
                child,
                port,
                storage,
            };
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while std::time::Instant::now() < deadline {
                if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
                    return db;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            panic!("isolated ablation SurrealDB did not start");
        }

        fn sql(&self, body: &str) -> String {
            use std::io::{Read, Write};
            let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", self.port))
                .expect("connect ablation db");
            let full = format!("USE NS ab1_inject DB memwall; {body}");
            let request = format!(
                "POST /sql HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: text/plain\r\nAccept: application/json\r\nAuthorization: Basic {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                self.port,
                ab1_base64("root:ab1_pw_007"),
                full.len(),
                full
            );
            stream.write_all(request.as_bytes()).expect("write ab1 sql");
            let mut response = Vec::new();
            stream
                .read_to_end(&mut response)
                .expect("read ab1 response");
            String::from_utf8(response).expect("ab1 response is UTF-8")
        }
    }

    impl Drop for AblationDb {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = std::fs::remove_dir_all(&self.storage);
        }
    }

    fn ab1_base64(input: &str) -> String {
        const TABLE: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let bytes = input.as_bytes();
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let mut n: u32 = 0;
            for &b in chunk {
                n = (n << 8) | b as u32;
            }
            n <<= 8 * (3 - chunk.len());
            for i in 0..4 {
                if i <= chunk.len() {
                    out.push(TABLE[((n >> (18 - 6 * i)) & 63) as usize] as char);
                } else {
                    out.push('=');
                }
            }
        }
        out
    }

    fn forward_timed(
        verified: &VerifiedManifest,
        ids: &[u32],
    ) -> (crate::trace::qwen36::ProducedTrace, u128) {
        let tape = encode_token_tape(ids).expect("tape encodes");
        let digest = Digest::sha256(&tape);
        let t0 = std::time::Instant::now();
        let pt = produce_real_qwen36_trace(verified, ids, &digest).expect("forward runs");
        (pt, t0.elapsed().as_millis())
    }

    fn rank0_coords(
        candidates: &[crate::extract::ExtractedCandidate],
    ) -> Vec<([f64; 4], [f64; 2])> {
        let mut out = Vec::new();
        for c in candidates {
            if c.cell_kind != CellKind::Rank0Atom {
                continue;
            }
            if let (Some(h), Some(t)) = (c.h3_lorentz, c.t2_phase) {
                out.push((h, t));
            }
        }
        out
    }

    fn activations_of(pt: &crate::trace::qwen36::ProducedTrace) -> Vec<TraceEnvelope> {
        pt.envelopes
            .iter()
            .filter(|e| e.kind == TraceKind::Activation)
            .cloned()
            .collect()
    }

    /// Decodes the single (KeyValue, F32) envelope as row-major [T, D].
    /// Fails closed unless exactly one exists with a 2-D shape matching
    /// its payload and finite values.
    fn kv_f32_rows(
        pt: &crate::trace::qwen36::ProducedTrace,
    ) -> Result<(usize, usize, Vec<f32>), String> {
        use crate::trace::TraceDtype;
        let hits: Vec<&TraceEnvelope> = pt
            .envelopes
            .iter()
            .filter(|e| e.kind == TraceKind::KeyValue && e.dtype == TraceDtype::F32)
            .collect();
        if hits.len() != 1 {
            return Err(format!("expected one F32 KV envelope, got {}", hits.len()));
        }
        let e = hits[0];
        if e.shape.len() != 2 {
            return Err(format!("KV shape is not 2-D: {:?}", e.shape));
        }
        let (t, d) = (e.shape[0] as usize, e.shape[1] as usize);
        if e.payload.len() != t * d * 4 {
            return Err("KV payload length mismatches shape".into());
        }
        let rows: Vec<f32> = e
            .payload
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        if rows.is_empty() || !rows.iter().all(|v| v.is_finite()) {
            return Err("KV rows are empty or non-finite".into());
        }
        Ok((t, d, rows))
    }

    /// Suffix-aligned KV comparison: arm-B rows [Tb-Ta..Tb) vs arm-A rows.
    /// Returns (rms divergence, bit-identity). Fails closed unless the
    /// widths match and arm B covers arm A.
    fn kv_suffix_divergence(
        a: &(usize, usize, Vec<f32>),
        b: &(usize, usize, Vec<f32>),
    ) -> Result<(f64, bool), String> {
        if a.1 != b.1 {
            return Err("KV width mismatch across arms".into());
        }
        if b.0 < a.0 || a.0 == 0 {
            return Err("KV arm-B does not cover arm-A".into());
        }
        let d = a.1;
        let suffix = &b.2[(b.0 - a.0) * d..];
        let bitid = a.2.as_slice() == suffix;
        let mut sum = 0.0f64;
        for (x, y) in a.2.iter().zip(suffix.iter()) {
            let diff = *x as f64 - *y as f64;
            sum += diff * diff;
        }
        Ok(((sum / a.2.len() as f64).sqrt(), bitid))
    }

    /// Which envelope kinds respond to prepended context? Runs arm-A
    /// (prefix alone) and arm-B (context + prefix) once, then compares every
    /// (kind, dtype) group suffix-wise. F32 groups get L2 + bit-identity;
    /// Q8 groups get byte-identity only. Informs which envelopes the
    /// ablation arms may honestly measure.
    #[test]
    #[ignore]
    fn probe_envelope_context_response() {
        const TOKENIZER: &str =
            "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/tokenizer.json";
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo = crate_dir.parent().expect("repo root").to_path_buf();
        let corpus = std::fs::read(repo.join("history.txt")).expect("corpus reads");
        let corpus_text = String::from_utf8(corpus).expect("corpus is UTF-8");
        let bpe = ByteLevelBpe::load(TOKENIZER).expect("bpe loads");
        let chunks = split_line_windows(&bpe, &corpus_text, 3, 40, 120)
            .expect("chunker runs on ASCII corpus");
        assert!(chunks.len() >= 9, "need 9 windows");
        // Same link as A1 row w24: X = w24 prefix, ctx = 60 toks of w12.
        let x_full = &chunks[8].token_ids;
        let x = &x_full[..x_full.len() / 2];
        let ctx = &chunks[4].token_ids[..60];
        let mut b_tape = Vec::with_capacity(60 + x.len());
        b_tape.extend_from_slice(ctx);
        b_tape.extend_from_slice(x);
        let verified = load_frozen_roster().expect("roster rehash");
        let (pt_a, ms_a) = forward_timed(&verified, x);
        let (pt_b, ms_b) = forward_timed(&verified, &b_tape);
        println!(
            "envresp xlen={} blen={} msA={ms_a} msB={ms_b}",
            x.len(),
            b_tape.len()
        );
        use crate::trace::TraceDtype;
        use std::collections::BTreeMap;
        let mut groups_a: BTreeMap<(String, String), Vec<&TraceEnvelope>> = BTreeMap::new();
        let mut groups_b: BTreeMap<(String, String), Vec<&TraceEnvelope>> = BTreeMap::new();
        for e in &pt_a.envelopes {
            groups_a
                .entry((format!("{:?}", e.kind), format!("{:?}", e.dtype)))
                .or_default()
                .push(e);
        }
        for e in &pt_b.envelopes {
            groups_b
                .entry((format!("{:?}", e.kind), format!("{:?}", e.dtype)))
                .or_default()
                .push(e);
        }
        for (key, ga) in &groups_a {
            let Some(gb) = groups_b.get(key) else {
                println!("envresp group={key:?} missing_in_B na={}", ga.len());
                continue;
            };
            if gb.len() < ga.len() {
                println!(
                    "envresp group={key:?} shape_differs na={} nb={}",
                    ga.len(),
                    gb.len()
                );
                continue;
            }
            let off = gb.len() - ga.len();
            let mut bitid = true;
            let mut l2_sum = 0.0f64;
            let mut l2_n = 0u64;
            let is_f32 = ga.iter().all(|e| e.dtype == TraceDtype::F32);
            for (i, ea) in ga.iter().enumerate() {
                let eb = gb[off + i];
                if ea.payload.len() != eb.payload.len() {
                    bitid = false;
                    continue;
                }
                if ea.payload.as_ref() != eb.payload.as_ref() {
                    bitid = false;
                }
                if is_f32 && ea.payload.len() % 4 == 0 {
                    for (ca, cb) in ea
                        .payload
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(eb.payload.as_chunks::<4>().0.iter())
                    {
                        let d = f32::from_le_bytes(*ca) as f64 - f32::from_le_bytes(*cb) as f64;
                        if d.is_finite() {
                            l2_sum += d * d;
                            l2_n += 1;
                        }
                    }
                }
            }
            let l2 = if l2_n > 0 {
                (l2_sum / l2_n as f64).sqrt()
            } else {
                f64::NAN
            };
            println!(
                "envresp group={key:?} na={} nb={} bitid={bitid} rms={l2:.10}",
                ga.len(),
                gb.len()
            );
        }
        for key in groups_b.keys() {
            if !groups_a.contains_key(key) {
                println!("envresp group={key:?} missing_in_A");
            }
        }
    }

    #[test]
    #[ignore]
    fn ablation1_injection_effect() {
        const M: usize = 8;
        const T: usize = 4;
        const K: u32 = 3;
        const BUDGET: usize = 60;
        const TOKENIZER: &str =
            "/Users/stewartbarteau/models/fused/Qwen3.6-MoE-Combined/tokenizer.json";
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo = crate_dir.parent().expect("repo root").to_path_buf();
        let corpus = std::fs::read(repo.join("history.txt")).expect("corpus reads");
        let corpus_digest = Digest::sha256(&corpus);
        let corpus_text = String::from_utf8(corpus).expect("corpus is UTF-8");
        let bpe = ByteLevelBpe::load(TOKENIZER).expect("bpe loads");
        let mut chunks = split_line_windows(&bpe, &corpus_text, 3, 40, 120)
            .expect("chunker runs on ASCII corpus");
        assert!(
            chunks.len() >= M + T,
            "need >= 12 eligible windows, got {}",
            chunks.len()
        );
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!(c.index, i, "chunk ordinals are dense");
        }
        chunks.truncate(M + T);
        let (mem, tests) = (&chunks[..M], &chunks[M..]);

        let t0 = std::time::Instant::now();
        let verified = load_frozen_roster().expect("roster rehash");
        let rehash_ms = t0.elapsed().as_millis();
        // Live nonperturbation receipt: every forward below asserts
        // max_logit/hidden_abs_diff == 0.0 from its own provenance.

        let abdb = AblationDb::start();
        let schema = std::fs::read_to_string(
            crate_dir.join("../hypermind-nested-kv/schema/0001_hypermind_graph.surql"),
        )
        .expect("0001 schema exists");
        let applied = abdb.sql(&schema);
        assert!(
            !applied.contains("\"status\":\"ERR\""),
            "schema applies: {applied}"
        );

        // ---- seed memory: forward each memory chunk, LAST rank-0 coords win.
        // (FIRST rank-0 proved input-degenerate: 7/8 seeds shared bit-identical
        // coords in the v1 run. LAST varies with input — the queries did.)
        let mut mem_text: BTreeMap<String, String> = BTreeMap::new();
        let mut seed_ms: u128 = 0;
        for c in mem {
            let (widx, text, ids) = (c.first_line, &c.text, &c.token_ids);
            let (pt, ms) = forward_timed(&verified, ids);
            seed_ms += ms;
            assert_eq!(
                pt.provenance.max_logit_abs_diff, 0.0,
                "live nonperturbation"
            );
            assert_eq!(
                pt.provenance.max_hidden_abs_diff, 0.0,
                "live nonperturbation"
            );
            let cands = extract(&activations_of(&pt)).expect("extract seed");
            let coords = rank0_coords(&cands);
            assert!(
                !coords.is_empty(),
                "seed window {widx} yields rank-0 coords"
            );
            let (h, t) = *coords.last().expect("non-empty");
            println!("seed w{widx} toks={} h3={h:.12?} t2={t:.12?}", ids.len());
            let cdigest = Digest::sha256(text.as_bytes());
            let seeded = abdb.sql(&format!(
                "CREATE atom:mem{widx} SET workspace = workspace:ab1mem, external_id = 'ab1_{widx}', \
                 content_kind = 'text', content_digest = '{}', domain = 'ab1-memory', \
                 rank = 0, salience = 0.5, provenance = 'ab1-seed', taint = 'clean'; \
                 CREATE atom_address:addr_mem{widx} SET workspace = workspace:ab1mem, atom = atom:mem{widx}, \
                 h3_lorentz = [{}, {}, {}, {}], t2_phase = [{}, {}], \
                 geometry_model = 'lorentz', geometry_version = 'v1', revision = 1;",
                cdigest.as_str(),
                h[0], h[1], h[2], h[3], t[0], t[1]
            ));
            assert!(
                !seeded.contains("\"status\":\"ERR\""),
                "seed {widx}: {seeded}"
            );
            mem_text.insert(format!("mem{widx}"), text.clone());
        }

        // ---- dual-arm runs: prefix alone vs prefix + top-K recalled text
        let ws = RecordId::parse("ab1mem").expect("workspace id");
        let cfg = RetrievalConfig {
            endpoint: format!("http://127.0.0.1:{}", abdb.port),
            namespace: "ab1_inject".into(),
            database: "memwall".into(),
            username: "root".into(),
            password: "ab1_pw_007".into(),
            workspace: ws,
            candidate_budget: 64,
            metric: RetrievalMetric::Hyperbolic,
        };
        let mut rows: Vec<(bool, f64, bool, f64, f64, f64, f64)> = Vec::new();
        let mut qa_ms: u128 = 0;
        let mut qb_ms: u128 = 0;
        for c in tests {
            let (widx, ids) = (c.first_line, &c.token_ids);
            let half = ids.len() / 2;
            let (prefix, target) = (&ids[..half], &ids[half..]);
            let (pt_a, ms_a) = forward_timed(&verified, prefix);
            qa_ms += ms_a;
            assert_eq!(
                pt_a.provenance.max_logit_abs_diff, 0.0,
                "live nonperturbation"
            );
            let cands_a = extract(&activations_of(&pt_a)).expect("extract query");
            let qc = rank0_coords(&cands_a);
            let q = *qc.last().expect("query coords");
            let bundle = retrieve(&cfg, q.0, q.1, "ab1-query", K).expect("retrieve");
            assert_eq!(bundle.members.len(), K as usize, "exactly K members");
            let mut rank_keys: Vec<String> = Vec::new();
            let mut dists: Vec<f64> = Vec::new();
            let mut recalled_texts: Vec<String> = Vec::new();
            for m in &bundle.members {
                let key = m
                    .atom
                    .as_str()
                    .strip_prefix("atom:")
                    .unwrap_or(m.atom.as_str());
                let text = mem_text
                    .get(key)
                    .unwrap_or_else(|| panic!("hit {key} is a seeded atom"));
                rank_keys.push(key.to_string());
                dists.push(m.score);
                recalled_texts.push(text.clone());
            }
            // Worst rank first, best rank adjacent to the prefix: truncation
            // drops the least relevant context first under budget pressure.
            let mut context_ids: Vec<u32> = Vec::new();
            for text in recalled_texts.iter().rev() {
                context_ids.extend(bpe.encode(text).expect("re-encode recall"));
            }
            let injected = build_injected_tape(&context_ids, prefix, BUDGET).expect("inject tape");
            let (pt_b, ms_b) = forward_timed(&verified, &injected.tokens);
            qb_ms += ms_b;
            assert_eq!(
                pt_b.provenance.max_logit_abs_diff, 0.0,
                "live nonperturbation"
            );
            assert_eq!(
                pt_a.provenance.model_digest.as_str(),
                pt_b.provenance.model_digest.as_str(),
                "same frozen model both arms"
            );
            assert_eq!(
                pt_a.provenance.config_digest.as_str(),
                pt_b.provenance.config_digest.as_str(),
                "same config both arms"
            );
            let sl_a = activation_slices(&activations_of(&pt_a), prefix.len()).expect("slices A");
            let sl_b =
                activation_slices(&activations_of(&pt_b), injected.tokens.len()).expect("slices B");
            // Schedule-v1 proxy: last 64-dim slice as pseudo-logits, target
            // class = target_id mod 64. Identical scoring both arms; a
            // task-quality claim needs trace-lane logit exposure.
            let target_class = target[0] % 64;
            let (hit_a, nll_a) =
                score_next_token(sl_a.last().expect("last A"), target_class).expect("score A");
            let (hit_b, nll_b) =
                score_next_token(sl_b.last().expect("last B"), target_class).expect("score B");
            let (l2, cos) = slice_divergence(&sl_a, &sl_b).expect("divergence");
            assert!(
                nll_a.is_finite() && nll_b.is_finite() && l2.is_finite() && cos.is_finite(),
                "finite scores"
            );
            let b_suffix = &sl_b[sl_b.len() - sl_a.len()..];
            let bit_identical = sl_a.as_slice() == b_suffix;
            // Honest causal sensor: the (KeyValue, F32) rows respond to
            // prepended context (probe receipt); activations are the control.
            let kv_a = kv_f32_rows(&pt_a).expect("KV A");
            let kv_b = kv_f32_rows(&pt_b).expect("KV B");
            assert_eq!(kv_a.0, prefix.len(), "KV-A rows match prefix");
            assert_eq!(kv_b.0, injected.tokens.len(), "KV-B rows match tape");
            let (kv_rms, kv_bitid) = kv_suffix_divergence(&kv_a, &kv_b).expect("KV divergence");
            assert!(kv_rms.is_finite(), "finite KV rms");
            println!(
                "row w{widx} preflen={} ctx={} prefixtok={} ctrunc={} qh3={:.10?} qt2={:.10?} rank={rank_keys:?} dists={dists:.10?} hitA={hit_a} nllA={nll_a:.6} hitB={hit_b} nllB={nll_b:.6} l2={l2:.10} cos={cos:.10} bitid={bit_identical} kvT={}x{} kvRms={kv_rms:.10} kvBitid={kv_bitid} msA={ms_a} msB={ms_b}",
                prefix.len(),
                injected.context_tokens,
                injected.prefix_tokens,
                injected.context_truncated,
                q.0,
                q.1,
                kv_a.0,
                kv_a.1,
            );
            rows.push((hit_a, nll_a, hit_b, nll_b, l2, cos, kv_rms));
        }
        let n = rows.len() as f64;
        let mean_nll_a = rows.iter().map(|r| r.1).sum::<f64>() / n;
        let mean_nll_b = rows.iter().map(|r| r.3).sum::<f64>() / n;
        let mean_l2 = rows.iter().map(|r| r.4).sum::<f64>() / n;
        let mean_cos = rows.iter().map(|r| r.5).sum::<f64>() / n;
        let mean_kv = rows.iter().map(|r| r.6).sum::<f64>() / n;
        let hits_a = rows.iter().filter(|r| r.0).count();
        let hits_b = rows.iter().filter(|r| r.2).count();
        println!(
            "ab1 corpus={} rehash_ms={rehash_ms} seed_ms={seed_ms} qA_ms={qa_ms} qB_ms={qb_ms} hitsA={hits_a}/{T} hitsB={hits_b}/{T} meanNllA={mean_nll_a:.4} meanNllB={mean_nll_b:.4} meanL2={mean_l2:.4} meanCos={mean_cos:.4} meanKvRms={mean_kv:.6}",
            corpus_digest.as_str(),
        );
        assert_eq!(rows.len(), T);
    }
}
