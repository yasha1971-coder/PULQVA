"""CI only: pinned Deno download and independent ZIP hash, never execute Deno."""
import hashlib
import json
import os
from pathlib import Path
import stat
import tempfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[3]


def main():
    target, member = {
        "Linux": ("x86_64-unknown-linux-gnu", "deno"),
        "Windows": ("x86_64-pc-windows-msvc", "deno.exe"),
    }[os.environ["RUNNER_OS"]]
    candidate = json.loads((ROOT / "sidecars/deno/CANDIDATE.json").read_text())
    proof = json.loads((ROOT / "sidecars/deno/ARTIFACT_VERIFICATION.json").read_text())
    assert candidate["runtime"]["version"] == "2.9.7"
    assets = [x for x in candidate["assets"] if x["target"] == target]
    receipts = [x for x in proof["receipts"] if x["target"] == target]
    assert len(assets) == len(receipts) == 1
    asset, expected = assets[0], receipts[0]
    assert expected["archive"] == {k: asset[k] for k in ("name", "bytes", "sha256")}
    assert asset["url"] == f'https://github.com/denoland/deno/releases/download/v2.9.7/{asset["name"]}'
    assert 0 < asset["bytes"] <= 64 * 1024 * 1024
    directory = Path(tempfile.mkdtemp(prefix="pulqva-deno-reference-", dir=os.environ["RUNNER_TEMP"]))
    archive_path = directory / asset["name"]
    archive_hash, size = hashlib.sha256(), 0
    with urllib.request.urlopen(asset["url"], timeout=90) as source, archive_path.open("xb") as output:
        while chunk := source.read(1024 * 1024):
            size += len(chunk)
            assert size <= asset["bytes"]
            output.write(chunk)
            archive_hash.update(chunk)
    assert size == asset["bytes"] and archive_hash.hexdigest() == asset["sha256"]
    executable_hash, executable_size = hashlib.sha256(), 0
    with zipfile.ZipFile(archive_path) as archive:
        entries = archive.infolist()
        assert len(entries) == 1
        entry = entries[0]
        assert entry.filename == entry.orig_filename == member
        assert not entry.is_dir() and not entry.flag_bits & 1
        assert stat.S_IFMT(entry.external_attr >> 16) in (0, stat.S_IFREG)
        assert not entry.external_attr & 0x10
        assert entry.file_size == expected["executable"]["bytes"] <= 256 * 1024 * 1024
        with archive.open(entry) as source:
            while chunk := source.read(1024 * 1024):
                executable_size += len(chunk)
                assert executable_size <= entry.file_size
                executable_hash.update(chunk)
    assert executable_size == expected["executable"]["bytes"]
    assert executable_hash.hexdigest() == expected["executable"]["sha256"]
    receipt = {"target": target, "version": "2.9.7", "member": member,
               "archive_sha256": archive_hash.hexdigest(), "archive_size": size,
               "executable_sha256": executable_hash.hexdigest(), "executable_size": executable_size}
    (Path(os.environ["RUNNER_TEMP"]) / "deno-compat-reference.json").write_text(json.dumps(receipt, indent=2))
    with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as env:
        for key, value in {"PULQVA_DENO_FIXTURE": archive_path,
                           "PULQVA_DENO_REFERENCE_SHA256": executable_hash.hexdigest(),
                           "PULQVA_DENO_REFERENCE_SIZE": executable_size}.items():
            env.write(f"{key}={value}\n")
    print("PULQVA_DENO_REFERENCE " + json.dumps(receipt, sort_keys=True))


if __name__ == "__main__":
    main()
