//! Native typed distillation manifest and frozen-artifact roster (Lane A).
//!
//! The manifest is an operator-authored, line-oriented `key = value` roster.
//! It is parsed as typed text lines; no JSON or Python crosses this boundary.
//! Every listed artifact is rehashed from disk before any database write, and
//! the teacher identity (provider/model/revision) is kept in the admission
//! audit record only. It never enters a candidate preimage.

use crate::{Digest, RecordId};
use sha2::{Digest as ShaDigest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read as IoRead};

pub const PROJECTOR_VERSION: &str = "globalmind-distill-v1";

const MODALITIES: [&str; 6] = ["text", "image", "audio", "video", "code", "multimodal"];

#[derive(Clone, Debug)]
pub struct ArtifactEntry {
    pub role: String,
    pub path: String,
    pub sha256: Digest,
}

#[derive(Clone, Debug)]
pub struct ModelEntry {
    pub provider: String,
    pub name: String,
    pub revision: String,
    pub modality: String,
    pub eligibility_receipt_digest: Option<Digest>,
    pub judge_receipt_digest: Option<Digest>,
    pub artifacts: Vec<ArtifactEntry>,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub workspace: RecordId,
    pub workspace_slug: String,
    pub method: String,
    pub configuration_digest: Digest,
    pub tpe_version: String,
    pub tpe_configuration_digest: Digest,
    pub models: Vec<ModelEntry>,
}

#[derive(Clone, Debug)]
pub struct VerifiedArtifact {
    pub role: String,
    pub path: String,
    pub sha256: Digest,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub struct VerifiedModel {
    pub provider: String,
    pub name: String,
    pub revision: String,
    pub modality: String,
    pub source_id: RecordId,
    pub admission_id: RecordId,
    pub eligibility_receipt_digest: Option<Digest>,
    pub judge_receipt_digest: Option<Digest>,
    pub artifacts: Vec<VerifiedArtifact>,
    pub roster_digest: Digest,
}

#[derive(Clone, Debug)]
pub struct VerifiedManifest {
    pub workspace: RecordId,
    pub method: String,
    pub configuration_digest: Digest,
    pub tpe_version: String,
    pub tpe_configuration_digest: Digest,
    pub models: Vec<VerifiedModel>,
    pub input_digest: Digest,
    pub run_id: RecordId,
    pub run_receipt_digest: Digest,
    pub opened_event_id: RecordId,
}

impl Manifest {
    pub fn load(path: &str) -> Result<Self, String> {
        let file = File::open(path).map_err(|error| format!("open manifest: {error}"))?;
        let mut pairs = BTreeMap::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line =
                line.map_err(|error| format!("read manifest line {}: {error}", index + 1))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("manifest line {} is not key = value", index + 1))?;
            let key = key.trim().to_string();
            if pairs
                .insert(key.clone(), value.trim().to_string())
                .is_some()
            {
                return Err(format!("manifest key is duplicated: {key}"));
            }
        }

