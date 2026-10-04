#!/usr/bin/env python3
"""Run the bounded native readiness-diagnostics contract against one measured test binary."""
from __future__ import annotations
import copy
import hashlib
import re
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

MAX_PROBE_OUTPUT = 16 * 1024
TEST_FLAGS = ["--exact", "--format=pretty", "--color=never", "--test-threads=1"]


def execution_verdict(test: str, proc: subprocess.CompletedProcess) -> tuple[str, str | None]:
    """Exit zero alone is NOT evidence that an exact libtest probe ran.

    Pin to the documented pretty output of Rust 1.91. Format drift fails closed.
    These are trusted synthetic libtests, not a parser for arbitrary program output.
    """
    if max(len(proc.stdout), len(proc.stderr)) > MAX_PROBE_OUTPUT:
        return "ERROR", "probe_output_exceeds_limit"
    try:
        output = proc.stdout.decode("utf-8")
        proc.stderr.decode("utf-8")
    except UnicodeDecodeError:
        return "ERROR", "probe_output_not_utf8"
    if proc.returncode != 0:
        return "FAIL", "probe_process_failed"
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    tests = [line for line in lines if line.startswith("test ") and not line.startswith("test result:")]
    summaries = [line for line in lines if line.startswith("test result:")]
    running = [line for line in lines if line.startswith("running ")]
    summary = r"test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; [0-9]+ filtered out; finished in [0-9]+(?:\.[0-9]+)?s"
    if (running != ["running 1 test"] or tests != [f"test {test} ... ok"]
            or len(summaries) != 1 or re.fullmatch(summary, summaries[0]) is None):
        return "ERROR", "exact_test_execution_not_proven"
    return "PASS", None


def measured_identity(manifest: dict, binary: Path) -> dict:
    """Read actual bytes again; never close by comparing the plan to itself."""
    identity = copy.deepcopy(manifest["identity"])
    paths = {
        "runtime": binary,
        "fixtures": ROOT / "crates/pulqva-privacy/src/arti_readiness/tests.rs",
        "evaluator": Path(__file__),
        "codec": Path(__file__).with_name("matrix_receipt.py"),
    }
    for name, path in paths.items():
        try:
            identity["components"][name] = component("captured", sha256=sha256_file(path))
        except OSError:
            identity["components"][name] = component("unavailable", reason="identity_read_failed")
    return identity


def collect(binary: Path, out: Path, manifest: dict) -> int:
    recorder = MatrixRecorder(manifest)
    # Persist criteria BEFORE executing a probe. Never overwrite another generation.
    write_exclusive(out / "manifest.json", manifest)
    results = {"fresh_process_per_probe": True,
               "binary_sha256": manifest["identity"]["components"]["runtime"]["sha256"],
               "probes": []}
    env = dict(os.environ, RUST_TEST_NOCAPTURE="0")
    try:
        for spec in PROBES:
            if measured_identity(manifest, binary) != manifest["identity"]:
                recorder.fail_closed("frozen_identity_changed_before_probe")
                break
            started = time.monotonic_ns()
            row = {"id": spec["id"], "test": spec["test"], "pid_isolated": True}
            stop = False
            try:
                proc = subprocess.run([str(binary), spec["test"], *TEST_FLAGS],
                                      cwd=ROOT, env=env, capture_output=True, timeout=30)
                status, reason = execution_verdict(spec["test"], proc)
                stdout, stderr = proc.stdout, proc.stderr
                row["returncode"] = proc.returncode
            except subprocess.TimeoutExpired as exc:
                stdout, stderr = exc.stdout or b"", exc.stderr or b""
                status, reason = "ERROR", "probe_process_timeout"
                row["timeout"] = True
            except OSError:
                stdout, stderr = b"", b""
                status, reason = "ERROR", "probe_process_start_failed"
                stop = True
            duration_ms = (time.monotonic_ns() - started) / 1_000_000
            row.update(result=status, reason=reason,
                       stdout_sha256=sha256_bytes(stdout), stderr_sha256=sha256_bytes(stderr),
                       stdout_hex=stdout[:MAX_PROBE_OUTPUT].hex(),
                       stderr_hex=stderr[:MAX_PROBE_OUTPUT].hex(),
                       output_truncated=max(len(stdout), len(stderr)) > MAX_PROBE_OUTPUT)
            results["probes"].append(row)
            recorder.record(spec["id"], result=status, duration_ms=duration_ms,
                            observed={"returncode": row.get("returncode"),
                                      "execution_check": reason or "one_exact_test_passed",
                                      "stdout_sha256": row["stdout_sha256"],
                                      "stderr_sha256": row["stderr_sha256"]},
                            evidence_refs=[f"test-results.json#{spec['id']}"], reason=reason)
            if stop:
                break
    except (Exception, KeyboardInterrupt):
        # Persist an explicitly unsafe/incomplete generation, never an accidental green.
        recorder.fail_closed("collector_interrupted_or_failed")
    receipt = recorder.close(measured_identity(manifest, binary), early_exit_reason="probe_not_executed")
    seen = {row["id"] for row in results["probes"]}
    for row in receipt["probes"]:
        if row["id"] not in seen:
            results["probes"].append({"id": row["id"], "result": "SKIP", "reason": row["reason"]})
    write_exclusive(out / "test-results.json", results)
    receipt["raw_evidence_sha256"] = sha256_file(out / "test-results.json")
    write_exclusive(out / "matrix_receipt.json", receipt)
    verdict = verify(manifest, receipt)
    print("PULQVA_READINESS_CONTRACT_" + verdict)
    return 0 if verdict == "PASS" else 1


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
                "argv_flags": TEST_FLAGS, "RUST_TEST_NOCAPTURE": "0",
                "exact_tests": [p["test"] for p in PROBES],
            }))),
            "fixtures": component("captured", sha256=sha256_file(
                ROOT / "crates/pulqva-privacy/src/arti_readiness/tests.rs")),
            "evaluator": component("captured", sha256=sha256_file(Path(__file__))),
            "codec": component("captured", sha256=sha256_file(Path(__file__).with_name("matrix_receipt.py"))),
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
            {k: value for k, value in spec.items() if k != "test"}
            | {"depends_on": [], "expected": {"contract": spec["expected"],
                 "test": spec["test"], "executed": 1, "passed": 1}}
            for spec in PROBES
        ],
    }
    return collect(binary, out, manifest)

if __name__ == "__main__":
    raise SystemExit(main())
