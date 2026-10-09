#!/usr/bin/env python3
"""Read-only continuity checks against an independently admitted v1 checkout.

Trust constants come from PR93 checkpoint 6073593473, not a candidate manifest.
The caller must own both checkouts exclusively; this is not a filesystem sandbox
or an authorization/signature service. Legitimate INDEX/state migrations require
separate reviewed admission, never an automatic reseal. Not wired to the v1 gate.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path, PurePosixPath

HEX40 = re.compile(r"[0-9a-f]{40}\Z")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
TRUST_COMMIT = "df5ac954485920c49e422fdb1eb5e9d3dd43c648"
TRUST_TREE = "32f9849e413a975fe541463ad17dd55828d0a5a6"
TRUST_INDEX_BLOB = "ac3ee2841e3b3ffa39e66d4d5aec8591bc4d79c7"
TRUST_INDEX_SHA256 = "cd71d449d9b6c2e81eb1231234f9ccf2fdf68c1739a218cb5d8d43eece5f2b22"
TRUST_GUARD_SHA256 = "724976ff71c55df814aa6a57bf084dbaa8a19c8236503562200a6661eef5833f"
ALLOWED = {"NEXT.md", "PROJECT_STATE.json", "kernel/CORE_CONTRACT.md",
           "kernel/PRIVACY_INVARIANTS.md", "kernel/UX_CONTRACT.md",
           "kernel/KERNEL_VERSION", "recovery/INDEX.json"}
MAX_JSON_BYTES = 1024 * 1024
TRANSITION_ID = "T069-retained-commons-index-v1"
TRANSITION_COMMIT = "3ca7a85ef5de3f9108cc3213da173a6f65356c87"
TRANSITION_TREE = "730e2faff454856d2fdaff51b47d17a77140ee79"
TRANSITION_INDEX_BLOB = "dd3711d6512cec3907c3ac16708d80a18ccef401"
TRANSITION_INDEX_SHA256 = "9d0ee616c8c2bcb25e35381de018eb7b999a864fe46208b69ead66c536c27b87"
# Exact, reviewed transition; never a caller-supplied reseal allowlist.
INDEX_SEAL_TRANSITIONS = {
    "tools/ci/media_evidence.py": (
        "16ec35eb27ae9647ff98cdcfb1c8fa38d6f1cabd", "e76f8d57c470b4ab17424675cefacfe39db6232a"),
    "tools/ci/test_media_evidence.py": (
        "9d465048e7a3d4858b8b2e6b7ac1a7de4283552e", "61e3cfb16747a5b11953ce623ea2437929df0c82"),
    "tools/ci/readiness_contract_matrix.py": (
        "a29d825489f82c7fbe900edfb3fa546ac2878bc5", "54de3f91b10bc9050c6f64fb62b531f65d23fa19"),
    ".github/workflows/ytdlp-tor-media-check.yml": (
        "1f31f15118a29b90c4b91789ef765a1200dd7a8c", "ac6943553e21ebb3ab0024b34d374258ea23e438"),
    "tasks/READY/T069-G2D-lane-a-retained-profile.md": (
        None, "774551d101164b4a085ff2953fbc6e60bfbea586"),
    "recovery/evidence/accepted-g2c2-par01.zip": (
        None, "117515714670ebf0af171d569367e2643241d42c"),
}


def unique_object(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError("duplicate JSON key: " + key)
        obj[key] = value
    return obj


def _nonfinite(_):
    raise ValueError("non-finite JSON number")


def load(path: Path) -> dict:
    with path.open("rb") as stream:
        raw = stream.read(MAX_JSON_BYTES + 1)
    if len(raw) > MAX_JSON_BYTES:
        raise ValueError("manifest exceeds byte limit")
    obj = json.loads(raw, object_pairs_hook=unique_object, parse_constant=_nonfinite)
    if not isinstance(obj, dict):
        raise ValueError("JSON object required")
    return obj


def _env() -> dict[str, str]:
    # Ignore injected GIT_DIR, index, worktree, config and replacement policy.
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(("GIT_", "PYTHON"))}
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
               GIT_CONFIG_SYSTEM=os.devnull, GIT_NO_REPLACE_OBJECTS="1",
               GIT_NO_LAZY_FETCH="1", GIT_TERMINAL_PROMPT="0",
               GIT_OPTIONAL_LOCKS="0", PYTHONDONTWRITEBYTECODE="1")
    return env


def _git_bytes(root: Path, *args: str) -> bytes:
    return subprocess.check_output(
        ["git", "--no-replace-objects", "-c", "core.fsmonitor=false", "-c",
         "protocol.allow=never", "-C", str(root), *args], env=_env(),
        stderr=subprocess.DEVNULL, timeout=20)


def git(root: Path, *args: str) -> str:
    return _git_bytes(root, *args).decode("utf-8").strip()


def _owned(root: Path, name: str) -> Path:
    rel = PurePosixPath(name)
    if (not name or rel.is_absolute() or ".." in rel.parts or "\\" in name
            or rel.as_posix() != name):
        raise ValueError("unapproved path")
    path = root
    for part in rel.parts:
        path = path / part
        if path.is_symlink():
            raise ValueError("symlink path")
    if not path.is_file() or not path.resolve().is_relative_to(root):
        raise ValueError("file absent or escapes root")
    return path


def safe_file(root: Path, name: str) -> Path:
    if name not in ALLOWED:
        raise ValueError("unapproved path")
    return _owned(root, name)


def _blob(raw: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()


def _repository(root: Path) -> Path:
    root = root.absolute()
    if root.resolve() != root or not (root / ".git").is_dir() or (root / ".git").is_symlink():
        raise ValueError("require an owned normal checkout, without symlink roots")
    if git(root, "rev-parse", "--show-toplevel") != str(root):
        raise ValueError("wrong repository root")
    for name in ("info/grafts", "shallow", "objects/info/alternates"):
        p = root / ".git" / name
        if p.exists() and (p.is_symlink() or p.stat().st_size):
            raise ValueError("unsupported graft, shallow history or alternate objects")
    return root


def _tracked(root: Path, revision: str) -> dict[str, tuple[str, str]]:
    rows = {}
    for entry in _git_bytes(root, "ls-tree", "-rz", "--full-tree", revision).split(b"\0"):
        if not entry:
            continue
        meta, name = entry.split(b"\t", 1)
        mode, kind, sha = meta.decode().split()
        if kind != "blob" or mode not in ("100644", "100755"):
            raise ValueError("non-regular tracked object")
        rows[name.decode("utf-8")] = (mode, sha)
    return rows


def _verify_worktree(root: Path, tracked: dict[str, tuple[str, str]]) -> None:
    for name, (_, sha) in tracked.items():
        if _blob(_owned(root, name).read_bytes()) != sha:
            raise ValueError("working file differs from Git: " + name)


def _checkpoint(root: Path, commit: str, tree: str,
                index_blob: str, index_sha256: str) -> tuple[dict, dict]:
    root = _repository(root)
    if (git(root, "rev-parse", "HEAD") != commit
            or git(root, "rev-parse", "HEAD^{tree}") != tree):
        raise ValueError("unadmitted historical checkpoint")
    raw = _owned(root, "recovery/INDEX.json").read_bytes()
    if (_blob(raw) != index_blob
            or hashlib.sha256(raw).hexdigest() != index_sha256):
        raise ValueError("trusted INDEX changed")
    guard = _owned(root, "scripts/recovery_guard.py")
    if hashlib.sha256(guard.read_bytes()).hexdigest() != TRUST_GUARD_SHA256:
        raise ValueError("untrusted v1 validator")
    tracked = _tracked(root, commit)
    _verify_worktree(root, tracked)
    result = subprocess.run([sys.executable, "-I", "-B", str(guard)], cwd=root,
                            env=_env(), capture_output=True, timeout=30, check=True)
    if b"PULQVA recovery guard: PASS" not in result.stdout:
        raise ValueError("historical v1 acceptance missing")
    _verify_worktree(root, tracked)
    return load(root / "recovery/INDEX.json"), tracked


def _trusted_policy(trusted_root: Path) -> tuple[dict, dict]:
    return _checkpoint(trusted_root, TRUST_COMMIT, TRUST_TREE,
                       TRUST_INDEX_BLOB, TRUST_INDEX_SHA256)


def _protected_paths(root: Path, tracked: dict) -> set[str]:
    protected = {n for n in tracked if n.startswith(
        ("kernel/", "recovery/evidence/", "recovery/legacy/"))}
    protected.update(("PROJECT_STATE.json", "recovery/INDEX.json", "AGENTS.md",
                      "scripts/recovery_guard.py", "decisions/INDEX.json"))
    protected.update(row["path"] for row in load(root / "decisions/INDEX.json")["decisions"])
    return protected


def _check_index_delta(before: dict, after: dict) -> None:
    if set(after) != set(before) | {"lane_a_pending"}:
        raise ValueError("unreviewed INDEX fields")
    for key in before:
        if key != "sealed_files" and after[key] != before[key]:
            raise ValueError("historical INDEX policy changed: " + key)
    old, new = before["sealed_files"], after["sealed_files"]
    delta = {name: (old.get(name), new.get(name)) for name in old.keys() | new.keys()
             if old.get(name) != new.get(name)}
    if delta != INDEX_SEAL_TRANSITIONS:
        raise ValueError("INDEX seal delta differs from reviewed transition")
    lane = after["lane_a_pending"]
    if (not isinstance(lane, dict) or lane.get("base") != TRUST_COMMIT
            or lane.get("status") != "LOCAL_SYNTHETIC_PASS_HOSTED_PENDING"
            or lane.get("branch") != "feat/T069-retained-profile"):
        raise ValueError("historical lane metadata escalated")


def _transition_policy(trusted_root: Path, before: dict, original: dict,
                       transition_root: Path) -> tuple[dict, dict]:
    root = _repository(transition_root)
    after, transitioned = _checkpoint(root, TRANSITION_COMMIT, TRANSITION_TREE,
                                      TRANSITION_INDEX_BLOB, TRANSITION_INDEX_SHA256)
    # Pin the native parent, not the synthetic PR merge or a candidate-selected ancestor.
    if git(root, "rev-parse", TRANSITION_COMMIT + "^@") != TRUST_COMMIT:
        raise ValueError("reviewed transition has different parents")
    _check_index_delta(before, after)
    for name in sorted(_protected_paths(trusted_root, original) - {"recovery/INDEX.json"}):
        if transitioned.get(name) != original[name]:
            raise ValueError("transition changes protected history/policy: " + name)
    return after, transitioned


def validate(root: Path, manifest: dict, *, expected_head: str,
             trusted_root: Path | None = None,
             transition_root: Path | None = None) -> dict:
    if trusted_root is None:
        raise ValueError("independent trusted checkout required")
    root = _repository(root)
    if root == trusted_root.resolve():
        raise ValueError("candidate cannot be its own trusted checkout")
    if not isinstance(manifest, dict) or set(manifest) != {"schema", "protocol", "parent", "target", "scope"}:
        raise ValueError("handoff fields differ from contract")
    if type(manifest["schema"]) is not int or manifest["schema"] != 2 or manifest["protocol"] != "pulqva-recovery-handoff-v2":
        raise ValueError("wrong handoff schema")
    if not isinstance(expected_head, str) or not HEX40.fullmatch(expected_head):
        raise ValueError("invalid expected head")
    parent, target = manifest["parent"], manifest["target"]
    if parent != {"commit": TRUST_COMMIT, "index_blob": TRUST_INDEX_BLOB,
                  "index_sha256": TRUST_INDEX_SHA256}:
        raise ValueError("candidate-selected historical baseline rejected")
    if not isinstance(target, dict) or set(target) != {"commit", "tree", "files"}:
        raise ValueError("target fields differ from contract")
    if manifest["scope"] != "recovery-continuity-only":
        raise ValueError("scope escalation")
    if (expected_head == TRUST_COMMIT or git(root, "rev-parse", "HEAD") != expected_head
            or target["commit"] != expected_head
            or git(root, "rev-parse", "HEAD^{tree}") != target["tree"]):
        raise ValueError("target must be the admitted strict descendant")
    git(root, "merge-base", "--is-ancestor", TRUST_COMMIT, expected_head)
    index, original = _trusted_policy(trusted_root)
    policy_root = trusted_root
    if transition_root is not None:
        if transition_root.resolve() in (root, trusted_root.resolve()):
            raise ValueError("transition checkpoint must be independently owned")
        git(root, "merge-base", "--is-ancestor", TRANSITION_COMMIT, expected_head)
        index, original = _transition_policy(trusted_root, index, original, transition_root)
        policy_root = transition_root
    tracked = _tracked(root, expected_head)
    _verify_worktree(root, tracked)
    if git(root, "rev-parse", "HEAD:kernel") != index["kernel_tree"]:
        raise ValueError("frozen Kernel changed")
    # Preserve accepted history and authority, independent of candidate hash maps.
    protected = _protected_paths(policy_root, original)
    for name in sorted(protected):
        if tracked.get(name) != original[name]:
            raise ValueError("protected history/policy changed; explicit transition required: " + name)
    files = target["files"]
    if not isinstance(files, dict) or set(files) != ALLOWED:
        raise ValueError("mandatory manifest inventory mismatch")
    for name, expected in files.items():
        if (not isinstance(expected, str) or not HEX40.fullmatch(expected)
                or tracked.get(name, (None, None))[1] != expected
                or _blob(safe_file(root, name).read_bytes()) != expected):
            raise ValueError("working file or git blob mismatch: " + name)
    return {"status": "PASS", "source": expected_head, "trusted_source": TRUST_COMMIT,
            "scope": "recovery-continuity-only",
            "transition": TRANSITION_ID if transition_root is not None else None}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--trusted-root", required=True, type=Path)
    parser.add_argument("--transition-root", type=Path)
    parser.add_argument("--expected-head", required=True)
    parser.add_argument("--manifest", type=Path)
    args = parser.parse_args(argv)
    try:
        root = _repository(args.root)
        # Caller/CI supplies the admitted SHA; deriving a manifest is not approval.
        manifest = load(args.manifest) if args.manifest else {
            "schema": 2, "protocol": "pulqva-recovery-handoff-v2",
            "parent": {"commit": TRUST_COMMIT, "index_blob": TRUST_INDEX_BLOB,
                       "index_sha256": TRUST_INDEX_SHA256},
            "target": {"commit": args.expected_head,
                       "tree": git(root, "rev-parse", "HEAD^{tree}"),
                       "files": {n: git(root, "rev-parse", "HEAD:" + n) for n in sorted(ALLOWED)}},
            "scope": "recovery-continuity-only"}
        answer = validate(root, manifest, expected_head=args.expected_head,
                          trusted_root=args.trusted_root, transition_root=args.transition_root)
    except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError) as exc:
        print(json.dumps({"status": "FAIL", "scope": "recovery-continuity-only",
                          "error": type(exc).__name__ + ": " + str(exc)}, sort_keys=True), file=sys.stderr)
        return 1
    print(json.dumps(answer, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
