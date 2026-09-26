"""Temporary per-executable Windows CI rule; never change profile defaults."""
import json
from pathlib import Path
import subprocess
import sys
import uuid

from deno_restricted_probe import run_restricted


def emit_diagnostics(mode, status, stdout=None, stderr=None):
    # CI fixture only: bounded streams, escaped control characters.
    def bounded(value):
        if isinstance(value, bytes):
            value = value.decode("utf-8", errors="replace")
        return (value or "")[-4096:]
    print("PULQVA_FIREWALL_DIAGNOSTIC " + json.dumps({
        "mode": mode, "status": status,
        "stdout_tail": bounded(stdout), "stderr_tail": bounded(stderr),
    }, ensure_ascii=True), file=sys.stderr, flush=True)


def run_windows_restricted(executable, directory):
    rule = "PULQVA-Deno-" + uuid.uuid4().hex
    script = Path(__file__).with_name("deno_windows_firewall.ps1")
    command = ["powershell.exe", "-NoProfile", "-NonInteractive", "-File", str(script)]

    def apply(mode):
        try:
            result = subprocess.run([*command, "-Mode", mode, "-RuleName", rule,
                                     "-Executable", str(executable)],
                                    capture_output=True, text=True, timeout=30)
        except subprocess.TimeoutExpired as error:
            emit_diagnostics(mode, "timeout-30s", error.stdout, error.stderr)
            raise
        emit_diagnostics(mode, "exit-" + str(result.returncode), stderr=result.stderr)
        if result.returncode:
            raise RuntimeError("Firewall " + mode + " failed: " + result.stderr[:2000])
        return json.loads(result.stdout) if mode != "Remove" else None

    try:
        evidence = apply("Install")
        result = run_restricted(executable, directory)
        if apply("Verify") != evidence:
            raise RuntimeError("Firewall evidence changed during fixtures")
    finally:
        apply("Remove")
    evidence["cleanup_verified"] = True
    evidence["scope"] = ("Effective outbound rule for exact Deno executable during fixtures; "
                         "not packet capture, DNS-service isolation or a child-process sandbox")
    result["network_gate"] = evidence
    return result
