"""CI only: authenticate pinned archive, independently hash its executable without running it."""
import hashlib
import json
import os
from pathlib import Path
import tarfile
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[3]
platform = {"Windows": "windows-x86_64", "Linux": "linux-x86_64"}[os.environ["RUNNER_OS"]]
proof = json.loads((ROOT / "sidecars/ffmpeg/SOURCE_PROOF.json").read_text())
items = [item for item in proof["artifacts"] if item["platform"] == platform]
assert len(items) == 1
item = items[0]
checksums = [line.split() for line in (ROOT / "sidecars/ffmpeg/SHA256SUMS").read_text().splitlines() if line.strip()]
assert [sha for sha, asset in checksums if asset == item["asset"]] == [item["sha256"]]
assert item["url"].startswith("https://github.com/BtbN/FFmpeg-Builds/releases/download/")
assert 0 < item["size_bytes"] <= 256 * 1024 * 1024
path = Path(os.environ["RUNNER_TEMP"]) / item["asset"]
digest, size = hashlib.sha256(), 0
with urllib.request.urlopen(item["url"], timeout=90) as response, path.open("xb") as target:
    while chunk := response.read(1024 * 1024):
        size += len(chunk)
        assert size <= item["size_bytes"], "archive exceeds pinned size"
        target.write(chunk)
        digest.update(chunk)
assert size == item["size_bytes"] and digest.hexdigest() == item["sha256"], "archive identity mismatch"

output_hash, output_size = hashlib.sha256(), 0
def consume(stream):
    global output_size
    while chunk := stream.read(1024 * 1024):
        output_size += len(chunk)
        assert output_size <= 512 * 1024 * 1024
        output_hash.update(chunk)

if item["asset"].endswith(".zip"):
    member = item["asset"][:-4] + "/bin/ffmpeg.exe"
    with zipfile.ZipFile(path) as archive:
        entries = archive.infolist()
        candidates = [entry for entry in entries if entry.filename == member]
        assert len(candidates) == 1 and not candidates[0].is_dir()
        with archive.open(candidates[0]) as stream:
            consume(stream)
else:
    member = item["asset"][:-7] + "/bin/ffmpeg"
    with tarfile.open(path, "r:xz") as archive:
        entries = archive.getmembers()
        candidates = [entry for entry in entries if entry.name == member]
        assert len(candidates) == 1 and candidates[0].isfile()
        with archive.extractfile(candidates[0]) as stream:
            consume(stream)
assert output_size > 0
receipt = {"platform": platform, "archive_sha256": digest.hexdigest(), "archive_size": size,
           "member": member, "member_count": len(entries), "sha256": output_hash.hexdigest(), "byte_size": output_size}
(Path(os.environ["RUNNER_TEMP"]) / "ffmpeg-compat-reference.json").write_text(json.dumps(receipt, indent=2))
with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as env:
    env.write(f"PULQVA_FFMPEG_FIXTURE={path}\n")
    env.write(f"PULQVA_FFMPEG_REFERENCE_SHA256={output_hash.hexdigest()}\n")
    env.write(f"PULQVA_FFMPEG_REFERENCE_SIZE={output_size}\n")
print("PULQVA_FFMPEG_REFERENCE " + json.dumps(receipt, sort_keys=True))
