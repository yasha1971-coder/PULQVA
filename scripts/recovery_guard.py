#!/usr/bin/env python3
"""Check the sealed continuation envelope. No network, mutation or test reruns."""
from __future__ import annotations
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import sys
import zipfile


def strict_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON key: ' + key)
        result[key] = value
    return result


def load(path):
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=strict_object)


def owned(root, name):
    rel = PurePosixPath(name)
    if not name or rel.is_absolute() or '..' in rel.parts or '\\' in name:
        raise ValueError('unsafe recovery path')
    path = root.joinpath(*rel.parts)
    if any(p.is_symlink() for p in [path, *path.parents] if p != root.parent):
        raise ValueError('symlink in recovery path')
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('recovery path escapes root')
    return path


def blob(data):
    return hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.DEVNULL,
                                   timeout=20).decode().strip()


def validate(root: Path, check_git: bool = True):
    """Raise on inconsistent/absent evidence; return the only declared next action.

    check_git=False is for isolated synthetic unit fixtures only, not acceptance.
    This verifies internal consistency/bytes, not the truth of every historical claim.
    """
    root = root.resolve()
    index = load(root / 'recovery/INDEX.json')
    if index['schema'] != 1 or index['repository'] != 'yasha1971-coder/PULQVA':
        raise ValueError('unknown recovery contract')
    if index['branch'] != 'feat/T069-intent-contract' or index['pr'] != 80:
        raise ValueError('unexpected development route; explicit handoff required')
    sealed = index['sealed_files']
    for required in ('PROJECT_STATE.json', 'NEXT.md', 'kernel/CORE_CONTRACT.md',
                     'kernel/PRIVACY_INVARIANTS.md', 'kernel/UX_CONTRACT.md',
                     'kernel/KERNEL_VERSION', 'decisions/INDEX.json'):
        if required not in sealed:
            raise ValueError('unsealed mandatory file: ' + required)
    for name, expected in sealed.items():
        if not re.fullmatch('[0-9a-f]{40}', expected) or blob(owned(root, name).read_bytes()) != expected:
            raise ValueError('sealed file changed: ' + name)
    state = load(root / 'PROJECT_STATE.json')
    nxt = (root / 'NEXT.md').read_text(encoding='utf-8')
    for key in ('active_task', 'next_task', 'last_completed_subtask', 'next_subtask',
                'last_verified_commit', 'execution_mode'):
        if state.get(key) != index['state_contract'][key]:
            raise ValueError('state contract mismatch: ' + key)
    if 'ONE NEXT ACTION' not in nxt or state['next_subtask'] not in nxt:
        raise ValueError('missing next subtask/action')
    if not state.get('known_blockers') or 'scope_note' not in state:
        raise ValueError('scope or blockers lost')
    if state['execution_mode'] != 'interactive_only_owner_cancelled_autonomy':
        raise ValueError('autonomy must remain cancelled')
    evidence = state['verification_context']
    for key, value in index['g1_contract'].items():
        if evidence.get(key) != value:
            raise ValueError('G1 evidence mismatch: ' + key)
    adr = load(root / 'decisions/INDEX.json')
    ids = [entry['id'] for entry in adr['decisions']]
    paths = [entry['path'] for entry in adr['decisions']]
    if len(set(ids)) != len(ids) or len(set(paths)) != len(paths):
        raise ValueError('duplicate ADR id/path')
    actual = sorted(str(p.relative_to(root)) for p in (root / 'decisions').glob('ADR-*.md'))
    if actual != sorted(paths):
        raise ValueError('ADR inventory differs from index')
    for entry in adr['decisions']:
        if not Path(entry['path']).name.startswith(entry['id'] + '-'):
            raise ValueError('ADR id/path mismatch')
        if blob(owned(root, entry['path']).read_bytes()) != entry['blob']:
            raise ValueError('ADR bytes changed without index update')
    if not any(a['legacy_id'] == 'ADR-0007' and a['canonical_id'] == 'ADR-0009'
               and a.get('source_commit') for a in adr['legacy_aliases']):
        raise ValueError('Windows ADR alias lost')
    archive = index['model_archive']
    data = owned(root, archive['path']).read_bytes()
    if hashlib.sha256(data).hexdigest() != archive['sha256']:
        raise ValueError('model archive digest mismatch')
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        if len(z.namelist()) != len(set(z.namelist())) or sorted(z.namelist()) != sorted(archive['members']):
            raise ValueError('archive members missing/duplicated')
        for name, expected in archive['members'].items():
            info = z.getinfo(name)
            if info.file_size > 1024 * 1024 or hashlib.sha256(z.read(name)).hexdigest() != expected:
                raise ValueError('archive member mismatch')
        receipt = json.loads(z.read('receipt.json'), object_pairs_hook=strict_object)
        cases = receipt['cases']
        if receipt.get('protocol') != 't069f-real-probe-matrix-v2' or receipt.get('all_ok') is not True:
            raise ValueError('historical model receipt differs')
        if [c['id'] for c in cases] != archive['case_ids'] or not all(c.get('ok') is True for c in cases):
            raise ValueError('historical model case inventory differs')
        if 'T069F_FEATURE_HEAD=' + archive['source_commit'] not in z.read('provenance.txt').decode():
            raise ValueError('model source mismatch')
    if state['historical_model_evidence']['source_head'] != archive['source_commit']:
        raise ValueError('state points at different model evidence')
    if check_git:
        for name, expected in index['product_trees'].items():
            if git(root, 'rev-parse', 'HEAD:' + name) != expected:
                raise ValueError('product tree changed; record pending generation: ' + name)
        for sha in (index['source_snapshot'], evidence['implementation_commit']):
            git(root, 'cat-file', '-e', sha + '^{commit}')
        if git(root, 'rev-parse', evidence['implementation_commit'] + '^{tree}') != evidence['checkout_tree']:
            raise ValueError('G1 implementation tree mismatch')
        if git(root, 'rev-parse', 'HEAD:kernel') != index['kernel_tree']:
            raise ValueError('kernel changed')
    return {'repository': index['repository'], 'branch': index['branch'], 'pr': index['pr'],
            'last_completed_subtask': state['last_completed_subtask'],
            'next_subtask': state['next_subtask'], 'next_action': index['next_action'],
            'mode': state['execution_mode'], 'scope': 'continuity only; not product acceptance'}


if __name__ == '__main__':
    try:
        answer = validate(Path(__file__).resolve().parents[1])
        print(json.dumps(answer, ensure_ascii=False, indent=2))
        print('PULQVA recovery guard: PASS')
    except (KeyError, TypeError, ValueError, OSError, subprocess.SubprocessError, zipfile.BadZipFile) as exc:
        print('PULQVA recovery guard: FAIL:', str(exc), file=sys.stderr)
        sys.exit(1)