        validate_manifest_keys(&pairs)?;
        let workspace_slug = required(&pairs, "workspace")?;
        let workspace = RecordId::parse(workspace_slug.clone())?;
        let method = required(&pairs, "method")?;
        if method.trim().is_empty() {
            return Err("manifest method must be non-empty".into());
        }
        let configuration_digest = Digest::parse(required(&pairs, "configuration_digest")?)?;
        let tpe_version = required(&pairs, "tpe_version")?;
        let tpe_configuration_digest =
            Digest::parse(required(&pairs, "tpe_configuration_digest")?)?;
        let models = parse_models(&pairs)?;
        if models.is_empty() {
            return Err("manifest must list at least one model".into());
        }
        Ok(Self {
            workspace,
            workspace_slug,
            method,
            configuration_digest,
            tpe_version,
            tpe_configuration_digest,
            models,
        })
    }

    /// G0: rehash every rostered artifact from disk and fail closed on any
    /// mismatch. No database write may precede this gate.
    pub fn verify_artifacts(&self) -> Result<VerifiedManifest, String> {
        let mut verified = Vec::new();
        for model in &self.models {
            let mut artifacts = Vec::new();
            let mut roster = Sha256::new();
            roster.update(b"globalmind-manifest-v1\n");
            roster.update(model.provider.as_bytes());
            roster.update(b"\n");
            roster.update(model.name.as_bytes());
            roster.update(b"\n");
            roster.update(model.revision.as_bytes());
            roster.update(b"\n");
            roster.update(model.modality.as_bytes());
            roster.update(b"\n");
            for artifact in &model.artifacts {
                if !artifact.path.starts_with('/') {
                    return Err(format!("artifact path must be absolute: {}", artifact.path));
                }
                let (bytes, actual) = sha256_file(&artifact.path)?;
                if actual != artifact.sha256 {
                    return Err(format!(
                        "artifact digest mismatch for {}: roster {} != disk {}",
                        artifact.path,
                        artifact.sha256.as_str(),
                        actual.as_str()
                    ));
                }
                roster.update(artifact.role.as_bytes());
                roster.update(b"\n");
                roster.update(artifact.path.as_bytes());
                roster.update(b"\n");
                roster.update(artifact.sha256.as_str().as_bytes());
                roster.update(b"\n");
                roster.update(bytes.to_le_bytes());
                roster.update(b"\n");
                artifacts.push(VerifiedArtifact {
                    role: artifact.role.clone(),
                    path: artifact.path.clone(),
                    sha256: artifact.sha256.clone(),
                    bytes,
                });
            }
            let roster_digest = Digest::parse(hex_digest(roster))?;
            let source_id = RecordId::parse(format!("source_{}", roster_digest.as_str()))?;
            let admission_digest = digest_fields(
                b"globalmind-admission-v1",
                &[
                    roster_digest.as_str(),
                    model
                        .eligibility_receipt_digest
                        .as_ref()
                        .map_or("UNQUALIFIED", Digest::as_str),
                    model
                        .judge_receipt_digest
                        .as_ref()
                        .map_or("UNJUDGED", Digest::as_str),
                ],
            );
            verified.push(VerifiedModel {
                provider: model.provider.clone(),
                name: model.name.clone(),
                revision: model.revision.clone(),
                modality: model.modality.clone(),
                source_id,
                admission_id: RecordId::parse(format!("admission_{}", admission_digest.as_str()))?,
                eligibility_receipt_digest: model.eligibility_receipt_digest.clone(),
                judge_receipt_digest: model.judge_receipt_digest.clone(),
                artifacts,
                roster_digest,
            });
        }
        let mut fields = vec![
            self.configuration_digest.as_str(),
            self.method.as_str(),
            self.tpe_version.as_str(),
            self.tpe_configuration_digest.as_str(),
        ];
        fields.extend(verified.iter().map(|model| model.roster_digest.as_str()));
        let input_digest = digest_fields(b"globalmind-run-input-v1", &fields);
        let run_receipt_digest = digest_fields(
            b"globalmind-run-open-v1",
            &[
                self.workspace.as_str(),
                self.method.as_str(),
                self.configuration_digest.as_str(),
                self.tpe_version.as_str(),
                self.tpe_configuration_digest.as_str(),
                input_digest.as_str(),
            ],
        );
        Ok(VerifiedManifest {
            workspace: self.workspace.clone(),
            method: self.method.clone(),
            configuration_digest: self.configuration_digest.clone(),
            tpe_version: self.tpe_version.clone(),
            tpe_configuration_digest: self.tpe_configuration_digest.clone(),
            models: verified,
            input_digest: input_digest.clone(),
            run_id: RecordId::parse(format!("run_{}", input_digest.as_str()))?,
            opened_event_id: RecordId::parse(format!("run_event_{}", run_receipt_digest.as_str()))?,
            run_receipt_digest,
        })
    }
}

fn required(pairs: &BTreeMap<String, String>, key: &str) -> Result<String, String> {
    pairs
        .get(key)
        .cloned()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("manifest is missing required key: {key}"))
}

