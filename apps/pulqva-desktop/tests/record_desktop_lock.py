"""Validate recovery pins and emit the Cargo-generated lock for retrieval.

Temporary text transport complements the CI artifact, not a hand-built lock.
Only public dependency metadata is emitted. Remove after lock is committed.
"""
import base64
import gzip
import hashlib
from pathlib import Path
import tomllib

path = Path(__file__).resolve().parents[1] / "src-tauri/Cargo.lock"
raw = path.read_bytes()
lock = tomllib.loads(raw.decode())
expected = {"tauri": "2.11.6", "tauri-build": "2.6.3",
            "tauri-runtime": "2.11.3", "tauri-runtime-wry": "2.11.4",
            "tauri-macros": "2.6.3", "tauri-codegen": "2.6.3", "tauri-utils": "2.9.3"}
for name, version in expected.items():
    actual = [p["version"] for p in lock["package"] if p["name"] == name]
    if actual != [version]:
        raise ValueError(f"unexpected {name} versions: {actual}")
print("PULQVA_DESKTOP_LOCK_SHA256=" + hashlib.sha256(raw).hexdigest())
print("PULQVA_DESKTOP_LOCK_GZIP_BASE64=" + base64.b64encode(gzip.compress(raw, mtime=0)).decode())
