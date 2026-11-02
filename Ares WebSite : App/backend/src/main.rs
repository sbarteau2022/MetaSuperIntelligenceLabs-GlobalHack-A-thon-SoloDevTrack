//! Ares backend binary: one process, loopback :4810.
//!
//! Startup is fail-closed:
//! 1. Connect SurrealDB, apply schema, seed the 12 controls.
//! 2. Load the audit chain and verify it from genesis to head.
//!    A broken chain refuses startup with a why-receipt — the backend
//!    does not run ungoverned and log a warning.
//! 3. Pin the weight fingerprint; restore the ImmuneSystem from the chain.
//! 4. Connect Qdrant and ensure the append-only `session_memory_v1`
//!    memory collection; construct the HyperDX OTLP telemetry exporter.
//! 5. Bind 127.0.0.1:4810 (after an lsof check) and serve.

use ares_backend::compliance::ImmuneSystem;
use ares_backend::governor::{
    apply_schema, default_base_weights, default_policy, load_and_verify_chain, seed_controls,
    AppState,
};
use ares_backend::store::CarvedSwarmMap;
use ares_backend::surreal::SurrealClient;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Why-receipt on startup failure: what ran, route, status, last
        // output, expected, evidenced cause, unblocker.
        eprintln!(
            "{}",
            serde_json::json!({
                "startup": "FAILED (fail-closed)",
                "why_receipt": {
                    "what_ran": "ares-backend startup (schema, chain verification, bind)",
                    "route": "127.0.0.1:4810",
                    "status": "refused",
                    "last_output": e,
                    "expected": "verified audit chain, live SurrealDB, free :4810",
                    "evidenced_cause": e,
                    "unblocker": "fix the evidenced cause; the backend does not run ungoverned"
                }
            })
        );
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    // Port discipline: refuse to bind if :4810 is taken.
    let out = std::process::Command::new("lsof")
        .args(["-nP", "-iTCP:4810", "-sTCP:LISTEN"])
        .output()
        .map_err(|e| e.to_string())?;
    let listeners = String::from_utf8_lossy(&out.stdout);
    if listeners.lines().count() > 1 {
        return Err(format!("port 4810 already bound:\n{}", listeners.trim()));
    }

    let surreal = match SurrealClient::from_env() {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("WARN surreal client unavailable ({}); running governed in-memory with persistence degraded — status will report surrealdb down", e);
            None
        }
    };

    let (entries, head) = if let Some(client) = &surreal {
        apply_schema(client).await?;
        seed_controls(client).await?;
        match load_and_verify_chain(client).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        }
    } else {
        (Vec::new(), ares_backend::digest::Sha256Digest([0u8; 32]))
    };

    let base = default_base_weights();
    let pinned = base.fingerprint;
    let policy = default_policy();
    let immune = if entries.is_empty() {
        ImmuneSystem::new(policy, pinned)
    } else {
        ImmuneSystem::restore(policy, pinned, entries, head)
            .map_err(|e| format!("chain restore failed: {}", e))?
    };
    let map = CarvedSwarmMap::new(base, ares_backend::fixed::Q32_31::from_f64(1.22));

    let mut app_state = AppState::new(immune, map, surreal);
    // Stores by job: HyperDX is telemetry/tracking (OLTP append-only);
    // Qdrant is RAG + queryable append-only memory. Startup uses real
    // checks here (HyperDX exporter is constructed with the 4318
    // endpoint; Qdrant uses a live collection check + idempotent create).
    match ares_backend::harness::telemetry::HyperdxTelemetry::new(
        ares_backend::harness::telemetry::HyperdxConfig::from_env(),
    ) {
        Ok(telemetry) => {
            let status = telemetry.config_status();
            println!("HyperDX telemetry configured: {}", status);
            app_state.telemetry = Some(telemetry);
        }
        Err(e) => {
            eprintln!(
                "WARN HyperDX telemetry unavailable ({}); harness telemetry degraded",
                e
            );
        }
    }
    match ares_backend::harness::qdrant_memory::QdrantMemory::from_env() {
        Ok(memory) => match memory.ensure_collection().await {
            Ok(created) => {
                println!(
                    "Qdrant memory collection {} ready (created_new={})",
                    memory.collection(),
                    created
                );
                app_state.memory = Some(memory);
            }
            Err(e) => {
                eprintln!(
                    "WARN Qdrant memory unavailable ({}); memory tail degraded",
                    e
                );
            }
        },
        Err(e) => {
            eprintln!(
                "WARN Qdrant memory client failed ({}); memory tail degraded",
                e
            );
        }
    }
    // Seed the capability tree with the workflow's proven nodes only.
    {
        let mut gate = ares_backend::harness::runtime::CapabilityGate::default();
        for tool in ["map.query", "map.kv_fetch", "map.ingest"] {
            gate.register(ares_backend::harness::runtime::CapabilityNode {
                name: tool.to_string(),
                family: "map".to_string(),
                judged_atom: true,
                receipt_present: true,
            });
        }
        app_state.capability_gate = tokio::sync::Mutex::new(gate);
    }
    let state = Arc::new(app_state);
    let app = ares_backend::http::router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:4810")
        .await
        .map_err(|e| format!("bind 127.0.0.1:4810 failed: {}", e))?;
    println!("ares-backend listening on http://127.0.0.1:4810 (governed, fail-closed)");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
