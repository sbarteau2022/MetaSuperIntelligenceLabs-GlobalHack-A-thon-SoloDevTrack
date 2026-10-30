// sqlite_sink — durable SQLite telemetry sink for harness events.
//
// Stores tool-call and verdict events in a local SQLite DB with a
// sha256 hash chain (prev_hash -> event_hash, GENESIS origin), mirroring
// the in-memory `SessionLog` chain in session.rs but crash-durable.
// Reopening the same path resumes the chain gaplessly; replay is a pure
// read path. Writes carry caller-supplied idempotency keys (UNIQUE):
// recording the same key twice returns the original seq, never a duplicate.
//
// Doctrine: no serde_json anywhere in this crate. Payload columns are
// stored as opaque TEXT and hashed as `|`-joined bytes; nothing is
// parsed. rusqlite is used with `bundled` only (no serde_json feature).

use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// One durable event. Native struct; never serialized to JSON.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkEvent {
    pub seq: i64,
    pub kind: String, // "tool_call" | "verdict"
    pub tool: String,
    pub args_json: String, // opaque text, never parsed
    pub packet_id: String,
    pub outcome: String,
    pub judges: String,
    pub dt_ns: i64,
    pub ok: bool,
    pub receipt_sha256: Option<String>,
    pub prev_hash: String,
    pub event_hash: String,
}

/// Append-only SQLite sink with a verified hash chain.
pub struct TelemetrySink {
    conn: Connection,
    #[allow(dead_code)]
    path: PathBuf,
    next_seq: i64,
    prev_hash: String,
}

