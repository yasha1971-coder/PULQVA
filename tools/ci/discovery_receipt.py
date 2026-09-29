"""Allowlisted per-OS step evidence; no API calls or raw log/body collection.

Consumes GitHub's step OUTCOME, not conclusion (which can mask allowed failures).
This is not a cross-job collector, a provenance signature, or an E2E file receipt.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
from dataclasses import asdict
from pathlib import Path

from ci_evidence_guard import Verdict, evaluate

CODE_STEPS = ('checkout', 'python', 'guard', 'rust', 'contracts', 'compile')
LIVE_STEPS = ('sidecars', 'live', 'receipt')
OUTCOMES = frozenset(('success', 'failure', 'skipped', 'cancelled'))


def summarize(source_sha: str, checkout_sha: str, steps: dict) -> dict:
    """Expected steps are declared here, never inferred from returned evidence."""
    if not isinstance(steps, dict):
        raise ValueError('step context must be an object')
    normalized = []
    for name in CODE_STEPS + LIVE_STEPS:
        item = steps.get(name)
        if not isinstance(item, dict):
            continue
        outcome = item.get('outcome')
        if not isinstance(outcome, str) or outcome not in OUTCOMES:
            outcome = 'unknown'
        normalized.append(dict(name=name, source_sha=checkout_sha,
                               status='completed', conclusion=outcome))

    def verdict(names: tuple[str, ...]) -> Verdict:
        return evaluate(source_sha, names,
                        (item for item in normalized if item['name'] in names))

    code, live = verdict(CODE_STEPS), verdict(LIVE_STEPS)
    if code.state != 'PASS' and live.state == 'PASS':
        live = Verdict('INCOMPLETE', problems=('live_without_verified_preflight',))
    return dict(schema_version=1, source_sha=source_sha, checkout_sha=checkout_sha,
                scope='per-OS steps before diagnostic upload; NOT overall CI or file E2E',
                code=asdict(code), live=asdict(live), steps=normalized)


def main() -> int:
    try:
        raw = os.environ['PULQVA_STEP_CONTEXT']
        if len(raw) > 65536:
            raise ValueError('step context too large')
        checkout = subprocess.run(['git', 'rev-parse', 'HEAD'], check=True,
                                  capture_output=True, text=True, timeout=10).stdout.strip()
        if re.fullmatch(r'[0-9a-f]{40}', checkout) is None:
            raise ValueError('invalid checkout SHA')
        data = summarize(os.environ['PULQVA_SOURCE_SHA'], checkout, json.loads(raw))
        for key in ('GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT'):
            value = os.environ.get(key, '')
            data[key.lower()] = value if re.fullmatch(r'[0-9]{1,20}', value) else None
        runner = os.environ.get('RUNNER_OS', '')
        data['runner_os'] = runner if runner in ('Linux', 'Windows') else 'Unknown'
        destination = Path(os.environ['PULQVA_EVIDENCE_DIR'])
        destination.mkdir(parents=True, exist_ok=True)
        (destination / 'stage-status.json').write_text(
            json.dumps(data, sort_keys=True, indent=2) + '\n', encoding='utf-8')
        print('PULQVA_STAGE_EVIDENCE code=' + data['code']['state']
              + ' live=' + data['live']['state'])
        # The original failed step still fails the job. Evidence creation is not
        # a replacement gate and never changes its outcome to green.
        return 0
    except (KeyError, ValueError, OSError, subprocess.SubprocessError):
        print('PULQVA_STAGE_EVIDENCE_UNAVAILABLE')
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
