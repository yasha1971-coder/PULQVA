#!/usr/bin/env python3
"""Record exact native object/archive fingerprints for Windows Arti builds.

Diagnostic only. Never substitutes for the pinned final executable SHA-256.
"""
import hashlib
import json
import os
import pathlib
import sys

SUFFIXES = {".obj", ".o", ".lib", ".a", ".rlib", ".rmeta"}
MAX_FILES = 200000

def digest(path):
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def collect(root):
    root = pathlib.Path(root).resolve(strict=True)
    if not root.is_dir():
        raise ValueError("target root is not a directory")
    entries = []
    for p in sorted(root.rglob("*")):
        if p.is_symlink():
            raise ValueError("symlink inside native target")
        if not p.is_file() or p.suffix.lower() not in SUFFIXES:
            continue
        if len(entries) >= MAX_FILES:
            raise ValueError("object inventory exceeds bound")
        relative = p.relative_to(root).as_posix()
        entries.append({"path": relative, "size": p.stat().st_size, "sha256": digest(p)})
    if not entries:
        raise ValueError("no native build objects found")
    aggregate = hashlib.sha256()
    for item in entries:
        aggregate.update(json.dumps(item, sort_keys=True, separators=(",", ":")).encode())
        aggregate.update(b"\n")
    return {"schema": 1, "purpose": "diagnostic-only-not-release-acceptance",
            "file_count": len(entries), "manifest_sha256": aggregate.hexdigest(),
            "files": entries}

def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: arti_windows_repro_fingerprint.py TARGET_DIR RECEIPT.json")
    out = pathlib.Path(sys.argv[2])
    if out.exists() or out.is_symlink():
        raise SystemExit("refusing to overwrite receipt")
    result = collect(sys.argv[1])
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("x", encoding="utf-8") as f:
        json.dump(result, f, sort_keys=True, indent=2)
        f.write("\n")
    print("PULQVA_ARTI_NATIVE_FINGERPRINT files=" + str(result["file_count"]) +
          " manifest_sha256=" + result["manifest_sha256"])

if __name__ == "__main__":
    main()
