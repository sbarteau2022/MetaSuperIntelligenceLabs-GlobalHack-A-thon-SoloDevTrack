//! Candidate identity for shared knowledge, separate from model-bound tensor witnesses.
//! Admission still requires independent evidence and a real cross-model replay.

use crate::{Digest, RecordId};

pub const CANONICALIZER_VERSION: &str = "shared-knowledge-v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KnowledgeKind {
    Proposition,
    Procedure,
    Constraint,
    Capability,
    Observation,
}

impl KnowledgeKind {
    fn tag(self) -> u8 {
        match self {
            Self::Proposition => 1,
            Self::Procedure => 2,
            Self::Constraint => 3,
            Self::Capability => 4,
            Self::Observation => 5,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedKnowledgeCandidate {
    pub kind: KnowledgeKind,
    pub domain: String,
    pub scope: String,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub validity: String,
    pub abstain_when: String,
}

impl SharedKnowledgeCandidate {
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        let fields = [
            self.domain.as_str(),
            self.scope.as_str(),
            self.subject.as_str(),
            self.predicate.as_str(),
            self.object.as_str(),
            self.validity.as_str(),
            self.abstain_when.as_str(),
        ];
        if fields.iter().any(|field| field.trim().is_empty()) {
            return Err("shared knowledge requires all semantic and abstention fields".into());
        }
        if fields
            .iter()
            .any(|field| field.trim() != *field || field.contains('\0'))
        {
            return Err("shared knowledge fields must be trimmed and contain no NUL".into());
        }
        if fields.iter().any(|field| field.len() > 65_536) {
            return Err("shared knowledge field exceeds 64 KiB".into());
        }
        let mut bytes = Vec::new();
        bytes.extend_from_slice(CANONICALIZER_VERSION.as_bytes());
        bytes.push(0);
        bytes.push(self.kind.tag());
        for field in fields {
            let len = u32::try_from(field.len()).map_err(|_| "semantic field too long")?;
            bytes.extend_from_slice(&len.to_le_bytes());
            bytes.extend_from_slice(field.as_bytes());
        }
        Ok(bytes)
    }

    pub fn semantic_key(&self) -> Result<Digest, String> {
        Ok(Digest::sha256(&self.canonical_bytes()?))
    }

    pub fn stable_id(&self) -> Result<RecordId, String> {
        RecordId::parse(format!("shared_{}", self.semantic_key()?))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedOrigin {
    pub model_digest: Digest,
    pub run: RecordId,
    pub source_digest: Digest,
    pub task_receipt_digest: Digest,
    pub independent_judge_digest: Digest,
}

impl SharedOrigin {
    pub fn validate(&self) -> Result<(), String> {
        if self.task_receipt_digest == self.independent_judge_digest {
            return Err("independent judge must have a distinct receipt".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> SharedKnowledgeCandidate {
        SharedKnowledgeCandidate {
            kind: KnowledgeKind::Proposition,
            domain: "geography".into(),
            scope: "France capital as of source date".into(),
            subject: "France".into(),
            predicate: "capital".into(),
            object: "Paris".into(),
            validity: "Current only while supported by source".into(),
            abstain_when: "Source is stale or conflicting".into(),
        }
    }

    #[test]
    fn key_is_stable_and_excludes_origin_identity() {
        let first = candidate();
        let key = first.semantic_key().unwrap();
        assert_eq!(key, candidate().semantic_key().unwrap());
        assert_eq!(first.stable_id().unwrap().as_str(), format!("shared_{key}"));
        let origin_a = SharedOrigin {
            model_digest: Digest::sha256(b"first frozen roster"),
            run: RecordId::parse("run_a").unwrap(),
            source_digest: Digest::sha256(b"source a"),
            task_receipt_digest: Digest::sha256(b"task a"),
            independent_judge_digest: Digest::sha256(b"judge a"),
        };
        let mut origin_b = origin_a.clone();
        origin_b.model_digest = Digest::sha256(b"second frozen roster");
        origin_b.run = RecordId::parse("run_b").unwrap();
        assert_ne!(origin_a.model_digest, origin_b.model_digest);
        assert_eq!(key, first.semantic_key().unwrap());
        assert!(origin_a.validate().is_ok());
    }

    #[test]
    fn semantic_and_admission_fields_fail_closed() {
        let mut value = candidate();
        let key = value.semantic_key().unwrap();
        value.object = "Lyon".into();
        assert_ne!(key, value.semantic_key().unwrap());
        value.object.clear();
        assert!(value.semantic_key().is_err());
        value.object = " Paris".into();
        assert!(value.semantic_key().is_err());
        let receipt = Digest::sha256(b"same receipt");
        let origin = SharedOrigin {
            model_digest: Digest::sha256(b"model"),
            run: RecordId::parse("run").unwrap(),
            source_digest: Digest::sha256(b"source"),
            task_receipt_digest: receipt.clone(),
            independent_judge_digest: receipt,
        };
        assert!(origin.validate().is_err());
    }
}
