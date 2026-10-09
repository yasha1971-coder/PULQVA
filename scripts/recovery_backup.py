#!/usr/bin/env python3
"""Export Git history + committed evidence; verify a restore without a remote."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def run(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, stderr=subprocess.STDOUT, timeout=90)


def main():
    root = Path(__file__).resolve().parents[1]
    out = Path(sys.argv[1]).resolve()
    if out.is_relative_to(root):
        raise ValueError('backup destination must be outside the source tree')
    out.mkdir(parents=True, exist_ok=True)
    head = run('git', 'rev-parse', 'HEAD', cwd=root).decode().strip()
    branch = 'recovery-snapshot-' + head
    ref = 'refs/heads/' + branch
    run('git', 'update-ref', ref, head, '0' * 40, cwd=root)
    try:
        bundle = out / 'PULQVA.bundle'
        run('git', 'bundle', 'create', str(bundle), '--all', cwd=root)
        verify = run('git', 'bundle', 'verify', str(bundle), cwd=root)
        refs = run('git', 'bundle', 'list-heads', str(bundle), cwd=root)
        (out / 'refs.txt').write_bytes(refs)
        with tempfile.TemporaryDirectory(prefix='pulqva-restore-') as temp:
            restored = Path(temp) / 'repo'
            run('git', '-c', 'protocol.file.allow=always', 'clone', '--branch', branch,
                str(bundle), str(restored))
            if run('git', 'rev-parse', 'HEAD', cwd=restored).decode().strip() != head:
                raise ValueError('restore checkout mismatch')
            fsck = run('git', 'fsck', '--full', cwd=restored)
            original = Path(temp) / 'original'
            transition = Path(temp) / 'transition'
            for destination, revision in (
                (original, 'df5ac954485920c49e422fdb1eb5e9d3dd43c648'),
                (transition, '3ca7a85ef5de3f9108cc3213da173a6f65356c87'),
            ):
                run('git', '-c', 'protocol.file.allow=always', 'clone',
                    '--no-local', '--no-checkout', str(restored), str(destination))
                run('git', '-C', str(destination), 'checkout', '--detach', revision)
            original_v1 = run(sys.executable, 'scripts/recovery_guard.py', cwd=original)
            transition_v1 = run(sys.executable, 'scripts/recovery_guard.py', cwd=transition)
            handoff = run(sys.executable, '-B', 'scripts/recovery_handoff_v2.py',
                          '--root', str(restored), '--expected-head', head,
                          '--trusted-root', str(original),
                          '--transition-root', str(transition), cwd=restored)
            (out / 'restore-check.txt').write_bytes(
                verify + fsck + original_v1 + transition_v1 + handoff)
        receipt = {'protocol': 'pulqva-cold-restore-v2', 'source_checkout': head,
                   'snapshot_branch': branch, 'git_bundle_sha256': hashlib.sha256(bundle.read_bytes()).hexdigest(),
                   'restore': 'PASS', 'admission': 'v1-original+v1-transition+v2-target', 'restored_from': 'local bundle, no remote fetch',
                   'run_id': os.getenv('GITHUB_RUN_ID'), 'run_attempt': os.getenv('GITHUB_RUN_ATTEMPT'),
                   'coverage': 'all refs fetched into CI checkout; their reachable Git history; committed evidence',
                   'limitations': ['not all GitHub comments/reviews or Actions logs',
                                   'not LFS objects, releases, packages, model or sidecar binaries',
                                   'backup must be copied outside Actions retention; no recurring schedule',
                                   'not a build or product E2E test']}
        (out / 'restore_receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
        (out / 'RESTORE.txt').write_text('PULQVA source and continuity backup. Not a runnable application.\n'
            'Verify PULQVA.bundle against git_bundle_sha256 in restore_receipt.json.\n'
            'git clone --branch ' + branch + ' PULQVA.bundle restored\n'
            'cd restored\npython3 scripts/recovery_handoff_v2.py --help\nRead START_HERE.md.\n'
            'Do not push, merge or run model/network tests as part of recovery.\n')
        print(json.dumps(receipt, indent=2))
    finally:
        run('git', 'update-ref', '-d', ref, head, cwd=root)


if __name__ == '__main__':
    main()
