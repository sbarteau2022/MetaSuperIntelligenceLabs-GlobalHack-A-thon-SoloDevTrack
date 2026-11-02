//! Lane 4 — independent judge.
//!
//! Consumes frozen candidate bytes plus evidence and emits immutable
//! pass/reject receipts. The judge is structurally independent: it receives
//! candidates by value (frozen bytes), holds thresholds fixed at
//! construction, exposes no mutation or threshold-tuning API, and its
//! receipt digest binds candidate digest + evidence digest + the frozen
//! policy digest. Novelty is judged against a digest set fixed at
//! construction; the judge cannot learn new thresholds from results.

use crate::extract::ExtractedCandidate;
use crate::{CellKind, Digest};
use sha2::{Digest as ShaDigest, Sha256};

pub const JUDGE_VERSION: &str = "globalmind-judge-v1";

/// Frozen judging policy. Constructed once, before any candidate is seen;
/// there is deliberately no setter.
#[derive(Clone, Copy, Debug)]
pub struct JudgePolicy {
    /// Minimum finite payload energy (mean square) for rank-0 admission.
    pub min_mean_square: f64,
    /// Maximum admitted chart rank.
    pub max_chart_rank: u8,
}

impl JudgePolicy {
    pub fn validate(self) -> Result<Self, String> {
        if !self.min_mean_square.is_finite() || self.min_mean_square < 0.0 {
            return Err("judge min_mean_square must be finite and nonnegative".into());
        }
        if self.max_chart_rank == 0 || self.max_chart_rank > 8 {
            return Err("judge max_chart_rank must be in 1..=8".into());
        }
        Ok(self)
    }

