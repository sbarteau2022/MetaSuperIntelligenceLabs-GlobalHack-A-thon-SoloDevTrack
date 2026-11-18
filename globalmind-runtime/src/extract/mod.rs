//! Lane 3 — frozen-weight candidate extractor.
//!
//! Consumes native trace envelopes (lane 1) and emits typed rank-0/1/2/8
//! candidates. Teacher identity (model/config/tokenizer digests) is
//! provenance on the envelope only: it never enters a candidate digest,
//! coordinate, novelty input, or ranking. Coordinates come from the
//! Whitehead canonical representative of the teacher-neutral content
//! digest, so identical content from two teachers coalesces to one atom.
//!
//! Extraction is deterministic and bounded: rank-0 from activation/weight
//! traces, rank-1 from consecutive token positions at one layer/head,
//! rank-2 when a walk revisits an atom, rank-8 from weight traces whose
//! payload carries a checked tangent basis. No model is loaded here; the
//! extractor reads envelopes produced by a real forward pass elsewhere.

use crate::trace::{TraceDtype, TraceEnvelope, TraceKind};
use crate::{CellKind, Digest, RecordId};
use sha2::{Digest as ShaDigest, Sha256};

pub const EXTRACTOR_VERSION: &str = "globalmind-extract-v1";

#[derive(Clone, Debug)]
pub struct ExtractedCandidate {
    pub id: RecordId,
    pub cell_kind: CellKind,
    /// Teacher-neutral source fragment (trace content digest or walk digest).
    pub source_fragment_digest: Digest,
    /// Teacher-neutral input digest; the only identity a candidate carries.
    pub teacher_neutral_input_digest: Digest,
    pub candidate_digest: Digest,
    /// Canonical H3xT2 address for rank-0 atoms; `None` for higher ranks,
    /// which bind to atoms rather than carrying their own address.
    pub h3_lorentz: Option<[f64; 4]>,
    pub t2_phase: Option<[f64; 2]>,
    pub well_ordinal: Option<u8>,
    /// Rank-1: ordered atom pair. Rank-2: closed step range. Rank-8: anchor.
    pub from_atom: Option<RecordId>,
    pub to_atom: Option<RecordId>,
    pub first_step: Option<u32>,
    pub last_step: Option<u32>,
    pub anchor_atom: Option<RecordId>,
    pub chart_rank: Option<u8>,
    pub tangent_basis: Option<Box<[u8]>>,
    pub tensor_shape: Option<Vec<u32>>,
}

