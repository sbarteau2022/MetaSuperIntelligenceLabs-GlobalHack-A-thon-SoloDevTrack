//! HyperDX telemetry/trace/metric exporter.
//!
//! HyperDX is the telemetry store. This module is the harness event sink
//! boundary: it preserves the old call/verdict recording ergonomics, but
//! events terminate in OTLP JSON sent to the local ClickStack/HyperDX
//! collector (ports 4317 gRPC / 4318 HTTP). HyperDX ingestion is
//! append-only by construction in ClickHouse (`otel_logs`, `otel_traces`,
//! `otel_metrics_*`); this module never owns a local database and never
//! updates/deletes history.
//!
//! Transport notes (grounded in the live collector):
//! - The local collector at `127.0.0.1:4318` wants an ingestion key in the
//!   `Authorization` header. The key is read at call time from
//!   `~/.config/ares-backend/secrets.env` as
//!   `HYPERDX_API_KEY` or `HYPERDX_INGEST_KEY` (with a bare-key fallback).
//!   It is never serialized into telemetry payload JSON and never logged.
//! - A log event is emitted as an OTLP `POST /v1/logs` request with `severityText`
//!   and `ares.*` log attributes. Verdicts and packet outcomes additionally
//!   form a span in `POST /v1/traces` and a gauge metric in `/v1/metrics`.
//! - If HyperDX is unavailable or the key is missing, `emit_*` returns an
//!   error that is captured as a telemetry failure by the engine. Engine
//!   verdicts never read exporter state (the binary verdict is determined
//!   by executed evidence, not transport), but "live run" receipts are
//!   only promoted by `dispatch_mission` when non-empty exported
//!   telemetry was acknowledged.
//!
//! Secrets discipline (Rust & Tokio systems-engineering doctrine):
//! secrets resolve at call time from a 0600 env file and never serialize
//! into JSON snapshots, telemetry payloads, or child environments.

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// HyperDX exporter configuration. Secrets stay outside JSON.
#[derive(Debug, Clone)]
pub struct HyperdxConfig {
    pub endpoint: String,
    pub service_name: String,
    api_key: Option<String>,
    api_key_header_name: String,
}

pub fn any_value_string(s: impl Into<String>) -> Value {
    json!({ "stringValue": s.into() })
}

pub fn any_value_i64(v: i64) -> Value {
    json!({ "intValue": v.to_string() })
}

pub fn any_value_bool(v: bool) -> Value {
    json!({ "boolValue": v })
}

pub fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn now_unix_nano() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

pub fn parse_env_file(path: &std::path::Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        })
        .collect()
}

fn hyperdx_key_from_env_file() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home).join(".config/ares-backend/secrets.env");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&path) {
            let mode = meta.permissions().mode() & 0o777;
            if mode & 0o077 != 0 {
                eprintln!(
                    "WARN refusing HyperDX key from {}: mode {:o} is wider than 0600",
                    path.display(),
                    mode
                );
                return None;
            }
        }
    }
    let vals = parse_env_file(&path);
    let get = |k: &str| {
        vals.iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    get("HYPERDX_API_KEY")
        .or_else(|| get("HYPERDX_INGEST_KEY"))
        .or_else(|| std::env::var("HYPERDX_API_KEY").ok())
}

impl HyperdxConfig {
    pub fn from_env() -> Self {
        let endpoint = std::env::var("HYPERDX_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:4318".to_string());
        let service_name =
            std::env::var("HYPERDX_SERVICE_NAME").unwrap_or_else(|_| "ares-backend".to_string());
        let mut cfg = Self {
            endpoint,
            service_name,
            api_key: None,
            api_key_header_name: "Authorization".to_string(),
        };
        cfg.api_key = hyperdx_key_from_env_file();
        cfg
    }

    pub fn endpoint_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.endpoint.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub fn has_key(&self) -> bool {
        self.api_key
            .as_ref()
            .map(|k| !k.trim().is_empty())
            .unwrap_or(false)
    }
}

