#!/usr/bin/env python3
"""One-off native dependency trial; never rewrites the checked-out package graph."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]
PINS = {"reqwest": "0.13.5", "rustls": "0.23.43", "webpki-roots": "1.0.9", "tokio": "1.53.1"}
DEPENDENCIES = '''\n# T068-B0 candidate only; import Cargo's verified lock before shipping adoption.
reqwest = { version = "=0.13.5", default-features = false, features = ["rustls-no-provider", "socks"] }
rustls = { version = "=0.23.43", default-features = false, features = ["ring", "std", "tls12"] }
webpki-roots = "=1.0.9"
tokio = { version = "=1.53.1", default-features = false, features = ["rt", "net", "time"] }
'''

def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def prepare() -> None:
    if not os.environ.get("GITHUB_ENV") or not os.environ.get("RUNNER_TEMP"):
        raise RuntimeError("candidate preparation is restricted to explicit GitHub CI")
    destination = Path(tempfile.mkdtemp(prefix="pulqva-t068-https-", dir=os.environ["RUNNER_TEMP"])).resolve()
    # Copy only tracked regular files. Never follow a link out of the checkout.
    paths = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT).decode("utf-8").split("\0")
    for relative in filter(None, paths):
        source = ROOT / relative
        if source.is_symlink() or not source.is_file() or not source.resolve().is_relative_to(ROOT):
            raise RuntimeError("candidate copy requires regular repository-owned files")
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    manifest = destination / "crates/pulqva-discovery/Cargo.toml"
    original = tomllib.loads(manifest.read_text(encoding="utf-8"))
    if set(PINS) & original.get("dependencies", {}).keys():
        raise RuntimeError("candidate dependencies already adopted; remove this one-off generator")
    # This known manifest currently has a single final [dependencies] section.
    text = manifest.read_text(encoding="utf-8")
    updated = text + DEPENDENCIES
    parsed = tomllib.loads(updated)
    for name, version in PINS.items():
        entry = parsed["dependencies"][name]
        actual = entry if isinstance(entry, str) else entry["version"]
        if actual != "=" + version:
            raise RuntimeError("dependency was not inserted in the dependencies table")
    manifest.write_text(updated, encoding="utf-8", newline="\n")
    examples = manifest.parent / "examples"
    examples.mkdir(exist_ok=True)
    shutil.copyfile(ROOT / "tools/t068_https_probe.rs", examples / "t068_https_probe.rs")
    baseline = digest(ROOT / "Cargo.lock")
    (destination / "baseline-lock.sha256").write_text(baseline + "\n", encoding="ascii")
    with Path(os.environ["GITHUB_ENV"]).open("a", encoding="utf-8") as output:
        output.write(f"T068_CANDIDATE={destination.as_posix()}\n")
    print("PULQVA_T068_CANDIDATE_COPY_READY")

def verify(destination: Path) -> None:
    destination = destination.resolve(strict=True)
    runner = Path(os.environ["RUNNER_TEMP"]).resolve(strict=True)
    if not destination.is_relative_to(runner) or not destination.name.startswith("pulqva-t068-https-"):
        raise RuntimeError("not an owned CI candidate directory")
    if digest(ROOT / "Cargo.lock") != (destination / "baseline-lock.sha256").read_text().strip():
        raise RuntimeError("shipping lock changed during candidate trial")
    before = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    after = tomllib.loads((destination / "Cargo.lock").read_text(encoding="utf-8"))
    identities = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in after["package"]}
    for package in before["package"]:
        if "source" in package:
            key = (package["name"], package["version"], package["source"], package.get("checksum"))
            if key not in identities:
                raise RuntimeError(f"existing dependency identity changed: {package['name']}")
    for name, version in PINS.items():
        if not any(p["name"] == name and p["version"] == version for p in after["package"]):
            raise RuntimeError(f"candidate pin missing: {name}")
    lock = destination / "Cargo.lock"
    receipt = {"scope": "candidate graph only; not live E2E or shipping adoption",
               "checkout_sha": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
               "runner_os": os.environ.get("RUNNER_OS"), "pins": PINS,
               "baseline_sha256": digest(ROOT / "Cargo.lock"), "lock_sha256": digest(lock),
               "lock_bytes": lock.stat().st_size, "existing_registry_identities_preserved": True}
    (destination / "candidate-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print("PULQVA_T068_CARGO_LOCK " + json.dumps(receipt, sort_keys=True))

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("operation", choices=["prepare", "verify"])
    args = parser.parse_args()
    if args.operation == "prepare":
        prepare()
    else:
        verify(Path(os.environ["T068_CANDIDATE"]))
