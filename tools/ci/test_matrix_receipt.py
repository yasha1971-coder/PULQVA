"""Deterministic codec probes; each test gets fresh data and owned file fixtures.

Direct execution emits its own declared matrix, NOT model/Tor E2E evidence.
"""
import copy
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

from matrix_receipt import (PROTOCOL, MatrixRecorder, canonical, digest, strict_load,
                            validate_manifest, verify, write_exclusive, COMPARISON_CONTROLS)


def fixture_plan():
    captured = {'status': 'captured', 'sha256': 'a' * 64}
    return dict(protocol=PROTOCOL, boundary='synthetic-codec-unit-test', generation_id='fixture-1',
                identity=dict(source_sha='1' * 40, checkout_sha='2' * 40,
                              run_id=None, run_id_reason='unit test',
                              job_id=None, job_id_reason='unit test',
                              attempt=None, attempt_reason='unit test',
                              components={name: dict(captured) for name in
                                          ('runtime', 'model', 'prompt', 'schema',
                                           'flags', 'fixtures', 'evaluator')}),
                environment=dict(os='fixture', arch='fixture', toolchain='fixture'),
                isolation=dict(status='verified', strategy='fresh test-owned fixtures',
                               evidence_refs=['test_matrix_receipt.py:setUp']),
                probes=[dict(id=pid, hypothesis='fixed classifier distinguishes ' + pid,
                             invariant_set=['same evaluator', 'fresh fixture'],
                             expected={'fixture_accepted': True}, depends_on=[])
                        for pid in ('A', 'B')])



def comparison_plan():
    plan = fixture_plan()
    controls = {key: digest({'synthetic_control': key}) for key in COMPARISON_CONTROLS}
    plan['comparison'] = dict(schema=1, question='Does the subject change the fixed outcome?',
        treatment='implementation under test', arms=[
            dict(id=pid, subject_sha256=digest({'synthetic_subject': pid}),
                 controls=copy.deepcopy(controls), probe_ids=[pid],
                 evidence_refs=['synthetic-measured-controls-' + pid]) for pid in ('A', 'B')])
    return plan


def failure_details(items):
    """unittest stores TestCase objects; preserve their IDs, not unserializable objects."""
    return [{'test': case.id(), 'detail': detail} for case, detail in items]


def add(rec, pid, result='PASS'):
    rec.record(pid, result=result, duration_ms=0.25, observed={'fixture': result},
               evidence_refs=['fixture-observation'], reason='fixture_error' if result == 'ERROR' else None)


