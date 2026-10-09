#!/usr/bin/env python3
"""Real-Git contracts using the original PULQVA history; no Git mocks/network.

The original v1 checkout and target descendant are fresh per probe. --matrix
extends the repository's existing MatrixRecorder, retaining each outcome durably.
"""
from __future__ import annotations
import copy
import hashlib
import json
import os
import platform
import subprocess
import sys
import tempfile
import time
import unittest
import uuid
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import recovery_handoff_v2 as h

ROOT = Path(__file__).resolve().parents[1]
HISTORY = Path(os.environ.get("PULQVA_TEST_HISTORY", str(ROOT))).resolve()
CASES = [
    ("01", "positive_exact_fixture", "A strict descendant with unchanged historical policy passes", True),
    ("02", "wrong_expected_head", "An independently supplied wrong HEAD is rejected", False),
    ("03", "wrong_target_commit", "Wrong declared target commit is rejected", False),
    ("04", "wrong_tree", "Wrong target tree is rejected", False),
    ("05", "wrong_parent_index", "Wrong original INDEX blob is rejected", False),
    ("06", "corrupt_parent_digest", "Wrong original INDEX SHA256 is rejected", False),
    ("07", "mutated_next", "Uncommitted NEXT mutation is rejected", False),
    ("08", "unapproved_manifest_file", "Traversal manifest entry is rejected", False),
    ("09", "symlink_target", "A symlink replacing a tracked file is rejected", False),
    ("10", "scope_escalation", "A product-release scope claim is rejected", False),
    ("11", "duplicate_json_key", "Duplicate JSON keys are rejected", False),
    ("12", "non_ancestor_fails_closed", "An actual orphan commit is not an admitted descendant", False),
    ("13", "committed_index_rewrite", "Rehashed committed INDEX rewrite is rejected", False),
    ("14", "committed_kernel_rewrite", "Rehashed committed frozen Kernel rewrite is rejected", False),
    ("15", "committed_authority_escalation", "Rehashed authority and release-state escalation is rejected", False),
    ("16", "self_parent", "Candidate cannot declare itself the historical parent", False),
    ("17", "malformed_historical_index", "A malformed historical INDEX is rejected before v1 execution", False),
    ("18", "changed_retained_evidence", "A changed retained archive is rejected outside the seven-file manifest", False),
    ("19", "approved_next_change", "A committed NEXT change is possible without resealing history", True),
    ("20", "missing_trust", "Missing independent trusted checkout fails closed", False),
    ("21", "forged_historical_guard", "An altered v1 guard cannot mint historical acceptance", False),
    ("22", "uncommitted_evidence", "Dirty retained evidence not listed in the manifest is rejected", False),
    ("23", "float_schema", "Float schema 2.0 cannot substitute for integer schema 2", False),
    ("24", "injected_git_environment", "Injected GIT_DIR and replacement policy cannot redirect validation", True),
]


def persist(path, raw):
    with Path(path).open("xb") as f:
        f.write(raw)
        f.flush()
        os.fsync(f.fileno())


def encode(obj):
    return json.dumps(obj, sort_keys=True, ensure_ascii=False, allow_nan=False, indent=2).encode() + b"\n"


