#!/usr/bin/env python3
"""CI-only artifact verification; does not wire Deno into PULQVA."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import stat
import subprocess
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]
MAX_ARCHIVE = 64 * 1024 * 1024
MAX_EXECUTABLE = 256 * 1024 * 1024


def unpack_verified(raw, asset, expected_name, directory):
    if len(raw) != asset["bytes"] or hashlib.sha256(raw).hexdigest() != asset["sha256"]:
        raise ValueError("archive identity mismatch")
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        entries = archive.infolist()
        if len(entries) != 1:
            raise ValueError("expected exactly one executable")
        info = entries[0]
        mode = info.external_attr >> 16
        if (info.filename != expected_name or info.orig_filename != expected_name
                or info.is_dir() or info.flag_bits & 1
                or stat.S_IFMT(mode) not in (0, stat.S_IFREG)
                or info.external_attr & 0x10
                or not 0 < info.file_size <= MAX_EXECUTABLE):
            raise ValueError("unsafe or unexpected archive entry")
        destination = directory / expected_name
        digest = hashlib.sha256()
        count = 0
        try:
            with archive.open(info) as source, destination.open("xb") as output:
                while chunk := source.read(1024 * 1024):
                    count += len(chunk)
                    if count > MAX_EXECUTABLE or count > info.file_size:
                        raise ValueError("expanded size exceeds bound")
                    digest.update(chunk)
                    output.write(chunk)
            if count != info.file_size:
                raise ValueError("expanded size mismatch")
        except Exception:
            # Directory is owned by this CI invocation.
            destination.unlink(missing_ok=True)
            raise
    if os.name != "nt":
        destination.chmod(0o700)
    return destination, {"bytes": count, "sha256": digest.hexdigest()}


def clean_environment(directory):
    # DENO_NO_UPDATE_CHECK documented at:
    # https://docs.deno.com/runtime/reference/env_variables/
    env = {key: value for key, value in os.environ.items()
           if key.upper() in ("SYSTEMROOT", "WINDIR")}
    env.update({
        "DENO_NO_UPDATE_CHECK": "1", "DENO_NO_PROMPT": "1",
        "DENO_DIR": str(directory / "cache"), "NO_COLOR": "1",
        "HOME": str(directory), "USERPROFILE": str(directory),
        "TMP": str(directory), "TEMP": str(directory), "TMPDIR": str(directory),
    })
    return env


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True,
                        choices=["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"])
    parser.add_argument("--receipt", required=True, type=Path)
    args = parser.parse_args()
    manifest = json.loads((ROOT / "sidecars/deno/CANDIDATE.json").read_text())
    asset = next(a for a in manifest["assets"] if a["target"] == args.target)
    windows = args.target.endswith("windows-msvc")
    if windows != (os.name == "nt") or platform.machine().lower() not in ("amd64", "x86_64"):
        raise ValueError("runner does not match candidate target")
    url = "https://github.com/denoland/deno/releases/download/v" + manifest["runtime"]["version"] + "/" + asset["name"]
    if asset["url"] != url or not 0 < asset["bytes"] <= MAX_ARCHIVE:
        raise ValueError("unexpected candidate URL or size")
    # CI acquisition only; no application traffic or runtime module downloads.
    with urllib.request.urlopen(url, timeout=45) as response:
        raw = response.read(MAX_ARCHIVE + 1)
    with tempfile.TemporaryDirectory(prefix="pulqva-deno-") as temp:
        directory = Path(temp).resolve()
        executable, identity = unpack_verified(raw, asset, "deno.exe" if windows else "deno", directory)
        version = subprocess.run([str(executable), "--version"], cwd=directory,
                                 env=clean_environment(directory), stdin=subprocess.DEVNULL,
                                 capture_output=True, text=True, timeout=20, check=True)
        first = version.stdout.splitlines()[0].split()
        if first[:2] != ["deno", manifest["runtime"]["version"]]:
            raise ValueError("runtime version mismatch")
        receipt = {
            "schema_version": 1, "target": args.target,
            "archive": {"name": asset["name"], "bytes": len(raw),
                        "sha256": hashlib.sha256(raw).hexdigest()},
            "executable": identity, "version_output": version.stdout.strip(),
            "observed_platform": platform.platform(), "libc": platform.libc_ver(),
            "scope": "local --version only; not permission isolation or YouTube compatibility",
        }
        args.receipt.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
        print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()
