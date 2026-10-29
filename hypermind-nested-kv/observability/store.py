"""Offline SQLite administration. Not a runtime adapter or model transport."""
import argparse
import hashlib
from functools import lru_cache
import json
import os
from pathlib import Path
import sqlite3
import time

HERE = Path(__file__).resolve().parent
DEFAULT_DB = HERE / 'store' / 'traceability.sqlite3'
ZERO = '0' * 64

def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()).hexdigest()

def connect(path=DEFAULT_DB, readonly=False):
    path = Path(path).resolve()
    if readonly:
        db = sqlite3.connect(path.as_uri() + '?mode=ro', uri=True, timeout=5)
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        db = sqlite3.connect(path, timeout=5)
        os.chmod(path, 0o600)
    db.row_factory = sqlite3.Row
    db.execute('PRAGMA foreign_keys=ON')
    db.execute('PRAGMA recursive_triggers=ON')
    db.execute('PRAGMA busy_timeout=5000')
    if not readonly:
        db.execute('PRAGMA journal_mode=WAL')
        db.execute('PRAGMA synchronous=FULL')
    return db

def execute_migration(db, sql):
    statement = ''
    for line in sql.splitlines(True):
        statement += line
        if sqlite3.complete_statement(statement):
            db.execute(statement)
            statement = ''
    if statement.strip():
        raise ValueError('Incomplete migration')

@lru_cache(maxsize=4)
def expected_schema(sql):
    reference = sqlite3.connect(':memory:')
    try:
        execute_migration(reference, sql)
        return list(reference.execute("SELECT type,name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name"))
    finally:
        reference.close()