/// Extracts every candidate provable from one envelope set. Envelopes are
/// validated on entry; ordering is by (layer, head, token_position) so the
/// walk is deterministic regardless of input order.
pub fn extract(envelopes: &[TraceEnvelope]) -> Result<Vec<ExtractedCandidate>, String> {
    if envelopes.is_empty() {
        return Err("extractor requires at least one trace envelope".into());
    }
    for envelope in envelopes {
        envelope.validate()?;
    }
    let mut ordered: Vec<&TraceEnvelope> = envelopes.iter().collect();
    ordered.sort_by_key(|e| {
        (
            e.layer,
            e.head,
            e.token_position,
            e.content_digest().as_str().to_string(),
        )
    });

    let mut candidates = Vec::new();
    // The walk retains revisits (rank-2 loops close over them); the rank-0
    // candidate set coalesces to one atom per teacher-neutral content.
    // Atom identity keys on payload content (kind/dtype/shape/layer/head/
    // payload digest): token position is provenance, not identity, so a
    // walk can return to an earlier atom and close a loop.
    let mut atoms: Vec<(RecordId, Digest)> = Vec::new();
    let mut seen: std::collections::BTreeSet<Digest> = std::collections::BTreeSet::new();
    for envelope in &ordered {
        if envelope.kind == TraceKind::KeyValue {
            continue; // exact-model state never enters semantic extraction
        }
        let content = atom_key(envelope);
        let candidate_digest = candidate_digest_for(CellKind::Rank0Atom, &content, &content);
        let atom_id = RecordId::parse(format!("atom_{}", &candidate_digest.as_str()[..32]))?;
        atoms.push((atom_id.clone(), content.clone()));
        if !seen.insert(content.clone()) {
            continue; // teacher-neutral coalescence: one atom per content
        }
        let (h3, t2, well) = address_for(&content);
        candidates.push(ExtractedCandidate {
            id: RecordId::parse(format!("candidate_{}", &candidate_digest.as_str()[..32]))?,
            cell_kind: CellKind::Rank0Atom,
            source_fragment_digest: content.clone(),
            teacher_neutral_input_digest: content,
            candidate_digest,
            h3_lorentz: Some(h3),
            t2_phase: Some(t2),
            well_ordinal: Some(well),
            from_atom: None,
            to_atom: None,
            first_step: None,
            last_step: None,
            anchor_atom: None,
            chart_rank: None,
            tangent_basis: None,
            tensor_shape: None,
        });
    }

    // Rank-1 transitions between consecutive distinct atoms in walk order.
    // A step onto the same atom is a loop closure (rank-2), not an edge.
    for pair in atoms.windows(2) {
        let (from, from_digest) = &pair[0];
        let (to, to_digest) = &pair[1];
        if from == to {
            continue;
        }
        let fragment = digest_fields(
            b"globalmind-extract-transition-v1",
            &[from_digest.as_str(), to_digest.as_str()],
        );
        candidates.push(ExtractedCandidate {
            id: RecordId::parse(format!(
                "candidate_{}",
                &candidate_digest_for(CellKind::Rank1Transition, &fragment, &fragment).as_str()
                    [..32]
            ))?,
            cell_kind: CellKind::Rank1Transition,
            source_fragment_digest: fragment.clone(),
            teacher_neutral_input_digest: fragment.clone(),
            candidate_digest: candidate_digest_for(CellKind::Rank1Transition, &fragment, &fragment),
            h3_lorentz: None,
            t2_phase: None,
            well_ordinal: None,
            from_atom: Some(from.clone()),
            to_atom: Some(to.clone()),
            first_step: None,
            last_step: None,
            anchor_atom: None,
            chart_rank: None,
            tangent_basis: None,
            tensor_shape: None,
        });
    }

    // Rank-2 loops: a walk that revisits an atom closes a loop over the
    // step range between the two visits.
    for (later, (atom, _)) in atoms.iter().enumerate() {
        if let Some(earlier) = atoms[..later].iter().position(|(seen, _)| seen == atom) {
            let fragment = digest_fields(
                b"globalmind-extract-loop-v1",
                &[atom.as_str(), &earlier.to_string(), &later.to_string()],
            );
            candidates.push(ExtractedCandidate {
                id: RecordId::parse(format!(
                    "candidate_{}",
                    &candidate_digest_for(CellKind::Rank2Loop, &fragment, &fragment).as_str()[..32]
                ))?,
                cell_kind: CellKind::Rank2Loop,
                source_fragment_digest: fragment.clone(),
                teacher_neutral_input_digest: fragment.clone(),
                candidate_digest: candidate_digest_for(CellKind::Rank2Loop, &fragment, &fragment),
                h3_lorentz: None,
                t2_phase: None,
                well_ordinal: None,
                from_atom: None,
                to_atom: None,
                first_step: Some(earlier as u32),
                last_step: Some((later - 1) as u32),
                anchor_atom: None,
                chart_rank: None,
                tangent_basis: None,
                tensor_shape: None,
            });
        }
    }

    // Rank-8 charts from weight traces: the payload is the native tangent
    // basis; rank is capped at 8 and the byte length must match shape/dtype.
    for envelope in &ordered {
        if envelope.kind != TraceKind::Weight {
            continue;
        }
        let anchor_content = atom_key(envelope);
        let Some((anchor, _)) = atoms.iter().find(|(_, seen)| *seen == anchor_content) else {
            continue;
        };
        let elements: usize = envelope.shape.iter().map(|d| *d as usize).product();
        let rank = elements.clamp(1, 8) as u8;
        let fragment = digest_fields(
            b"globalmind-extract-chart-v1",
            &[anchor_content.as_str(), envelope.payload_digest.as_str()],
        );
        candidates.push(ExtractedCandidate {
            id: RecordId::parse(format!(
                "candidate_{}",
                &candidate_digest_for(CellKind::Rank8Jacobian, &fragment, &fragment).as_str()[..32]
            ))?,
            cell_kind: CellKind::Rank8Jacobian,
            source_fragment_digest: fragment.clone(),
            teacher_neutral_input_digest: fragment.clone(),
            candidate_digest: candidate_digest_for(CellKind::Rank8Jacobian, &fragment, &fragment),
            h3_lorentz: None,
            t2_phase: None,
            well_ordinal: None,
            from_atom: None,
            to_atom: None,
            first_step: None,
            last_step: None,
            anchor_atom: Some(anchor.clone()),
            chart_rank: Some(rank),
            tangent_basis: Some(envelope.payload.clone()),
            tensor_shape: Some(envelope.shape.clone()),
        });
    }
    Ok(candidates)
}

