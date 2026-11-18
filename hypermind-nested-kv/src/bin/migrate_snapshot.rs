use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::Path;

struct Atom {
    id: u64,
    coord: [f64; 4],
    phase: [f64; 2],
    text: String,
    salience: f64,
    provenance: String,
    taint: String,
    rank: u8,
    domain: String,
    pinned: bool,
}

fn sha256(bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(bytes);
    format!("{:x}", hash.finalize())
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn fnv1a(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn rotate_rank(vector: &[f32], rank: f32) -> Vec<f32> {
    let dimension = vector.len();
    let mut output = vector.to_vec();
    if rank == 0.0 {
        return output;
    }
    for index in 0..dimension / 2 {
        let fraction = 1.0 - (2 * index) as f32 / dimension as f32;
        let angle = rank * std::f32::consts::FRAC_PI_2 * fraction;
        let (cosine, sine) = (angle.cos(), angle.sin());
        let (x, y) = (vector[2 * index], vector[2 * index + 1]);
        output[2 * index] = x * cosine - y * sine;
        output[2 * index + 1] = x * sine + y * cosine;
    }
    output
}

fn calibrate_idf(texts: &[String]) -> HashMap<String, f64> {
    let count = texts.len().max(1) as f64;
    let mut frequencies = HashMap::<String, usize>::new();
    for text in texts {
        let mut seen = HashSet::new();
        for token in tokenize(text) {
            if seen.insert(token.clone()) {
                *frequencies.entry(token).or_default() += 1;
            }
        }
    }
    frequencies
        .into_iter()
        .map(|(token, frequency)| (token, ((count + 1.0) / (frequency as f64 + 1.0)).ln() + 1.0))
        .collect()
}

fn embed_direction(text: &str, idf: &HashMap<String, f64>, rank: u8) -> [f64; 3] {
    let mut vector = vec![0.0_f64; 256];
    for token in tokenize(text) {
        let weight = idf.get(&token).copied().unwrap_or(1.0);
        let hash = fnv1a(&token);
        vector[(hash % 256) as usize] += weight;
        vector[((hash >> 16) % 256) as usize] += 0.5 * weight;
    }
    let vector32: Vec<f32> = vector.iter().map(|value| *value as f32).collect();
    let vector: Vec<f64> = rotate_rank(&vector32, f32::from(rank))
        .into_iter()
        .map(f64::from)
        .collect();
    let norm = vector
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt()
        .max(1e-9);
    let mut output = [0.0_f64; 3];
    for (index, value) in vector.iter().enumerate() {
        let lane = (index / 32) % 3;
        let salt = 1.0 + 0.13 * ((index % 32) as f64 / 32.0);
        output[lane] += (value / norm) * salt;
    }
    let output_norm = (output[0] * output[0] + output[1] * output[1] + output[2] * output[2])
        .sqrt()
        .max(1e-9);
    [
        output[0] / output_norm,
        output[1] / output_norm,
        output[2] / output_norm,
    ]
}

fn address(
    text: &str,
    salience: f64,
    idf: &HashMap<String, f64>,
    rank: u8,
) -> ([f64; 4], [f64; 2]) {
    let direction = embed_direction(text, idf, rank);
    let radius = (1.0 - salience.clamp(0.0, 1.0)) * 2.5 + 0.1;
    let spatial = [
        direction[0] * radius,
        direction[1] * radius,
        direction[2] * radius,
    ];
    let x0 =
        (1.0 + spatial[0] * spatial[0] + spatial[1] * spatial[1] + spatial[2] * spatial[2]).sqrt();
    let theta_fast = spatial[1].atan2(spatial[0]);
    let spatial_radius = (spatial[0] * spatial[0] + spatial[1] * spatial[1]).sqrt();
    (
        [x0, spatial[0], spatial[1], spatial[2]],
        [theta_fast, spatial[2].atan2(spatial_radius)],
    )
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization")
}

fn array<const N: usize>(values: &[f64; N]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn string(value: &Value, field: &str, id: u64) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("ledger atom {id} has no string {field}"))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: migrate_snapshot SNAPSHOT LEDGER OUTPUT".into());
    }
    let snapshot_path = Path::new(&args[1]);
    let ledger_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);
    let snapshot_bytes = std::fs::read(snapshot_path).map_err(|error| error.to_string())?;
    let snapshot_digest = sha256(&snapshot_bytes);
    let expected_digest = std::fs::read_to_string(snapshot_path.with_extension("sha256"))
        .map_err(|error| error.to_string())?
        .trim()
        .to_owned();
    if snapshot_digest != expected_digest {
        return Err(format!(
            "snapshot digest mismatch: expected {expected_digest}, got {snapshot_digest}"
        ));
    }

    let ledger_bytes = std::fs::read(ledger_path).map_err(|error| error.to_string())?;
    let ledger_digest = sha256(&ledger_bytes);
    let mut sources = BTreeMap::<u64, Value>::new();
    for (line_number, line) in String::from_utf8_lossy(&ledger_bytes).lines().enumerate() {
        let value: Value = serde_json::from_str(line)
            .map_err(|error| format!("ledger line {}: {error}", line_number + 1))?;
        if value.get("op").and_then(Value::as_str) != Some("deposit")
            || value.get("kind").and_then(Value::as_str) != Some("atom")
        {
            continue;
        }
        let id = value
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("ledger line {} has no numeric id", line_number + 1))?;
        if sources.insert(id, value).is_some() {
            return Err(format!("duplicate ledger atom id {id}"));
        }
    }
    let texts: Vec<String> = sources
        .values()
        .map(|value| {
            value
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    let idf = calibrate_idf(&texts);
    let mut atoms = Vec::with_capacity(sources.len());
    for (id, source) in sources {
        let text = string(&source, "text", id)?;
        let salience = source
            .get("salience")
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("ledger atom {id} has no numeric salience"))?;
        let rank = source
            .get("rank")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            .min(1) as u8;
        let (coord, phase) = address(&text, salience, &idf, rank);
        atoms.push(Atom {
            id,
            coord,
            phase,
            text,
            salience,
            provenance: string(&source, "provenance", id)?,
            taint: string(&source, "taint", id)?,
            rank,
            domain: string(&source, "domain", id)?,
            pinned: source
                .get("pinned")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        });
    }

    let mut output = String::new();
    writeln!(output, "UPSERT workspace:hypermind CONTENT {{ slug: 'hypermind', created_at: time::now() }} RETURN NONE;")
        .map_err(|error| error.to_string())?;
    for chunk in atoms.chunks(100) {
        writeln!(output, "BEGIN TRANSACTION;").map_err(|error| error.to_string())?;
        for atom in chunk {
            let content_digest = sha256(atom.text.as_bytes());
            writeln!(output, "CREATE atom:{} CONTENT {{ workspace: workspace:hypermind, external_id: {}, content_kind: 'text', content_text: {}, content_uri: NONE, content_digest: '{}', domain: {}, rank: {}, salience: {:?}, provenance: {}, taint: {}, pinned: {}, current_address: atom_address:{}, created_at: time::now(), updated_at: time::now() }} RETURN NONE;", atom.id, quoted(&atom.id.to_string()), quoted(&atom.text), content_digest, quoted(&atom.domain), atom.rank, atom.salience, quoted(&atom.provenance), quoted(&atom.taint), atom.pinned, atom.id)
                .map_err(|error| error.to_string())?;
            writeln!(output, "CREATE atom_address:{} CONTENT {{ workspace: workspace:hypermind, atom: atom:{}, h3_lorentz: [{}], t2_phase: [{}], geometry_model: 'hash-idf-rank', geometry_version: 'H3xT2-Lorentz-canonical-v2', revision: 1, active: true, created_at: time::now() }} RETURN NONE;", atom.id, atom.id, array(&atom.coord), array(&atom.phase))
                .map_err(|error| error.to_string())?;
        }
        writeln!(output, "COMMIT TRANSACTION;").map_err(|error| error.to_string())?;
    }
    writeln!(output, "UPSERT schema_migration:atoms_v1 CONTENT {{ version: 'atoms-v1', digest: '{}:{}', applied_at: time::now() }} RETURN NONE;", snapshot_digest, ledger_digest)
        .map_err(|error| error.to_string())?;
    std::fs::write(output_path, output).map_err(|error| error.to_string())?;
    println!(
        "atoms={} snapshot_sha256={} ledger_sha256={} output={}",
        atoms.len(),
        snapshot_digest,
        ledger_digest,
        output_path.display()
    );
    Ok(())
}
