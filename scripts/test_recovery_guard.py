#!/usr/bin/env python3
"""Independent recovery-contract probes against one unchanged validator."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import time
from recovery_guard import blob, load, validate

ROOT = Path(__file__).resolve().parents[1]


def save(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def seal(root, name):
    index = load(root / 'recovery/INDEX.json')
    index['sealed_files'][name] = blob((root / name).read_bytes())
    save(root / 'recovery/INDEX.json', index)


def fixture(root):
    index = load(ROOT / 'recovery/INDEX.json')
    state = dict(index['state_contract'])
    state.update(known_blockers=['synthetic fixture only'], scope_note='not product evidence',
                 verification_context=index['g1_contract'],
                 historical_model_evidence={'source_head': index['model_archive']['source_commit']})
    save(root / 'PROJECT_STATE.json', state)
    (root / 'NEXT.md').write_text('ONE NEXT ACTION: ' + state['next_subtask'])
    for name in index['sealed_files']:
        p = root / name
        if not p.exists():
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text('synthetic ' + name)
    adr = load(ROOT / 'decisions/INDEX.json')
    for row in adr['decisions']:
        p = root / row['path']
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text('# ' + row['id'] + '\nSynthetic fixture, not an ADR.\n')
        row['blob'] = blob(p.read_bytes())
    save(root / 'decisions/INDEX.json', adr)
    dst = root / index['model_archive']['path']
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(ROOT / index['model_archive']['path'], dst)
    for name in index['sealed_files']:
        index['sealed_files'][name] = blob((root / name).read_bytes())
    save(root / 'recovery/INDEX.json', index)


def mutate_state(root, key, value):
    p = root / 'PROJECT_STATE.json'
    state = load(p)
    state[key] = value
    save(p, state)
    seal(root, 'PROJECT_STATE.json')


def duplicate_adr(root):
    p = root / 'decisions/INDEX.json'
    data = load(p)
    data['decisions'].append(data['decisions'][0].copy())
    save(p, data)
    seal(root, 'decisions/INDEX.json')


def mismatched_g1(root):
    p = root / 'PROJECT_STATE.json'
    data = load(p)
    data['verification_context']['implementation_commit'] = '0' * 40
    save(p, data)
    seal(root, 'PROJECT_STATE.json')


def unsafe_path(root):
    p = root / 'recovery/INDEX.json'
    data = load(p)
    data['sealed_files']['../outside'] = '0' * 40
    save(p, data)


def wrong_provenance(root):
    p = root / 'recovery/INDEX.json'
    data = load(p)
    data['model_archive']['source_commit'] = '0' * 40
    save(p, data)


CASES = [
    ('REC-01', 'A complete consistent envelope identifies G2 without chat', lambda r: None, True),
    ('REC-02', 'Unsealed state changes are detected', lambda r: (r/'PROJECT_STATE.json').write_text('{}'), False),
    ('REC-03', 'Logical task drift fails even after rehash', lambda r: mutate_state(r, 'next_subtask', 'T068'), False),
    ('REC-04', 'A new duplicate ADR id is rejected', duplicate_adr, False),
    ('REC-05', 'A missing accepted archive fails', lambda r: (r/'recovery/evidence/t069f-attempt9.zip').unlink(), False),
    ('REC-06', 'Corrupt archive bytes fail', lambda r: (r/'recovery/evidence/t069f-attempt9.zip').write_bytes(b'corrupt'), False),
    ('REC-07', 'Recovery paths cannot escape root', unsafe_path, False),
    ('REC-08', 'G1 cannot be rebound to another commit', mismatched_g1, False),
    ('REC-09', 'Cancelled autonomy cannot silently become enabled', lambda r: mutate_state(r, 'execution_mode', 'autonomous'), False),
    ('REC-10', 'An unindexed ADR is rejected', lambda r: (r/'decisions/ADR-9999-extra.md').write_text('extra'), False),
    ('REC-11', 'Duplicate JSON keys are rejected', lambda r: (r/'recovery/INDEX.json').write_text('{"schema":1,"schema":1}'), False),
    ('REC-12', 'Changed model provenance is rejected', wrong_provenance, False),
    ('REC-13', 'Discarded blockers are rejected', lambda r: mutate_state(r, 'known_blockers', []), False),
    ('REC-14', 'A missing kernel contract fails', lambda r: (r/'kernel/CORE_CONTRACT.md').unlink(), False),
]


def main():
    rows = []
    for probe, hypothesis, mutation, expected in CASES:
        start = time.monotonic_ns()
        result, detail = 'ERROR', None
        try:
            with tempfile.TemporaryDirectory(prefix='pulqva-recovery-probe-') as temp:
                root = Path(temp)
                fixture(root)
                mutation(root)
                try:
                    observed = validate(root, check_git=False)
                    accepted = observed['next_subtask'] == 'T069-G2'
                except (ValueError, KeyError, TypeError, OSError):
                    accepted = False
                result = 'PASS' if accepted == expected else 'FAIL'
                detail = {'accepted': accepted}
        except Exception as exc:
            detail = {'error_type': type(exc).__name__}
        rows.append({'id': probe, 'hypothesis': hypothesis,
                     'invariant_set': ['same validator source', 'fresh owned fixture', 'no network or model', 'no hot fixes'],
                     'result': result, 'duration_ms': (time.monotonic_ns()-start)/1e6,
                     'expected': {'accepted': expected}, 'observed': detail})
    output = {'protocol': 'pulqva-continuity-probe-matrix-v1', 'boundary': 'recovery validator, synthetic fixtures',
              'generation': 'fixed validator source digests below',
              'source_sha256': {n: hashlib.sha256((ROOT/'scripts'/n).read_bytes()).hexdigest()
                                for n in ('recovery_guard.py', 'test_recovery_guard.py')},
              'isolation': 'one fresh TemporaryDirectory per probe; shared read-only input ZIP; no shared mutable runtime',
              'probes': rows, 'all_ok': all(r['result']=='PASS' for r in rows),
              'limitations': ['No Git ancestry in these synthetic cases; real bundle restore checks that separately.',
                              'Not the real-model matrix implementation of T069-G2.']}
    if len(sys.argv) > 1:
        save(Path(sys.argv[1]), output)
    print(json.dumps(output, indent=2))
    return 0 if output['all_ok'] else 1


if __name__ == '__main__':
    sys.exit(main())
