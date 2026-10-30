//! Runtime capability DAG dispatcher (N3 audit Part 3, §19).
//!
//! The vault on disk (`CapabilityTreeSitter/`, `MasterDesignDocument/`,
//! `MasterToolingIndex`, `MasterSkillsIndex`) is static registry. This module
//! is the runtime side: it loads the capability manifest into nodes, wires
//! the hardcoded §19 family-dependency edges, validates the DAG (Kahn +
//! unknown-endpoint check), and admits tool calls through [`CapabilityGate`].
//!
//! Doctrine, same as `registry.rs` / `mcp_client.rs`: NO serde_json anywhere
//! in this crate. The manifest is parsed with a minimal hand scanner that
//! extracts the four string fields per object (`source_root`, `source_path`,
//! `category`, `dest_path`). Serde derive on structs is fine; serde_json is
//! not a dependency and must not become one.
//!
//! §19 family shape (hardcoded):
//!   roots: 01-FormalMethods, 02-SystemsKernels (no incoming)
//!   03-AgenticRuntime depends on 01 + 02
//!   04-Observability + 05-Verification depend on 03
//!   06-DataKnowledge depends on 03 + 05
//!   07-SecuritySandbox depends on all (policy gate)

use serde::{Deserialize, Serialize};

/// The seven §19 families, in canonical order.
pub const KNOWN_FAMILIES: [&str; 7] = [
    "01-FormalMethods",
    "02-SystemsKernels",
    "03-AgenticRuntime",
    "04-Observability",
    "05-Verification",
    "06-DataKnowledge",
    "07-SecuritySandbox",
];

/// Hardcoded §19 family-dependency edges (dependency -> dependent).
pub fn family_edges() -> Vec<(String, String)> {
    let raw = [
        ("01-FormalMethods", "03-AgenticRuntime"),
        ("02-SystemsKernels", "03-AgenticRuntime"),
        ("03-AgenticRuntime", "04-Observability"),
        ("03-AgenticRuntime", "05-Verification"),
        ("03-AgenticRuntime", "06-DataKnowledge"),
        ("05-Verification", "06-DataKnowledge"),
        ("01-FormalMethods", "07-SecuritySandbox"),
        ("02-SystemsKernels", "07-SecuritySandbox"),
        ("03-AgenticRuntime", "07-SecuritySandbox"),
        ("04-Observability", "07-SecuritySandbox"),
        ("05-Verification", "07-SecuritySandbox"),
        ("06-DataKnowledge", "07-SecuritySandbox"),
    ];
    raw.iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// One runtime capability node: one manifest entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityNode {
    /// `category/leaf`, e.g. `03-AgenticRuntime/agents-sdk`.
    pub id: String,
    /// §19 family, one of [`KNOWN_FAMILIES`] (== manifest `category`).
    pub family: String,
    /// Vault path (manifest `dest_path`).
    pub skill_path: String,
    /// Where the skill was sourced from (manifest `source_root`).
    pub source_root: String,
    /// sha256 hex of the canonical `family|id|skill_path|source_root` fields.
    pub digest: String,
}

/// Runtime capability DAG: nodes plus family-granularity edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDag {
    pub nodes: Vec<CapabilityNode>,
    /// Family-granularity edges (`family -> family`); nodes carry family.
    pub edges: Vec<(String, String)>,
}

/// Frozen-policy hook, mirroring [`crate::policy_kernel::FrozenPolicyKernel`]
/// semantics: judge / safety / egress / receipt targets are immutable.
pub fn check_mutation(target: &str) -> Result<(), String> {
    crate::policy_kernel::FrozenPolicyKernel::default().check_mutation(target)
}

fn leaf_of(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    trimmed.rsplit('/').next().unwrap_or(trimmed)
}

fn digest_of(family: &str, id: &str, skill_path: &str, source_root: &str) -> String {
    use sha2::{Digest, Sha256};
    let canon = format!("{}|{}|{}|{}", family, id, skill_path, source_root);
    let mut h = Sha256::new();
    h.update(canon.as_bytes());
    hex::encode(h.finalize())
}