    pub fn digest(&self) -> Digest {
        digest_fields(
            b"globalmind-judge-policy-v1",
            &[
                JUDGE_VERSION,
                &format!("{:?}", self.min_mean_square),
                &self.max_chart_rank.to_string(),
            ],
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JudgeVerdict {
    Pass,
    Reject,
}

#[derive(Clone, Debug)]
pub struct JudgeReceipt {
    pub candidate_digest: Digest,
    pub verdict: JudgeVerdict,
    pub reason: &'static str,
    pub policy_digest: Digest,
    pub evidence_digest: Digest,
    pub receipt_digest: Digest,
    pub novelty_receipt_digest: Digest,
}

/// Evidence the judge is allowed to see: frozen payload statistics computed
/// by the caller from the candidate's source trace, plus the set of
/// already-known candidate digests for the novelty check.
#[derive(Clone, Debug)]
pub struct JudgeEvidence {
    pub mean_square: f64,
    pub finite: bool,
    pub known_candidate_digests: Vec<Digest>,
}

pub struct IndependentJudge {
    policy: JudgePolicy,
}

impl IndependentJudge {
    pub fn new(policy: JudgePolicy) -> Result<Self, String> {
        Ok(Self {
            policy: policy.validate()?,
        })
    }

    pub fn policy_digest(&self) -> Digest {
        self.policy.digest()
    }

    /// Judges one frozen candidate. The candidate is taken by reference to
    /// its immutable extracted form; the judge has no API to modify it.
    pub fn judge(
        &self,
        candidate: &ExtractedCandidate,
        evidence: &JudgeEvidence,
    ) -> Result<JudgeReceipt, String> {
        if !evidence.mean_square.is_finite() && evidence.finite {
            return Err("judge evidence claims finite with non-finite mean_square".into());
        }
        let novel = !evidence
            .known_candidate_digests
            .contains(&candidate.candidate_digest);
        let novelty_receipt_digest = digest_fields(
            b"globalmind-judge-novelty-v1",
            &[
                candidate.candidate_digest.as_str(),
                if novel { "novel" } else { "duplicate" },
                self.policy.digest().as_str(),
            ],
        );
        let (verdict, reason) = if !novel {
            (JudgeVerdict::Reject, "duplicate candidate digest")
        } else if !evidence.finite {
            (JudgeVerdict::Reject, "non-finite trace evidence")
        } else if candidate.cell_kind == CellKind::Rank0Atom
            && evidence.mean_square < self.policy.min_mean_square
        {
            (JudgeVerdict::Reject, "rank-0 energy below frozen floor")
        } else if candidate.cell_kind == CellKind::Rank8Jacobian
            && candidate.chart_rank.unwrap_or(0) > self.policy.max_chart_rank
        {
            (JudgeVerdict::Reject, "chart rank above frozen cap")
        } else if candidate.cell_kind == CellKind::Rank8Jacobian
            && candidate
                .tangent_basis
                .as_ref()
                .is_none_or(|b| b.is_empty())
        {
            (JudgeVerdict::Reject, "chart without tangent basis")
        } else {
            (JudgeVerdict::Pass, "admitted by frozen policy")
        };
        let evidence_digest = digest_fields(
            b"globalmind-judge-evidence-v1",
            &[
                candidate.candidate_digest.as_str(),
                &format!("{:?}", evidence.mean_square),
                if evidence.finite {
                    "finite"
                } else {
                    "nonfinite"
                },
                &evidence.known_candidate_digests.len().to_string(),
            ],
        );
        let receipt_digest = digest_fields(
            b"globalmind-judge-receipt-v1",
            &[
                candidate.candidate_digest.as_str(),
                match verdict {
                    JudgeVerdict::Pass => "pass",
                    JudgeVerdict::Reject => "reject",
                },
                reason,
                self.policy.digest().as_str(),
                evidence_digest.as_str(),
                novelty_receipt_digest.as_str(),
            ],
        );
        Ok(JudgeReceipt {
            candidate_digest: candidate.candidate_digest.clone(),
            verdict,
            reason,
            policy_digest: self.policy.digest(),
            evidence_digest,
            receipt_digest,
            novelty_receipt_digest,
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extract::extract;
    use crate::trace::{TraceDtype, TraceEnvelope, TraceKind};

    fn candidate() -> ExtractedCandidate {
        let payload: Vec<u8> = [1.0f32, 2.0].iter().flat_map(|v| v.to_le_bytes()).collect();
        let envelope = TraceEnvelope {
            model_digest: Digest::parse("a".repeat(64)).unwrap(),
            config_digest: Digest::parse("b".repeat(64)).unwrap(),
            tokenizer_digest: Digest::parse("c".repeat(64)).unwrap(),
            kind: TraceKind::Activation,
            dtype: TraceDtype::F32,
            shape: vec![2],
            layer: 0,
            head: 0,
            token_position: 0,
            observer_receipt_digest: Digest::parse("d".repeat(64)).unwrap(),
            payload_digest: Digest::sha256(&payload),
            payload: payload.into_boxed_slice(),
        };
        extract(&[envelope]).unwrap().remove(0)
    }

    fn judge() -> IndependentJudge {
        IndependentJudge::new(JudgePolicy {
            min_mean_square: 0.5,
            max_chart_rank: 8,
        })
        .unwrap()
    }

    #[test]
    fn pass_reject_and_duplicate_paths() {
        let candidate = candidate();
        let evidence = JudgeEvidence {
            mean_square: 2.5,
            finite: true,
            known_candidate_digests: vec![],
        };
        let receipt = judge().judge(&candidate, &evidence).unwrap();
        assert_eq!(receipt.verdict, JudgeVerdict::Pass);

        let duplicate = JudgeEvidence {
            known_candidate_digests: vec![candidate.candidate_digest.clone()],
            ..JudgeEvidence {
                mean_square: 2.5,
                finite: true,
                known_candidate_digests: vec![],
            }
        };
        assert_eq!(
            judge().judge(&candidate, &duplicate).unwrap().verdict,
            JudgeVerdict::Reject
        );

        let weak = JudgeEvidence {
            mean_square: 0.1,
            finite: true,
            known_candidate_digests: vec![],
        };
        assert_eq!(
            judge().judge(&candidate, &weak).unwrap().verdict,
            JudgeVerdict::Reject
        );
    }

    #[test]
    fn receipts_are_deterministic_and_policy_bound() {
        let candidate = candidate();
        let evidence = JudgeEvidence {
            mean_square: 2.5,
            finite: true,
            known_candidate_digests: vec![],
        };
        let a = judge().judge(&candidate, &evidence).unwrap();
        let b = judge().judge(&candidate, &evidence).unwrap();
        assert_eq!(a.receipt_digest, b.receipt_digest);
        let other = IndependentJudge::new(JudgePolicy {
            min_mean_square: 0.25,
            max_chart_rank: 8,
        })
        .unwrap();
        assert_ne!(
            other.judge(&candidate, &evidence).unwrap().receipt_digest,
            a.receipt_digest
        );
    }
}
