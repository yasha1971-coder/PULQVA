"""Finite evidence codec for existing CI harnesses; never launches or repairs a runtime.

The verifier requires the independently retained PRE-RUN manifest, not a plan inferred
from results. It checks consistency, not whether a trusted collector told the truth.
Capture/isolation must be wired into each real harness before live acceptance.
"""
from __future__ import annotations

import copy
import hashlib
import json
import math
from pathlib import Path
import re

PROTOCOL = 'pulqva-evidence-boundary-v1'
RESULTS = frozenset(('PASS', 'FAIL', 'SKIP', 'ERROR'))
SPEC_KEYS = ('id', 'hypothesis', 'invariant_set', 'expected', 'depends_on')
MAX_BYTES = 1024 * 1024


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def canonical(value: object) -> bytes:
    """Local format identity, not an implementation of RFC 8785/JCS."""
    data = json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(',', ':'), allow_nan=False).encode('utf-8')
    require(len(data) <= MAX_BYTES, 'evidence document exceeds byte limit')
    return data


def digest(value: object) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def strict_load(path: Path) -> dict:
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'duplicate JSON field')
            result[key] = value
        return result
    with path.open('rb') as stream:
        data = stream.read(MAX_BYTES + 1)
    require(len(data) <= MAX_BYTES, 'evidence document exceeds byte limit')
    def invalid_constant(_):
        raise ValueError('non-finite JSON number')
    value = json.loads(data, object_pairs_hook=pairs, parse_constant=invalid_constant)
    require(isinstance(value, dict), 'expected an evidence object')
    canonical(value)
    return value