fn parse_models(pairs: &BTreeMap<String, String>) -> Result<Vec<ModelEntry>, String> {
    let model_indices = indexed_children(pairs.keys(), "model.")?;
    require_contiguous("model", &model_indices)?;
    let mut models = Vec::new();
    for model_index in model_indices {
        let prefix = format!("model.{model_index}.");
        let get = |field: &str| {
            required(pairs, &format!("{prefix}{field}"))
                .map_err(|_| format!("manifest is missing required key: {prefix}{field}"))
        };
        let modality = get("modality")?;
        if !MODALITIES.contains(&modality.as_str()) {
            return Err(format!(
                "model.{model_index}.modality is not an admitted modality"
            ));
        }
        let artifact_indices = indexed_children(pairs.keys(), &format!("{prefix}artifact."))?;
        require_contiguous(&format!("model.{model_index}.artifact"), &artifact_indices)?;
        let mut artifacts = Vec::new();
        let mut artifact_roles = BTreeSet::new();
        let mut artifact_paths = BTreeSet::new();
        for artifact_index in artifact_indices {
            let aprefix = format!("{prefix}artifact.{artifact_index}.");
            let aget = |field: &str| {
                required(pairs, &format!("{aprefix}{field}"))
                    .map_err(|_| format!("manifest is missing required key: {aprefix}{field}"))
            };
            let role = aget("role")?;
            let path = aget("path")?;
            if !artifact_roles.insert(role.clone()) {
                return Err(format!("model.{model_index} repeats artifact role {role}"));
            }
            if !artifact_paths.insert(path.clone()) {
                return Err(format!("model.{model_index} repeats artifact path {path}"));
            }
            artifacts.push(ArtifactEntry {
                role,
                path,
                sha256: Digest::parse(aget("sha256")?)?,
            });
        }
        if artifacts.is_empty() {
            return Err(format!(
                "model.{model_index} must list at least one artifact"
            ));
        }
        models.push(ModelEntry {
            provider: get("provider")?,
            name: get("name")?,
            revision: get("revision")?,
            modality,
            eligibility_receipt_digest: optional_digest(
                pairs,
                &format!("{prefix}eligibility_receipt_digest"),
            )?,
            judge_receipt_digest: optional_digest(pairs, &format!("{prefix}judge_receipt_digest"))?,
            artifacts,
        });
    }
    Ok(models)
}

fn validate_manifest_keys(pairs: &BTreeMap<String, String>) -> Result<(), String> {
    for key in pairs.keys() {
        if matches!(
            key.as_str(),
            "workspace"
                | "method"
                | "configuration_digest"
                | "tpe_version"
                | "tpe_configuration_digest"
        ) {
            continue;
        }
        let parts = key.split('.').collect::<Vec<_>>();
        let valid_model = parts.len() == 3
            && parts[0] == "model"
            && parts[1].parse::<usize>().is_ok()
            && matches!(
                parts[2],
                "provider"
                    | "name"
                    | "revision"
                    | "modality"
                    | "eligibility_receipt_digest"
                    | "judge_receipt_digest"
            );
        let valid_artifact = parts.len() == 5
            && parts[0] == "model"
            && parts[1].parse::<usize>().is_ok()
            && parts[2] == "artifact"
            && parts[3].parse::<usize>().is_ok()
            && matches!(parts[4], "role" | "path" | "sha256");
        if !valid_model && !valid_artifact {
            return Err(format!("manifest contains unknown key: {key}"));
        }
    }
    Ok(())
}

fn indexed_children<'a>(
    keys: impl Iterator<Item = &'a String>,
    prefix: &str,
) -> Result<Vec<usize>, String> {
    let mut indices = BTreeSet::new();
    for key in keys.filter(|key| key.starts_with(prefix)) {
        let suffix = &key[prefix.len()..];
        let index = suffix
            .split('.')
            .next()
            .ok_or_else(|| format!("invalid indexed key: {key}"))?
            .parse::<usize>()
            .map_err(|_| format!("invalid index in manifest key: {key}"))?;
        indices.insert(index);
    }
    Ok(indices.into_iter().collect())
}

fn require_contiguous(label: &str, indices: &[usize]) -> Result<(), String> {
    for (expected, actual) in indices.iter().copied().enumerate() {
        if expected != actual {
            return Err(format!("{label} indices must be contiguous from zero"));
        }
    }
    Ok(())
}

fn optional_digest(pairs: &BTreeMap<String, String>, key: &str) -> Result<Option<Digest>, String> {
    pairs.get(key).cloned().map(Digest::parse).transpose()
}

fn sha256_file(path: &str) -> Result<(u64, Digest), String> {
    let file = File::open(path).map_err(|error| format!("open artifact {path}: {error}"))?;
    let mut hasher = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 262_144];
    let mut reader = BufReader::new(file);
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("read artifact {path}: {error}"))?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        hasher.update(&buffer[..read]);
    }
    Ok((bytes, Digest::parse(hex_digest(hasher))?))
}

fn hex_digest(hasher: Sha256) -> String {
    let digest = hasher.finalize();
    let mut value = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("writing to String cannot fail");
    }
    value
}

fn digest_fields(domain: &[u8], fields: &[&str]) -> Digest {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for field in fields {
        hasher.update((field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    Digest::parse(hex_digest(hasher)).expect("SHA-256 is a valid Digest")
}
