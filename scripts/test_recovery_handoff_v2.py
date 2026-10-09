#!/usr/bin/env python3
"""Deterministic, offline negative contracts for recovery handoff v2."""
import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import recovery_handoff_v2 as h

HEAD = "a" * 40
PARENT = "b" * 40
TREE = "c" * 40
INDEX = b'{"schema":1}\n'
INDEX_BLOB = hashlib.sha1(b"blob " + str(len(INDEX)).encode() + b"\0" + INDEX).hexdigest()


def blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


class HandoffContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.files = {}
        for name in h.ALLOWED:
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(name.encode())
            self.files[name] = blob(name.encode())
        self.manifest = {
            "schema": 2, "protocol": "pulqva-recovery-handoff-v2",
            "parent": {"commit": PARENT, "index_blob": INDEX_BLOB,
                       "index_sha256": hashlib.sha256(INDEX).hexdigest()},
            "target": {"commit": HEAD, "tree": TREE, "files": self.files},
            "scope": "recovery-continuity-only"}
        self.calls = []

        def fake_git(root, *args):
            self.calls.append(args)
            if args == ("rev-parse", "HEAD"):
                return HEAD
            if args == ("rev-parse", "HEAD^{tree}"):
                return TREE
            if args == ("merge-base", "--is-ancestor", PARENT, HEAD):
                return ""
            if args == ("rev-parse", PARENT + ":recovery/INDEX.json"):
                return INDEX_BLOB
            if len(args) == 2 and args[0] == "rev-parse" and args[1].startswith("HEAD:"):
                return self.files[args[1][5:]]
            raise AssertionError(args)

        self.gitpatch = patch.object(h, "git", side_effect=fake_git)
        self.gitpatch.start()
        self.addCleanup(self.gitpatch.stop)
        self.subpatch = patch.object(h.subprocess, "check_output", return_value=INDEX)
        self.subpatch.start()
        self.addCleanup(self.subpatch.stop)

    def check(self, manifest=None, expected_head=HEAD):
        return h.validate(self.root, manifest or self.manifest, expected_head=expected_head)

    def test_01_positive_exact_fixture(self):
        self.assertEqual(self.check()["status"], "PASS")

    def test_02_wrong_expected_head(self):
        with self.assertRaises(ValueError):
            self.check(expected_head="d" * 40)

    def test_03_wrong_target_commit(self):
        m = copy.deepcopy(self.manifest)
        m["target"]["commit"] = "d" * 40
        with self.assertRaises(ValueError):
            self.check(m)

    def test_04_wrong_tree(self):
        m = copy.deepcopy(self.manifest)
        m["target"]["tree"] = "d" * 40
        with self.assertRaises(ValueError):
            self.check(m)

    def test_05_wrong_parent_index(self):
        m = copy.deepcopy(self.manifest)
        m["parent"]["index_blob"] = "d" * 40
        with self.assertRaises(ValueError):
            self.check(m)

    def test_06_corrupt_parent_digest(self):
        m = copy.deepcopy(self.manifest)
        m["parent"]["index_sha256"] = "d" * 64
        with self.assertRaises(ValueError):
            self.check(m)

    def test_07_mutated_next(self):
        (self.root / "NEXT.md").write_text("mutated")
        with self.assertRaises(ValueError):
            self.check()

    def test_08_unapproved_manifest_file(self):
        m = copy.deepcopy(self.manifest)
        m["target"]["files"]["../escape"] = "d" * 40
        with self.assertRaises(ValueError):
            self.check(m)

    def test_09_symlink_target(self):
        path = self.root / "NEXT.md"
        path.unlink()
        path.symlink_to(self.root / "PROJECT_STATE.json")
        with self.assertRaises(ValueError):
            self.check()

    def test_10_scope_escalation(self):
        m = copy.deepcopy(self.manifest)
        m["scope"] = "release-accepted"
        with self.assertRaises(ValueError):
            self.check(m)

    def test_11_duplicate_json_key(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "bad.json"
            p.write_text('{"schema":2,"schema":2}')
            with self.assertRaises(ValueError):
                h.load(p)

    def test_12_non_ancestor_fails_closed(self):
        original = h.git
        def bad(root, *args):
            if args[:2] == ("merge-base", "--is-ancestor"):
                raise h.subprocess.CalledProcessError(1, "git")
            return original(root, *args)
        with patch.object(h, "git", side_effect=bad):
            with self.assertRaises(h.subprocess.CalledProcessError):
                self.check()


if __name__ == "__main__":
    unittest.main()