/// Extract one `"key" : "value"` string field from a flat JSON object slice.
/// Tolerates whitespace around the colon; handles the standard `\"` `\\`
/// `\n` `\r` `\t` `\/` escapes. First well-formed occurrence wins.
fn field(obj: &str, key: &str) -> Option<String> {
    let kpat = format!("\"{}\"", key);
    let bytes = obj.as_bytes();
    let mut search = 0usize;
    while search < obj.len() {
        let rel = obj[search..].find(&kpat)?;
        let mut j = search + rel + kpat.len();
        while j < bytes.len()
            && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r')
        {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b':' {
            search += rel + 1;
            continue;
        }
        j += 1;
        while j < bytes.len()
            && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r')
        {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'"' {
            search += rel + 1;
            continue;
        }
        j += 1;
        let mut out = String::new();
        let mut k = j;
        while k < bytes.len() {
            match bytes[k] {
                b'\\' if k + 1 < bytes.len() => {
                    match bytes[k + 1] {
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        c => {
                            out.push('\\');
                            out.push(c as char);
                        }
                    }
                    k += 2;
                }
                b'"' => return Some(out),
                c => {
                    out.push(c as char);
                    k += 1;
                }
            }
        }
        return None;
    }
    None
}

/// Split manifest text into top-level `{...}` object slices, ignoring braces
/// inside JSON strings. Same escape-aware scanning idiom as `mcp_client`.
fn split_objects(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut objs = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for (i, &b) in bytes.iter().enumerate() {
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            b'}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        objs.push(&text[start..=i]);
                    }
                }
            }
            _ => {}
        }
    }
    objs
}

impl CapabilityDag {
    /// Load nodes from manifest JSON text with the hand scanner (no
    /// serde_json), then attach the hardcoded §19 family edges.
    pub fn load_from_manifest(manifest_text: &str) -> Result<Self, String> {
        let mut nodes = Vec::new();
        for (idx, obj) in split_objects(manifest_text).iter().enumerate() {
            let source_root = field(obj, "source_root")
                .ok_or_else(|| format!("manifest object {} missing field 'source_root'", idx))?;
            let source_path = field(obj, "source_path")
                .ok_or_else(|| format!("manifest object {} missing field 'source_path'", idx))?;
            let category = field(obj, "category")
                .ok_or_else(|| format!("manifest object {} missing field 'category'", idx))?;
            let dest_path = field(obj, "dest_path")
                .ok_or_else(|| format!("manifest object {} missing field 'dest_path'", idx))?;
            let leaf = leaf_of(&source_path);
            let leaf = if leaf.is_empty() {
                leaf_of(&dest_path)
            } else {
                leaf
            };
            let id = format!("{}/{}", category, leaf);
            let digest = digest_of(&category, &id, &dest_path, &source_root);
            nodes.push(CapabilityNode {
                id,
                family: category,
                skill_path: dest_path,
                source_root,
                digest,
            });
        }
        Ok(Self {
            nodes,
            edges: family_edges(),
        })
    }

    /// Validate: unknown-family / unknown-endpoint check first, then Kahn
    /// cycle detection over the family graph.
    pub fn validate_dag(&self) -> Result<(), String> {
        for n in &self.nodes {
            if !KNOWN_FAMILIES.contains(&n.family.as_str()) {
                return Err(format!(
                    "unknown family endpoint '{}' on node '{}'",
                    n.family, n.id
                ));
            }
        }
        for (a, b) in &self.edges {
            if !KNOWN_FAMILIES.contains(&a.as_str()) {
                return Err(format!(
                    "unknown endpoint '{}' in edge ('{}','{}')",
                    a, a, b
                ));
            }
            if !KNOWN_FAMILIES.contains(&b.as_str()) {
                return Err(format!(
                    "unknown endpoint '{}' in edge ('{}','{}')",
                    b, a, b
                ));
            }
        }
        use std::collections::{HashMap, HashSet};
        let mut fams: HashSet<&str> = self.nodes.iter().map(|n| n.family.as_str()).collect();
        for (a, b) in &self.edges {
            fams.insert(a.as_str());
            fams.insert(b.as_str());
        }
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut indeg: HashMap<&str, usize> = HashMap::new();
        for &f in &fams {
            adj.entry(f).or_default();
            indeg.entry(f).or_insert(0);
        }
        for (a, b) in &self.edges {
            adj.get_mut(a.as_str()).unwrap().push(b.as_str());
            *indeg.get_mut(b.as_str()).unwrap() += 1;
        }
        let mut queue: Vec<&str> = indeg
            .iter()
            .filter(|&(_, &v)| v == 0)
            .map(|(&k, _)| k)
            .collect();
        let mut seen = 0usize;
        while let Some(n) = queue.pop() {
            seen += 1;
            for &m in &adj[&n] {
                let d = indeg.get_mut(m).unwrap();
                *d = d.saturating_sub(1);
                if *d == 0 {
                    queue.push(m);
                }
            }
        }
        if seen != fams.len() {
            return Err("capability DAG cycle detected over family edges".into());
        }
        Ok(())
    }