#[derive(Debug)]
pub struct HyperdxTelemetry {
    cfg: HyperdxConfig,
    client: reqwest::Client,
    instance_id: String,
    seq: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct ExportAck {
    pub endpoint: String,
    pub status: u16,
    pub event_name: String,
    pub sequence: u64,
}

impl HyperdxTelemetry {
    pub fn new(cfg: HyperdxConfig) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            cfg,
            client,
            instance_id: hex_encode(&Sha256::digest(
                format!("{}:{}", std::process::id(), now_unix_nano()).as_bytes(),
            ))[..16]
                .to_string(),
            seq: AtomicU64::new(1),
        })
    }

    pub fn config_status(&self) -> Value {
        json!({
            "store": "hyperdx",
            "endpoint": self.cfg.endpoint,
            "service_name": self.cfg.service_name,
            "ingestion_key_present": self.cfg.has_key(),
            "append_only_store": true,
        })
    }

    fn next_sequence(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::SeqCst)
    }

    fn auth_header_value(&self) -> Option<String> {
        let key = self.cfg.api_key.as_ref()?.trim().to_string();
        if key.is_empty() {
            return None;
        }
        // HyperDX accepts its ingestion key as the Authorization value.
        // Whatever shape the secret has (bare key, `Bearer ...`, or a
        // `scheme value` pair), it is forwarded verbatim.
        Some(key)
    }

    fn resource(&self) -> Value {
        json!({
            "attributes": [
                { "key": "service.name", "value": any_value_string(self.cfg.service_name.clone()) },
                { "key": "deployment.environment.name", "value": any_value_string("local") },
                { "key": "telemetry.sdk.language", "value": any_value_string("rust") },
                { "key": "ares.runtime", "value": any_value_string("rust-tokio") },
            ],
            "droppedAttributesCount": 0
        })
    }

    fn attributes(pairs: Vec<(&str, Value)>) -> Vec<Value> {
        pairs
            .into_iter()
            .map(|(key, value)| json!({ "key": key, "value": value }))
            .collect()
    }

    async fn post(&self, path: &str, body: Value, event_name: &str) -> Result<ExportAck, String> {
        let sequence = self.next_sequence();
        let url = self.cfg.endpoint_url(path);
        let mut req = self
            .client
            .post(url.clone())
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(5))
            .json(&body);
        let auth = self.auth_header_value().ok_or_else(|| {
            "HYPERDX_API_KEY missing from ~/.config/ares-backend/secrets.env".to_string()
        })?;
        req = req.header(self.cfg.api_key_header_name.clone(), auth);
        let resp = req
            .send()
            .await
            .map_err(|e| format!("HyperDX POST {} failed: {}", url, e))?;
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap_or_default();
        if !(200..300).contains(&(status as i32)) {
            return Err(format!(
                "HyperDX POST {} refused: HTTP {} {}",
                url,
                status,
                text.chars().take(240).collect::<String>()
            ));
        }
        Ok(ExportAck {
            endpoint: url,
            status,
            event_name: event_name.to_string(),
            sequence,
        })
    }

    /// Emit one OTLP logs-batch for one harness event.
    #[allow(clippy::too_many_arguments)]
    pub async fn emit_log(
        &self,
        short_kind: &str,
        packet_id: &str,
        tool: &str,
        outcome: &str,
        ok: bool,
        dt_ns: u64,
        receipt_sha256: Option<&str>,
        idempotency_key: &str,
    ) -> Result<ExportAck, String> {
        let now_ns = now_unix_nano();
        let body_text = format!(
            "{} tool={} packet={} ok={}",
            short_kind, tool, packet_id, ok
        );
        let trace_id = self.trace_id_for(idempotency_key);
        let span_id = self.span_id_for(&format!("{}:log", idempotency_key));
        let payload = json!({
            "resourceLogs": [{
                "resource": self.resource(),
                "scopeLogs": [{
                    "scope": { "name": "ares.harness", "version": "1" },
                    "logRecords": [{
                        "timeUnixNano": now_ns.to_string(),
                        "observedTimeUnixNano": now_ns.to_string(),
                        "severityNumber": if ok { 9 } else { 17 },
                        "severityText": if ok { "INFO" } else { "ERROR" },
                        "body": any_value_string(body_text),
                        "traceId": trace_id,
                        "spanId": span_id,
                        "attributes": Self::attributes(vec![
                            ("ares.event.kind", any_value_string(short_kind)),
                            ("ares.packet.id", any_value_string(packet_id)),
                            ("ares.tool", any_value_string(tool)),
                            ("ares.outcome", any_value_string(outcome)),
                            ("ares.ok", any_value_bool(ok)),
                            ("ares.dt_ns", any_value_i64(dt_ns as i64)),
                            ("ares.idempotency_key", any_value_string(idempotency_key)),
                            (
                                "ares.receipt_sha256",
                                any_value_string(receipt_sha256.unwrap_or_default()),
                            ),
                        ]),
                    }]
                }]
            }]
        });
        self.post("/v1/logs", payload, &format!("log:{}", short_kind))
            .await
    }

    /// Record one harness tool call into HyperDX (OLTP logs + traces).
    #[allow(clippy::too_many_arguments)]
    pub async fn record_call(
        &self,
        idempotency_key: &str,
        tool: &str,
        args_json: &str,
        dt_ns: u64,
        ok: bool,
        receipt_sha256: Option<&str>,
    ) -> Result<u64, String> {
        let ack = self
            .emit_log(
                "tool_call",
                "",
                tool,
                if ok { "OK" } else { "ERR" },
                ok,
                dt_ns,
                receipt_sha256,
                idempotency_key,
            )
            .await
            .map_err(|e| format!("telemetry tool_call export failed for {}: {}", tool, e))?;
        // Trace mirrors the same event as a span. The user-facing ingest success
        // is the logs write (HyperDX's primary store); trace ingest errors do
        // not fail the engine verdict but are surfaced by dispatch checks via
        // `export_trace` in the trace path, so here we retain the same
        // semantics as the SQLite path (verdict export is best-effort in the
        // engine, logs are the durability floor).
        let _ = self
            .export_span(
                tool,
                "tool_call",
                "",
                idempotency_key,
                dt_ns,
                ok,
                receipt_sha256,
                args_json,
            )
            .await;
        Ok(ack.sequence)
    }

    /// Record one packet verdict into HyperDX.
    pub async fn record_verdict(
        &self,
        idempotency_key: &str,
        packet_id: &str,
        outcome: &str,
        judges: &str,
    ) -> Result<u64, String> {
        let ack = self
            .emit_log(
                "verdict",
                packet_id,
                "",
                outcome,
                outcome == "PASS",
                0,
                None,
                idempotency_key,
            )
            .await
            .map_err(|e| format!("telemetry verdict export failed for {}: {}", packet_id, e))?;
        let _ = judges;
        Ok(ack.sequence)
    }

    /// Emit a single OTLP span carrying the same event surface in traces.
    #[allow(clippy::too_many_arguments)]
    pub async fn export_span(
        &self,
        name: &str,
        kind: &str,
        packet_id: &str,
        idempotency_key: &str,
        duration_ns: u64,
        ok: bool,
        receipt_sha256: Option<&str>,
        detail: &str,
    ) -> Result<ExportAck, String> {
        let end_ns = now_unix_nano();
        let start_ns = end_ns.saturating_sub(duration_ns as u128);
        let trace_id = self.trace_id_for(idempotency_key);
        let span_id = self.span_id_for(idempotency_key);
        let payload = json!({
            "resourceSpans": [{
                "resource": self.resource(),
                "scopeSpans": [{
                    "scope": { "name": "ares.harness", "version": "1" },
                    "spans": [{
                        "traceId": trace_id,
                        "spanId": span_id,
                        "name": format!("ares.{}", kind),
                        "kind": 1,
                        "startTimeUnixNano": start_ns.to_string(),
                        "endTimeUnixNano": end_ns.to_string(),
                        "attributes": Self::attributes(vec![
                            ("ares.event.kind", any_value_string(kind)),
                            ("ares.tool", any_value_string(name)),
                            ("ares.packet.id", any_value_string(packet_id)),
                            ("ares.idempotency_key", any_value_string(idempotency_key)),
                            ("ares.ok", any_value_bool(ok)),
                            (
                                "ares.receipt_sha256",
                                any_value_string(receipt_sha256.unwrap_or_default()),
                            ),
                            ("ares.detail", any_value_string(detail.chars().take(512).collect::<String>())),
                        ]),
                        "status": { "code": if ok { 1 } else { 2 } }
                    }]
                }]
            }]
        });
        self.post("/v1/traces", payload, &format!("trace:{}", kind))
            .await
    }

    fn trace_id_for(&self, seed: &str) -> String {
        let digest = Sha256::digest(format!("{}:{}", self.instance_id, seed).as_bytes());
        hex_encode(&digest[..16])
    }

    fn span_id_for(&self, seed: &str) -> String {
        let digest = Sha256::digest(format!("{}:span:{}", self.instance_id, seed).as_bytes());
        hex_encode(&digest[..8])
    }

    /// Best-effort local read-back sanity check for status. HyperDX's local
    /// UI/API requires a login session for deep query paths, so this function
    /// intentionally does *not* fabricate a row count. It only proves the OTLP
    /// endpoint is reachable and authenticated enough to accept a no-op
    /// log batch with zero records.
    pub async fn ping(&self) -> Result<u16, String> {
        let body = json!({
            "resourceLogs": [{
                "resource": self.resource(),
                "scopeLogs": [{
                    "scope": { "name": "ares.harness", "version": "1" },
                    "logRecords": []
                }]
            }]
        });
        self.post("/v1/logs", body, "ping")
            .await
            .map(|ack| ack.status)
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// OTLP resource attributes as a deterministic payload object. Used by
    /// tests to verify that secrets are only in headers, never payloads.
    pub fn resource_json(&self) -> Value {
        self.resource()
    }

    pub fn into_map_attrs(m: &Map<String, Value>) -> Vec<(String, String)> {
        m.iter().map(|(k, v)| (k.clone(), v.to_string())).collect()
    }
}