class HandoffContracts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="pulqva-hv2-")
        self.addCleanup(self.temp.cleanup)
        self.owned = Path(self.temp.name).resolve()
        self.trusted = self.owned / "trusted"
        self.root = self.owned / "candidate"
        self.env = h._env()
        self.env.update(GIT_AUTHOR_NAME="PULQVA synthetic fixture",
                        GIT_AUTHOR_EMAIL="fixture@example.invalid",
                        GIT_COMMITTER_NAME="PULQVA synthetic fixture",
                        GIT_COMMITTER_EMAIL="fixture@example.invalid",
                        GIT_AUTHOR_DATE="2026-10-09T00:00:00+00:00",
                        GIT_COMMITTER_DATE="2026-10-09T00:00:00+00:00")
        for root in (self.trusted, self.root):
            subprocess.run(["git", "-c", "protocol.file.allow=always", "clone", "--no-local",
                            "--no-checkout", str(HISTORY), str(root)], env=self.env,
                           check=True, capture_output=True, timeout=20)
            self.cmd(root, "checkout", "--detach", h.TRUST_COMMIT)
        (self.root / "handoff-test-fixture.txt").write_text("synthetic descendant; not release\n")
        self.commit()

    def cmd(self, root, *args):
        return subprocess.check_output(["git", "--no-replace-objects", "-c", "core.fsmonitor=false",
                                        "-c", "core.hooksPath=/dev/null", "-C", str(root), *args],
                                       env=self.env, stderr=subprocess.DEVNULL, timeout=20).decode().strip()

    def commit(self):
        self.cmd(self.root, "add", "-A")
        self.cmd(self.root, "-c", "commit.gpgsign=false", "commit", "--allow-empty", "-m", "controlled fixture mutation")

    def manifest(self):
        return {"schema": 2, "protocol": "pulqva-recovery-handoff-v2",
                "parent": {"commit": h.TRUST_COMMIT, "index_blob": h.TRUST_INDEX_BLOB,
                           "index_sha256": h.TRUST_INDEX_SHA256},
                "target": {"commit": self.cmd(self.root, "rev-parse", "HEAD"),
                           "tree": self.cmd(self.root, "rev-parse", "HEAD^{tree}"),
                           "files": {n: self.cmd(self.root, "rev-parse", "HEAD:" + n) for n in h.ALLOWED}},
                "scope": "recovery-continuity-only"}

    def exercise(self, name, expected):
        path = self.root / "NEXT.md"
        if name == "mutated_next": path.write_text("uncommitted mutation\n")
        if name == "symlink_target":
            path.unlink()
            path.symlink_to(self.root / "PROJECT_STATE.json")
        if name == "non_ancestor_fails_closed":
            self.cmd(self.root, "checkout", "--orphan", "orphan-fixture")
            self.commit()
        if name == "committed_index_rewrite":
            p = self.root / "recovery/INDEX.json"
            obj = h.load(p); obj["next_action"] = "unreviewed rewrite"; p.write_bytes(encode(obj)); self.commit()
        if name == "committed_kernel_rewrite":
            (self.root / "kernel/CORE_CONTRACT.md").write_text("unreviewed frozen Kernel change\n"); self.commit()
        if name == "committed_authority_escalation":
            p = self.root / "PROJECT_STATE.json"; obj = h.load(p)
            obj["phase"] = "release"; obj["execution_authorization"]["max_mutating_tasks_per_cycle"] = 99
            p.write_bytes(encode(obj)); self.commit()
        if name == "malformed_historical_index":
            (self.trusted / "recovery/INDEX.json").write_text("{broken")
        if name == "changed_retained_evidence":
            p = self.root / h.load(self.trusted / "recovery/INDEX.json")["model_archive"]["path"]
            p.write_bytes(b"tampered retained evidence"); self.commit()
        if name == "approved_next_change":
            with path.open("a") as f: f.write("\nPending synthetic handoff; no acceptance escalation.\n")
            self.commit()
        if name == "forged_historical_guard":
            (self.trusted / "scripts/recovery_guard.py").write_text("print('PULQVA recovery guard: PASS')\n")
        if name == "uncommitted_evidence":
            (self.root / "recovery/evidence/g1-observation.json").write_text("{}")
        m = self.manifest(); head = m["target"]["commit"]
        if name == "wrong_expected_head": head = "d" * 40
        if name == "wrong_target_commit": m["target"]["commit"] = "d" * 40
        if name == "wrong_tree": m["target"]["tree"] = "d" * 40
        if name == "wrong_parent_index": m["parent"]["index_blob"] = "d" * 40
        if name == "corrupt_parent_digest": m["parent"]["index_sha256"] = "d" * 64
        if name == "unapproved_manifest_file": m["target"]["files"]["../escape"] = "d" * 40
        if name == "scope_escalation": m["scope"] = "release-accepted"
        if name == "self_parent": m["parent"]["commit"] = head
        if name == "float_schema": m["schema"] = 2.0
        before = {str(p.name): self.cmd(p, "status", "--porcelain", "--untracked-files=all") for p in (self.root, self.trusted)}
        accepted = False; error = None; old_env = os.environ.copy()
        try:
            if name == "duplicate_json_key":
                p = self.owned / "duplicate.json"; p.write_text('{"schema":2,"schema":2}')
                m = h.load(p)
            if name == "injected_git_environment":
                os.environ.update(GIT_DIR=str(self.trusted / ".git"), GIT_NO_REPLACE_OBJECTS="0")
            result = h.validate(self.root, m, expected_head=head,
                                trusted_root=None if name == "missing_trust" else self.trusted)
            accepted = result["status"] == "PASS"
        except (ValueError, subprocess.CalledProcessError) as exc:
            error = type(exc).__name__ + ": " + str(exc)
        finally:
            os.environ.clear(); os.environ.update(old_env)
        after = {str(p.name): self.cmd(p, "status", "--porcelain", "--untracked-files=all") for p in (self.root, self.trusted)}
        out = os.environ.get("PULQVA_PROBE_OUT")
        if out:
            persist(Path(out) / "fixture.json", encode({"case": name, "manifest": m,
                    "expected_acceptance": expected, "observed_acceptance": accepted,
                    "rejection": error, "before": before, "after": after,
                    "trusted_commit": h.TRUST_COMMIT, "real_git": True, "mocked": False}))
            persist(Path(out) / "target.patch", self.cmd(self.root, "diff", "--binary", h.TRUST_COMMIT, "HEAD").encode())
        self.assertEqual(before, after, "validator mutated a checkout")
        self.assertEqual(accepted, expected, error or "unexpected acceptance")


