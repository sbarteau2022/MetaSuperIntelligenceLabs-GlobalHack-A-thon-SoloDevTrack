use rgah::agent_step::{Evidence, Retriever, run_step};
use rgah::n3_retriever::N3Retriever;

const FACT: &str = "The Zorblax reactor at Quillan Station vents coolant every 47 minutes.";

fn root() -> String {
    let dir = std::env::temp_dir().join(format!("rgah-n3-retriever-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.to_string_lossy().to_string()
}

/// Live gate: real hypermind-mcp in isolated mode (HYPERMIND_SURREAL=off).
/// Set RGAH_HYPERMIND_MCP to the binary and run with --ignored.
#[test]
#[ignore = "requires hypermind-mcp binary (RGAH_HYPERMIND_MCP)"]
fn retrieves_ingested_fact_through_map_query() {
    let bin = std::env::var("RGAH_HYPERMIND_MCP").expect("RGAH_HYPERMIND_MCP");
    let mut r = N3Retriever::spawn(&bin, &root()).expect("spawn");
    r.ingest(FACT).expect("ingest");
    let hits = r.lookup("how often does the Zorblax reactor vent coolant").expect("query");
    assert!(hits.contains("47 minutes"), "retrieval missed the ingested fact: {hits:.300}");
    // Model-facing evidence is plain text: no JSON structure may cross into the prompt.
    assert!(!hits.contains('{') && !hits.contains('"'), "JSON leaked into evidence: {hits:.300}");
}

#[test]
#[ignore = "requires hypermind-mcp binary (RGAH_HYPERMIND_MCP)"]
fn search_action_feeds_retrieved_fact_into_evidence() {
    let bin = std::env::var("RGAH_HYPERMIND_MCP").expect("RGAH_HYPERMIND_MCP");
    let mut r = N3Retriever::spawn(&bin, &root()).expect("spawn");
    r.ingest(FACT).expect("ingest");
    let mut ev = Evidence::new();
    let sig = run_step(
        "ACTION search :: how often does the Zorblax reactor vent coolant",
        &mut r,
        &mut ev,
    )
    .expect("step");
    assert!(!sig.verifier_pass);
    assert!(ev.render().contains("47 minutes"));
}
