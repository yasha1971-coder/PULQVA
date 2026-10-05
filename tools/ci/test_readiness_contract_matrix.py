"""Admission regressions; subprocess fixtures are NOT a native Rust/Tor proof."""
from __future__ import annotations
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import readiness_contract_matrix as runner
from matrix_receipt import strict_load, verify
from test_matrix_receipt import fixture_plan


def completed(name: str, *, code: int = 0, text: str | None = None):
    output = text if text is not None else (
        f"\nrunning 1 test\ntest {name} ... ok\n\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out; finished in 0.00s\n\n")
    return subprocess.CompletedProcess([], code, output.encode(), b"")


class NativeAdmissionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="pulqva-native-admission-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.binary = self.root / "synthetic-binary"
        self.binary.write_bytes(b"one immutable fixture; never executed")
        self.out = self.root / "evidence"
        self.out.mkdir()
        self.plan = fixture_plan()
        self.plan["probes"] = [
            {k: copy.deepcopy(v) for k, v in p.items() if k != "test"} | {"depends_on": []}
            for p in runner.PROBES]
        self.plan["identity"] = runner.measured_identity(self.plan, self.binary)
        self.name = runner.PROBES[0]["test"]

    def test_one_exact_test_can_pass(self):
        """Only a named successful test with count one can establish PASS."""
        self.assertEqual(runner.execution_verdict(self.name, completed(self.name)), ("PASS", None))

    def test_zero_tests_is_not_green(self):
        """Exit zero with no executed tests must be ERROR, not PASS."""
        p = completed(self.name, text="running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 0.00s\n")
        self.assertEqual(p.returncode, 0)  # This satisfied the previous admission predicate.
        self.assertEqual(runner.execution_verdict(self.name, p)[0], "ERROR")

    def test_wrong_ignored_or_duplicate_tests_are_not_green(self):
        """Wrong, ignored, repeated or multi-test output cannot stand in for the probe."""
        good = completed(self.name).stdout.decode()
        for text in [good.replace(self.name, "other::test"), good.replace("... ok", "... ignored"),
                     good.replace("1 passed; 0 failed; 0 ignored", "0 passed; 0 failed; 1 ignored"),
                     good.replace("running 1 test", "running 2 tests"), good + good,
                     good + "test another::test ... ok\n", ""]:
            with self.subTest(text=text):
                self.assertEqual(runner.execution_verdict(self.name, completed(self.name, text=text))[0], "ERROR")

    def test_nonzero_cannot_pass_valid_looking_output(self):
        """A process failure cannot be rescued by a success-looking stdout."""
        self.assertEqual(runner.execution_verdict(self.name, completed(self.name, code=7))[0], "FAIL")

    def test_bad_encoding_and_oversize_are_errors(self):
        """Invalid UTF-8 and oversized output are explicit capture errors."""
        for data in [b"\xff", b"x" * (runner.MAX_PROBE_OUTPUT + 1)]:
            p = subprocess.CompletedProcess([], 0, data, b"")
            self.assertEqual(runner.execution_verdict(self.name, p)[0], "ERROR")

    def test_manifest_precedes_execution_and_raw_bytes_are_retained(self):
        """Criteria exist before every subprocess; raw bytes survive with verified hashes."""
        def invoke(argv, **kwargs):
            self.assertEqual(strict_load(self.out / "manifest.json"), self.plan)
            self.assertFalse((self.out / "matrix_receipt.json").exists())
            self.assertEqual(kwargs["env"]["RUST_TEST_NOCAPTURE"], "0")
            self.assertIn("--exact", argv)
            return completed(argv[1])
        with patch.object(runner.subprocess, "run", side_effect=invoke) as called:
            self.assertEqual(runner.collect(self.binary, self.out, self.plan), 0)
            self.assertEqual(called.call_count, len(runner.PROBES))
        raw = strict_load(self.out / "test-results.json")
        receipt = strict_load(self.out / "matrix_receipt.json")
        for spec, row in zip(runner.PROBES, raw["probes"]):
            self.assertEqual(bytes.fromhex(row["stdout_hex"]), completed(spec["test"]).stdout)
            self.assertEqual(hashlib.sha256(bytes.fromhex(row["stdout_hex"])).hexdigest(), row["stdout_sha256"])
        self.assertEqual(receipt["raw_evidence_sha256"], runner.sha256_file(self.out / "test-results.json"))
        self.assertEqual(verify(self.plan, receipt), "PASS")

    def test_runtime_change_before_first_probe_prevents_execution(self):
        """Changed runtime bytes stop the entire generation before spawning a probe."""
        self.binary.write_bytes(b"changed")
        with patch.object(runner.subprocess, "run") as called:
            self.assertEqual(runner.collect(self.binary, self.out, self.plan), 1)
            called.assert_not_called()
        receipt = strict_load(self.out / "matrix_receipt.json")
        self.assertFalse(receipt["intact"])
        self.assertEqual([r["result"] for r in receipt["probes"]], ["SKIP"] * 6)

    def test_runtime_change_between_probes_stops_and_preserves_first_observation(self):
        """A generation cannot continue after binary mutation or erase prior outcomes."""
        def invoke(argv, **kwargs):
            self.binary.write_bytes(b"changed by synthetic test")
            return completed(argv[1])
        with patch.object(runner.subprocess, "run", side_effect=invoke) as called:
            self.assertEqual(runner.collect(self.binary, self.out, self.plan), 1)
            self.assertEqual(called.call_count, 1)
        receipt = strict_load(self.out / "matrix_receipt.json")
        self.assertEqual([r["result"] for r in receipt["probes"]], ["PASS"] + ["SKIP"] * 5)
        self.assertFalse(receipt["accepted"])

    def test_start_error_still_records_all_planned_outcomes(self):
        """A failed subprocess start records ERROR and explicit SKIPs, not a missing report."""
        with patch.object(runner.subprocess, "run", side_effect=OSError("private local path")):
            self.assertEqual(runner.collect(self.binary, self.out, self.plan), 1)
        receipt = strict_load(self.out / "matrix_receipt.json")
        self.assertEqual([r["result"] for r in receipt["probes"]], ["ERROR"] + ["SKIP"] * 5)
        self.assertNotIn("private local path", (self.out / "test-results.json").read_text())
        self.assertEqual(len(strict_load(self.out / "test-results.json")["probes"]), 6)

    def test_independent_probes_continue_after_timeout(self):
        """A timed-out synthetic process does not suppress other independent safe probes."""
        count = 0
        def invoke(argv, **kwargs):
            nonlocal count
            count += 1
            if count == 1:
                raise subprocess.TimeoutExpired(argv, 30, output=b"partial")
            return completed(argv[1])
        with patch.object(runner.subprocess, "run", side_effect=invoke):
            self.assertEqual(runner.collect(self.binary, self.out, self.plan), 1)
        receipt = strict_load(self.out / "matrix_receipt.json")
        self.assertEqual([r["result"] for r in receipt["probes"]], ["ERROR"] + ["PASS"] * 5)

    def test_previous_manifest_is_not_overwritten(self):
        """An existing manifest blocks execution and is never replaced."""
        p = self.out / "manifest.json"
        p.write_bytes(b"original")
        with patch.object(runner.subprocess, "run") as called:
            with self.assertRaises(FileExistsError):
                runner.collect(self.binary, self.out, self.plan)
            called.assert_not_called()
        self.assertEqual(p.read_bytes(), b"original")


if __name__ == "__main__":
    unittest.main()