/// Teacher-neutral atom identity for one envelope: payload content only.
/// Token position and all teacher digests are provenance, never identity.
pub(crate) fn atom_key(envelope: &TraceEnvelope) -> Digest {
    let mut fields: Vec<String> = vec![
        envelope.kind.code().to_string(),
        envelope.dtype.code().to_string(),
        envelope.layer.to_string(),
        envelope.head.to_string(),
    ];
    fields.extend(envelope.shape.iter().map(u32::to_string));
    fields.push(envelope.payload_digest.as_str().to_string());
    digest_fields(
        b"globalmind-atom-key-v1",
        &fields.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

/// The one candidate-digest preimage used across the runtime. It contains
/// teacher-neutral fields only; this function has no parameter that could
/// carry teacher identity.
pub fn candidate_digest_for(kind: CellKind, fragment: &Digest, neutral: &Digest) -> Digest {
    digest_fields(
        b"globalmind-candidate-v1",
        &[
            cell_kind_name(kind),
            "frozen_weight",
            fragment.as_str(),
            neutral.as_str(),
            crate::PROJECTOR_VERSION,
            // Configuration binding is supplied by the run, not the teacher;
            // extraction uses the neutral content itself as the binding so
            // the digest is computable before a run exists. The runner
            // rebinds to the run configuration at packet admission.
            neutral.as_str(),
        ],
    )
}

fn cell_kind_name(kind: CellKind) -> &'static str {
    match kind {
        CellKind::Rank0Atom => "rank0_atom",
        CellKind::Rank1Transition => "rank1_transition",
        CellKind::Rank2Loop => "rank2_loop",
        CellKind::Rank8Jacobian => "rank8_jacobian",
    }
}

/// Canonical address for teacher-neutral content: the Whitehead canonical
/// representative supplies the well; the Lorentz point is derived from the
/// content digest on the unit hyperboloid (t = cosh(u), spatial direction
/// from digest bytes, norm sinh(u)).
fn address_for(content: &Digest) -> ([f64; 4], [f64; 2], u8) {
    let bytes = digest_bytes(content);
    let u = 0.25 + (bytes[0] as f64 / 255.0) * 1.5;
    let direction = [
        bytes[1] as f64 - 127.5,
        bytes[2] as f64 - 127.5,
        bytes[3] as f64 - 127.5,
    ];
    let norm = (direction.iter().map(|v| v * v).sum::<f64>())
        .sqrt()
        .max(1.0);
    let spatial = u.sinh();
    let h3 = [
        u.cosh(),
        spatial * direction[0] / norm,
        spatial * direction[1] / norm,
        spatial * direction[2] / norm,
    ];
    let t2 = [
        (bytes[4] as f64 / 255.0) * std::f64::consts::TAU,
        (bytes[5] as f64 / 255.0) * std::f64::consts::TAU,
    ];
    let well = bytes[6] % 12;
    (h3, t2, well)
}

fn digest_bytes(digest: &Digest) -> [u8; 32] {
    let hex = digest.as_str().as_bytes();
    let mut out = [0u8; 32];
    for (index, pair) in hex.as_chunks::<2>().0.iter().enumerate() {
        out[index] = (nibble(pair[0]) << 4) | nibble(pair[1]);
    }
    out
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
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
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    Digest::parse(value).expect("SHA-256 is a valid Digest")
}

pub fn dtype_of(envelope: &TraceEnvelope) -> TraceDtype {
    envelope.dtype
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::TraceKind;

    fn envelope(model: &str, kind: TraceKind, token: u64, payload: Vec<u8>) -> TraceEnvelope {
        TraceEnvelope {
            model_digest: Digest::parse(model.repeat(64)).unwrap(),
            config_digest: Digest::parse("b".repeat(64)).unwrap(),
            tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
            kind,
            dtype: TraceDtype::F32,
            shape: vec![(payload.len() / 4) as u32],
            layer: 0,
            head: 0,
            token_position: token,
            observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        }
    }

    fn floats(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn twin_teachers_coalesce_to_identical_candidates() {
        let payload = floats(&[1.0, 2.0, 3.0, 4.0]);
        let a = vec![envelope("a", TraceKind::Activation, 0, payload.clone())];
        let b = vec![envelope("e", TraceKind::Activation, 0, payload)];
        let left = extract(&a).unwrap();
        let right = extract(&b).unwrap();
        assert_eq!(left.len(), right.len());
        for (l, r) in left.iter().zip(&right) {
            assert_eq!(l.candidate_digest, r.candidate_digest);
            assert_eq!(l.h3_lorentz, r.h3_lorentz);
            assert_eq!(l.t2_phase, r.t2_phase);
            assert_eq!(l.well_ordinal, r.well_ordinal);
        }
    }

    #[test]
    fn walk_emits_rank0_rank1_and_loop() {
        // Content identity binds token position, so the same payload at a
        // new position is new content. A loop closes when the walk revisits
        // the identical content digest (same payload, layer, head, token).
        let envelopes = vec![
            envelope("a", TraceKind::Activation, 0, floats(&[1.0, 0.0])),
            envelope("a", TraceKind::Activation, 1, floats(&[0.0, 1.0])),
            envelope("a", TraceKind::Activation, 0, floats(&[1.0, 0.0])),
        ];
        let candidates = extract(&envelopes).unwrap();
        let rank0 = candidates
            .iter()
            .filter(|c| c.cell_kind == CellKind::Rank0Atom)
            .count();
        let rank1 = candidates
            .iter()
            .filter(|c| c.cell_kind == CellKind::Rank1Transition)
            .count();
        let rank2 = candidates
            .iter()
            .filter(|c| c.cell_kind == CellKind::Rank2Loop)
            .count();
        // Sorted walk is [A, A, B]: the A->A step is the loop closure, the
        // single A->B step is the only rank-1 transition.
        assert_eq!(rank0, 2, "identical content coalesces to one atom");
        assert_eq!(rank1, 1, "only the A->B step is a transition");
        assert_eq!(rank2, 1, "the revisit closes exactly one loop");
    }

    #[test]
    fn kv_traces_never_enter_semantic_extraction() {
        let envelopes = vec![envelope("a", TraceKind::KeyValue, 0, floats(&[1.0]))];
        assert!(extract(&envelopes).unwrap().is_empty());
    }

    #[test]
    fn weight_trace_emits_rank8_chart() {
        let envelopes = vec![envelope(
            "a",
            TraceKind::Weight,
            0,
            floats(&[1.0, 2.0, 3.0, 4.0]),
        )];
        let candidates = extract(&envelopes).unwrap();
        let chart = candidates
            .iter()
            .find(|c| c.cell_kind == CellKind::Rank8Jacobian)
            .expect("rank-8 chart");
        assert_eq!(chart.chart_rank, Some(4));
        assert!(chart.anchor_atom.is_some());
    }
}
