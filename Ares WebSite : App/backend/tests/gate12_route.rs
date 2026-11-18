//! Gate 12 HTTP route tests: `POST /api/compliance/core-scope`.
//! The handler existed but was NOT mounted in the router. These tests
//! prove the route is live, permit-gated, and single-use through HTTP,
//! with no live SurrealDB or Qdrant (AppState surreal=None, memory=None).

use ares_backend::compliance::{CoreScopeChange, ImmuneSystem, OverlapPolicyMatrix};
use ares_backend::fixed::Q32_31;
use ares_backend::governor::AppState;
use ares_backend::store::{CarvedSwarmMap, FrozenBaseWeights};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use std::sync::Arc;
use tower::ServiceExt;

fn state() -> Arc<AppState> {
    let base = FrozenBaseWeights::freeze("test-model", 2, vec![0.25f32; 64]);
    let fp = base.fingerprint;
    let map = CarvedSwarmMap::new(base, Q32_31::from_f64(1.22));
    let immune = ImmuneSystem::new(
        OverlapPolicyMatrix::minimal_adjacent(Q32_31::from_f64(0.08)),
        fp,
    );
    Arc::new(AppState::new(immune, map, None))
}

async fn post(state: Arc<AppState>, body: serde_json::Value) -> (StatusCode, serde_json::Value) {
    let app = ares_backend::http::router(state);
    let req = Request::builder()
        .method("POST")
        .uri("/api/compliance/core-scope")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn core_scope_route_without_permit_is_forbidden() {
    let (status, body) = post(state(), serde_json::json!({"scope": "model_swap"})).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        body.get("why_receipt").is_some(),
        "failure must carry a why-receipt"
    );
}

#[tokio::test]
async fn core_scope_route_unknown_scope_is_bad_request() {
    let (status, _body) = post(state(), serde_json::json!({"scope": "not_a_scope"})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn core_scope_route_with_issued_permit_passes_once() {
    let st = state();
    // Operator issuance, recorded in the audit chain (the only way a
    // permit exists; the backend never mints one for itself).
    let seq = {
        let mut immune = st.immune.lock().await;
        immune
            .issue_human_permit(&CoreScopeChange::ModelSwap, "operator-stewart")
            .sequence()
    };
    let (status, body) = post(
        st.clone(),
        serde_json::json!({
            "scope": "model_swap",
            "human_permit_sequence": seq,
            "issued_by": "operator-stewart",
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body: {}", body);
    assert!(body["permit"]["chain_digest"].as_str().is_some());

    // Single-use: the same permit sequence must now fail.
    let (status2, _) = post(
        st,
        serde_json::json!({
            "scope": "model_swap",
            "human_permit_sequence": seq,
            "issued_by": "operator-stewart",
        }),
    )
    .await;
    assert_eq!(status2, StatusCode::FORBIDDEN);
}
