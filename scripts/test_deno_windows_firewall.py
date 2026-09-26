import contextlib
import io
import json
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import deno_windows_firewall as firewall


class FirewallTests(unittest.TestCase):
    def invoke(self, effects, probe_result=None, probe_error=None):
        output = io.StringIO()
        with patch.object(firewall.subprocess, "run", side_effect=effects) as run, \
                patch.object(firewall, "run_restricted", return_value=probe_result,
                             side_effect=probe_error) as probe, \
                contextlib.redirect_stderr(output):
            error = None
            result = None
            try:
                result = firewall.run_windows_restricted(Path("deno.exe"), Path("fixture"))
            except Exception as exc:
                error = exc
        modes = [call.args[0][call.args[0].index("-Mode") + 1] for call in run.call_args_list]
        self.assertTrue(all(call.kwargs["timeout"] == 30 for call in run.call_args_list))
        return result, error, modes, probe, output.getvalue()

    @staticmethod
    def ok(body="{}"):
        return subprocess.CompletedProcess([], 0, body, "phase\n")

    def test_timeout_keeps_partial_output_and_cleans_without_probe(self):
        timeout = subprocess.TimeoutExpired([], 30, output=b"partial", stderr=b"create-rule-begin\x1b")
        result, error, modes, probe, log = self.invoke([timeout, self.ok("")])
        self.assertIs(error, timeout)
        self.assertIsNone(result)
        self.assertEqual(modes, ["Install", "Remove"])
        probe.assert_not_called()
        self.assertIn("create-rule-begin", log)
        self.assertIn("\\u001b", log)

    def test_install_failure_stops_probe_and_cleans(self):
        _, error, modes, probe, _ = self.invoke([subprocess.CompletedProcess([], 1, "", "failure"), self.ok("")])
        self.assertIsInstance(error, RuntimeError)
        self.assertEqual(modes, ["Install", "Remove"])
        probe.assert_not_called()

    def test_probe_failure_still_cleans(self):
        failure = RuntimeError("probe failed")
        _, error, modes, _, _ = self.invoke([self.ok(), self.ok("")], probe_error=failure)
        self.assertIs(error, failure)
        self.assertEqual(modes, ["Install", "Remove"])

    def test_success_requires_matching_evidence_and_cleanup(self):
        result, error, modes, _, _ = self.invoke([self.ok(), self.ok(), self.ok("")], probe_result={})
        self.assertIsNone(error)
        self.assertEqual(modes, ["Install", "Verify", "Remove"])
        self.assertTrue(result["network_gate"]["cleanup_verified"])
        for effects in ([self.ok(), self.ok('{"changed":true}'), self.ok("")],
                        [self.ok(), self.ok(), subprocess.TimeoutExpired([], 30)]):
            result, error, _, _, _ = self.invoke(effects, probe_result={})
            self.assertIsNone(result)
            self.assertIsNotNone(error)

    def test_diagnostics_are_bounded_and_single_line(self):
        output = io.StringIO()
        with contextlib.redirect_stderr(output):
            firewall.emit_diagnostics("Install", "timeout", b"a" * 10000, "\n" * 10000)
        lines = output.getvalue().splitlines()
        self.assertEqual(len(lines), 1)
        data = json.loads(lines[0].split(" ", 1)[1])
        self.assertEqual(len(data["stdout_tail"]), 4096)
        self.assertEqual(len(data["stderr_tail"]), 4096)