class MatrixReceiptTests(unittest.TestCase):
    def setUp(self):
        self.plan = fixture_plan()
        self.rec = MatrixRecorder(self.plan)

    def complete(self):
        add(self.rec, 'A')
        add(self.rec, 'B')
        return self.rec.close(self.plan['identity'])

    def test_complete_matrix_passes(self):
        """All declared measured probes and complete identity permit scoped PASS."""
        self.assertEqual(verify(self.plan, self.complete()), 'PASS')

    def test_failure_does_not_suppress_independent_probe(self):
        """A failed expectation cannot erase the next safe independent outcome."""
        add(self.rec, 'A', 'FAIL')
        add(self.rec, 'B')
        receipt = self.rec.close(self.plan['identity'])
        self.assertEqual([x['result'] for x in receipt['probes']], ['FAIL', 'PASS'])
        self.assertEqual(verify(self.plan, receipt), 'FAIL')

    def test_early_exit_keeps_every_planned_probe(self):
        """Readiness failure retains all unstarted probes without invented durations."""
        receipt = self.rec.close(self.plan['identity'], early_exit_reason='readiness_failed')
        self.assertEqual(verify(self.plan, receipt), 'INCOMPLETE')
        self.assertEqual([r['id'] for r in receipt['probes']], ['A', 'B'])
        self.assertTrue(all(r['result'] == 'SKIP' and r['duration_ms'] is None
                            and r['reason'] == 'readiness_failed' for r in receipt['probes']))

    def test_missing_duplicate_extra_reordered_results_rejected(self):
        """Result inventory cannot redefine the predeclared probe set or order."""
        good = self.complete()
        for rows in (good['probes'][:1], good['probes'] * 2,
                     [good['probes'][0]] * 2, list(reversed(good['probes']))):
            bad = copy.deepcopy(good)
            bad['probes'] = rows
            with self.assertRaises(ValueError):
                verify(self.plan, bad)

    def test_changed_criteria_and_manifest_rejected(self):
        """Changing a hypothesis/invariant/expected result cannot repair a generation."""
        good = self.complete()
        for key, value in (('hypothesis', 'different'), ('invariant_set', ['different']),
                           ('expected', 'different')):
            bad = copy.deepcopy(good)
            bad['probes'][0][key] = value
            with self.assertRaises(ValueError):
                verify(self.plan, bad)
        bad = copy.deepcopy(good)
        bad['manifest_sha256'] = 'b' * 64
        with self.assertRaises(ValueError):
            verify(self.plan, bad)

    def test_source_checkout_run_and_generation_must_match(self):
        """Old or unattributed source/runtime identities cannot supply new evidence."""
        good = self.complete()
        for key in ('source_sha', 'checkout_sha', 'run_id', 'job_id', 'attempt', 'components'):
            bad = copy.deepcopy(good)
            bad['identity'][key] = 'changed'
            with self.assertRaises(ValueError):
                verify(self.plan, bad)
        bad = copy.deepcopy(good)
        bad['generation_id'] = 'other-generation'
        with self.assertRaises(ValueError):
            verify(self.plan, bad)

    def test_invalid_or_absent_durations_rejected(self):
        """Missing, negative, Boolean and nonfinite timing cannot establish PASS."""
        good = self.complete()
        for duration in (None, True, -1, '2', float('nan'), float('inf')):
            bad = copy.deepcopy(good)
            bad['probes'][0]['duration_ms'] = duration
            with self.assertRaises(ValueError):
                verify(self.plan, bad)

    def test_outcome_and_aggregate_cannot_be_forged(self):
        """Unknown result or a hand-written green aggregate fails verification."""
        good = self.complete()
        for field, value in (('overall', 'FAIL'), ('accepted', 1)):
            bad = copy.deepcopy(good)
            bad[field] = value
            with self.assertRaises(ValueError):
                verify(self.plan, bad)
        bad = copy.deepcopy(good)
        bad['probes'][0]['result'] = 'UNKNOWN'
        with self.assertRaises(ValueError):
            verify(self.plan, bad)

    def test_unavailable_identity_or_unverified_isolation_cannot_pass(self):
        """All expected answers are insufficient without runtime identity and isolation."""
        for kind in ('identity', 'isolation'):
            plan = copy.deepcopy(self.plan)
            if kind == 'identity':
                plan['identity']['components']['model'] = dict(status='unavailable', sha256=None,
                                                               reason='not captured')
            else:
                plan['isolation'].update(status='unverified', reason='shared state not checked')
            rec = MatrixRecorder(plan)
            add(rec, 'A'); add(rec, 'B')
            self.assertEqual(verify(plan, rec.close(plan['identity'])), 'INCOMPLETE')

    def test_identity_change_and_integrity_stop_fail_closed(self):
        """Runtime change or privacy violation ends acceptance and forbids more probes."""
        add(self.rec, 'A')
        self.rec.fail_closed('privacy_violation')
        with self.assertRaises(ValueError):
            add(self.rec, 'B')
        receipt = self.rec.close(self.plan['identity'])
        self.assertEqual(verify(self.plan, receipt), 'ERROR')
        self.assertEqual(receipt['probes'][1]['reason'], 'privacy_violation')
        rec = MatrixRecorder(self.plan)
        self.assertEqual(verify(self.plan, rec.close({})), 'ERROR')

    def test_record_once_close_once_and_defensive_copy(self):
        """A recorded result cannot be replaced; the manifest is a frozen copy."""
        self.plan['probes'][0]['expected'] = 'mutated caller'
        self.assertNotEqual(self.rec.manifest(), self.plan)
        add(self.rec, 'A')
        with self.assertRaises(ValueError):
            add(self.rec, 'A')
        self.rec.close(self.rec.manifest()['identity'])
        with self.assertRaises(ValueError):
            add(self.rec, 'B')
        with self.assertRaises(ValueError):
            self.rec.close(self.plan['identity'])

    def test_execution_order_and_dependency_are_explicit(self):
        """No out-of-order execution or PASS downstream of a failed prerequisite."""
        with self.assertRaises(ValueError):
            add(self.rec, 'B')
        self.plan['probes'][1]['depends_on'] = ['A']
        rec = MatrixRecorder(self.plan)
        add(rec, 'A', 'FAIL')
        with self.assertRaises(ValueError):
            add(rec, 'B')
        self.assertEqual(verify(self.plan, rec.close(self.plan['identity'])), 'FAIL')

    def test_invalid_manifest_rejected_before_capture(self):
        """An empty, duplicated or incomplete declaration cannot start capture."""
        for mutate in (lambda p: p.update(probes=[]),
                       lambda p: p['probes'].append(copy.deepcopy(p['probes'][0])),
                       lambda p: p['probes'][0].update(depends_on=['B']),
                       lambda p: p['identity']['components'].pop('runtime'),
                       lambda p: p['identity']['components']['runtime'].update(
                           status='not_applicable', sha256=None, reason='not valid')):
            plan = copy.deepcopy(self.plan)
            mutate(plan)
            with self.assertRaises(ValueError):
                validate_manifest(plan)

    def test_missing_observation_evidence_and_skip_reason_rejected(self):
        """Empty evidence and unexplained missing measurements are not successful tests."""
        good = self.complete()
        for field, value in (('observed', None), ('evidence_refs', [])):
            bad = copy.deepcopy(good)
            bad['probes'][0][field] = value
            with self.assertRaises(ValueError):
                verify(self.plan, bad)
        rec = MatrixRecorder(self.plan)
        with self.assertRaises(ValueError):
            rec.record('A', result='SKIP', duration_ms=None, observed=None,
                       evidence_refs=[], timing_reason='not_started')

    def test_strict_json_and_non_overwriting_output(self):
        """Duplicate JSON and replacement of a previous receipt are rejected."""
        with tempfile.TemporaryDirectory(prefix='pulqva-matrix-test-') as root:
            p = Path(root) / 'matrix_receipt.json'
            p.write_text('{"a":1,"a":2}', encoding='utf-8')
            with self.assertRaises(ValueError):
                strict_load(p)
            before = p.read_bytes()
            with self.assertRaises(FileExistsError):
                write_exclusive(p, self.complete())
            self.assertEqual(p.read_bytes(), before)

    def test_failed_test_details_remain_serializable(self):
        """A failed self-test must not prevent writing the other probe outcomes."""
        class Fails(unittest.TestCase):
            def runTest(self):
                self.fail('fixed synthetic failure')
        class Errors(unittest.TestCase):
            def runTest(self):
                raise ValueError('fixed synthetic error')
        result = unittest.TestResult()
        Fails().run(result)
        Errors().run(result)
        doc = dict(errors=failure_details(result.errors), failures=failure_details(result.failures))
        self.assertEqual(len(doc['errors']), 1)
        self.assertEqual(len(doc['failures']), 1)
        self.assertEqual(json.loads(canonical(doc)), doc)

    def test_cli_verifies_against_separate_manifest(self):
        """The actual CLI requires a trusted manifest and exits nonzero on mismatch."""
        with tempfile.TemporaryDirectory(prefix='pulqva-matrix-cli-') as root:
            root = Path(root)
            write_exclusive(root / 'manifest.json', self.plan)
            write_exclusive(root / 'matrix_receipt.json', self.complete())
            cli = Path(__file__).with_name('matrix_receipt.py')
            args = [sys.executable, str(cli), str(root / 'manifest.json'), str(root / 'matrix_receipt.json')]
            result = subprocess.run(args, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 0)
            self.assertEqual(result.stdout.strip(), 'PULQVA_MATRIX_PASS')
            (root / 'manifest.json').write_text('{"private":"do not echo"}', encoding='utf-8')
            result = subprocess.run(args, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 2)
            self.assertEqual(result.stdout.strip(), 'PULQVA_MATRIX_INVALID')


    def test_comparison_matched_controls_pass(self):
        """Different declared subjects with identical measured controls permit comparison capture."""
        plan = comparison_plan(); rec = MatrixRecorder(plan)
        add(rec, 'A'); add(rec, 'B')
        receipt = rec.close(plan['identity'])
        self.assertEqual(verify(plan, receipt), 'PASS')
        self.assertEqual(receipt['comparison_status'], 'MATCHED')
        self.assertEqual(receipt['comparison_sha256'], digest(plan['comparison']))

    def test_comparison_missing_controls_rejected(self):
        """Missing conditions cannot be replaced by a caveat or inferred from another arm."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls'].pop('corpus')
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_unknown_controls_rejected(self):
        """Unknown/unmeasured conditions reject admission even when both arms use the same placeholder."""
        for unknown in (None, '', 'unknown', 'a'*63, True):
            plan = comparison_plan()
            for arm in plan['comparison']['arms']: arm['controls']['environment'] = unknown
            with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_all_additional_controls_must_match(self):
        """A control added by a collector also participates in exact matching."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['load'] = 'b'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_probe_coverage_and_unique_assignment(self):
        """A comparison cannot omit, duplicate or import probes from another plan."""
        for ids in ([], ['A'], ['B', 'B'], ['foreign']):
            plan = comparison_plan(); plan['comparison']['arms'][1]['probe_ids'] = ids
            with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_duplicate_arms_and_same_subject_rejected(self):
        """Duplicated arms or an unchanged subject cannot mint a treatment comparison."""
        for key in ('id', 'subject_sha256'):
            plan = comparison_plan(); arms = plan['comparison']['arms']
            arms[1][key] = arms[0][key]
            with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_schema_and_evidence_required(self):
        """Malformed declarations, unknown extensions and missing provenance stop before capture."""
        for mutate in (lambda c: c.update(schema=True), lambda c: c.update(question=''),
                       lambda c: c.update(treatment=''), lambda c: c.update(waiver='accept anyway'),
                       lambda c: c['arms'][0].update(evidence_refs=[])):
            plan = comparison_plan(); mutate(plan['comparison'])
            with self.assertRaises(ValueError): MatrixRecorder(plan)
        plan = comparison_plan(); plan['comparison'] = None
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_frozen_plan_cannot_be_repaired_in_place(self):
        """Changing comparison controls after capture invalidates the old manifest receipt binding."""
        plan = comparison_plan(); rec = MatrixRecorder(plan); add(rec, 'A'); add(rec, 'B')
        receipt = rec.close(plan['identity'])
        changed = copy.deepcopy(plan)
        for arm in changed['comparison']['arms']: arm['controls']['corpus'] = 'c'*64
        with self.assertRaises(ValueError): verify(changed, receipt)
        receipt['comparison_sha256'] = '0'*64
        with self.assertRaises(ValueError): verify(plan, receipt)

    def test_single_object_receipt_cannot_claim_comparison(self):
        """Historical single-object success remains valid but cannot be relabelled a matched comparison."""
        receipt = self.complete(); receipt['comparison_status'] = 'MATCHED'
        with self.assertRaises(ValueError): verify(self.plan, receipt)

    def test_comparison_mismatched_corpus_rejected(self):
        """Unequal corpus invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['corpus'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_request_rejected(self):
        """Unequal request invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['request'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_expected_output_rejected(self):
        """Unequal expected_output invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['expected_output'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_environment_rejected(self):
        """Unequal environment invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['environment'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_toolchain_rejected(self):
        """Unequal toolchain invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['toolchain'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_model_policy_rejected(self):
        """Unequal model_policy invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['model_policy'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_flags_rejected(self):
        """Unequal flags invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['flags'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_resource_budget_rejected(self):
        """Unequal resource_budget invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['resource_budget'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_cache_policy_rejected(self):
        """Unequal cache_policy invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['cache_policy'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_transport_policy_rejected(self):
        """Unequal transport_policy invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['transport_policy'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_isolation_rejected(self):
        """Unequal isolation invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['isolation'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_metric_rejected(self):
        """Unequal metric invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['metric'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)

    def test_comparison_mismatched_evaluator_rejected(self):
        """Unequal evaluator invalidates comparison before the main runtime can be invoked."""
        plan = comparison_plan(); plan['comparison']['arms'][1]['controls']['evaluator'] = 'f'*64
        with self.assertRaises(ValueError): MatrixRecorder(plan)


def run_matrix(destination: Path) -> int:
    import os
    tests = list(unittest.defaultTestLoader.loadTestsFromTestCase(MatrixReceiptTests))
    root = Path(__file__).resolve().parents[2]
    sha = subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD'], text=True).strip()
    def file_hash(path):
        h = hashlib.sha256()
        with Path(path).open('rb') as stream:
            for block in iter(lambda: stream.read(65536), b''):
                h.update(block)
        return dict(status='captured', sha256=h.hexdigest())
    def identity():
        result = dict(source_sha=sha, checkout_sha=sha, components=dict(
            runtime=file_hash(sys.executable), evaluator=file_hash(Path(__file__).with_name('matrix_receipt.py')),
            fixtures=file_hash(__file__), flags=dict(status='captured', sha256=digest(['sequential', 'fresh TestCase'])),
            **{k: dict(status='not_applicable', sha256=None, reason='codec unit tests; no model runtime')
               for k in ('model', 'prompt', 'schema')}))
        for key, env in (('run_id', 'GITHUB_RUN_ID'), ('job_id', 'PULQVA_JOB_ID'), ('attempt', 'GITHUB_RUN_ATTEMPT')):
            result[key] = os.environ.get(env)
            if result[key] is None:
                result[key + '_reason'] = 'not supplied by this local/CI invocation'
        return result
    plan = fixture_plan()
    plan.update(boundary='G2A-receipt-codec-contract-only', generation_id=str(uuid.uuid4()),
                identity=identity(), environment=dict(os=platform.platform(), arch=platform.machine(),
                                                       toolchain=sys.version),
                probes=[dict(id=t._testMethodName, hypothesis=t.shortDescription() or t._testMethodName,
                             invariant_set=['unchanged codec/test source', 'fresh test instance',
                                            'no model/Tor/public-network execution'],
                             expected='unit assertions pass', depends_on=[]) for t in tests])
    plan['identity']['working_tree_note'] = 'Exact file digests bind any local overlay; source SHA is its Git base.'
    plan['isolation'] = dict(status='verified', strategy='sequential fresh TestCase/setUp; owned TemporaryDirectory per IO case',
                             evidence_refs=['test_matrix_receipt.py:setUp', 'test_matrix_receipt.py:TemporaryDirectory'])
    destination.parent.mkdir(parents=True, exist_ok=True)
    write_exclusive(destination.with_name('manifest.json'), plan)  # Before executing any probe.
    recorder = MatrixRecorder(plan)
    raw = []
    for test in tests:
        result = unittest.TestResult()
        started = time.monotonic_ns()
        test.run(result)
        duration = (time.monotonic_ns() - started) / 1_000_000
        status = 'ERROR' if result.errors else 'FAIL' if result.failures else 'SKIP' if result.skipped else 'PASS'
        raw.append(dict(id=test._testMethodName, result=status, errors=failure_details(result.errors), failures=failure_details(result.failures)))
        recorder.record(test._testMethodName, result=status, duration_ms=duration,
                        observed=dict(tests_run=result.testsRun, errors=len(result.errors), failures=len(result.failures)),
                        evidence_refs=['test-results.json#' + test._testMethodName],
                        reason='unit_test_did_not_complete' if status in ('ERROR', 'SKIP') else None)
    write_exclusive(destination.with_name('test-results.json'), {'results': raw})
    final_identity = identity()
    final_identity['working_tree_note'] = plan['identity']['working_tree_note']
    receipt = recorder.close(final_identity)
    receipt['raw_evidence_sha256'] = hashlib.sha256(destination.with_name('test-results.json').read_bytes()).hexdigest()
    write_exclusive(destination, receipt)
    outcome = verify(strict_load(destination.with_name('manifest.json')), strict_load(destination))
    print('PULQVA_G2A_MATRIX_' + outcome + ' probes=' + str(len(tests)))
    if outcome != 'PASS':
        print(json.dumps(raw, indent=2))
    return 0 if outcome == 'PASS' else 1


if __name__ == '__main__':
    if len(sys.argv) == 2:
        raise SystemExit(run_matrix(Path(sys.argv[1])))
    unittest.main()
