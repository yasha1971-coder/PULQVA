#!/usr/bin/env python3
"""Run the bounded native readiness-diagnostics contract against one measured test binary."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from matrix_receipt import MatrixRecorder, canonical, verify, write_exclusive

PROBES = [
    {
        "id": "RD-WIRE",
        "test": "arti_readiness::tests::rd_wire_request_is_fixed_remote_domain_connect",
        "hypothesis": "The readiness CONNECT request remains the fixed RFC1928 remote-domain request.",
        "invariant_set": ["fixed_destination", "remote_dns_via_socks", "no_new_network_action"],
        "expected": "exact fixed request bytes",
    },
    {
        "id": "RD-REPLY",
        "test": "arti_readiness::tests::rd_reply_classifies_all_256_codes",
        "hypothesis": "Every SOCKS5 REP byte maps to a fixed bounded classification without changing acceptance.",
        "invariant_set": ["all_256_rep_values", "rep_zero_only_success", "no_raw_error_text"],
        "expected": "256 deterministic classifications",
    },
    {
        "id": "RD-TRACE",
        "test": "arti_readiness::tests::action_trace_matches_pre_diagnostics_policy_for_all_25_outcome_pairs",
        "hypothesis": "Passive diagnostics preserve the pre-existing IPv4/IPv6 dispatcher action trace.",
        "invariant_set": ["ipv4_then_ipv6", "first_success_stops", "protocol_error_stops", "retry_pair_aggregates"],
        "expected": "all 25 outcome pairs preserve action order/result category",
    },
    {
        "id": "RD-CAUSES",
        "test": "arti_readiness::tests::rd_io_classification_distinguishes_timeout_kinds",
        "hypothesis": "Platform timeout-like I/O kinds are recorded distinctly without changing retry policy.",
        "invariant_set": ["wouldblock_distinct", "timedout_distinct", "retry_policy_unchanged"],
        "expected": "fixed I/O-kind mapping",
    },
    {
        "id": "RD-CAP",
        "test": "arti_readiness::tests::rd_trace_is_bounded_and_counts_omissions",
        "hypothesis": "The in-memory readiness trace is bounded and explicitly counts omitted observations.",
        "invariant_set": ["fixed_capacity", "no_disk_log", "omissions_explicit"],
        "expected": "capacity enforced and omitted count exact",
    },
    {
        "id": "RD-PRIVATE",
        "test": "arti_readiness::tests::rd_trace_contains_only_fixed_privacy_safe_fields",
        "hypothesis": "Trace data contains only fixed classifications/timing, never raw host, port, path, or error text.",
        "invariant_set": ["no_hostname", "no_port", "no_path", "no_raw_error"],
        "expected": "privacy-safe bounded trace representation",
    },
]

def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def git(*args: str) -> str:
    return subprocess.check_output(["git", "-C", str(ROOT), *args], text=True, timeout=20).strip()

def compile_test_binary() -> Path:
    cmd = ["cargo", "+1.91.0", "test", "--locked", "-p", "pulqva-privacy",
           "--lib", "--no-run", "--message-format=json"]
    proc = subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, timeout=240)
    if proc.returncode != 0:
        raise RuntimeError(f"compile:{proc.returncode}")
    executables = []
    for line in proc.stdout.splitlines():
        try:
            item = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (item.get("reason") == "compiler-artifact"
                and item.get("profile", {}).get("test") is True
                and item.get("target", {}).get("name") == "pulqva_privacy"
                and item.get("executable")):
            executables.append(Path(item["executable"]))
    if len(executables) != 1 or not executables[0].is_file():
        raise RuntimeError("compile:unique-test-binary-not-found")
    return executables[0]

def component(status: str, *, sha256: str | None = None, reason: str | None = None) -> dict:
    out = {"status": status, "sha256": sha256}
    if reason is not None:
        out["reason"] = reason
    return out

def main() -> int:
    if len(sys.argv) != 2:
        raise SystemExit("expected output directory")
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=False)
    try:
        binary = compile_test_binary()
    except Exception as exc:
        (out / "runner-error.json").write_bytes(
            canonical({"status": "ERROR", "stage": "compile_test_binary",
                       "classification": str(exc).split(":", 1)[0]}) + b"\n")
        return 2

    source_sha = os.environ.get("PULQVA_SOURCE_SHA") or git("rev-parse", "HEAD")
    checkout_sha = git("rev-parse", "HEAD")
    run_id = os.environ.get("GITHUB_RUN_ID")
    attempt = os.environ.get("GITHUB_RUN_ATTEMPT")
    identity = {
        "source_sha": source_sha,
        "checkout_sha": checkout_sha,
        "run_id": run_id,
        "run_id_reason": None if run_id else "not_running_in_github_actions",
        "job_id": None,
        "job_id_reason": "numeric GitHub job id is not available inside the runner environment",
        "attempt": attempt,
        "attempt_reason": None if attempt else "not_running_in_github_actions",
        "components": {
            "runtime": component("captured", sha256=sha256_file(binary)),
            "model": component("not_applicable", reason="deterministic native contract; no model"),
            "prompt": component("not_applicable", reason="deterministic native contract; no prompt"),
            "schema": component("not_applicable", reason="deterministic native contract; no model schema"),
            "flags": component("captured", sha256=sha256_bytes(canonical({
                "binary": "pulqva_privacy libtest",
                "mode": "fresh process per exact test",
                "toolchain": "1.91.0",
            }))),
            "fixtures": component("captured", sha256=sha256_file(
                ROOT / "crates/pulqva-privacy/src/arti_readiness/tests.rs")),
            "evaluator": component("captured", sha256=sha256_file(Path(__file__))),
        },
    }
    manifest = {
        "protocol": "pulqva-evidence-boundary-v1",
        "boundary": "T069-G2B-readiness-diagnostics-native-contract",
        "generation_id": f"{checkout_sha[:12]}-{os.environ.get('GITHUB_RUN_ID', 'local')}-{os.environ.get('GITHUB_RUN_ATTEMPT', '0')}",
        "identity": identity,
        "environment": {
            "os": platform.system(),
            "arch": platform.machine(),
            "toolchain": subprocess.check_output(
                ["rustc", "+1.91.0", "--version"], text=True, timeout=20).strip(),
        },
        "isolation": {
            "status": "verified",
            "strategy": "same measured libtest binary; one fresh process per exact synthetic-state probe; no live Tor/network",
            "evidence_refs": ["test-results.json#fresh_process_per_probe"],
        },
        "probes": [
            {k: value for k, value in spec.items() if k != "test"} | {"depends_on": []}
            for spec in PROBES
        ],
    }
    recorder = MatrixRecorder(manifest)
    results = {"fresh_process_per_probe": True, "binary_sha256": sha256_file(binary), "probes": []}
    for spec in PROBES:
        started = time.monotonic_ns()
        try:
            proc = subprocess.run([str(binary), spec["test"], "--exact", "--nocapture"],
                                  cwd=ROOT, capture_output=True, timeout=30)
            duration_ms = (time.monotonic_ns() - started) / 1_000_000
            stdout_sha = sha256_bytes(proc.stdout)
            stderr_sha = sha256_bytes(proc.stderr)
            row = {"id": spec["id"], "test": spec["test"], "pid_isolated": True,
                   "returncode": proc.returncode, "stdout_sha256": stdout_sha,
                   "stderr_sha256": stderr_sha}
            results["probes"].append(row)
            recorder.record(
                spec["id"],
                result="PASS" if proc.returncode == 0 else "FAIL",
                duration_ms=duration_ms,
                observed={"returncode": proc.returncode,
                          "stdout_sha256": stdout_sha, "stderr_sha256": stderr_sha},
                evidence_refs=[f"test-results.json#{spec['id']}"],
            )
        except subprocess.TimeoutExpired as exc:
            duration_ms = (time.monotonic_ns() - started) / 1_000_000
            stdout = exc.stdout or b""
            stderr = exc.stderr or b""
            results["probes"].append({
                "id": spec["id"], "test": spec["test"], "pid_isolated": True,
                "timeout": True, "stdout_sha256": sha256_bytes(stdout),
                "stderr_sha256": sha256_bytes(stderr),
            })
            recorder.record(
                spec["id"], result="ERROR", duration_ms=duration_ms,
                observed={"timeout": True}, evidence_refs=[f"test-results.json#{spec['id']}"],
                reason="probe_process_timeout",
            )
    receipt = recorder.close(identity, early_exit_reason="probe_not_executed")
    write_exclusive(out / "manifest.json", manifest)
    write_exclusive(out / "test-results.json", results)
    write_exclusive(out / "matrix_receipt.json", receipt)
    verdict = verify(manifest, receipt)
    print("PULQVA_READINESS_CONTRACT_" + verdict)
    return 0 if verdict == "PASS" else 1

if __name__ == "__main__":
    raise SystemExit(main())
