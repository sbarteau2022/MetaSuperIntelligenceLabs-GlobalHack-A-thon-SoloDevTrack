import concurrent.futures
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
import store

H = 'a' * 64
J = 'b' * 64

class StoreTests(unittest.TestCase):
    def setUp(self):
        # Scratch remains inside the canonical repo per AGENTS.md.
        self.tmp = tempfile.TemporaryDirectory(dir=ROOT / 'store')
        self.path = Path(self.tmp.name) / 'test.sqlite3'
        self.db = store.initialize(self.path)
        for sql in [
            "INSERT INTO projects VALUES('p',?)",
            "INSERT INTO artifacts VALUES('artifact',?,0,'metadata','available')",
            "INSERT INTO config_snapshots VALUES('cfg','artifact',?)",
            "INSERT INTO sessions VALUES('session','p',?,'cfg')",
        ]:
            self.db.execute(sql, (H,))
        self.db.execute("INSERT INTO tasks VALUES('task','session','artifact','artifact')")
        self.db.execute("INSERT INTO runs VALUES('run','task',NULL,'cfg','trace')")
        for actor, kind in [('meta','meta'), ('agent','agent'), ('swarm','swarm')]:
            self.db.execute('INSERT INTO actors VALUES(?,?,?,?)', (actor,'p',kind,H))
        self.db.execute("INSERT INTO spans VALUES('span','run',NULL,'meta','meta')")
        self.db.execute("INSERT INTO capabilities VALUES('tool','tool',?,?,NULL)", (H,H))
        self.db.execute("INSERT INTO invocations VALUES('invoke','span','tool',?,3)", (H,))
        self.db.execute("INSERT INTO attempts VALUES('attempt','invoke',1,?)", (H,))
        self.db.commit()
        self.n = 0

    def tearDown(self):
        self.db.close()
        self.tmp.cleanup()

    def header(self, **changes):
        self.n += 1
        value = dict(id=f'event-{self.n}', run_id='run', span_id='span', actor_id='meta',
                     producer_id='test', producer_seq=self.n, occurred_at_us=10,
                     boot_id='boot', monotonic_ns=self.n, event_type='check')
        value.update(changes)
        return value

    def fact(self, table, **values):
        return {'table':table, 'values':values}

    def put(self, table, **values):
        return store.append(self.db, self.header(), [self.fact(table, **values)])

    def check(self, result='pass', evidence='artifact', id='check'):
        self.put('acceptance_checks',id=id,attempt_id='attempt',criteria_id='artifact',result=result,
                 evidence_id=evidence,checker_version_hash=H)

    def decision(self, **overrides):
        values = dict(id='decision',meta_actor_id='meta',decision='activate',justification_id='artifact',
                      estimated_cost_microusd=20,budget_microusd=100,max_agents=1,expires_at_us=100,
                      approval='approved',policy_id='artifact')
        values.update(overrides)
        self.put('meta_decisions',**values)

    def lease(self, **overrides):
        values = dict(id='lease',decision_id='decision',agent_id='agent',swarm_id='swarm',
                      reserved_cost_microusd=50,expires_at_us=90)
        values.update(overrides)
        self.put('agent_leases',**values)

    def graphs(self, kind='dag'):
        self.db.execute('INSERT INTO graph_versions VALUES(?,?,?,?,NULL)',('g','run',kind,H))
        for node in 'abc':
            self.db.execute('INSERT INTO graph_nodes VALUES(?,?,?,?,NULL)',(node,'g','task',H))
        self.db.commit()

    def edge(self, a, b):
        self.put('graph_edges',id=f'{a}-{b}',graph_id='g',source_id=a,target_id=b,relation='depends_on')

    def cache(self):
        self.db.execute("INSERT INTO caches VALUES('kv','meta',?,'transformer_kv',100,20)",(H,))
        self.db.commit()

    def test_initialize_idempotent(self):
        other=store.initialize(self.path)
        self.assertEqual(other.execute('PRAGMA user_version').fetchone()[0],1)
        other.close()

    def test_pragmas(self):
        for key, value in [('foreign_keys',1),('journal_mode','wal'),('synchronous',2)]:
            self.assertEqual(self.db.execute(f'PRAGMA {key}').fetchone()[0],value)

    def test_unrelated_database_rejected(self):
        path=Path(self.tmp.name)/'other.db'
        db=sqlite3.connect(path)
        db.execute('CREATE TABLE unrelated(x)')
        db.close()
        with self.assertRaises(ValueError): store.initialize(path)

    def test_drift_rejected(self):
        self.db.execute('DROP TRIGGER schema_migrations_no_update')
        self.db.execute("UPDATE schema_migrations SET checksum='wrong'")
        self.db.commit()
        with self.assertRaises(ValueError): store.initialize(self.path)

    def test_empty_verify(self):
        self.assertEqual(store.verify(self.db)['events'],0)

    def test_append_verify(self):
        self.check()
        self.assertEqual(store.verify(self.db)['events'],1)

    def test_replay_idempotent(self):
        h=self.header()
        self.assertEqual(store.append(self.db,h),store.append(self.db,h))
        self.assertEqual(store.verify(self.db)['events'],1)

    def test_conflicting_replay_rejected(self):
        h=self.header()
        store.append(self.db,h)
        h['event_type']='security'
        with self.assertRaises(ValueError): store.append(self.db,h)

    def test_unknown_payload_rejected(self):
        with self.assertRaises(ValueError): store.append(self.db,self.header(payload='secret'))
        with self.assertRaises(ValueError): self.put('acceptance_checks',id='x',result='pass',raw_args='secret')

    def test_invalid_table_rejected(self):
        with self.assertRaises(ValueError): self.put('events; DROP TABLE events',id='x')

    def test_nan_rejected(self):
        self.graphs()
        with self.assertRaises(ValueError):
            self.put('retrieval_queries',id='q',query_hash=H,method_version_hash=H,method='lexical',
                     requested_k=1,returned_count=1,duration_ns=1,mode='primary',participation_ratio=float('nan'))

    def test_wrong_numeric_type_rejected(self):
        with self.assertRaises(ValueError): self.put('resource_samples',id='r',value='2')

    def test_missing_fk_rolls_back_event(self):
        with self.assertRaises(sqlite3.IntegrityError): self.check(evidence='missing')
        self.assertEqual(store.verify(self.db)['events'],0)

    def test_invalid_status(self):
        with self.assertRaises(sqlite3.IntegrityError): self.check(result='success')

    def test_false_pass_without_evidence(self):
        with self.assertRaises(sqlite3.IntegrityError): self.check(evidence=None)

    def test_false_objective_pass(self):
        self.check(result='fail')
        with self.assertRaises(sqlite3.IntegrityError): self.put('objective_outcomes',id='out',check_id='check',result='pass')

    def test_valid_objective_pass(self):
        self.check()
        self.put('objective_outcomes',id='out',check_id='check',result='pass')
        self.assertEqual(store.verify(self.db)['events'],2)

    def test_all_tables_append_only(self):
        for row in self.db.execute("SELECT name FROM sqlite_master WHERE type='table'"):
            name=row[0]
            triggers=[r[0] for r in self.db.execute("SELECT name FROM sqlite_master WHERE type='trigger' AND tbl_name=?",(name,))]
            self.assertIn(name+'_no_update',triggers)
            self.assertIn(name+'_no_delete',triggers)
            self.assertIn(name+'_no_replace',triggers)
        self.check()
        for sql in ["UPDATE acceptance_checks SET result='fail'",'DELETE FROM events',"INSERT OR REPLACE INTO projects VALUES('p',?)"]:
            with self.assertRaises(sqlite3.IntegrityError): self.db.execute(sql,(J,) if '?' in sql else ())
        self.db.rollback()

    def test_event_hash_tampering_detected(self):
        store.append(self.db,self.header())
        self.db.execute('DROP TRIGGER events_no_update')
        self.db.execute('UPDATE events SET event_hash=?',(J,))
        self.db.commit()
        with self.assertRaises(ValueError): store.verify(self.db)

    def test_fact_tampering_detected(self):
        self.check()
        self.db.execute('DROP TRIGGER acceptance_checks_no_update')
        self.db.execute("UPDATE acceptance_checks SET result='fail'")
        self.db.commit()
        with self.assertRaises(ValueError): store.verify(self.db)

    def test_late_occurrence_allowed(self):
        store.append(self.db,self.header(occurred_at_us=20))
        store.append(self.db,self.header(occurred_at_us=1,boot_id='new-boot',monotonic_ns=0))
        self.assertEqual(store.verify(self.db)['events'],2)

    def test_cross_run_span_rejected(self):
        self.db.execute("INSERT INTO runs VALUES('run2','task',NULL,'cfg','trace2')")
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError): store.append(self.db,self.header(run_id='run2'))

    def test_retry_requires_material_change(self):
        self.db.execute("INSERT INTO attempts VALUES('retry','invoke',2,?)",(H,))
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('recovery_edges',id='r',from_attempt_id='attempt',to_attempt_id='retry',kind='retry',reason_code_hash=H)

    def test_retry_limit(self):
        with self.assertRaises(sqlite3.IntegrityError): self.db.execute("INSERT INTO attempts VALUES('retry','invoke',4,?)",(J,))
        self.db.rollback()

    def test_approved_lease(self):
        self.decision()
        self.lease()
        self.assertEqual(store.verify(self.db)['events'],2)

    def test_denied_lease(self):
        self.decision(approval='denied')
        with self.assertRaises(sqlite3.IntegrityError): self.lease()

    def test_budget_overrun(self):
        self.decision()
        with self.assertRaises(sqlite3.IntegrityError): self.lease(reserved_cost_microusd=101)

    def test_lease_expired(self):
        self.decision()
        with self.assertRaises(sqlite3.IntegrityError): self.lease(expires_at_us=9)

    def test_lease_exceeds_decision_expiry(self):
        self.decision()
        with self.assertRaises(sqlite3.IntegrityError): self.lease(expires_at_us=101)

    def test_agent_limit(self):
        self.decision()
        self.lease()
        self.db.execute("INSERT INTO actors VALUES('agent2','p','agent',?)",(H,))
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError): self.lease(id='lease2',agent_id='agent2')

    def test_meta_role_enforced(self):
        with self.assertRaises(sqlite3.IntegrityError): self.decision(meta_actor_id='agent')

    def test_dag_cycle_rejected(self):
        self.graphs()
        self.edge('a','b')
        self.edge('b','c')
        with self.assertRaises(sqlite3.IntegrityError): self.edge('c','a')

    def test_knowledge_cycle_allowed(self):
        self.graphs('knowledge')
        self.edge('a','b')
        self.edge('b','a')
        self.assertEqual(store.verify(self.db)['events'],2)

    def test_cross_identity_kv_rejected(self):
        self.cache()
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('kv_operations',id='op',cache_id='kv',identity_hash=J,key_hash=H,operation='read',result='hit')

    def test_kv_capacity(self):
        self.cache()
        with self.assertRaises(sqlite3.IntegrityError): self.put('kv_samples',id='s',cache_id='kv',used_bytes=101,used_tokens=1,entries_count=1,quality='measured')

    def test_kv_minmax(self):
        self.cache()
        for i,value in enumerate([0,50,100]):
            self.put('kv_samples',id=f's{i}',cache_id='kv',used_bytes=value,used_tokens=0,entries_count=0,quality='measured')
        row=self.db.execute('SELECT * FROM kv_usage_extrema').fetchone()
        self.assertEqual((row['sampled_min_bytes'],row['sampled_max_bytes'],row['samples']),(0,100,3))

    def test_missing_measurement_not_zero(self):
        self.put('resource_samples',id='r',metric='energy_uj',sample_kind='delta',quality='unavailable',value=None,interval_start_us=0,interval_end_us=1,source_hash=H)
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('resource_samples',id='r2',metric='energy_uj',sample_kind='delta',quality='measured',value=None,interval_start_us=0,interval_end_us=1,source_hash=H)

    def test_real_roundtrip(self):
        self.put('retrieval_queries',id='q',query_hash=H,method_version_hash=H,method='lexical',requested_k=1,returned_count=1,duration_ns=1,mode='primary',participation_ratio=1)
        self.assertEqual(store.verify(self.db)['events'],1)

    def test_backup_and_readonly(self):
        self.check()
        target=Path(self.tmp.name)/'backup.db'
        store.backup(self.db,target)
        read=store.connect(target,readonly=True)
        self.assertEqual(store.verify(read)['events'],1)
        with self.assertRaises(sqlite3.OperationalError): read.execute("INSERT INTO projects VALUES('x',?)",(H,))
        read.close()
        with self.assertRaises(ValueError): store.backup(self.db,target)

    def test_crash_uncommitted_transaction_rolls_back(self):
        code="import sqlite3,os,sys; d=sqlite3.connect(sys.argv[1]); d.execute('BEGIN IMMEDIATE'); d.execute(\"INSERT INTO projects VALUES('crash',?)\",('a'*64,)); os._exit(9)"
        result=subprocess.run([sys.executable,'-c',code,str(self.path)],capture_output=True)
        self.assertEqual(result.returncode,9)
        self.assertIsNone(self.db.execute("SELECT id FROM projects WHERE id='crash'").fetchone())
        self.assertEqual(store.verify(self.db)['events'],0)

    def test_concurrent_pressure_and_merkle(self):
        def worker(worker_id):
            db=store.connect(self.path)
            try:
                for i in range(260):
                    h=dict(id=f'w{worker_id}-{i}',run_id='run',span_id='span',actor_id='meta',producer_id=f'w{worker_id}',producer_seq=i,occurred_at_us=i,boot_id='boot',monotonic_ns=i,event_type='telemetry')
                    store.append(db,h)
            finally: db.close()
        start=time.monotonic()
        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(worker,range(4)))
        result=store.verify(self.db)
        self.assertEqual((result['events'],result['merkle_roots']),(1040,1))
        self.assertGreater(time.monotonic()-start,0)

    def test_success_requires_execution_evidence(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('invocation_events',id='done',attempt_id='attempt',phase='completed',status='succeeded')
        self.put('invocation_events',id='dispatch',attempt_id='attempt',phase='dispatched',status='pending')
        self.put('invocation_events',id='start',attempt_id='attempt',phase='started',status='running')
        self.put('invocation_events',id='done',attempt_id='attempt',phase='completed',status='succeeded')
        row=self.db.execute('SELECT * FROM invocation_summary').fetchone()
        self.assertEqual((row['did_dispatch'],row['did_start'],row['execution_status'],row['latest_check']),(1,1,'succeeded',None))

    def test_phase_status_mismatch(self):
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('invocation_events',id='bad',attempt_id='attempt',phase='requested',status='succeeded')

    def test_cross_run_check_rejected(self):
        self.db.execute("INSERT INTO runs VALUES('run2','task',NULL,'cfg','trace2')")
        self.db.commit()
        f=self.fact('acceptance_checks',id='c',attempt_id='attempt',criteria_id='artifact',result='pass',evidence_id='artifact',checker_version_hash=H)
        with self.assertRaises(sqlite3.IntegrityError):
            store.append(self.db,self.header(run_id='run2',span_id=None),[f])

    def test_unique_replace_without_recursive_triggers(self):
        self.db.execute('PRAGMA recursive_triggers=OFF')
        with self.assertRaises(sqlite3.IntegrityError):
            self.db.execute("INSERT OR REPLACE INTO runs VALUES('other','task',NULL,'cfg','trace')")
        self.db.rollback()
        self.assertIsNotNone(self.db.execute("SELECT id FROM runs WHERE id='run'").fetchone())

    def test_cumulative_budget_cap(self):
        self.decision(max_agents=2)
        self.lease(reserved_cost_microusd=60)
        self.db.execute("INSERT INTO actors VALUES('agent2','p','agent',?)",(H,))
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError):
            self.lease(id='lease2',agent_id='agent2',reserved_cost_microusd=41)

    def test_schema_tampering_detected(self):
        self.db.execute('DROP TRIGGER projects_no_delete')
        with self.assertRaises(ValueError): store.verify(self.db)

    def test_migration_failure_atomic(self):
        fake=Path(self.tmp.name)/'broken'
        (fake/'migrations').mkdir(parents=True)
        (fake/'migrations'/'001_initial.sql').write_text('CREATE TABLE partial(x);\nINVALID SQL;\n')
        old=store.HERE
        store.HERE=fake
        path=fake/'db.sqlite3'
        try:
            with self.assertRaises(sqlite3.OperationalError): store.initialize(path)
            db=sqlite3.connect(path)
            self.assertEqual(list(db.execute("SELECT name FROM sqlite_master WHERE type='table'")),[])
            db.close()
        finally: store.HERE=old

    def test_deterministic_schema_generation(self):
        path=ROOT/'migrations'/'001_initial.sql'
        before=path.read_bytes()
        subprocess.run([sys.executable,str(ROOT/'build_schema.py')],check=True,capture_output=True)
        self.assertEqual(path.read_bytes(),before)

    def test_atom_admission_and_usage(self):
        self.check()
        self.db.execute("INSERT INTO atoms VALUES('atom','p',?)",(H,))
        self.db.commit()
        self.put('atom_revisions',id='rev',atom_id='atom',content_id='artifact',rank=0,embedding_hash=H,method_version_hash=H,judge_check_id='check',salience=0.5)
        for action in ['added','retrieved','presented','verified_used']:
            self.put('atom_operations',id=action,atom_id='atom',revision_id='rev',operation=action,evidence_id='artifact')
        rows=list(self.db.execute('SELECT operation,operations FROM atom_activity'))
        self.assertEqual(len(rows),4)
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('atom_operations',id='unsupported',atom_id='atom',revision_id='rev',operation='verified_used')
        self.assertEqual(store.verify(self.db)['events'],6)

    def test_atom_failed_judge_rejected(self):
        self.check(result='fail')
        self.db.execute("INSERT INTO atoms VALUES('atom','p',?)",(H,))
        self.db.commit()
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('atom_revisions',id='rev',atom_id='atom',content_id='artifact',rank=0,embedding_hash=H,method_version_hash=H,judge_check_id='check',salience=0.5)

    def test_hunger_is_derived_without_rewrite(self):
        for i,action in enumerate(['observed','observed','sated']):
            self.put('hunger_events',id=f'h{i}',gap_hash=H,tap='query_thin',action=action,depth=2,threshold=0.3)
        row=self.db.execute('SELECT * FROM hunger_summary').fetchone()
        self.assertEqual((row['hits'],row['latest_action']),(2,'sated'))

    def test_rerank_trajectory_and_novelty(self):
        self.db.execute("INSERT INTO trajectories VALUES('t','run',NULL,?)",(H,))
        self.db.commit()
        self.put('rerank_runs',id='rank',ranker_version_hash=H,criteria_id='artifact',candidate_count=1)
        self.put('rerank_candidates',id='candidate',rerank_id='rank',trajectory_id='t',old_rank=2,new_rank=1,score=0.9,selected=1)
        with self.assertRaises(sqlite3.IntegrityError):
            self.put('novelty_assessments',id='novel',candidate_id='artifact',comparison_set_hash=H,method_version_hash=H,score=1.0,threshold=0.5,decision='admit',evidence_id='artifact')
        self.assertEqual(store.verify(self.db)['events'],2)

    def test_shared_resource_excluded_from_actor_sum(self):
        self.db.execute("INSERT INTO allocations VALUES('pool','meta',?,'weights','shared')",(H,))
        self.db.commit()
        self.put('resource_samples',id='shared',allocation_id='pool',metric='allocated_bytes',sample_kind='gauge',quality='measured',value=100,interval_start_us=0,interval_end_us=1,source_hash=H)
        self.put('resource_samples',id='own',metric='tokens_output',sample_kind='delta',quality='measured',value=10,interval_start_us=0,interval_end_us=1,source_hash=H)
        rows=list(self.db.execute('SELECT metric,delta_total FROM resource_by_actor'))
        self.assertEqual([tuple(r) for r in rows],[('tokens_output',10)])

if __name__=='__main__': unittest.main(verbosity=2)
