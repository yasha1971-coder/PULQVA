"""Independent byte check for the fixed public C2 fixture, not a Tor attestation.

Does not download, repair, rewrite or synthesize receipts. Run before uploading
an artifact. Exactly receipt.json + selected.webm must remain in the bundle.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import stat
import sys

CAP = 8 * 1024 * 1024


def require(condition: bool) -> None:
    if not condition:
        raise ValueError('invalid E2E evidence')


def unique_object(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        require(key not in result)
        result[key] = value
    return result


def integer(value: object, low: int, high: int) -> bool:
    return type(value) is int and low <= value <= high


def verify(bundle: Path, source_sha: str, platform: str = 'linux') -> dict:
    require(re.fullmatch(r'[0-9a-f]{40}', source_sha) is not None)
    require(stat.S_ISDIR(bundle.lstat().st_mode))
    require({item.name for item in bundle.iterdir()} == {'receipt.json', 'selected.webm'})
    receipt_path, media_path = bundle / 'receipt.json', bundle / 'selected.webm'
    for path in (receipt_path, media_path):
        require(stat.S_ISREG(path.lstat().st_mode))
    require(0 < receipt_path.stat().st_size <= 65536)
    # Read caps also apply after stat: evidence is assumed exclusively owned.
    with receipt_path.open('rb') as handle:
        raw = handle.read(65537)
    require(len(raw) <= 65536)
    data = json.loads(raw, object_pairs_hook=unique_object)
    require(isinstance(data, dict))
    require(data.get('schema') == 2 and data.get('source_sha') == source_sha)
    require(platform in {'linux', 'windows'})
    require(data.get('scope') == f'{platform}-live-request-choice-file')
    require(data.get('request') == 'countdown')
    for name in ('core_retrieval_used', 'file_downloaded', 'cancelled_search_rejected', 'stale_results_cleared'):
        require(data.get(name) is True)
    require(data.get('publisher_authenticated') is False)
    require(data.get('windows_e2e_verified') is (platform == 'windows'))
    choices = data.get('choices')
    require(isinstance(choices, list) and 2 <= len(choices) <= 10)
    require(integer(data.get('choice_count'), 2, 10) and data['choice_count'] == len(choices))
    require(type(data.get('selected_index')) is int and data['selected_index'] == 1)
    for index, row in enumerate(choices):
        require(isinstance(row, dict) and type(row.get('index')) is int and row['index'] == index)
    selected, file = choices[1], data.get('file')
    require(isinstance(file, dict))
    require(file.get('path') == 'selected.webm' and type(file.get('selected_index')) is int and file['selected_index'] == 1)
    require(isinstance(selected.get('locator'), str) and data.get('selected_locator') == selected['locator'])
    require(selected['locator'].startswith('https://upload.wikimedia.org/wikipedia/commons/'))
    require(file.get('title') == selected.get('title') and isinstance(file.get('title'), str))
    size = selected.get('declared_size')
    require(integer(size, 1, CAP) and type(file.get('byte_size')) is int and file['byte_size'] == size)
    for value, width in ((selected.get('declared_sha1'), 40), (file.get('sha1'), 40), (file.get('sha256'), 64)):
        require(isinstance(value, str) and re.fullmatch('[0-9a-f]{' + str(width) + '}', value) is not None)
    require(media_path.stat().st_size == size)
    # SHA-1 is only provider compatibility; SHA-256 identifies the saved bytes.
    sha1 = hashlib.sha1(usedforsecurity=False)
    sha256 = hashlib.sha256()
    actual = 0
    with media_path.open('rb') as handle:
        while True:
            chunk = handle.read(min(65536, size + 1 - actual))
            if not chunk:
                break
            actual += len(chunk)
            require(actual <= size)
            sha1.update(chunk)
            sha256.update(chunk)
    require(actual == size)
    require(sha1.hexdigest() == selected['declared_sha1'] == file['sha1'])
    require(sha256.hexdigest() == file['sha256'])
    return {'bytes': actual, 'sha256': sha256.hexdigest()}


def main() -> int:
    try:
        require(len(sys.argv) == 4)
        verify(Path(sys.argv[1]), sys.argv[2], sys.argv[3])
    except (OSError, ValueError, TypeError, KeyError, IndexError, AttributeError):
        print('PULQVA_FILE_EVIDENCE_REJECTED')
        return 1
    print('PULQVA_FILE_EVIDENCE_OK')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
