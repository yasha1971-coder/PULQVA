"""Narrow repository contract checks, not a general GitHub YAML validator."""
from pathlib import Path
import re
import unittest

from discovery_receipt import CODE_STEPS, LIVE_STEPS


class WorkflowContractTests(unittest.TestCase):
    def setUp(self):
        root = Path(__file__).resolve().parents[2]
        self.text = (root / '.github/workflows/commons-tor-discovery-check.yml').read_text()

    def test_expected_step_ids_and_preflight_order(self):
        ids = re.findall(r'^        id: (\w+)$', self.text, re.MULTILINE)
        self.assertEqual(ids, list(CODE_STEPS + LIVE_STEPS) + ['evidence'])
        self.assertNotIn('continue-on-error:', self.text)
        self.assertIn('cargo +1.91.0 test --locked -p pulqva-core -p pulqva-discovery -- --nocapture', self.text)

    def test_failure_evidence_is_explicit_and_not_empty_path(self):
        self.assertIn("always() && steps.checkout.outcome == 'success' && steps.python.outcome == 'success'", self.text)
        self.assertIn("always() && steps.evidence.outcome == 'success'", self.text)
        self.assertIn('path: ${{ runner.temp }}/pulqva-discovery-evidence/stage-status.json', self.text)
        self.assertIn('PULQVA_STEP_CONTEXT: ${{ toJSON(steps) }}', self.text)
        # runner context is valid in step env, not in job-level env.
        before_steps = self.text.split('    steps:', 1)[0]
        self.assertNotIn('runner.', before_steps)
        self.assertIn('          PULQVA_EVIDENCE_DIR: ${{ runner.temp }}', self.text)


    def test_all_targets_and_privacy_tests_are_required_before_live_work(self):
        block = self.text.split('        id: contracts\n', 1)[1].split('      - name:', 1)[0]
        self.assertIn('        shell: bash\n', block)
        self.assertIn('          set -euo pipefail\n', block)
        self.assertIn('test "$(git rev-parse HEAD)" = "$PULQVA_SOURCE_SHA"', block)
        commands = [
            'cargo +1.91.0 check --workspace --all-targets --locked --keep-going',
            'cargo +1.91.0 test --workspace --all-targets --locked --no-run',
            'cargo +1.91.0 test --locked -p pulqva-privacy --lib -- --nocapture',
            'cargo +1.91.0 test --locked -p pulqva-core -p pulqva-discovery -- --nocapture',
        ]
        positions = [block.index(command) for command in commands]
        self.assertEqual(positions, sorted(positions))
        self.assertNotIn('|| true', block)
        self.assertNotIn('set +e', block)

if __name__ == '__main__':
    unittest.main()
