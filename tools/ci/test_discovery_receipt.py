import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from discovery_receipt import CODE_STEPS, LIVE_STEPS, summarize

SHA = '1' * 40


def steps():
    return {name: {'outcome': 'success'} for name in CODE_STEPS + LIVE_STEPS}


class ReceiptTests(unittest.TestCase):
    def test_positive_is_scoped_not_global_e2e(self):
        result = summarize(SHA, SHA, steps())
        self.assertEqual(result['code']['state'], 'PASS')
        self.assertEqual(result['live']['state'], 'PASS')
        self.assertIn('NOT overall CI or file E2E', result['scope'])

    def test_live_failure_does_not_erase_code_result(self):
        data = steps()
        data['live']['outcome'] = 'failure'
        data['receipt']['outcome'] = 'skipped'
        result = summarize(SHA, SHA, data)
        self.assertEqual(result['code']['state'], 'PASS')
        self.assertEqual(result['live']['state'], 'FAIL')
        self.assertEqual(result['live']['failed'], ('live',))

    def test_contract_failure_and_skipped_live_stay_distinct(self):
        data = steps()
        data['contracts']['outcome'] = 'failure'
        for name in ('compile',) + LIVE_STEPS:
            data[name]['outcome'] = 'skipped'
        result = summarize(SHA, SHA, data)
        self.assertEqual(result['code']['state'], 'FAIL')
        self.assertEqual(result['live']['state'], 'INCOMPLETE')

    def test_outcome_not_masking_conclusion_is_used(self):
        data = steps()
        data['contracts'] = {'outcome': 'failure', 'conclusion': 'success'}
        result = summarize(SHA, SHA, data)
        self.assertEqual(result['code']['state'], 'FAIL')
        self.assertNotEqual(result['live']['state'], 'PASS')

    def test_stale_checkout_cannot_pass(self):
        result = summarize(SHA, '2' * 40, steps())
        self.assertEqual(result['code']['state'], 'INCOMPLETE')
        self.assertEqual(result['live']['state'], 'INCOMPLETE')

    def test_missing_or_invalid_step_cannot_pass(self):
        for bad in (None, {}, {'outcome': ['success']}, {'outcome': 'neutral'}):
            data = steps()
            data['contracts'] = bad
            self.assertNotEqual(summarize(SHA, SHA, data)['code']['state'], 'PASS')

    def test_outputs_unknown_steps_and_messages_are_not_persisted(self):
        data = steps()
        data['live']['outputs'] = {'url': 'https://PRIVATE.invalid/SECRET'}
        data['UNKNOWN-PRIVATE'] = {'outcome': 'SECRET'}
        output = json.dumps(summarize(SHA, SHA, data))
        self.assertNotIn('PRIVATE', output)
        self.assertNotIn('SECRET', output)
        self.assertNotIn('://', output)

    def test_empty_context_is_not_green(self):
        result = summarize(SHA, SHA, {})
        self.assertEqual(result['code']['state'], 'INCOMPLETE')
        self.assertEqual(result['live']['state'], 'INCOMPLETE')

    def test_real_cli_writes_evidence_after_failed_stage(self):
        with tempfile.TemporaryDirectory(prefix='pulqva ci ') as tmp:
            root = Path(tmp)
            subprocess.run(['git', 'init', '-q', tmp], check=True)
            subprocess.run(['git', '-C', tmp, '-c', 'user.name=Fixture', '-c',
                            'user.email=fixture@example.invalid', 'commit', '-qm',
                            'fixture', '--allow-empty'], check=True)
            sha = subprocess.check_output(['git', '-C', tmp, 'rev-parse', 'HEAD'], text=True).strip()
            data = steps()
            data['live']['outcome'] = 'failure'
            data['receipt']['outcome'] = 'skipped'
            env = dict(os.environ, PULQVA_SOURCE_SHA=sha,
                       PULQVA_STEP_CONTEXT=json.dumps(data),
                       PULQVA_EVIDENCE_DIR=str(root / 'safe evidence'),
                       GITHUB_RUN_ID='123', GITHUB_RUN_ATTEMPT='1')
            script = Path(__file__).with_name('discovery_receipt.py').resolve()
            proc = subprocess.run([sys.executable, str(script)], cwd=tmp, env=env,
                                  capture_output=True, text=True, timeout=15)
            self.assertEqual(proc.returncode, 0, proc.stdout)
            receipt = json.loads((root / 'safe evidence/stage-status.json').read_text())
            self.assertEqual(receipt['checkout_sha'], sha)
            self.assertEqual(receipt['live']['state'], 'FAIL')

    def test_non_object_context_is_rejected(self):
        with self.assertRaises(ValueError):
            summarize(SHA, SHA, [])


if __name__ == '__main__':
    unittest.main()
