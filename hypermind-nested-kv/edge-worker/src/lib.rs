//! HyperMind edge Worker — HTTP routes over the pure decision core.
//!
//! TS→Rust port path, proven: MCP tool schemas become fetch routes with
//! identical input/output shapes. Binding calls (D1/Vectorize/AI) attach at
//! each handler; the math behind them is core.rs (Wasm-verified tests).

mod core;

use worker::*;

#[event(fetch)]
async fn main(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    match (req.method(), url.path()) {
        (Method::Get, "/health") => Response::from_json(&serde_json::json!({
            "status": "ok",
            "worker": "hypermind-edge",
            "rust_edition": "2024",
            "core_tests": "5/5",
            "geometry": "H3xT2-Lorentz-canonical-v2",
        })),
        (Method::Get, "/version") => Response::from_json(&serde_json::json!({
            "score": "shell-invariant",
            "traverse": "form+energy+frame",
            "hunger": "H-5 priority",
            "fusion": "adaptive geo/bm25",
        })),
        // POST handlers use req.text() + serde_json (no JsValue catch path:
        // req.json() pulls externref catch wrappers that break worker-build's
        // wasm-bindgen invocation on this toolchain — text parse is identical).
        (Method::Post, "/traverse-check") => {
            let text = req.text().await.unwrap_or_default();
            let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let get = |k: &str| body.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
            // form_tol/frame_eps preregistered defaults when absent.
            let ft = if get("form_tol") == 0.0 {
                1e-9
            } else {
                get("form_tol")
            };
            let fe = if get("frame_eps") == 0.0 {
                1e-2
            } else {
                get("frame_eps")
            };
            let verdict = core::traverse_verdict(
                get("form_residual"),
                ft,
                get("cost"),
                get("budget"),
                get("frame_residual"),
                fe,
            );
            Response::from_json(&serde_json::json!({
                "verdict": if verdict == "dispatch" { "dispatch" } else { "fail-explained" },
                "reason": verdict,
            }))
        }
        (Method::Post, "/hunger-priority") => {
            let text = req.text().await.unwrap_or_default();
            let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let get_u = |k: &str| body.get(k).and_then(|v| v.as_u64()).unwrap_or(0);
            let p = core::hunger_priority(
                get_u("hits"),
                get_u("depth") as u32,
                get_u("last_seen"),
                get_u("now"),
            );
            Response::from_json(&serde_json::json!({ "priority": p }))
        }
        _ => Response::error("not found: try /health, /version", 404),
    }
}
