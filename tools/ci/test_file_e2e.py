"""Synthetic local evidence tests; no fixture here is a real network download."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from verify_file_e2e import CAP, verify

SHA = '1' * 40


class FileEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='pulqva-c2-evidence-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        payload = b'abc'
        self.h1 = hashlib.sha1(payload, usedforsecurity=False).hexdigest()
        self.h256 = hashlib.sha256(payload).hexdigest()
        self.data = {
            'schema': 2, 'scope': 'linux-live-request-choice-file', 'source_sha': SHA,
            'request': 'countdown', 'choice_count': 2, 'selected_index': 1,
            'core_retrieval_used': True, 'file_downloaded': True,
            'cancelled_search_rejected': True, 'stale_results_cleared': True,
            'publisher_authenticated': False, 'windows_e2e_verified': False,
            'choices': [{'index': i, 'page_id': i+1, 'title': f'File:{i}.webm',
                         'locator': f'https://upload.wikimedia.org/wikipedia/commons/a/ab/{i}.webm',
                         'declared_size': 3, 'declared_sha1': self.h1} for i in range(2)],
            'selected_locator': 'https://upload.wikimedia.org/wikipedia/commons/a/ab/1.webm',
            'file': {'path': 'selected.webm', 'byte_size': 3, 'sha1': self.h1,
                     'sha256': self.h256, 'selected_index': 1, 'title': 'File:1.webm'},
        }
        (self.root / 'selected.webm').write_bytes(payload)
        self.write()

    def write(self):
        (self.root / 'receipt.json').write_text(json.dumps(self.data), encoding='utf-8')

    def test_positive_fixture_checks_actual_bytes_and_leaves_them_unchanged(self):
        before = (self.root / 'receipt.json').read_bytes()
        self.assertEqual(verify(self.root, SHA), {'bytes': 3, 'sha256': self.h256})
        self.assertEqual((self.root / 'receipt.json').read_bytes(), before)
        self.assertEqual((self.root / 'selected.webm').read_bytes(), b'abc')

    def test_windows_receipt_is_platform_bound(self):
        self.data['scope'] = 'windows-live-request-choice-file'
        self.data['windows_e2e_verified'] = True
        self.write()
        self.assertEqual(verify(self.root, SHA, 'windows'), {'bytes': 3, 'sha256': self.h256})
        with self.assertRaises(ValueError):
            verify(self.root, SHA, 'linux')

    def test_same_size_corruption_does_not_pass(self):
        (self.root / 'selected.webm').write_bytes(b'abd')
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_missing_file_never_passes_from_receipt_alone(self):
        (self.root / 'selected.webm').unlink()
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_stale_sha_and_wrong_selection_are_rejected(self):
        with self.assertRaises(ValueError): verify(self.root, '2' * 40)
        self.data['selected_index'] = 0; self.write()
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_receipt_paths_cannot_select_an_external_file(self):
        self.data['file']['path'] = '../selected.webm'; self.write()
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_source_and_digest_mismatches_are_rejected(self):
        for section, key, bad in ((self.data, 'selected_locator', 'https://example.invalid/x'),
                                  (self.data['file'], 'sha1', '0' * 40),
                                  (self.data['file'], 'sha256', '0' * 64)):
            old = section[key]; section[key] = bad; self.write()
            with self.assertRaises(ValueError): verify(self.root, SHA)
            section[key] = old

    def test_size_budget_false_success_and_extra_files_are_rejected(self):
        self.data['choices'][1]['declared_size'] = CAP + 1; self.write()
        with self.assertRaises(ValueError): verify(self.root, SHA)
        self.data['choices'][1]['declared_size'] = 3
        self.data['file_downloaded'] = False; self.write()
        with self.assertRaises(ValueError): verify(self.root, SHA)
        self.data['file_downloaded'] = True; self.write()
        (self.root / 'arti.toml').write_text('must not be uploaded')
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_duplicate_json_fields_are_rejected(self):
        text = json.dumps(self.data).replace('"schema": 2', '"schema": 2, "schema": 2', 1)
        (self.root / 'receipt.json').write_text(text)
        with self.assertRaises(ValueError): verify(self.root, SHA)

    @unittest.skipUnless(os.name == 'posix', 'Unix symlink fixture')
    def test_symlink_does_not_pass(self):
        file = self.root / 'selected.webm'; file.unlink()
        file.symlink_to('receipt.json')
        with self.assertRaises(ValueError): verify(self.root, SHA)

    def test_cli_is_fail_closed_without_raw_error_details(self):
        script = Path(__file__).with_name('verify_file_e2e.py')
        result = subprocess.run([sys.executable, str(script), str(self.root), SHA, 'linux'],
                                capture_output=True, text=True, timeout=10)
        self.assertEqual((result.returncode, result.stdout.strip()), (0, 'PULQVA_FILE_EVIDENCE_OK'))
        (self.root / 'selected.webm').unlink()
        result = subprocess.run([sys.executable, str(script), str(self.root), SHA, 'linux'],
                                capture_output=True, text=True, timeout=10)
        self.assertEqual((result.returncode, result.stdout.strip()), (1, 'PULQVA_FILE_EVIDENCE_REJECTED'))
        self.assertNotIn(str(self.root), result.stdout + result.stderr)

    def test_workflow_requires_file_oracle_and_bounded_native_process(self):
        root = Path(__file__).resolve().parents[2]
        text = (root / '.github/workflows/commons-tor-discovery-check.yml').read_text()
        live = text.split('        id: live\n', 1)[1].split('      - uses:', 1)[0]
        self.assertIn('timeout --signal=TERM --kill-after=10s 300s', live)
        self.assertNotIn('--foreground', live)
        self.assertIn('verify_file_e2e.py "$EVIDENCE" "$PULQVA_SOURCE_SHA" "$PLATFORM"', live)
        self.assertIn('--example real_commons_file', text)
        self.assertNotIn('continue-on-error:', text)
        self.assertIn('sidecars/yt-dlp/SHA256SUMS', text)
        source = (root / 'crates/pulqva-discovery/examples/real_commons_file.rs').read_text()
        self.assertLess(source.index('let receipt = retrieve_choice('), source.index('cancellation.cancel();'))
        self.assertNotIn('Command::new', source)  # No second downloader in this harness.


if __name__ == '__main__':
    unittest.main()