def text(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and len(value) <= 4096


def strings(value: object, *, empty: bool = False) -> bool:
    return (isinstance(value, list) and (empty or bool(value))
            and all(text(x) for x in value) and len(set(value)) == len(value))



# A comparison may vary its subject, not its measurement conditions. Collectors
# measure these identities before capture and bind them into final_identity too.
# The codec checks declarations/consistency; it cannot authenticate a dishonest
# collector or infer comparability from an absent comparison declaration.
COMPARISON_CONTROLS = (
    'corpus', 'request', 'expected_output', 'environment', 'toolchain',
    'model_policy', 'flags', 'resource_budget', 'cache_policy',
    'transport_policy', 'isolation', 'metric', 'evaluator',
)


def validate_comparison(comparison: dict, probe_ids: set[str]) -> None:
    """Reject unequal/unknown controls before MatrixRecorder can be constructed.

    Subject hashes may differ by the declared experimental factor. Every other
    condition must be captured and identical. Windows/Linux functional gates are
    separate strata, not a matched performance comparison. Historical single-
    object manifests remain valid but do not gain comparative acceptance.
    """
    require(isinstance(comparison, dict), 'comparison declaration must be an object')
    require(set(comparison) == {'schema', 'question', 'treatment', 'arms'},
            'incomplete or unknown comparison fields')
    require(type(comparison['schema']) is int and comparison['schema'] == 1,
            'unsupported comparison schema')
    require(text(comparison['question']) and text(comparison['treatment']),
            'comparison question/treatment must be predeclared')
    arms = comparison['arms']
    require(isinstance(arms, list) and 2 <= len(arms) <= 16, 'invalid comparison arms')
    names, covered, subjects = set(), set(), set()
    reference = None
    for arm in arms:
        require(isinstance(arm, dict) and set(arm) ==
                {'id', 'subject_sha256', 'controls', 'probe_ids', 'evidence_refs'},
                'incomplete or unknown comparison arm')
        require(text(arm['id']) and arm['id'] not in names, 'duplicate comparison arm')
        names.add(arm['id'])
        subject = arm['subject_sha256']
        require(isinstance(subject, str) and re.fullmatch('[0-9a-f]{64}', subject),
                'missing comparison subject identity')
        subjects.add(subject)
        controls = arm['controls']
        require(isinstance(controls, dict) and set(COMPARISON_CONTROLS) <= set(controls),
                'missing comparison controls')
        for name, value in controls.items():
            require(text(name) and isinstance(value, str) and
                    re.fullmatch('[0-9a-f]{64}', value), 'unknown comparison condition')
        if reference is None:
            reference = controls
        require(controls == reference, 'comparison INVALID: conditions differ; prepare a new generation')
        require(strings(arm['evidence_refs']), 'comparison lacks control evidence references')
        ids = arm['probe_ids']
        require(strings(ids) and set(ids) <= probe_ids and not (set(ids) & covered),
                'comparison probe missing, foreign or multiply assigned')
        covered.update(ids)
    require(len(subjects) >= 2, 'comparison has no different experimental subjects')
    require(covered == probe_ids, 'comparison must cover every declared probe')


def validate_manifest(plan: dict) -> None:
    canonical(plan)
    require(plan.get('protocol') == PROTOCOL, 'unknown matrix protocol')
    for name in ('boundary', 'generation_id'):
        require(text(plan.get(name)), 'missing boundary/generation')
    identity = plan.get('identity')
    require(isinstance(identity, dict), 'missing identity')
    for name in ('source_sha', 'checkout_sha'):
        require(isinstance(identity.get(name), str)
                and re.fullmatch('[0-9a-f]{40}', identity[name]) is not None,
                'source and checkout must be full Git SHAs')
    for name in ('run_id', 'job_id', 'attempt'):
        value = identity.get(name)
        require(name in identity and (value is None or
                (isinstance(value, str) and re.fullmatch('[0-9]{1,20}', value))),
                'invalid or absent run provenance')
        if value is None:
            require(text(identity.get(name + '_reason')), 'missing provenance reason')
    components = identity.get('components')
    require(isinstance(components, dict) and bool(components), 'missing frozen components')
    for name in ('runtime', 'model', 'prompt', 'schema', 'flags', 'fixtures', 'evaluator'):
        require(name in components, 'missing component identity')
    for name, value in components.items():
        require(text(name) and isinstance(value, dict), 'invalid component')
        status = value.get('status')
        require(status in ('captured', 'unavailable', 'not_applicable'), 'invalid digest status')
        if name in ('runtime', 'evaluator'):
            require(status != 'not_applicable', 'runtime/evaluator identity is required')
        sha = value.get('sha256')
        if status == 'captured':
            require(isinstance(sha, str) and re.fullmatch('[0-9a-f]{64}', sha),
                    'invalid captured digest')
        else:
            require(sha is None and text(value.get('reason')), 'missing digest reason')
    environment = plan.get('environment')
    require(isinstance(environment, dict) and
            all(text(environment.get(x)) for x in ('os', 'arch', 'toolchain')),
            'missing execution environment')
    isolation = plan.get('isolation')
    require(isinstance(isolation, dict) and
            isolation.get('status') in ('verified', 'unverified') and
            text(isolation.get('strategy')) and
            strings(isolation.get('evidence_refs'), empty=True), 'invalid isolation record')
    if isolation['status'] == 'verified':
        require(bool(isolation['evidence_refs']), 'verified isolation lacks evidence')
    else:
        require(text(isolation.get('reason')), 'unverified isolation lacks reason')
    probes = plan.get('probes')
    require(isinstance(probes, list) and 0 < len(probes) <= 256, 'invalid probe count')
    seen = set()
    for probe in probes:
        require(isinstance(probe, dict) and all(k in probe for k in SPEC_KEYS),
                'incomplete probe declaration')
        pid = probe['id']
        require(isinstance(pid, str) and re.fullmatch('[A-Za-z0-9_-]{1,80}', pid)
                and pid not in seen, 'duplicate/invalid probe ID')
        require(text(probe['hypothesis']) and strings(probe['invariant_set']),
                'missing hypothesis/invariants')
        require(strings(probe['depends_on'], empty=True) and
                set(probe['depends_on']) <= seen, 'dependency missing or out of order')
        require(probe['expected'] is not None, 'missing expected outcome')
        seen.add(pid)
    if 'comparison' in plan:
        validate_comparison(plan['comparison'], seen)


def aggregate(plan: dict, rows: list[dict], *, intact: bool, closed: bool) -> str:
    if not intact or any(row['result'] == 'ERROR' for row in rows):
        return 'ERROR'
    if any(row['result'] == 'FAIL' for row in rows):
        return 'FAIL'
    unavailable = any(item['status'] == 'unavailable'
                      for item in plan['identity']['components'].values())
    if (not closed or unavailable or plan['isolation']['status'] != 'verified'
            or any(row['result'] != 'PASS' for row in rows)):
        return 'INCOMPLETE'
    return 'PASS'


def verify(plan: dict, receipt: dict) -> str:
    """Reject altered criteria/provenance and recompute acceptance independently."""
    validate_manifest(plan)
    canonical(receipt)
    require(receipt.get('protocol') == PROTOCOL, 'unknown receipt protocol')
    require(receipt.get('manifest_sha256') == digest(plan), 'wrong frozen manifest')
    for name in ('boundary', 'generation_id', 'identity', 'environment', 'isolation'):
        require(receipt.get(name) == plan[name], 'receipt provenance differs from plan')
    require(type(receipt.get('closed')) is bool and type(receipt.get('intact')) is bool,
            'invalid boundary state')
    if not receipt['intact']:
        require(text(receipt.get('integrity_reason')), 'missing integrity failure reason')
    require(strings(receipt.get('limitations')), 'missing scope limitations')
    if 'comparison' in plan:
        require(receipt.get('comparison_sha256') == digest(plan['comparison']) and
                receipt.get('comparison_status') == 'MATCHED', 'missing comparison binding')
    else:
        require('comparison_sha256' not in receipt and 'comparison_status' not in receipt,
                'single-object evidence cannot assert comparison acceptance')
    rows = receipt.get('probes')
    require(isinstance(rows, list) and len(rows) == len(plan['probes']),
            'missing or extra planned probes')
    seen = {}
    for spec, row in zip(plan['probes'], rows):
        require(isinstance(row, dict) and all(row.get(k) == spec[k] for k in SPEC_KEYS),
                'probe order, hypothesis or invariant changed')
        result, duration = row.get('result'), row.get('duration_ms')
        require(result in RESULTS, 'invalid probe outcome')
        require('duration_ms' in row and 'observed' in row, 'missing outcome fields')
        require(strings(row.get('evidence_refs'), empty=True), 'invalid evidence references')
        if duration is None:
            require(text(row.get('timing_reason')) and result in ('SKIP', 'ERROR'),
                    'untimed probe cannot establish PASS or FAIL')
        else:
            require(type(duration) in (int, float) and math.isfinite(duration)
                    and duration >= 0, 'invalid duration')
        if result in ('SKIP', 'ERROR'):
            require(text(row.get('reason')), 'missing skip/error reason')
        if result in ('PASS', 'FAIL'):
            require(row['observed'] is not None and bool(row['evidence_refs']),
                    'executed outcome lacks observations/evidence')
        if any(seen[dep] != 'PASS' for dep in spec['depends_on']):
            require(result == 'SKIP', 'dependent probe ran after unmet prerequisite')
        seen[spec['id']] = result
    expected = aggregate(plan, rows, intact=receipt['intact'], closed=receipt['closed'])
    require(receipt.get('overall') == expected and
            type(receipt.get('accepted')) is bool and
            receipt['accepted'] == (expected == 'PASS'), 'incorrect aggregate acceptance')
    return expected


class MatrixRecorder:
    """One manifest, one write per probe, one close. No execution, retry or repair API."""
    def __init__(self, manifest: dict):
        validate_manifest(manifest)
        self._plan = copy.deepcopy(manifest)
        self._rows = {}
        self._closed = False
        self._intact = True
        self._integrity_reason = None

    def manifest(self) -> dict:
        return copy.deepcopy(self._plan)

    def record(self, pid: str, *, result: str, duration_ms: float | None,
               observed: object, evidence_refs: list[str], reason: str | None = None,
               timing_reason: str | None = None) -> None:
        require(not self._closed and self._intact, 'closed/unsafe boundary')
        require(pid not in self._rows, 'probe result already recorded')
        spec = next((p for p in self._plan['probes'] if p['id'] == pid), None)
        require(spec is not None, 'probe was not predeclared')
        next_id = next(p['id'] for p in self._plan['probes'] if p['id'] not in self._rows)
        require(pid == next_id, 'probe execution order differs from manifest')
        if any(self._rows.get(dep, {}).get('result') != 'PASS' for dep in spec['depends_on']):
            require(result == 'SKIP', 'unmet prerequisite')
        row = dict(copy.deepcopy(spec), result=result, duration_ms=duration_ms,
                   observed=copy.deepcopy(observed), evidence_refs=list(evidence_refs),
                   reason=reason, timing_reason=timing_reason)
        candidate = self._snapshot(extra=(pid, row))
        verify(self._plan, candidate)
        self._rows[pid] = row

    def fail_closed(self, reason: str) -> None:
        require(not self._closed and text(reason), 'invalid integrity stop')
        self._intact = False
        self._integrity_reason = reason

    def _snapshot(self, extra=None) -> dict:
        recorded = dict(self._rows)
        if extra is not None:
            recorded[extra[0]] = extra[1]
        rows = []
        for spec in self._plan['probes']:
            row = recorded.get(spec['id'])
            if row is None:
                row = dict(spec, result='SKIP', duration_ms=None, observed=None,
                           evidence_refs=[], reason=self._integrity_reason or 'not_executed',
                           timing_reason='probe_not_started')
            rows.append(copy.deepcopy(row))
        doc = {k: copy.deepcopy(self._plan[k]) for k in
               ('boundary', 'generation_id', 'identity', 'environment', 'isolation')}
        overall = aggregate(self._plan, rows, intact=self._intact, closed=self._closed)
        doc.update(protocol=PROTOCOL, manifest_sha256=digest(self._plan), probes=rows,
                   closed=self._closed, intact=self._intact, integrity_reason=self._integrity_reason,
                   overall=overall, accepted=overall == 'PASS',
                   limitations=['Consistency only; collector must verify actual bytes and isolation.',
                                'Probe PASS does not itself confirm the underlying hypothesis.',
                                'Not whole-product acceptance or an authenticity signature.'])
        if 'comparison' in self._plan:
            doc['comparison_sha256'] = digest(self._plan['comparison'])
            doc['comparison_status'] = 'MATCHED'
        return doc

    def close(self, final_identity: dict, *, early_exit_reason: str = 'not_executed') -> dict:
        require(not self._closed and text(early_exit_reason), 'boundary already closed')
        if final_identity != self._plan['identity']:
            self.fail_closed('frozen_identity_changed')
        self._closed = True
        doc = self._snapshot()
        for row in doc['probes']:
            if row['id'] not in self._rows and self._intact:
                row['reason'] = early_exit_reason
        verify(self._plan, doc)
        return doc


def write_exclusive(path: Path, value: dict) -> None:
    """Never replace an older generation. Partial writes remain non-accepting JSON.

    Caller owns the output directory; this is not a hostile-filesystem sandbox.
    """
    data = canonical(value) + b'\n'
    with path.open('xb') as output:
        output.write(data)
        output.flush()


def main() -> int:
    import sys
    try:
        require(len(sys.argv) == 3, 'expected pre-run manifest and receipt paths')
        result = verify(strict_load(Path(sys.argv[1])), strict_load(Path(sys.argv[2])))
        print('PULQVA_MATRIX_' + result)
        return 0 if result == 'PASS' else 1
    except (KeyError, TypeError, ValueError, OSError, OverflowError):
        print('PULQVA_MATRIX_INVALID')  # No raw paths, inputs or error messages.
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
