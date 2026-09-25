#!/usr/bin/env python3
"""Read pinned PyInstaller data only; never execute the inspected binary.

Format reference: PyInstaller v6.22.0 PyInstaller/archive/readers.py.
This assessment utility accepts only the two hash-pinned official assets.
"""
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
PINNED = {
    "yt-dlp_linux": "58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a",
    "yt-dlp.exe": "66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a",
}


def inspect(path, asset):
    if path.stat().st_size > 64 * 1024 * 1024:
        raise ValueError("asset exceeds inspection bound")
    data = path.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    if digest != PINNED[asset]:
        raise ValueError("official asset hash mismatch")
    cookie = struct.Struct("!8sIIII64s")
    pos = data.rfind(b"MEI\x0c\x0b\x0a\x0b\x0e")
    if pos < 0:
        raise ValueError("missing CArchive cookie")
    _, length, offset, size, _, _ = cookie.unpack_from(data, pos)
    start = pos + cookie.size - length
    if not (0 <= start <= start + offset <= start + offset + size <= pos):
        raise ValueError("invalid CArchive bounds")
    toc = data[start + offset:start + offset + size]
    header = struct.Struct("!IIIIBc")
    scripts = json.loads((ROOT / "sidecars/deno/CANDIDATE.json").read_text())["ejs"]["scripts"]
    expected = {item["name"]: item["sha256"] for item in scripts}
    found = {}
    cursor = 0
    while cursor < len(toc):
        n, off, packed, unpacked, compressed, kind = header.unpack_from(toc, cursor)
        if n <= header.size or cursor + n > len(toc):
            raise ValueError("invalid TOC entry")
        name = toc[cursor + header.size:cursor + n].rstrip(b"\0").decode().replace("\\", "/")
        cursor += n
        leaf = "yt.solver." + name.rsplit("/", 1)[-1]
        if not name.startswith("yt_dlp_ejs/yt/solver/") or leaf not in expected:
            continue
        if leaf in found or kind not in (b"x", b"b") or compressed not in (0, 1):
            raise ValueError("ambiguous or unsupported EJS entry")
        if not (0 <= off <= off + packed <= offset) or unpacked > 1024 * 1024:
            raise ValueError("EJS entry exceeds bounds")
        payload = data[start + off:start + off + packed]
        if compressed:
            dec = zlib.decompressobj()
            payload = dec.decompress(payload, unpacked + 1)
            if not dec.eof or dec.unused_data or dec.unconsumed_tail:
                raise ValueError("invalid compressed EJS entry")
        actual = hashlib.sha256(payload).hexdigest()
        if len(payload) != unpacked or actual != expected[leaf]:
            raise ValueError("EJS content mismatch")
        found[leaf] = {"archive_path": name, "bytes": len(payload), "sha256": actual}
    if set(found) != set(expected):
        raise ValueError(f"missing expected EJS scripts; found {sorted(found)}")
    return {"asset": asset, "bytes": len(data), "sha256": digest,
            "ejs_release": "0.8.0", "scripts": found, "binary_executed": False}


if __name__ == "__main__":
    print(json.dumps(inspect(Path(sys.argv[1]), sys.argv[2]), indent=2))
