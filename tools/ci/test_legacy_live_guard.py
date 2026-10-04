"""Synthetic trigger-containment contracts; no live transport or inference."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

from legacy_live_guard import WORKFLOWS, assess
from matrix_receipt import MatrixRecorder, digest, verify, write_exclusive

ROOT = Path(__file__).resolve().parents[2]
BASELINE = json.loads(Path(__file__).with_name('legacy_live_baseline.json').read_text())
SHA = 'a' * 40


class LegacyLiveGuardTests(unittest.TestCase):
    def test_automatic_events_have_zero_live_budget(self):
        """A PR or push requests contracts, never a live generation."""
        for event in ('pull_request', 'push'):
            for workflow in WORKFLOWS:
                result = assess(event, workflow, SHA, SHA, '1')
                self.assertIs(result['admitted'], False)
                self.assertEqual(result['launch_budget'], 0)
                self.assertEqual(result['live_result'], 'NOT_TESTED')
                self.assertEqual(result['state'], 'NOT_REQUESTED')

    def test_manual_event_is_not_capture_or_isolation_proof(self):
        """Explicit dispatch alone cannot grant missing live collector authority."""
        for workflow in WORKFLOWS:
            result = assess('workflow_dispatch', workflow, SHA, SHA, '1')
            self.assertIs(result['admitted'], False)
            self.assertEqual(result['state'], 'BLOCKED')
            self.assertEqual(result['reason'], 'live_capture_admission_not_implemented')

    def test_stale_or_invalid_source_denied(self):
        """A stale checkout or malformed source cannot become an admission record."""
        for source, checkout in ((SHA, 'b' * 40), ('main', 'main'), ('', ''), ('A' * 40, 'A' * 40)):
            with self.assertRaises(ValueError):
                assess('pull_request', WORKFLOWS[0], source, checkout, '1')

    def test_unknown_event_or_workflow_denied(self):
        """Unknown triggers and out-of-scope jobs fail closed."""
        for event, workflow in (('schedule', WORKFLOWS[0]), ('pull_request_target', WORKFLOWS[0]), ('push', 'unknown')):
            with self.assertRaises(ValueError):
                assess(event, workflow, SHA, SHA, '1')

    def test_replays_and_malformed_attempts_never_admit(self):
        """Retrying the workflow cannot bypass absent live admission."""
        for attempt in ('1', '2', '999'):
            self.assertIs(assess('workflow_dispatch', WORKFLOWS[0], SHA, SHA, attempt)['admitted'], False)
        for attempt in ('', '0', '-1', '1\ntrue', '01'):
            with self.assertRaises(ValueError):
                assess('push', WORKFLOWS[0], SHA, SHA, attempt)

    def cli(self, event):
        with tempfile.TemporaryDirectory(prefix='pulqva-admission-') as td:
            root = Path(td)
            env = dict(os.environ, GITHUB_EVENT_NAME=event, GITHUB_RUN_ATTEMPT='1',
                       PULQVA_SOURCE_SHA=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                       GITHUB_OUTPUT=str(root/'output'), GITHUB_STEP_SUMMARY=str(root/'summary'),
                       PULQVA_ALLOW_LIVE='true')  # No environment override is accepted.
            command = [sys.executable, str(ROOT/'tools/ci/legacy_live_guard.py'), WORKFLOWS[0], str(root/'status')]
            p = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=10)
            result = json.loads((root/'status/live-status.json').read_text())
            self.assertEqual((root/'output').read_text(), 'admitted=false\n')
            self.assertIn('Live: NOT TESTED', (root/'summary').read_text())
            before = (root/'status/live-status.json').read_bytes()
            again = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=10)
            self.assertNotEqual(again.returncode, 0)
            self.assertEqual((root/'status/live-status.json').read_bytes(), before)
            return p, result

    def test_automatic_cli_persists_not_tested_exclusively(self):
        """The actual automatic CLI preserves explicit scope and refuses overwrite."""
        p, result = self.cli('pull_request')
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertEqual(result['live_result'], 'NOT_TESTED')

    def test_manual_cli_fails_visibly_without_launching(self):
        """The actual manual CLI is a failure, not a successful skipped live test."""
        p, result = self.cli('workflow_dispatch')
        self.assertEqual(p.returncode, 1, p.stderr)
        self.assertEqual(result['state'], 'BLOCKED')

    def check_workflow(self, name):
        text = (ROOT/f'.github/workflows/{name}.yml').read_text()
        job = BASELINE['jobs'][name]
        prefix, live = text.split(f'  {job}:\n', 1)
        self.assertIn('  pull_request:\n', prefix)
        self.assertIn('  push:\n', prefix)
        self.assertNotIn('    paths:', prefix)
        self.assertIn('  contracts:\n', prefix)
        self.assertIn('os: [ubuntu-24.04, windows-latest]', prefix)
        self.assertIn('  workflow_dispatch:\n', prefix)
        self.assertIn('needs: [admission, contracts]', live.split('    steps:', 1)[0])
        self.assertIn("github.event_name == 'workflow_dispatch' && needs.admission.outputs.admitted == 'true'", live)
        if name == 'ytdlp-tor-media-check':
            prep = '      - name: Build exact pinned Arti sidecar' + live.split('      - name: Build exact pinned Arti sidecar',1)[1].split('      - name: Prebuild existing fixture before freezing evidence',1)[0]
            self.assertEqual(hashlib.sha256(prep.encode()).hexdigest(), BASELINE['media_preparation_sha256'])
            self.assertIn('python3 tools/ci/media_evidence.py capture', live)
            self.assertIn('runs-on: ubuntu-24.04', live)
            self.assertIn('needs: contracts', prefix)
            self.assertIn('media-native-prerequisite-', prefix)
        else:
            self.assertEqual(hashlib.sha256(live.split('    steps:',1)[1].encode()).hexdigest(), BASELINE['steps_sha256'][name])
        self.assertIn(f'python3 tools/ci/legacy_live_guard.py {name}', prefix)
        self.assertIn('admitted: ${{ steps.admission.outputs.admitted }}', prefix)
        self.assertIn('live-status.json', prefix)
        self.assertIn('live NOT TESTED', prefix)
        self.assertNotIn('continue-on-error:', text)
        contracts = prefix.split('  contracts:',1)[1]
        self.assertNotIn('    if:', contracts.split('    steps:',1)[0])
        self.assertNotIn('cargo +1.91.0 install arti', contracts)
        self.assertNotIn('--example real_arti_readiness --', contracts)
        for command in BASELINE['contracts'][name]:
            self.assertIn(command, contracts)

    def test_readiness_workflow_separates_live_and_contracts(self):
        """Readiness keeps both OS contracts and unchanged, admission-gated live commands."""
        self.check_workflow('tor-readiness-check')

    def test_media_workflow_separates_live_and_contracts(self):
        """Media keeps both OS diagnostic tests and its original gated failure predicates."""
        self.check_workflow('ytdlp-tor-media-check')

    def test_metadata_workflow_separates_live_and_contracts(self):
        """Metadata retains both native vector/diagnostic contracts and gated live steps."""
        self.check_workflow('ytdlp-tor-metadata-check')

    def test_commons_workflow_separates_live_and_contracts(self):
        """Commons retains full preflight contracts, evidence controls and gated live steps."""
        self.check_workflow('commons-tor-discovery-check')
        text = (ROOT/'.github/workflows/commons-tor-discovery-check.yml').read_text().split('  live-discovery:',1)[0]
        self.assertIn('python -m unittest discover -s tools/ci -p "test_*.py" -v', text)


def run_matrix(destination: Path) -> int:
    tests = list(unittest.defaultTestLoader.loadTestsFromTestCase(LegacyLiveGuardTests))
    checkout = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    files = [Path(__file__), Path(__file__).with_name('legacy_live_guard.py'),
             Path(__file__).with_name('legacy_live_baseline.json'), Path(__file__).with_name('matrix_receipt.py')]
    files += [ROOT/f'.github/workflows/{w}.yml' for w in WORKFLOWS]
    def identity():
        captured = lambda data: dict(status='captured', sha256=hashlib.sha256(data).hexdigest())
        components = dict(runtime=captured(Path(sys.executable).read_bytes()),
                          evaluator=captured(files[1].read_bytes()), fixtures=captured(files[0].read_bytes()),
                          flags=dict(status='captured', sha256=digest(['sequential','fresh TestCase','no live runtime'])))
        components.update({str(p.relative_to(ROOT)): captured(p.read_bytes()) for p in files[2:]})
        for key in ('model','prompt','schema'):
            components[key] = dict(status='not_applicable', sha256=None, reason='synthetic trigger containment; no model')
        return dict(source_sha=os.environ.get('PULQVA_SOURCE_SHA', checkout), checkout_sha=checkout,
                    working_tree_note='Local overlays bound by component hashes; no future commit identity claimed.',
                    components=components, run_id=os.environ.get('GITHUB_RUN_ID'),
                    run_id_reason='not supplied locally', job_id=None, job_id_reason='numeric job ID not supplied',
                    attempt=os.environ.get('GITHUB_RUN_ATTEMPT'), attempt_reason='not supplied locally')
    plan = dict(protocol='pulqva-evidence-boundary-v1', boundary='G2C1-legacy-trigger-containment-only',
                generation_id=str(uuid.uuid4()), identity=identity(),
                environment=dict(os=platform.platform(),arch=platform.machine(),toolchain=sys.version),
                isolation=dict(status='verified', strategy='Fresh TestCase and owned temporary output for each CLI fixture; unchanged source.',
                               evidence_refs=['tools/ci/test_legacy_live_guard.py:cli']),
                probes=[dict(id=t._testMethodName,hypothesis=t.shortDescription(),
                             invariant_set=['no Tor or model launch','same measured source/interpreter','no repair during matrix'],
                             expected='assertions pass',depends_on=[]) for t in tests])
    destination.mkdir(parents=True,exist_ok=False)
    write_exclusive(destination/'manifest.json',plan)
    recorder = MatrixRecorder(plan)
    raw=[]
    for test in tests:
        if identity()!=plan['identity']:
            break
        result=unittest.TestResult(); started=time.monotonic_ns(); test.run(result)
        duration=(time.monotonic_ns()-started)/1_000_000
        status='ERROR' if result.errors else 'FAIL' if result.failures else 'SKIP' if result.skipped else 'PASS'
        raw.append(dict(id=test._testMethodName,result=status,errors=[(t.id(),str(e)) for t,e in result.errors],
                        failures=[(t.id(),str(e)) for t,e in result.failures]))
        recorder.record(test._testMethodName,result=status,duration_ms=duration,
                        observed=dict(tests_run=result.testsRun,errors=len(result.errors),failures=len(result.failures)),
                        evidence_refs=['test-results.json#'+test._testMethodName],
                        reason='synthetic_test_incomplete' if status in ('ERROR','SKIP') else None)
    write_exclusive(destination/'test-results.json',dict(results=raw))
    receipt=recorder.close(identity())
    receipt['raw_evidence_sha256']=hashlib.sha256((destination/'test-results.json').read_bytes()).hexdigest()
    write_exclusive(destination/'matrix_receipt.json',receipt)
    outcome=verify(plan,receipt)
    print('PULQVA_G2C1_MATRIX_'+outcome+' probes='+str(len(tests)))
    if outcome!='PASS': print(json.dumps(raw,indent=2))
    return 0 if outcome=='PASS' else 1


if __name__=='__main__':
    if len(sys.argv)==2:
        raise SystemExit(run_matrix(Path(sys.argv[1])))
    unittest.main()
