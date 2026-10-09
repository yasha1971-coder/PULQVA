#!/usr/bin/env python3
"""Read-only validation of a v2 recovery handoff manifest. No network or mutation."""
from __future__ import annotations
import hashlib
import json
import re
import subprocess
from pathlib import Path, PurePosixPath

HEX40 = re.compile(r"[0-9a-f]{40}\Z")
HEX64 = re.compile(r"[0-9a-f]{64}\Z")
ALLOWED = {"NEXT.md", "PROJECT_STATE.json", "kernel/CORE_CONTRACT.md",
           "kernel/PRIVACY_INVARIANTS.md", "kernel/UX_CONTRACT.md",
           "kernel/KERNEL_VERSION", "recovery/INDEX.json"}


def unique_object(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError("duplicate JSON key: " + key)
        obj[key] = value
    return obj


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", "-C", str(root), *args],
                                   stderr=subprocess.DEVNULL, timeout=20).decode().strip()


def safe_file(root: Path, name: str) -> Path:
    if name not in ALLOWED or PurePosixPath(name).is_absolute() or ".." in PurePosixPath(name).parts:
        raise ValueError("unapproved path")
    path = root / name
    if any(p.is_symlink() for p in [path, *path.parents] if p != root.parent):
        raise ValueError("symlink path")
    if not path.resolve().is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError("file absent or escapes root")
    return path


def validate(root: Path, manifest: dict, *, expected_head: str) -> dict:
    root = root.resolve()
    if manifest.get("schema") != 2 or manifest.get("protocol") != "pulqva-recovery-handoff-v2":
        raise ValueError("wrong handoff schema")
    if not isinstance(expected_head, str) or not HEX40.fullmatch(expected_head):
        raise ValueError("invalid expected head")
    if git(root, "rev-parse", "HEAD") != expected_head:
        raise ValueError("actual checkout differs from admitted head")
    parent, target = manifest["parent"], manifest["target"]
    for sha in (parent["commit"], parent["index_blob"], target["commit"], target["tree"]):
        if not isinstance(sha, str) or not HEX40.fullmatch(sha):
            raise ValueError("invalid git identity")
    if target["commit"] != expected_head or git(root, "rev-parse", "HEAD^{tree}") != target["tree"]:
        raise ValueError("target commit/tree mismatch")
    git(root, "merge-base", "--is-ancestor", parent["commit"], expected_head")
    if git(root, "rev-parse", parent["commit"] + ":recovery/INDEX.json") != parent["index_blob"]:
        raise ValueError("historical index blob mismatch")
    raw_parent = subprocess.check_output(
        ["git", "-C", str(root), "show", parent["commit"] + ":recovery/INDEX.json"], timeout=20)
    if not isinstance(parent.get("index_sha256"), str) or not HEX64.fullmatch(parent["index_sha256"]) or hashlib.sha256(raw_parent).hexdigest() != parent["index_sha256"]:
        raise ValueError("historical index digest mismatch")
    files = target["files"]
    if not isinstance(files, dict) or set(files) != ALLOWED:
        raise ValueError("mandatory manifest inventory mismatch")
    for name, expected in files.items():
        if not isinstance(expected, str) or not HEX40.fullmatch(expected):
            raise ValueError("invalid file blob")
        path = safe_file(root, name)
        raw = path.read_bytes()
        actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        if actual != expected or git(root, "rev-parse", "HEAD:" + name) != expected:
            raise ValueError("working file or git blob mismatch: " + name)
    if manifest.get("scope") != "recovery-continuity-only":
        raise ValueError("scope escalation")
    return {"status": "PASS", "source": expected_head, "scope": "recovery-continuity-only"}


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object)