    /// Match by full native id (`03-AgenticRuntime/agents-sdk`), by leaf dir
    /// name (`agents-sdk`), or by vault skill path / its leaf.
    pub fn resolve_tool(&self, tool_name: &str) -> Option<&CapabilityNode> {
        let t = tool_name.trim();
        if let Some(n) = self.nodes.iter().find(|n| n.id == t) {
            return Some(n);
        }
        if let Some(n) = self.nodes.iter().find(|n| leaf_of(&n.id) == t) {
            return Some(n);
        }
        if let Some(n) = self.nodes.iter().find(|n| n.skill_path == t) {
            return Some(n);
        }
        self.nodes.iter().find(|n| leaf_of(&n.skill_path) == t)
    }

    /// Fail-explained denial receipt: names the tool and the families
    /// searched, with remediation. Never panics on unknown input.
    pub fn deny_unknown(&self, tool: &str) -> String {
        let mut fams: Vec<&str> = self.nodes.iter().map(|n| n.family.as_str()).collect();
        fams.sort_unstable();
        fams.dedup();
        format!(
            "DENIED unknown-tool '{}': no capability node matches; searched {} nodes across {} families [{}]; declare the tool in the capability manifest or add it to the S19 family DAG before invocation.",
            tool,
            self.nodes.len(),
            fams.len(),
            fams.join(", ")
        )
    }

    /// Frozen-policy hook on the DAG (same semantics as the free function).
    pub fn check_mutation(&self, target: &str) -> Result<(), String> {
        check_mutation(target)
    }
}

/// Engine-facing admission gate over a loaded [`CapabilityDag`].
/// The engine-N3 wiring calls [`CapabilityGate::admit`]; unknown tools get
/// the fail-explained [`CapabilityDag::deny_unknown`] receipt as `Err`.
#[derive(Debug, Clone)]
pub struct CapabilityGate {
    pub dag: CapabilityDag,
}

impl CapabilityGate {
    pub fn new(dag: CapabilityDag) -> Self {
        Self { dag }
    }

    pub fn from_manifest(manifest_text: &str) -> Result<Self, String> {
        Ok(Self {
            dag: CapabilityDag::load_from_manifest(manifest_text)?,
        })
    }

    pub fn admit(&self, tool: &str) -> Result<&CapabilityNode, String> {
        self.dag
            .resolve_tool(tool)
            .ok_or_else(|| self.dag.deny_unknown(tool))
    }