impl TelemetrySink {
    /// Open (or create) the DB at `path`, creating parent dirs. An existing
    /// DB resumes: the chain continues from its last row, gaplessly.
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {}", parent.display(), e))?;
        }
        let conn = Connection::open(path).map_err(|e| format!("open {}: {}", path.display(), e))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                seq INTEGER PRIMARY KEY,
                kind TEXT NOT NULL,
                tool TEXT NOT NULL DEFAULT '',
                args_json TEXT NOT NULL DEFAULT '',
                packet_id TEXT NOT NULL DEFAULT '',
                outcome TEXT NOT NULL DEFAULT '',
                judges TEXT NOT NULL DEFAULT '',
                dt_ns INTEGER NOT NULL DEFAULT 0,
                ok INTEGER NOT NULL DEFAULT 0,
                receipt_sha256 TEXT,
                idempotency_key TEXT NOT NULL DEFAULT '',
                prev_hash TEXT NOT NULL,
                event_hash TEXT NOT NULL
            );
            CREATE UNIQUE INDEX IF NOT EXISTS idx_events_idem
                ON events(idempotency_key);",
        )
        .map_err(|e| format!("migrate: {}", e))?;
        // Resume: next seq continues past the max row; prev links to the
        // last hash. Aggregate without GROUP BY always yields exactly one
        // row (COALESCE covers the empty table); the last-hash lookup may
        // yield zero rows, which means GENESIS.
        let next_seq: i64 = conn
            .query_row("SELECT COALESCE(MAX(seq), -1) + 1 FROM events", [], |r| {
                r.get(0)
            })
            .map_err(|e| format!("resume seq: {}", e))?;
        let prev_hash: String = conn
            .query_row(
                "SELECT event_hash FROM events ORDER BY seq DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "GENESIS".to_string());
        Ok(Self {
            conn,
            path: path.to_path_buf(),
            next_seq,
            prev_hash,
        })
    }

    /// Record one MCP tool call. Idempotent on `idempotency_key`.
    /// Returns the event seq.
    #[allow(clippy::too_many_arguments)]
    pub fn record_call(
        &mut self,
        idempotency_key: &str,
        tool: &str,
        args_json: &str,
        dt_ns: u64,
        ok: bool,
        receipt_sha256: Option<&str>,
    ) -> Result<i64, String> {
        let payload = format!(
            "{}|{}|{}|{}|{}",
            tool,
            args_json,
            dt_ns,
            ok as u8,
            receipt_sha256.unwrap_or("")
        );
        self.insert(
            idempotency_key,
            "tool_call",
            &payload,
            tool,
            args_json,
            "",
            "",
            "",
            dt_ns as i64,
            ok,
            receipt_sha256,
        )
    }

    /// Record one packet verdict. Idempotent on `idempotency_key`.
    /// Returns the event seq.
    pub fn record_verdict(
        &mut self,
        idempotency_key: &str,
        packet_id: &str,
        outcome: &str,
        judges: &str,
    ) -> Result<i64, String> {
        let payload = format!("{}|{}|{}", packet_id, outcome, judges);
        self.insert(
            idempotency_key,
            "verdict",
            &payload,
            "",
            "",
            packet_id,
            outcome,
            judges,
            0,
            outcome == "PASS",
            None,
        )
    }

    /// Verify the full hash chain. Ok(count) when intact; Err names the
    /// first broken seq (gap, relink, or recompute mismatch).
    pub fn verify(&self) -> Result<u64, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT seq, kind, tool, args_json, packet_id, outcome, judges,
                        dt_ns, ok, receipt_sha256, prev_hash, event_hash
                 FROM events ORDER BY seq",
            )
            .map_err(|e| format!("verify prepare: {}", e))?;
        let mut prev = "GENESIS".to_string();
        let mut count: u64 = 0;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, i64>(7)?,
                    r.get::<_, i64>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, String>(10)?,
                    r.get::<_, String>(11)?,
                ))
            })
            .map_err(|e| format!("verify scan: {}", e))?;
        for (expected_seq, row) in rows.enumerate() {
            let expected_seq = expected_seq as i64;
            let (seq, kind, tool, args, packet, outcome, judges, dt, ok, receipt, ph, eh) =
                row.map_err(|e| format!("verify row: {}", e))?;
            if seq != expected_seq || ph != prev {
                return Err(format!("chain broken at seq={}", seq));
            }
            let payload = match kind.as_str() {
                "tool_call" => format!(
                    "{}|{}|{}|{}|{}",
                    tool,
                    args,
                    dt as u64,
                    ok,
                    receipt.as_deref().unwrap_or("")
                ),
                _ => format!("{}|{}|{}", packet, outcome, judges),
            };
            if hash_event(seq, &kind, &payload, &ph) != eh {
                return Err(format!("chain broken at seq={}", seq));
            }
            prev = eh;
            count += 1;
        }
        Ok(count)
    }

    /// Pure read path: events with seq >= `from_seq`, in order. Reading
    /// never mutates the DB, so replay is side-effect free and deterministic.
    pub fn replay_from(&self, from_seq: i64) -> Result<Vec<SinkEvent>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT seq, kind, tool, args_json, packet_id, outcome, judges,
                        dt_ns, ok, receipt_sha256, prev_hash, event_hash
                 FROM events WHERE seq >= ?1 ORDER BY seq",
            )
            .map_err(|e| format!("replay prepare: {}", e))?;
        let rows = stmt
            .query_map(params![from_seq], |r| {
                Ok(SinkEvent {
                    seq: r.get(0)?,
                    kind: r.get(1)?,
                    tool: r.get(2)?,
                    args_json: r.get(3)?,
                    packet_id: r.get(4)?,
                    outcome: r.get(5)?,
                    judges: r.get(6)?,
                    dt_ns: r.get(7)?,
                    ok: r.get::<_, i64>(8)? != 0,
                    receipt_sha256: r.get(9)?,
                    prev_hash: r.get(10)?,
                    event_hash: r.get(11)?,
                })
            })
            .map_err(|e| format!("replay scan: {}", e))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("replay row: {}", e))
    }

    /// True when no events are stored.
    pub fn is_empty(&self) -> Result<bool, String> {
        Ok(self.len()? == 0)
    }

    /// Number of events currently stored.
    pub fn len(&self) -> Result<u64, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM events", [], |r| r.get(0))
            .map_err(|e| format!("count: {}", e))
    }

    /// Flush and release the DB handle.
    pub fn close(self) -> Result<(), String> {
        self.conn.close().map_err(|(_, e)| e.to_string())?;
        Ok(())
    }

    /// Insert-or-return-existing under the idempotency key.
    #[allow(clippy::too_many_arguments)]
    fn insert(
        &mut self,
        idempotency_key: &str,
        kind: &str,
        payload: &str,
        tool: &str,
        args_json: &str,
        packet_id: &str,
        outcome: &str,
        judges: &str,
        dt_ns: i64,
        ok: bool,
        receipt_sha256: Option<&str>,
    ) -> Result<i64, String> {
        if !idempotency_key.is_empty() {
            // query_row errors on zero rows; map that to None explicitly.
            let mut stmt = self
                .conn
                .prepare("SELECT seq FROM events WHERE idempotency_key = ?1")
                .map_err(|e| format!("idempotency prepare: {}", e))?;
            let mut rows = stmt
                .query_map(params![idempotency_key], |r| r.get::<_, i64>(0))
                .map_err(|e| format!("idempotency lookup: {}", e))?;
            if let Some(row) = rows.next() {
                let seq = row.map_err(|e| format!("idempotency row: {}", e))?;
                return Ok(seq);
            }
        }
        let seq = self.next_seq;
        let event_hash = hash_event(seq, kind, payload, &self.prev_hash);
        self.conn
            .execute(
                "INSERT INTO events
                 (seq, kind, tool, args_json, packet_id, outcome, judges,
                  dt_ns, ok, receipt_sha256, idempotency_key, prev_hash, event_hash)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
                params![
                    seq,
                    kind,
                    tool,
                    args_json,
                    packet_id,
                    outcome,
                    judges,
                    dt_ns,
                    ok as i64,
                    receipt_sha256,
                    idempotency_key,
                    self.prev_hash,
                    event_hash,
                ],
            )
            .map_err(|e| format!("insert: {}", e))?;
        self.next_seq += 1;
        self.prev_hash = event_hash;
        Ok(seq)
    }
}

