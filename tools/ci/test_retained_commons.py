"""Deterministic readback controls, no Tor/model/download execution."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from media_evidence import verify_retained_commons


class RetainedCommonsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.source = 'a' * 40
        h1 = hashlib.sha1(b'abc').hexdigest()
        self.receipt = dict(schema=2, scope='linux-live-request-choice-file', source_sha=self.source,
            request='countdown', choice_count=2, selected_index=1, selected_locator='public-fixture',
            choices=[dict(index=0), dict(index=1, title='fixture', locator='public-fixture',
                declared_size=3, declared_sha1=h1)],
            file=dict(path='selected.webm', byte_size=3, selected_index=1, title='fixture',
                sha1=h1, sha256=hashlib.sha256(b'abc').hexdigest()),
            core_retrieval_used=True, file_downloaded=True, cancelled_search_rejected=True,
            stale_results_cleared=True, publisher_authenticated=False, windows_e2e_verified=False)
        (self.root/'selected.webm').write_bytes(b'abc')
        self.save()

    def save(self):
        (self.root/'receipt.json').write_text(json.dumps(self.receipt))

    def check(self):
        return verify_retained_commons(self.root, self.source)

    def test_valid_readback_retains_file(self):
        self.assertEqual(self.check()['byte_size'], 3)
        self.assertEqual((self.root/'selected.webm').read_bytes(), b'abc')

    def test_same_size_corruption(self):
        (self.root/'selected.webm').write_bytes(b'abd')
        with self.assertRaises(ValueError): self.check()

    def test_missing_file(self):
        (self.root/'selected.webm').unlink()
        with self.assertRaises(OSError): self.check()

    def test_stale_source(self):
        self.receipt['source_sha'] = 'b' * 40; self.save()
        with self.assertRaises(ValueError): self.check()

    def test_selection_and_bounds(self):
        original = copy.deepcopy(self.receipt)
        for key, value in [('path', '../selected.webm'), ('title', 'foreign'),
                           ('selected_index', True), ('byte_size', 4)]:
            with self.subTest(key=key):
                self.receipt = copy.deepcopy(original)
                self.receipt['file'][key] = value; self.save()
                with self.assertRaises(ValueError): self.check()

    def test_duplicate_keys(self):
        path = self.root/'receipt.json'
        path.write_text(path.read_text()[:-1] + ',"schema":2}')
        with self.assertRaises(ValueError): self.check()

    def test_symlink_file(self):
        path = self.root/'selected.webm'; path.unlink()
        try: path.symlink_to(self.root/'receipt.json')
        except OSError: self.skipTest('symlinks unavailable')
        with self.assertRaises(ValueError): self.check()

    def test_false_completion(self):
        self.receipt['file_downloaded'] = False; self.save()
        with self.assertRaises(ValueError): self.check()

    def test_oversized_receipt(self):
        (self.root/'receipt.json').write_bytes(b' ' * 65_537)
        with self.assertRaises(ValueError): self.check()