for number, name, hypothesis, expected in CASES:
    def probe(self, name=name, expected=expected): self.exercise(name, expected)
    probe.__doc__ = hypothesis
    setattr(HandoffContracts, "test_" + number + "_" + name, probe)


def matrix(out: Path) -> int:
    sys.path.insert(0, str(HISTORY / "tools/ci"))
    from matrix_receipt import MatrixRecorder, canonical, verify
    out.mkdir(parents=True, exist_ok=False)
    sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest()
    source = os.environ.get("PULQVA_SOURCE_SHA", h.git(HISTORY, "rev-parse", "HEAD"))
    paths = {"runtime": Path(sys.executable).resolve(), "validator": Path(h.__file__),
             "evaluator": Path(__file__), "codec": HISTORY / "tools/ci/matrix_receipt.py",
             "fixtures": HISTORY / "recovery/INDEX.json"}
    components = {k: {"status": "captured", "sha256": sha(p)} for k, p in paths.items()}
    components["flags"] = {"status": "captured", "sha256": hashlib.sha256(canonical({"argv": ["-B", "one_named_unittest"], "timeout": 20, "cases": CASES})).hexdigest()}
    for k in ("model", "prompt", "schema"):
        components[k] = {"status": "not_applicable", "sha256": None, "reason": "offline source-continuity validation, no inference"}
    identity = {"source_sha": source, "checkout_sha": h.git(HISTORY, "rev-parse", "HEAD"),
                "run_id": os.environ.get("GITHUB_RUN_ID"), "job_id": None,
                "attempt": os.environ.get("GITHUB_RUN_ATTEMPT"), "components": components,
                "run_id_reason": "local execution unless supplied by CI", "job_id_reason": "numeric job ID unavailable to collector",
                "attempt_reason": "local execution unless supplied by CI"}
    plan = {"protocol": "pulqva-evidence-boundary-v1", "boundary": "T069-HV2-independent-trust-correction",
            "generation_id": str(uuid.uuid4()), "identity": identity,
            "environment": {"os": platform.platform(), "arch": platform.machine(), "toolchain": platform.python_version() + " / " + h.git(HISTORY, "--version")},
            "isolation": {"status": "verified", "strategy": "Fresh child, HOME/TMPDIR, original-history checkout and target per probe; owned fixture mutations only before validate", "evidence_refs": ["fixture-provenance.json"]},
            "probes": [{"id": "HV2-" + num, "hypothesis": hyp,
                        "invariant_set": ["one unchanged validator and evaluator", "original independently pinned PULQVA history", "fresh owned Git checkouts", "no network or hot repairs"],
                        "expected": {"accepted": accept}, "depends_on": []} for num, _, hyp, accept in CASES]}
    persist(out / "fixture-provenance.json", canonical({"trusted_commit": h.TRUST_COMMIT, "trusted_tree": h.TRUST_TREE,
            "source_scope": "local candidate overlay unless CI; source SHA is the unmodified live base, checkout SHA is the actual history source; file digests identify tested overlay",
            "case_methods": ["test_" + n + "_" + name for n, name, _, _ in CASES]}) + b"\n")
    persist(out / "manifest.json", canonical(plan) + b"\n")
    recorder = MatrixRecorder(plan)
    try:
        for num, name, _, expected in CASES:
            pid = "HV2-" + num; pd = out / pid; pd.mkdir()
            home = pd / "home"; home.mkdir(); tmp = pd / "tmp"; tmp.mkdir()
            env = h._env(); env.update(HOME=str(home), TMPDIR=str(tmp), PULQVA_PROBE_OUT=str(pd), PULQVA_TEST_HISTORY=str(HISTORY))
            started = time.monotonic_ns()
            try:
                p = subprocess.run([sys.executable, "-B", str(Path(__file__).resolve()), "HandoffContracts.test_" + num + "_" + name, "-v"], env=env, capture_output=True, timeout=20)
                dt = (time.monotonic_ns() - started) / 1e6
                persist(pd / "stdout.txt", p.stdout); persist(pd / "stderr.txt", p.stderr)
                fixture = json.loads((pd / "fixture.json").read_text()) if (pd / "fixture.json").exists() else None
                good = p.returncode == 0 and b"Ran 1 test in " in p.stderr and p.stderr.rstrip().endswith(b"OK") and fixture is not None and fixture["observed_acceptance"] == expected
                row = {"id": pid, "result": "PASS" if good else "FAIL", "duration_ms": dt, "returncode": p.returncode,
                       "accepted": fixture["observed_acceptance"] if fixture else None,
                       "stdout_sha256": sha(pd / "stdout.txt"), "stderr_sha256": sha(pd / "stderr.txt")}
                reason = None
            except subprocess.TimeoutExpired as exc:
                dt = (time.monotonic_ns() - started) / 1e6
                persist(pd / "stdout.txt", exc.stdout or b""); persist(pd / "stderr.txt", exc.stderr or b"")
                row = {"id": pid, "result": "ERROR", "duration_ms": dt, "accepted": None, "returncode": None}
                reason = "bounded probe timeout"
            persist(pd / "outcome.json", canonical(row) + b"\n")
            with (out / "journal.jsonl").open("ab") as f: f.write(canonical(row) + b"\n"); f.flush(); os.fsync(f.fileno())
            recorder.record(pid, result=row["result"], duration_ms=dt, observed=row,
                            evidence_refs=[pid + "/outcome.json", pid + "/stdout.txt", pid + "/stderr.txt"], reason=reason)
            print(pid, row["result"], round(dt, 1), flush=True)
    finally:
        final = copy.deepcopy(identity)
        for k, p in paths.items(): final["components"][k]["sha256"] = sha(p)
        receipt = recorder.close(final, early_exit_reason="collector interrupted")
        receipt["limitations"] += ["Real original-history descendant fixtures, not the current PR93 handoff or product E2E.",
                "Historical INDEX/state transitions are intentionally rejected until independently reviewed; original v1 gate remains in effect."]
        persist(out / "matrix_receipt.json", canonical(receipt) + b"\n")
    result = verify(json.loads((out / "manifest.json").read_text()), json.loads((out / "matrix_receipt.json").read_text()))
    persist(out / "readback.json", canonical({"result": result, "matrix_sha256": sha(out / "matrix_receipt.json"), "probes": len(receipt["probes"])}) + b"\n")
    print("PROJECT_CODEC_VERIFY=" + result, flush=True)
    return 0 if result == "PASS" else 1


if __name__ == "__main__":
    if len(sys.argv) == 3 and sys.argv[1] == "--matrix":
        raise SystemExit(matrix(Path(sys.argv[2]).resolve()))
    unittest.main()
