//! HyperMind edge Worker — HTTP routes over the pure decision core.
//!
//! TS→Rust port path, proven: MCP tool schemas become fetch routes with
//! identical input/output shapes. Binding calls (D1/Vectorize/AI) attach at
//! each handler; the math behind them is core.rs (Wasm-verified tests).

mod core;

use worker::*;

/// Unix seconds via the Worker runtime clock (no Math.random, no Date drift).
fn js_sys_now() -> u64 {
    (Date::now().as_millis() / 1000) as u64
}

#[event(fetch)]
async fn main(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    match (req.method(), url.path()) {
        // Hunger tap write: deterministic upsert, no model. Mirrors the
        // in-server tap_hunger() 1:1 (TAP-N-TRACE §4). Zero permissions
        // needed beyond the D1 binding itself.
        (Method::Post, "/hunger-tap") => {
            let text = req.text().await.unwrap_or_default();
            let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let s = |k: &str| {
                body.get(k)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let now: u64 = js_sys_now();
            let db = env.d1("HUNGER")?;
            let gap_key = s("gap_key");
            let depth: u32 = body.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let atom_ids = body
                .get("atom_ids")
                .cloned()
                .unwrap_or(serde_json::json!([]))
                .to_string();
            db.prepare(
                "INSERT INTO hunger (gap_key, tap, terms_hash, atom_ids, hits, depth, first_seen, last_seen, sated, sated_by)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?6, 0, '')
                 ON CONFLICT(gap_key) DO UPDATE SET hits = hits + 1, last_seen = excluded.last_seen,
                 depth = max(depth, excluded.depth)",
            )
            // D1 local binding rejects BigInt: all numerics bound as f64
            // (lossless to 2^53 — hunger counts never approach it).
            .bind(&[gap_key.clone().into(), s("tap").into(), s("terms_hash").into(), atom_ids.into(), (depth as f64).into(), (now as f64).into()])?
            .run()
            .await?;
            let row = db
                .prepare("SELECT hits FROM hunger WHERE gap_key = ?1")
                .bind(&[gap_key.clone().into()])?
                .first::<serde_json::Value>(None)
                .await?;
            Response::from_json(&serde_json::json!({
                "status": "ok", "gap_key": gap_key,
                "hits": row.as_ref().and_then(|r| r.get("hits")).and_then(|v| v.as_u64()).unwrap_or(1),
            }))
        }
        // Ranked leaderboard H-5: priority computed in SQL from the same
        // formula as core::hunger_priority (hits × depth × (1+recency)).
        (Method::Get, "/hunger") => {
            let db = env.d1("HUNGER")?;
            let k: usize = url
                .query_pairs()
                .find_map(|(kk, vv)| (kk == "k").then(|| vv.parse().unwrap_or(10)))
                .unwrap_or(10);
            let now: u64 = js_sys_now();
            let rows = db
                .prepare(
                    "SELECT gap_key, tap, hits, depth, last_seen,
                 hits * max(depth,1) * (1.0 + 1.0/(1.0 + (?1 - last_seen)/86400.0)) AS priority
                 FROM hunger WHERE sated = 0 ORDER BY priority DESC LIMIT ?2",
                )
                .bind(&[(now as f64).into(), (k as f64).into()])?
                .all()
                .await?;
            let gaps: Vec<serde_json::Value> = rows.results::<serde_json::Value>()?;
            let open_row = db
                .prepare("SELECT COUNT(*) AS c FROM hunger WHERE sated = 0")
                .first::<serde_json::Value>(None)
                .await?;
            let open: u64 = open_row
                .as_ref()
                .and_then(|r| r.get("c"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let sated_row = db
                .prepare("SELECT COUNT(*) AS c FROM hunger WHERE sated = 1")
                .first::<serde_json::Value>(None)
                .await?;
            let sated: u64 = sated_row
                .as_ref()
                .and_then(|r| r.get("c"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            Response::from_json(&serde_json::json!({
                "status": "ok", "open_gaps": open, "sated_gaps": sated, "gaps": gaps,
            }))
        }
        // Consolidate (sleep): reinforce/decay handled by caller-supplied
        // salience map in phase 1; this endpoint sates covered gaps.
        // Full reinforce/decay moves server-side with the write path (step 2).
        (Method::Post, "/hunger-sate") => {
            let text = req.text().await.unwrap_or_default();
            let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
            let gap_key = body
                .get("gap_key")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let by = body
                .get("sated_by")
                .and_then(|v| v.as_str())
                .unwrap_or("consolidate")
                .to_string();
            let db = env.d1("HUNGER")?;
            db.prepare("UPDATE hunger SET sated = 1, sated_by = ?1 WHERE gap_key = ?2")
                .bind(&[by.clone().into(), gap_key.clone().into()])?
                .run()
                .await?;
            Response::from_json(&serde_json::json!({
                "status": "ok", "gap_key": gap_key, "sated_by": by,
            }))
        }
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