fn hash_event(seq: i64, kind: &str, payload: &str, prev: &str) -> String {
    let mut h = Sha256::new();
    h.update(seq.to_le_bytes());
    h.update(kind.as_bytes());
    h.update(payload.as_bytes());
    h.update(prev.as_bytes());
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static CTR: AtomicU64 = AtomicU64::new(0);

    /// Hermetic DB path: fresh unique dir under the OS temp dir on every
    /// call. Never inside the repo worktree (asserted below).
    fn test_db(name: &str) -> PathBuf {
        let n = CTR.fetch_add(1, Ordering::SeqCst);
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rgah-sqlite-sink-{}-{}-{}",
            std::process::id(),
            n,
            name
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir.join("sink.db")
    }

    #[test]
    fn open_creates_empty_db_outside_worktree() {
        let db = test_db("empty");
        // 1. hermetic: DB lives under temp_dir, not the repo worktree.
        assert!(db.starts_with(std::env::temp_dir()));
        let here = std::env::current_dir().expect("cwd");
        assert!(!db.starts_with(here), "DB must never land in worktree");
        let sink = TelemetrySink::open(&db).expect("open");
        // 2. file materialized by open.
        assert!(db.is_file());
        // 3. empty chain verifies with count 0.
        assert_eq!(sink.verify().expect("verify"), 0);
        // 4. replay of empty DB is empty, not an error.
        assert!(sink.replay_from(0).expect("replay").is_empty());
        sink.close().expect("close");
        // 5. file persists after close (durable, not in-memory).
        assert!(db.is_file());
    }

    #[test]
    fn record_chain_verifies_and_replays_in_order() {
        let db = test_db("chain");
        let mut sink = TelemetrySink::open(&db).expect("open");
        let s0 = sink
            .record_call(
                "k-ingest-1",
                "map.ingest",
                "{\"text\":\"hi\"}",
                4242,
                true,
                Some("abc123"),
            )
            .expect("call1");
        let s1 = sink
            .record_call("k-query-1", "map.query", "{\"q\":\"x\"}", 900, true, None)
            .expect("call2");
        let s2 = sink
            .record_verdict("k-verdict-1", "packet-01", "PASS", "judge-pass")
            .expect("verdict");
        // 1. seqs are gapless from 0.
        assert_eq!((s0, s1, s2), (0, 1, 2));
        // 2. full chain verifies with count 3.
        assert_eq!(sink.verify().expect("verify"), 3);
        // 3. replay returns all three in seq order with kinds intact.
        let evs = sink.replay_from(0).expect("replay");
        assert_eq!(evs.len(), 3);
        assert_eq!(
            evs.iter().map(|e| e.kind.clone()).collect::<Vec<_>>(),
            vec!["tool_call", "tool_call", "verdict"]
        );
        // 4. payloads round-trip byte-exact (opaque text, never parsed).
        assert_eq!(evs[0].args_json, "{\"text\":\"hi\"}");
        assert_eq!(evs[0].receipt_sha256.as_deref(), Some("abc123"));
        assert_eq!(evs[2].packet_id, "packet-01");
        // 5. suffix replay slices deterministically.
        let tail = sink.replay_from(2).expect("replay tail");
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].seq, 2);
        sink.close().expect("close");
    }

    #[test]
    fn reopen_continues_chain_gaplessly() {
        let db = test_db("reopen");
        let mut sink = TelemetrySink::open(&db).expect("open");
        sink.record_call("k-1", "map.ingest", "{}", 1, true, Some("r1"))
            .expect("c1");
        sink.record_verdict("k-2", "packet-01", "PASS", "j")
            .expect("v1");
        sink.close().expect("close");
        // Reopen the same file: chain must resume, not restart.
        let mut sink2 = TelemetrySink::open(&db).expect("reopen");
        // 1. pre-existing rows still verify.
        assert_eq!(sink2.verify().expect("verify"), 2);
        let s = sink2
            .record_verdict("k-3", "packet-02", "FAIL", "judge-x")
            .expect("v2");
        // 2. next seq continues gaplessly.
        assert_eq!(s, 2);
        // 3. extended chain (old + new rows) verifies as one chain.
        assert_eq!(sink2.verify().expect("verify2"), 3);
        // 4. linkage spans the reopen boundary.
        let evs = sink2.replay_from(0).expect("replay");
        assert_eq!(evs[2].prev_hash, evs[1].event_hash);
        sink2.close().expect("close");
    }

    #[test]
    fn idempotent_rewrite_returns_original_seq() {
        let db = test_db("idem");
        let mut sink = TelemetrySink::open(&db).expect("open");
        let a = sink
            .record_call("same-key", "map.query", "{\"q\":\"a\"}", 5, true, None)
            .expect("first");
        // Same key, DIFFERENT payload: must not append, returns original seq.
        let b = sink
            .record_call(
                "same-key",
                "map.query",
                "{\"q\":\"CHANGED\"}",
                6,
                false,
                Some("zzz"),
            )
            .expect("second");
        // 1. same seq returned.
        assert_eq!(a, b);
        // 2. row count unchanged (no duplicate row).
        assert_eq!(sink.len().expect("len"), 1);
        // 3. original payload preserved, not overwritten.
        let evs = sink.replay_from(0).expect("replay");
        assert_eq!(evs[0].args_json, "{\"q\":\"a\"}");
        // 4. chain still verifies.
        assert_eq!(sink.verify().expect("verify"), 1);
        // 5. verdict keys are idempotent too.
        let v1 = sink
            .record_verdict("vk", "packet-09", "PASS", "j")
            .expect("v1");
        let v2 = sink
            .record_verdict("vk", "packet-09", "PASS", "j")
            .expect("v2");
        assert_eq!(v1, v2);
        assert_eq!(sink.len().expect("len2"), 2);
        sink.close().expect("close");
    }

    #[test]
    fn tamper_is_detected_at_exact_seq() {
        let db = test_db("tamper");
        let mut sink = TelemetrySink::open(&db).expect("open");
        sink.record_call("k-1", "map.ingest", "{}", 1, true, Some("r1"))
            .expect("c1");
        sink.record_call("k-2", "map.query", "{}", 2, true, Some("r2"))
            .expect("c2");
        sink.record_verdict("k-3", "packet-01", "PASS", "j")
            .expect("v1");
        assert_eq!(sink.verify().expect("pre-tamper verify"), 3);
        sink.close().expect("close");
        // Attacker flips one column directly in the DB file.
        {
            let conn = Connection::open(&db).expect("attacker open");
            conn.execute("UPDATE events SET tool = 'map.EVIL' WHERE seq = 1", [])
                .expect("tamper");
        }
        let sink2 = TelemetrySink::open(&db).expect("reopen");
        // 1. verify fails ...
        let err = sink2.verify().expect_err("tamper must break verify");
        // 2. ... and names the exact broken seq.
        assert!(err.contains("seq=1"), "error must name seq=1, got: {}", err);
        // 3. untouched prefix still replays (detection, not destruction).
        let evs = sink2.replay_from(0).expect("replay");
        assert_eq!(evs.len(), 3);
        // 4. gap attack also detected: delete the middle row.
        sink2.close().expect("close2");
        {
            let conn = Connection::open(&db).expect("attacker open2");
            conn.execute("DELETE FROM events WHERE seq = 1", [])
                .expect("gap");
        }
        let sink3 = TelemetrySink::open(&db).expect("reopen2");
        let gerr = sink3.verify().expect_err("gap must break verify");
        assert!(
            gerr.contains("seq=2"),
            "gap surfaces at seq=2, got: {}",
            gerr
        );
        sink3.close().expect("close3");
    }

    #[test]
    fn replay_is_deterministic_and_side_effect_free() {
        let db = test_db("replay");
        let mut sink = TelemetrySink::open(&db).expect("open");
        for i in 0..5 {
            sink.record_verdict(
                &format!("k-{}", i),
                &format!("packet-{:02}", i),
                "PASS",
                "judge-pass",
            )
            .expect("verdict");
        }
        // 1. two replays return identical vectors.
        let r1 = sink.replay_from(0).expect("r1");
        let r2 = sink.replay_from(0).expect("r2");
        assert_eq!(r1, r2);
        assert_eq!(r1.len(), 5);
        // 2. replay does not mutate: count + verify unchanged after reads.
        assert_eq!(sink.len().expect("len"), 5);
        assert_eq!(sink.verify().expect("verify"), 5);
        // 3. out-of-range replay is empty, not an error.
        assert!(sink.replay_from(99).expect("r99").is_empty());
        sink.close().expect("close");
    }
}