    pub fn check_mutation(&self, target: &str) -> Result<(), String> {
        check_mutation(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Hermetic 6-node / 2-family canned manifest (inline, no fs reads).
    const CANNED: &str = r#"[
      {"source_root": "/r/skills", "source_path": "/r/skills/agents-sdk", "category": "03-AgenticRuntime", "dest_path": "/vault/03-AgenticRuntime/agents-sdk"},
      {"source_root": "/r/skills", "source_path": "/r/skills/ares-agents", "category": "03-AgenticRuntime", "dest_path": "/vault/03-AgenticRuntime/ares-agents"},
      {"source_root": "/r/skills", "source_path": "/r/skills/find-skills", "category": "03-AgenticRuntime", "dest_path": "/vault/03-AgenticRuntime/find-skills"},
      {"source_root": "/r/skills", "source_path": "/r/skills/skill-verify", "category": "05-Verification", "dest_path": "/vault/05-Verification/skill-verify"},
      {"source_root": "/r/skills", "source_path": "/r/skills/certify-or-propose-gaps", "category": "05-Verification", "dest_path": "/vault/05-Verification/certify-or-propose-gaps"},
      {"source_root": "/r/skills", "source_path": "/r/skills/codebase-pressure-test", "category": "05-Verification", "dest_path": "/vault/05-Verification/codebase-pressure-test"}
    ]"#;

    fn canned() -> CapabilityDag {
        CapabilityDag::load_from_manifest(CANNED).expect("canned manifest must load")
    }

    #[test]
    fn loads_six_nodes_two_families() {
        let dag = canned();
        // 1. six nodes loaded.
        assert_eq!(
            dag.nodes.len(),
            6,
            "expected 6 nodes, got {}",
            dag.nodes.len()
        );
        // 2. exactly the two canned families.
        let mut fams: Vec<&str> = dag.nodes.iter().map(|n| n.family.as_str()).collect();
        fams.sort_unstable();
        fams.dedup();
        assert_eq!(fams, vec!["03-AgenticRuntime", "05-Verification"]);
        // 3. id is category + leaf.
        let n = dag.resolve_tool("agents-sdk").expect("leaf must resolve");
        assert_eq!(n.id, "03-AgenticRuntime/agents-sdk");
        // 4. digests are sha256 hex, distinct per node.
        for n in &dag.nodes {
            assert_eq!(n.digest.len(), 64, "digest must be sha256 hex for {}", n.id);
        }
        let mut digests: Vec<&str> = dag.nodes.iter().map(|n| n.digest.as_str()).collect();
        digests.sort_unstable();
        digests.dedup();
        assert_eq!(digests.len(), 6, "digests must be distinct per node");
    }

    #[test]
    fn family_edges_acyclic_and_roots_have_no_incoming() {
        let dag = canned();
        // 6. shipped §19 edges validate clean.
        assert!(
            dag.validate_dag().is_ok(),
            "shipped family edges must be acyclic"
        );
        // 7. roots have no incoming edges.
        for root in ["01-FormalMethods", "02-SystemsKernels"] {
            assert!(
                dag.edges.iter().all(|(_, b)| b != root),
                "root {} must have no incoming edges",
                root
            );
        }
        // 8. policy gate: 07 depends on every other family.
        for fam in KNOWN_FAMILIES.iter().take(6) {
            assert!(
                dag.edges
                    .iter()
                    .any(|(a, b)| a == *fam && b == "07-SecuritySandbox"),
                "07-SecuritySandbox must depend on {}",
                fam
            );
        }
    }

    #[test]
    fn cycle_is_detected() {
        let mut dag = canned();
        // 9. inject the back edge 05 -> 03 alongside shipped 03 -> 05.
        dag.edges.push((
            "05-Verification".to_string(),
            "03-AgenticRuntime".to_string(),
        ));
        let err = dag.validate_dag().expect_err("cycle must fail validation");
        // 10. error names the cycle.
        assert!(
            err.contains("cycle"),
            "cycle error must say cycle, got: {}",
            err
        );
    }

    #[test]
    fn unknown_endpoint_is_rejected() {
        let mut dag = canned();
        dag.edges
            .push(("03-AgenticRuntime".to_string(), "99-Nope".to_string()));
        let err = dag.validate_dag().expect_err("unknown endpoint must fail");
        // 11. error names the unknown endpoint.
        assert!(err.contains("unknown"), "must say unknown, got: {}", err);
        assert!(
            err.contains("99-Nope"),
            "must name the endpoint, got: {}",
            err
        );
    }

    #[test]
    fn resolve_hit_and_miss() {
        let dag = canned();
        // 12. leaf hit.
        assert!(dag.resolve_tool("agents-sdk").is_some());
        // 13. full native id hit.
        let full = dag
            .resolve_tool("05-Verification/skill-verify")
            .expect("full id must resolve");
        assert_eq!(full.family, "05-Verification");
        // 14. vault path hit.
        assert!(
            dag.resolve_tool("/vault/03-AgenticRuntime/find-skills")
                .is_some()
        );
        // 15. miss yields None.
        assert!(dag.resolve_tool("nope-tool-xyz").is_none());
    }

    #[test]
    fn deny_unknown_names_tool_and_families() {
        let dag = canned();
        let receipt = dag.deny_unknown("nope-tool-xyz");
        // 16. receipt names the tool.
        assert!(
            receipt.contains("nope-tool-xyz"),
            "receipt must name tool: {}",
            receipt
        );
        // 17. receipt is a DENIED fail-explained receipt.
        assert!(
            receipt.contains("DENIED"),
            "receipt must say DENIED: {}",
            receipt
        );
        // 18. receipt lists families searched.
        assert!(
            receipt.contains("03-AgenticRuntime") && receipt.contains("05-Verification"),
            "receipt must list families searched: {}",
            receipt
        );
    }

    #[test]
    fn frozen_policy_denies_judge_family() {
        let dag = canned();
        // 19. judge-family mutation denied (mirrors FrozenPolicyKernel).
        let err = dag
            .check_mutation("judge")
            .expect_err("judge must be denied");
        assert!(
            err.contains("immutable"),
            "must say immutable, got: {}",
            err
        );
        // 20. safety / egress / receipt denied too.
        assert!(check_mutation("safety").is_err());
        assert!(check_mutation("egress").is_err());
        assert!(check_mutation("receipt").is_err());
        // 21. unfrozen target admitted.
        assert!(check_mutation("mission").is_ok());
        // 22. gate-level hook agrees.
        let gate = CapabilityGate::new(dag);
        assert!(gate.check_mutation("judge").is_err());
    }

    #[test]
    fn gate_admits_known_and_denies_unknown() {
        let gate = CapabilityGate::new(canned());
        // 23. known tool admitted with node ref.
        let node = gate
            .admit("agents-sdk")
            .expect("known tool must be admitted");
        assert_eq!(node.id, "03-AgenticRuntime/agents-sdk");
        // 24. unknown tool denied with fail-explained receipt naming it.
        let err = gate
            .admit("ghost-tool")
            .expect_err("unknown must be denied");
        assert!(
            err.contains("ghost-tool"),
            "gate denial must name tool: {}",
            err
        );
        assert!(
            err.contains("DENIED"),
            "gate denial must be DENIED receipt: {}",
            err
        );
    }
}
