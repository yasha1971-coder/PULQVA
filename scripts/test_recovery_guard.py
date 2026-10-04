#!/usr/bin/env python3
"""Independent recovery-contract probes against one unchanged validator."""
from __future__ import annotations
import hashlib
import os
import platform
import subprocess
import uuid
import json
from pathlib import Path
import shutil
import sys
import tempfile
import time
from recovery_guard import blob, load, validate

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools/ci'))
from matrix_receipt import MatrixRecorder, canonical, write_exclusive, verify


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
    declared = load(ROOT / 'PROJECT_STATE.json')
    if 'execution_authorization' in declared:
        state['execution_authorization'] = declared['execution_authorization']
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


def authority_change(root, key, value):
    p = root / 'PROJECT_STATE.json'
    state = load(p)
    state['execution_authorization'][key] = value
    save(p, state)
    seal(root, 'PROJECT_STATE.json')


def remove_authority(root):
    p = root / 'PROJECT_STATE.json'
    state = load(p)
    state.pop('execution_authorization', None)
    save(p, state)
    seal(root, 'PROJECT_STATE.json')


def cancelled_mode(root):
    p = root / 'PROJECT_STATE.json'
    state = load(p)
    state['execution_mode'] = 'interactive_only_owner_cancelled_autonomy'
    state['execution_authorization']['status'] = 'revoked'
    save(p, state)
    seal(root, 'PROJECT_STATE.json')
    p = root / 'recovery/INDEX.json'
    index = load(p)
    index['state_contract']['execution_mode'] = state['execution_mode']
    save(p, index)


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
    ('REC-15', 'Scheduled mode without owner record fails even after resealing', remove_authority, False),
    ('REC-16', 'Revoked authority cannot run scheduled mode', lambda r: authority_change(r, 'status', 'revoked'), False),
    ('REC-17', 'Owner stop restores valid cancelled mode without losing history', cancelled_mode, True),
    ('REC-18', 'Expanded execution grants cannot be accepted', lambda r: authority_change(r, 'grants', ['merge']), False),
    ('REC-19', 'Mutating work stays bounded to one task', lambda r: authority_change(r, 'max_mutating_tasks_per_cycle', 2), False),
    ('REC-20', 'Boolean cannot masquerade as an integer budget', lambda r: authority_change(r, 'max_live_generations_per_cycle', True), False),
    ('REC-21', 'Missing owner checkpoint denies authority', lambda r: authority_change(r, 'checkpoint', None), False),
]


def measured_identity():
    checkout = subprocess.check_output(['git', '-C', str(ROOT), 'rev-parse', 'HEAD'],
                                       text=True, timeout=10).strip()
    paths = {'runtime': Path(sys.executable).resolve(),
             'evaluator': ROOT / 'scripts/recovery_guard.py',
             'fixtures': Path(__file__), 'codec': ROOT / 'tools/ci/matrix_receipt.py',
             'state': ROOT / 'PROJECT_STATE.json', 'index': ROOT / 'recovery/INDEX.json',
             'adr_index': ROOT / 'decisions/INDEX.json',
             'archive': ROOT / load(ROOT / 'recovery/INDEX.json')['model_archive']['path']}
    components = {name: {'status': 'captured', 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()}
                  for name, p in paths.items()}
    for name in ('model', 'prompt', 'schema'):
        components[name] = {'status': 'not_applicable', 'sha256': None,
                            'reason': 'synthetic source recovery; no inference'}
    components['flags'] = {'status': 'captured', 'sha256': hashlib.sha256(
        canonical({'check_git': False, 'fresh_fixture': True})).hexdigest()}
    return {'source_sha': os.environ.get('PULQVA_SOURCE_SHA', checkout),
            'checkout_sha': checkout, 'components': components,
            'run_id': os.environ.get('GITHUB_RUN_ID'), 'run_id_reason': 'local run if absent',
            'job_id': None, 'job_id_reason': 'bind external numeric job from Actions job list',
            'attempt': os.environ.get('GITHUB_RUN_ATTEMPT'), 'attempt_reason': 'local run if absent',
            'working_tree_note': 'Prepared overlays are bound by actual component digests, not a future commit.'}


def main():
    destination = Path(sys.argv[1]) if len(sys.argv) > 1 else None
    plan = {'protocol': 'pulqva-evidence-boundary-v1', 'boundary': 'source recovery authority',
            'generation_id': str(uuid.uuid4()), 'identity': measured_identity(),
            'environment': {'os': platform.system(), 'arch': platform.machine(),
                            'toolchain': 'Python ' + platform.python_version()},
            'isolation': {'status': 'verified', 'strategy': 'Fresh TemporaryDirectory and independently loaded fixture per probe; source ZIP read-only.',
                          'evidence_refs': ['recovery-test-results.json']},
            'probes': [{'id': pid, 'hypothesis': hypothesis,
                        'invariant_set': ['same validator and fixtures', 'fresh owned root', 'no network or model', 'no hot fixes'],
                        'expected': {'accepted': expected}, 'depends_on': []}
                       for pid, hypothesis, _, expected in CASES]}
    recorder = MatrixRecorder(plan)
    if destination:
        destination.parent.mkdir(parents=True, exist_ok=True)
        write_exclusive(destination.with_name('recovery-manifest.json'), plan)
    rows = []
    for probe, hypothesis, mutation, expected in CASES:
        if measured_identity() != plan['identity']:
            recorder.fail_closed('frozen_identity_changed')
            break
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
        duration = (time.monotonic_ns()-start)/1e6
        rows.append({'id': probe, 'result': result, 'duration_ms': duration, 'observed': detail})
        recorder.record(probe, result=result, duration_ms=duration, observed=detail,
                        evidence_refs=['recovery-test-results.json#' + probe],
                        reason='synthetic_probe_error' if result=='ERROR' else None)
    raw = {'probes': rows}
    output = recorder.close(measured_identity())
    output['limitations'].extend(['Synthetic probes do not check Git history; source restore checks it separately.',
                                  'Recorded owner authority is not an authenticity signature or live scheduler verification.'])
    if destination:
        raw_path = destination.with_name('recovery-test-results.json')
        write_exclusive(raw_path, raw)
        output['raw_evidence_sha256'] = hashlib.sha256(raw_path.read_bytes()).hexdigest()
        write_exclusive(destination, output)
    print(json.dumps(output, indent=2))
    return 0 if verify(plan, output)=='PASS' else 1


if __name__ == '__main__':
    sys.exit(main())
