#!/usr/bin/env python3
"""Contain legacy live triggers; NOT a completed live-generation admission service.

These four old paths have no integrated, accepted live matrix collector. Keep their
commands for migration, but never mint launch authority from an event or a boolean.
Automatic deterministic contracts are separate. A manual attempt fails visibly.
No API/network access, runtime launcher, retry loop, or arbitrary manifest executor.
"""
from __future__ import annotations
import json
import os
from pathlib import Path
import re
import subprocess
import sys

WORKFLOWS = (
    'tor-readiness-check', 'ytdlp-tor-media-check',
    'ytdlp-tor-metadata-check', 'commons-tor-discovery-check',
)


def assess(event: str, workflow: str, source: str, checkout: str, attempt: str) -> dict:
    if workflow not in WORKFLOWS:
        raise ValueError('unknown_workflow')
    if not re.fullmatch('[0-9a-f]{40}', source) or checkout != source:
        raise ValueError('source_mismatch')
    if not re.fullmatch('[1-9][0-9]{0,5}', attempt):
        raise ValueError('invalid_attempt')
    automatic = event in ('pull_request', 'push')
    if not automatic and event != 'workflow_dispatch':
        raise ValueError('unexpected_event')
    return {
        'schema': 'pulqva-legacy-live-containment-v1',
        'source_sha': source, 'checkout_sha': checkout, 'workflow': workflow,
        'event': event, 'attempt': int(attempt), 'admitted': False,
        'state': 'NOT_REQUESTED' if automatic else 'BLOCKED',
        'live_result': 'NOT_TESTED', 'launch_budget': 0,
        'reason': 'automatic_contracts_only' if automatic else 'live_capture_admission_not_implemented',
        'scope': 'trigger containment only; no live test or product acceptance',
    }


def main() -> int:
    try:
        checkout = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, timeout=10).strip()
        status = assess(os.environ.get('GITHUB_EVENT_NAME', ''), sys.argv[1],
                        os.environ.get('PULQVA_SOURCE_SHA', ''), checkout,
                        os.environ.get('GITHUB_RUN_ATTEMPT', ''))
        if (sys.argv[1] == 'ytdlp-tor-media-check'
                and os.environ.get('GITHUB_EVENT_NAME') == 'workflow_dispatch'):
            try:
                from media_evidence import admit
                status = admit(dict(os.environ), Path(__file__).resolve().parents[2],
                               Path(os.environ.get('PULQVA_MEDIA_PREREQUISITE', '')))
            except (ValueError, KeyError, TypeError, OSError, subprocess.SubprocessError):
                status['reason'] = 'media_prerequisite_or_generation_invalid'
        output = Path(sys.argv[2])
        output.mkdir(parents=True, exist_ok=False)
        with (output / 'live-status.json').open('x', encoding='utf-8') as stream:
            json.dump(status, stream, sort_keys=True, indent=2)
            stream.write('\n')
        # Literal-only values: no untrusted event content in Actions command files.
        with open(os.environ['GITHUB_OUTPUT'], 'a', encoding='utf-8') as stream:
            stream.write('admitted=' + ('true' if status['admitted'] else 'false') + '\n')
        with open(os.environ['GITHUB_STEP_SUMMARY'], 'a', encoding='utf-8') as stream:
            stream.write('## Live: NOT TESTED\nAdmission state: ' + status['state'] + '. '
                         'Only a validated explicit Linux media generation can proceed. '
                         'Automatic events and other legacy paths have zero live budget. '
                         'Admission is not a completed live test or file success.\n')
        print('PULQVA_LIVE_NOT_TESTED ' + status['state'])
        return 0 if status['state'] == 'NOT_REQUESTED' or status['admitted'] else 1
    except (OSError, ValueError, KeyError, IndexError, subprocess.SubprocessError):
        print('PULQVA_LIVE_ADMISSION_ERROR')
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