def initialize(path=DEFAULT_DB):
    db = connect(path)
    sql = (HERE / 'migrations' / '001_initial.sql').read_text()
    checksum = hashlib.sha256(sql.encode()).hexdigest()
    try:
        db.execute('BEGIN IMMEDIATE')
        tables = [r[0] for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")]
        if tables:
            if 'schema_migrations' not in tables:
                raise ValueError('Refusing to adopt an unrelated database')
            rows = list(db.execute('SELECT version,checksum FROM schema_migrations'))
            if len(rows) != 1 or tuple(rows[0]) != (1, checksum):
                raise ValueError('Migration version/checksum mismatch')
        else:
            execute_migration(db, sql)
            db.execute('INSERT INTO schema_migrations VALUES(1,?,?)', (checksum, time.time_ns() // 1000))
            db.execute('PRAGMA user_version=1')
        db.commit()
    except BaseException:
        db.rollback()
        db.close()
        raise
    return db

def fact_tables(db):
    return {r[0]: {c[1]: c[2] for c in db.execute(f'PRAGMA table_info("{r[0]}")')}
            for r in db.execute("SELECT name FROM sqlite_master WHERE type='table'")
            if any(c[1] == 'event_seq' for c in db.execute(f'PRAGMA table_info("{r[0]}")'))}

def normalize_facts(db, facts):
    schemas = fact_tables(db)
    output = []
    seen = set()
    for fact in facts:
        if set(fact) != {'table', 'values'} or fact['table'] not in schemas:
            raise ValueError('Unknown fact schema')
        name, values = fact['table'], fact['values']
        schema = schemas[name]
        if set(values) - (set(schema) - {'event_seq'}):
            raise ValueError('Unknown or forbidden fact field')
        row = {k: values.get(k) for k in schema if k != 'event_seq'}
        for key, value in row.items():
            if value is None:
                continue
            expected = schema[key]
            if expected == 'TEXT' and not isinstance(value, str):
                raise ValueError('Expected metadata string')
            if expected == 'INTEGER' and (type(value) is not int):
                raise ValueError('Expected integer')
            if expected == 'REAL':
                if type(value) not in (int, float):
                    raise ValueError('Expected numeric value')
                row[key] = float(value)
        identity = (name, row['id'])
        if identity in seen:
            raise ValueError('Duplicate fact identity')
        seen.add(identity)
        output.append({'table': name, 'values': row})
    return sorted(output, key=lambda x: (x['table'], x['values']['id']))

HEADER_FIELDS = {'id', 'run_id', 'span_id', 'actor_id', 'producer_id', 'producer_seq',
                 'occurred_at_us', 'boot_id', 'monotonic_ns', 'event_type', 'supersedes_seq'}

def append(db, header, facts=()):
    """Atomic, idempotent event with typed facts. Caller must own outer-shell role."""
    if set(header) - HEADER_FIELDS:
        raise ValueError('Unknown event field; raw payloads forbidden')
    required = HEADER_FIELDS - {'span_id', 'actor_id', 'supersedes_seq'}
    if required - set(header):
        raise ValueError('Missing event fields')
    header = {k: header.get(k) for k in HEADER_FIELDS}
    normalized = normalize_facts(db, facts)
    body_hash = digest(normalized)
    db.execute('BEGIN IMMEDIATE')
    try:
        existing = db.execute('SELECT * FROM events WHERE producer_id=? AND producer_seq=?',
                              (header['producer_id'], header['producer_seq'])).fetchone()
        if existing:
            if any(existing[k] != v for k, v in header.items()) or existing['body_hash'] != body_hash:
                raise ValueError('Replay identity conflicts with committed content')
            db.rollback()
            return existing['seq']
        previous = db.execute('SELECT seq,event_hash,ingested_at_us FROM events ORDER BY seq DESC LIMIT 1').fetchone()
        event = dict(header, seq=previous['seq'] + 1 if previous else 1,
                     ingested_at_us=max(time.time_ns() // 1000, previous['ingested_at_us'] if previous else 0),
                     body_hash=body_hash, previous_hash=previous['event_hash'] if previous else ZERO)
        event['event_hash'] = digest(event)
        fields = list(event)
        db.execute(f"INSERT INTO events({','.join(fields)}) VALUES({','.join('?' for _ in fields)})",
                   [event[k] for k in fields])
        for fact in normalized:
            values = dict(fact['values'], event_seq=event['seq'])
            fields = list(values)
            db.execute(f"INSERT INTO {fact['table']}({','.join(fields)}) VALUES({','.join('?' for _ in fields)})",
                       [values[k] for k in fields])
        if event['seq'] % 1000 == 0:
            first = event['seq'] - 999
            hashes = [r[0] for r in db.execute('SELECT event_hash FROM events WHERE seq BETWEEN ? AND ? ORDER BY seq', (first, event['seq']))]
            db.execute('INSERT INTO ledger_roots VALUES(?,?,?,?,?)',
                       (f'root-{event["seq"]}', first, event['seq'], merkle(hashes), 'sha256_binary_duplicate_last'))
        db.commit()
        return event['seq']
    except BaseException:
        db.rollback()
        raise

def merkle(hashes):
    nodes = [bytes.fromhex(h) for h in hashes]
    if not nodes:
        return hashlib.sha256(b'').hexdigest()
    while len(nodes) > 1:
        if len(nodes) % 2:
            nodes.append(nodes[-1])
        nodes = [hashlib.sha256(nodes[i] + nodes[i + 1]).digest() for i in range(0, len(nodes), 2)]
    return nodes[0].hex()

def verify(db):
    """Consistent read snapshot; typed fact hashes, ordering, FKs and Merkle blocks."""
    db.execute('BEGIN')
    try:
        sql = (HERE / 'migrations' / '001_initial.sql').read_text()
        migrations = [tuple(r) for r in db.execute('SELECT version,checksum FROM schema_migrations')]
        if migrations != [(1, hashlib.sha256(sql.encode()).hexdigest())] or db.execute('PRAGMA user_version').fetchone()[0] != 1:
            raise ValueError('Migration verification failed')
        actual = [tuple(r) for r in db.execute("SELECT type,name,sql FROM sqlite_master WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name")]
        if actual != expected_schema(sql):
            raise ValueError('Schema integrity failed')
        if db.execute('PRAGMA integrity_check').fetchone()[0] != 'ok':
            raise ValueError('SQLite integrity failed')
        if list(db.execute('PRAGMA foreign_key_check')):
            raise ValueError('Foreign key check failed')
        tables = fact_tables(db)
        previous_hash, previous_time, count = ZERO, 0, 0
        for row in db.execute('SELECT * FROM events ORDER BY seq'):
            event = dict(row)
            count += 1
            if event['seq'] != count or event['previous_hash'] != previous_hash or event['ingested_at_us'] < previous_time:
                raise ValueError('Event ordering or hash chain failed')
            facts = []
            for name in tables:
                for fact in db.execute(f'SELECT * FROM {name} WHERE event_seq=? ORDER BY id', (count,)):
                    values = dict(fact)
                    values.pop('event_seq')
                    facts.append({'table': name, 'values': values})
            facts.sort(key=lambda x: (x['table'], x['values']['id']))
            if digest(facts) != event['body_hash']:
                raise ValueError('Typed fact integrity failed')
            event_hash = event.pop('event_hash')
            if digest(event) != event_hash:
                raise ValueError('Event integrity failed')
            previous_hash, previous_time = event_hash, event['ingested_at_us']
        roots = list(db.execute('SELECT * FROM ledger_roots ORDER BY last_seq'))
        if len(roots) != count // 1000:
            raise ValueError('Missing or unexpected Merkle checkpoints')
        for i, root in enumerate(roots):
            first, last = i * 1000 + 1, (i + 1) * 1000
            hashes = [r[0] for r in db.execute('SELECT event_hash FROM events WHERE seq BETWEEN ? AND ? ORDER BY seq', (first, last))]
            if (root['first_seq'], root['last_seq']) != (first, last) or merkle(hashes) != root['root_hash']:
                raise ValueError('Merkle verification failed')
        return {'events': count, 'merkle_roots': len(roots), 'last_event_hash': previous_hash, 'integrity': 'ok'}
    finally:
        db.rollback()

def backup(db, destination):
    path = Path(destination)
    if path.exists():
        raise ValueError('Backup destination already exists')
    # Exclusive creation prevents overwriting a raced-in file.
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
    os.close(fd)
    target = sqlite3.connect(path)
    try:
        db.backup(target)
    finally:
        target.close()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['init', 'verify', 'backup'])
    parser.add_argument('--db', type=Path, default=DEFAULT_DB)
    parser.add_argument('--destination', type=Path)
    args = parser.parse_args()
    if args.command == 'init':
        db = initialize(args.db)
        print(json.dumps({'database': str(args.db.resolve()), 'schema_version': 1, **verify(db)}))
    else:
        db = connect(args.db, readonly=True)
        if args.command == 'verify':
            print(json.dumps(verify(db)))
        else:
            if args.destination is None:
                parser.error('--destination required')
            backup(db, args.destination)
            print(json.dumps({'backup': str(args.destination.resolve())}))
    db.close()

if __name__ == '__main__':
    main()
