"""Reproduce database-only validation and persist bounded, honest evidence."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import sqlite3
import subprocess
import sys
import time

import store

HERE = Path(__file__).resolve().parent

def main():
    sql_path = HERE / 'migrations' / '001_initial.sql'
    before = sql_path.read_bytes()
    generation = subprocess.run([sys.executable, str(HERE/'build_schema.py')], capture_output=True, text=True)
    deterministic = generation.returncode == 0 and before == sql_path.read_bytes()
    started = time.monotonic()
    result = subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s', str(HERE/'tests'), '-v'], capture_output=True, text=True)
    elapsed = time.monotonic()-started
    system_sqlite = subprocess.run(['/usr/bin/sqlite3', ':memory:'], input=before.decode()+"\nSELECT COUNT(*) FROM sqlite_master WHERE type='table';\n", capture_output=True, text=True)
    db = store.connect(readonly=True)
    try:
        integrity = store.verify(db)
        counts = {kind: db.execute('SELECT COUNT(*) FROM sqlite_master WHERE type=?',(kind,)).fetchone()[0] for kind in ('table','index','trigger','view')}
    finally:
        db.close()
    passed = deterministic and result.returncode == 0 and system_sqlite.returncode == 0 and system_sqlite.stdout.strip() == str(counts['table'])
    evidence = {
        'recorded_at_utc':datetime.now(timezone.utc).isoformat(),
        'status':'PASS' if passed else 'FAIL',
        'scope':'SQLite foundation only; no live model/harness integration or full-system performance validation',
        'python':sys.version.split()[0], 'sqlite_python':sqlite3.sqlite_version,
        'platform':platform.platform(), 'schema_sha256':hashlib.sha256(before).hexdigest(),
        'deterministic_generation':deterministic, 'system_sqlite_migration_pass':system_sqlite.returncode == 0,
        'schema_objects':counts, 'suite_elapsed_seconds':round(elapsed,3),
        'suite_returncode':result.returncode, 'suite_output':result.stdout+result.stderr,
        'pressure_scope':'4 writers x 260 metadata events; includes one verified 1000-event Merkle block',
        'local_database':str(store.DEFAULT_DB), 'local_integrity':integrity,
        'independent_review':'Pending; adversarial self-review and regression tests completed',
    }
    (HERE/'VALIDATION.json').write_text(json.dumps(evidence,indent=2)+'\n')
    print(json.dumps({k:evidence[k] for k in ('status','schema_objects','suite_elapsed_seconds','local_integrity')}))
    if not passed:
        print(result.stdout+result.stderr+system_sqlite.stderr)
        raise SystemExit(1)

if __name__=='__main__': main()
