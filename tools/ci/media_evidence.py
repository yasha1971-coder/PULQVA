"""One Linux fixed-public media fixture, through the existing Rust/Tor path.

No build, shell, alternate downloader, automatic retry, or mutable test plan here.
Same-run native prerequisites are checked before admission and again before launch.
This is CI evidence capture, NOT an egress sandbox or retained-file acceptance.
"""
from __future__ import annotations

import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import selectors
import signal
import stat
import subprocess
import sys
import time

from matrix_receipt import (MatrixRecorder, canonical, digest, require, strict_load,
                            validate_manifest, verify, write_exclusive)
from readiness_contract_matrix import PROBES, execution_verdict, profile_spec

ROOT = Path(__file__).resolve().parents[2]
BOUNDARY = 'T069-G2C2-fixed-public-media-capture'
WORKFLOW = 'ytdlp-tor-media-check'
MARKER = b'PULQVA_YTDLP_TOR_MEDIA_OK'
MAX_OUTPUT = 64 * 1024
LEGACY_PROFILE = 'fixed-public-media'
RETAINED_PROFILE = 'retained-commons'
OUTER_TIMEOUT = 330  # Existing inner 90s readiness + 180s media, then cleanup allowance.


def verify_retained_commons(directory: Path, source: str) -> dict:
    """Read back the existing example's fixed output; never download or follow paths.

    Caller must establish successful child exit, cleanup, frozen identities and
    admission separately. This helper alone cannot mint live acceptance. Requires
    exclusive parent ownership; O_NOFOLLOW does not make parent races safe.
    """
    require(re.fullmatch('[0-9a-f]{40}', source) is not None, 'retained_source_format')
    require(stat.S_ISDIR(directory.lstat().st_mode), 'retained_directory')

    def read_fixed(name: str, cap: int) -> bytes:
        path = directory / name
        require(stat.S_ISREG(path.lstat().st_mode), 'retained_regular_file')
        flags = os.O_RDONLY | getattr(os, 'O_NOFOLLOW', 0) | getattr(os, 'O_NONBLOCK', 0)
        fd = os.open(path, flags)
        with os.fdopen(fd, 'rb') as stream:
            before = os.fstat(stream.fileno())
            require(stat.S_ISREG(before.st_mode) and 0 < before.st_size <= cap, 'retained_size_bound')
            data = stream.read(cap + 1)
            after = os.fstat(stream.fileno())
            require(len(data) == before.st_size == after.st_size and len(data) <= cap
                    and before.st_mtime_ns == after.st_mtime_ns, 'retained_changed_during_read')
            return data

    def unique(pairs):
        obj = {}
        for key, value in pairs:
            require(key not in obj, 'retained_duplicate_key')
            obj[key] = value
        return obj

    receipt = json.loads(read_fixed('receipt.json', 65_536), object_pairs_hook=unique,
                         parse_constant=lambda _: require(False, 'retained_nonfinite'))
    require(type(receipt) is dict and receipt.get('schema') == 2
            and receipt.get('scope') == 'linux-live-request-choice-file'
            and receipt.get('source_sha') == source and receipt.get('request') == 'countdown',
            'retained_identity')
    for key in ('core_retrieval_used', 'file_downloaded', 'cancelled_search_rejected', 'stale_results_cleared'):
        require(receipt.get(key) is True, 'retained_predicates')
    require(receipt.get('publisher_authenticated') is False
            and receipt.get('windows_e2e_verified') is False, 'retained_scope')
    choices = receipt.get('choices')
    require(type(choices) is list and 2 <= len(choices) <= 20
            and type(receipt.get('choice_count')) is int and receipt['choice_count'] == len(choices)
            and type(receipt.get('selected_index')) is int and receipt['selected_index'] == 1,
            'retained_choice_count')
    for index, row in enumerate(choices):
        require(type(row) is dict and type(row.get('index')) is int and row['index'] == index,
                'retained_choice_index')
    selected, file = choices[1], receipt.get('file')
    require(type(file) is dict and file.get('path') == 'selected.webm'
            and type(file.get('selected_index')) is int and file['selected_index'] == 1
            and type(selected.get('title')) is str and bool(selected['title'])
            and file.get('title') == selected['title']
            and type(selected.get('locator')) is str and bool(selected['locator'])
            and receipt.get('selected_locator') == selected['locator'], 'retained_selection')
    size = selected.get('declared_size')
    require(type(size) is int and 0 < size <= 8 * 1024 * 1024
            and type(file.get('byte_size')) is int and file['byte_size'] == size, 'retained_declared_size')
    for value, width in ((selected.get('declared_sha1'), 40), (file.get('sha1'), 40), (file.get('sha256'), 64)):
        require(type(value) is str and re.fullmatch('[0-9a-f]{' + str(width) + '}', value) is not None,
                'retained_digest_format')
    data = read_fixed('selected.webm', size)
    sha1, sha256 = hashlib.sha1(data).hexdigest(), hashlib.sha256(data).hexdigest()
    require(len(data) == size and sha1 == selected['declared_sha1'] == file['sha1']
            and sha256 == file['sha256'], 'retained_content_mismatch')
    return dict(byte_size=size, sha1=sha1, sha256=sha256, selected_index=1,
                scope='fixed Commons output readback only; not live admission')


def sha(path: Path) -> str:
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def captured(value: str) -> dict:
    return dict(status='captured', sha256=value)


def checkout(root: Path) -> str:
    return subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root,
                                   text=True, timeout=10).strip()


def verify_prerequisite(directory: Path, root: Path, source: str, run: str, attempt: str,
                        profile: str = "readiness") -> str:
    """Validate actual named test output, not exit status, a badge, or caller assertion."""
    contract = profile_spec(profile)
    specs = contract['probes']
    plan = strict_load(directory / 'manifest.json')
    receipt = strict_load(directory / 'matrix_receipt.json')
    results = strict_load(directory / 'test-results.json')
    require(verify(plan, receipt) == 'PASS', 'prerequisite_not_accepted')
    require(plan['boundary'] == contract['boundary'], 'wrong_boundary')
    identity = plan['identity']
    require((identity['source_sha'], identity['checkout_sha'], identity['run_id'], identity['attempt'])
            == (source, source, run, attempt), 'foreign_or_stale_prerequisite')
    require(plan['environment']['os'] == 'Linux', 'wrong_prerequisite_platform')
    require(receipt.get('raw_evidence_sha256') == sha(directory / 'test-results.json'), 'raw_digest')
    paths = {'evaluator': root / 'tools/ci/readiness_contract_matrix.py',
             'fixtures': root / contract['fixtures'],
             'codec': root / 'tools/ci/matrix_receipt.py'}
    for name, path in paths.items():
        require(identity['components'].get(name) == captured(sha(path)), 'prerequisite_code_changed')
    require([p['id'] for p in plan['probes']] == [p['id'] for p in specs], 'probe_inventory')
    rows = results.get('probes', [])
    require(len(rows) == len(specs) and results.get('fresh_process_per_probe') is True, 'raw_inventory')
    for spec, declaration, evidence in zip(specs, plan['probes'], rows):
        for key in ('id', 'hypothesis', 'invariant_set'):
            require(declaration[key] == spec[key], 'prerequisite_criteria_changed')
        require(declaration['expected'] == dict(contract=spec['expected'], test=spec['test'], executed=1, passed=1)
                and declaration['depends_on'] == [], 'prerequisite_expected_changed')
        require(evidence.get('id') == spec['id'] and evidence.get('test') == spec['test']
                and evidence.get('result') == 'PASS' and evidence.get('output_truncated') is False,
                'raw_probe_not_passed')
        stdout, stderr = bytes.fromhex(evidence['stdout_hex']), bytes.fromhex(evidence['stderr_hex'])
        require(hashlib.sha256(stdout).hexdigest() == evidence['stdout_sha256']
                and hashlib.sha256(stderr).hexdigest() == evidence['stderr_sha256'], 'output_digest')
        require(execution_verdict(spec['test'], subprocess.CompletedProcess([], evidence['returncode'], stdout, stderr))[0]
                == 'PASS', 'exact_native_test_not_executed')
    return digest(plan)


def admit(env: dict, root: Path, prerequisite: Path) -> dict:
    source, run, attempt = (env.get(k, '') for k in ('PULQVA_SOURCE_SHA', 'GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT'))
    require(env.get('GITHUB_EVENT_NAME') == 'workflow_dispatch', 'not_explicit_dispatch')
    require(attempt == '1', 'rerun_denied')
    require(re.fullmatch('[0-9a-f]{40}', source) is not None and
            env.get('PULQVA_REQUESTED_SOURCE') == source == env.get('GITHUB_SHA') == checkout(root), 'source_mismatch')
    require(re.fullmatch('[1-9][0-9]{0,19}', run) is not None, 'run_identity')
    require(env.get('PULQVA_CONTRACTS_RESULT') == 'success', 'contracts_not_successful')
    require(env.get('RUNNER_OS') == 'Linux', 'linux_only')
    profile = env.get('PULQVA_MEDIA_PROFILE', LEGACY_PROFILE)
    require(profile in (LEGACY_PROFILE, RETAINED_PROFILE), 'unknown_media_profile')
    native_digest = verify_prerequisite(prerequisite, root, source, run, attempt)
    extra = {}
    if profile == RETAINED_PROFILE:
        extra['retained_prerequisite_manifest_sha256'] = verify_prerequisite(
            prerequisite/'retained-commons', root, source, run, attempt, RETAINED_PROFILE)
    return dict(schema='pulqva-media-admission-v1', source_sha=source, checkout_sha=source,
                run_id=run, attempt=1, workflow=WORKFLOW, event='workflow_dispatch', admitted=True,
                generation_id=f'{source[:12]}-{run}-1-media', state='ADMITTED', launch_budget=1,
                live_result='NOT_TESTED', prerequisite_manifest_sha256=native_digest, profile=profile, **extra,
                scope='one selected fixed-public Linux profile; admission is not a file or whole-product result')


def bounded_process(argv: list[str], cwd: Path, env: dict, *, timeout: float = OUTER_TIMEOUT,
                    cap: int = MAX_OUTPUT) -> dict:
    """Drain pipes incrementally; kill only the owned POSIX process group on every exit."""
    require(os.name == 'posix', 'posix_capture_only')
    output = {'stdout': bytearray(), 'stderr': bytearray()}
    started = time.monotonic_ns()
    proc = None
    reason = None
    cleanup = False
    returncode = None
    try:
        proc = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        deadline = time.monotonic() + timeout
        with selectors.DefaultSelector() as selector:
            for name in output:
                pipe = getattr(proc, name)
                os.set_blocking(pipe.fileno(), False)
                selector.register(pipe, selectors.EVENT_READ, name)
            while selector.get_map():
                left = deadline - time.monotonic()
                if left <= 0:
                    reason = 'outer_deadline'; break
                overflow = False
                for key, _ in selector.select(min(left, 0.1)):
                    chunk = os.read(key.fd, 8192)
                    if not chunk:
                        selector.unregister(key.fileobj); continue
                    room = cap - len(output[key.data])
                    output[key.data].extend(chunk[:room])
                    if len(chunk) > room:
                        reason = 'output_limit'; overflow = True; break
                if overflow:
                    break
            if reason is None:
                try:
                    returncode = proc.wait(timeout=max(0.001, deadline - time.monotonic()))
                except subprocess.TimeoutExpired:
                    reason = 'outer_deadline'
    except (OSError, KeyboardInterrupt):
        reason = 'capture_interrupted_or_start_failed'
    finally:
        if proc is not None:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
                cleanup = True
            except ProcessLookupError:
                cleanup = True
            except OSError:
                cleanup = False
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                cleanup = False
            for name in output:
                getattr(proc, name).close()
            if not cleanup:
                reason = 'owned_group_cleanup_failed'
        else:
            cleanup = True  # No child was started.
    return dict(stdout=bytes(output['stdout']), stderr=bytes(output['stderr']),
                returncode=returncode, reason=reason, cleanup=cleanup,
                duration_ms=(time.monotonic_ns() - started) / 1_000_000)


def identities(root: Path, paths: dict[str, Path]) -> dict:
    measured = {name: captured(sha(path)) for name, path in paths.items()}
    require(subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--'], cwd=root, timeout=10).returncode == 0,
            'tracked_source_changed')
    return measured


def run_fixture(root: Path, output: Path, prerequisite: Path, binary: Path, arti: Path, ytdlp: Path,
                env: dict, *, retained: bool = False) -> int:
    # Deliberately cannot be called to bypass admission via a ready-made status file.
    admission = admit(env, root, prerequisite)
    require(retained == (admission['profile'] == RETAINED_PROFILE), 'capture_profile_mismatch')
    marker = b'PULQVA_COMMONS_FILE_E2E_OK' if retained else MARKER
    probe = 'RETAINED-FILE' if retained else 'MEDIA-FIXTURE'
    boundary = 'T069-G2D-retained-public-file' if retained else BOUNDARY
    fixture = ('crates/pulqva-discovery/examples/real_commons_file.rs' if retained
               else 'crates/pulqva-privacy/examples/real_ytdlp_tor_media.rs')
    require(platform.system() == 'Linux', 'linux_only')
    paths = {'runtime': binary.resolve(strict=True), 'arti': arti.resolve(strict=True),
             'ytdlp': ytdlp.resolve(strict=True), 'evaluator': Path(__file__).resolve(),
             'codec': root/'tools/ci/matrix_receipt.py',
             'fixtures': root/fixture,
             'native_collector': root/'tools/ci/readiness_contract_matrix.py',
             'workflow': root/'.github/workflows/ytdlp-tor-media-check.yml',
             'arti_pin': root/'sidecars/arti/VERSION', 'ytdlp_pin': root/'sidecars/yt-dlp/SHA256SUMS',
             'lockfile': root/'Cargo.lock'}
    if retained:
        for rel in ('crates/pulqva-core/src/journey.rs', 'crates/pulqva-discovery/src/artifact.rs',
                    'crates/pulqva-discovery/src/commons.rs', 'crates/pulqva-discovery/src/https.rs'):
            paths[rel] = root/rel
    components = identities(root, paths)
    pins = [line.split()[0] for line in paths['ytdlp_pin'].read_text().splitlines()
            if len(line.split()) == 2 and line.split()[1] == 'yt-dlp_linux']
    require(len(pins) == 1 and components['ytdlp']['sha256'] == pins[0], 'ytdlp_pin_mismatch')
    require(paths['arti_pin'].read_text().strip() == '2.6.0', 'arti_version_pin')
    for name in ('model', 'prompt', 'schema'):
        components[name] = dict(status='not_applicable', sha256=None, reason='fixed public media fixture; no local model')
    components['flags'] = captured(digest(dict(argv=['native_fixture', 'pinned_arti', 'pinned_ytdlp',
                                                    'new-retained-directory' if retained else '--diagnostics'],
                                              inner_budgets='unchanged Rust fixture', outer_timeout=OUTER_TIMEOUT,
                                              output_cap_per_stream=MAX_OUTPUT, calls=1, env='isolated_home_temp_minimal')))
    components['prerequisite'] = captured(admission['prerequisite_manifest_sha256'])
    if retained:
        components['retained_prerequisite'] = captured(admission['retained_prerequisite_manifest_sha256'])
    identity = dict(source_sha=admission['source_sha'], checkout_sha=checkout(root), run_id=admission['run_id'],
                    attempt='1', job_id=None, job_id_reason='numeric job ID bound externally by GitHub job API',
                    components=components)
    output.mkdir(parents=True, exist_ok=False, mode=0o700)
    temp, home = output/'owned-tmp', output/'owned-home'
    temp.mkdir(mode=0o700); home.mkdir(mode=0o700)
    child_env = dict(PATH='/usr/bin:/bin', LANG='C.UTF-8', HOME=str(home.resolve()),
                     TMPDIR=str(temp.resolve()), TMP=str(temp.resolve()), TEMP=str(temp.resolve()))
    if retained:
        child_env['PULQVA_SOURCE_SHA'] = admission['source_sha']
    retained_dir = output/'retained-file'
    argv = [str(paths['runtime']), str(paths['arti']), str(paths['ytdlp']),
            str(retained_dir.resolve()) if retained else '--diagnostics']
    plan = dict(protocol='pulqva-evidence-boundary-v1', boundary=boundary,
                generation_id=admission['generation_id'] + ('-retained' if retained else ''), identity=identity,
                environment=dict(os=platform.system(), arch=platform.machine(), toolchain='Rust 1.91.0; '+sys.version),
                isolation=dict(status='verified', strategy='One trusted public fixture, exclusive temp/home; one owned POSIX process group. No cross-probe state.',
                               evidence_refs=['launch.json', 'test-results.json#cleanup']),
                probes=[dict(id=probe, hypothesis=('The existing Commons core request/choice/retrieval fixture retains bytes matching selected metadata and independent readback.' if retained else 'The existing Tor-gated fixed-public media fixture reaches its original success marker within its unchanged internal budgets.'),
                             invariant_set=['one invocation; no repairs or rebuilds', 'same source/sidecar/config identities',
                                            'Tor-only route; existing dependent retries unchanged', 'no user input or inherited credentials'],
                             expected=dict(marker=marker.decode(), returncode=0, **({'independent_readback': True} if retained else {})), depends_on=[])])
    validate_manifest(plan)
    write_exclusive(output/'admission.json', admission)
    write_exclusive(output/'launch.json', dict(owned_namespace=True, process_group=True, calls=1,
                    isolated_env_keys=sorted(child_env), outer_timeout_seconds=OUTER_TIMEOUT,
                    limitations=['not an OS egress sandbox', 'malicious children escaping the group are outside this trusted-fixture scope']))
    write_exclusive(output/'manifest.json', plan)
    recorder = MatrixRecorder(plan)
    raw = dict(reason='not_started', cleanup=True, returncode=None, stdout=b'', stderr=b'', duration_ms=0.0)
    try:
        require(identities(root, paths) == {name: components[name] for name in paths}, 'prelaunch_identity_changed')
        require(admit(env, root, prerequisite) == admission, 'prelaunch_admission_changed')
        raw = bounded_process(argv, output, child_env)
        passed = (raw['reason'] is None and raw['cleanup'] is True and raw['returncode'] == 0
                  and raw['stdout'].splitlines().count(marker) == 1
                  and b'WINDOWS_FAIL_CLOSED' not in raw['stdout'])
        readback = None
        if retained and passed:
            try:
                readback = verify_retained_commons(retained_dir, admission['source_sha'])
            except (ValueError, OSError, KeyError, TypeError):
                passed = False
                raw['reason'] = 'retained_readback_failed'
        raw['retained_readback'] = readback
        result = 'ERROR' if raw['reason'] else 'PASS' if passed else 'FAIL'
        recorder.record(probe, result=result, duration_ms=raw['duration_ms'],
                        observed=dict(returncode=raw['returncode'], exact_marker=passed, cleanup=raw['cleanup']),
                        evidence_refs=['test-results.json', 'stdout.bin', 'stderr.bin'], reason=raw['reason'])
        final = copy.deepcopy(identity)
        final['components'].update(identities(root, paths))
        final['checkout_sha'] = checkout(root)
        require(admit(env, root, prerequisite) == admission, 'closeout_admission_changed')
    except (OSError, ValueError, KeyError, subprocess.SubprocessError, KeyboardInterrupt):
        recorder.fail_closed('capture_or_frozen_identity_failure')
        final = identity  # Explicitly unsafe, NOT a claim that identity remeasurement passed.
    receipt = recorder.close(final, early_exit_reason='fixture_not_launched')
    for name in ('stdout', 'stderr'):
        data = raw.pop(name)
        with (output/(name+'.bin')).open('xb') as stream:
            stream.write(data)
        raw[name+'_sha256'] = hashlib.sha256(data).hexdigest()
        raw[name+'_bytes'] = len(data)
    write_exclusive(output/'test-results.json', raw)
    receipt['raw_evidence_sha256'] = sha(output/'test-results.json')
    receipt['limitations'] += [('Fixed public request/choice/file only; no model/UI/Windows acceptance; requires owned parent paths.' if retained else 'Fixed fixture removes downloaded output; this is NOT a retained content-verified file.'),
                               'Tor retries are dependent observations inside ONE probe, not independent experiments.',
                               'Host termination may prevent closeout; missing receipt never means PASS.',
                               'No OS-wide egress, hostile-process isolation, provider authenticity or Windows-model claim.']
    receipt['retained_verified_file'] = bool(retained and raw.get('retained_readback') and verify(plan, receipt) == 'PASS')
    write_exclusive(output/'matrix_receipt.json', receipt)
    verdict = verify(plan, receipt)
    print('PULQVA_MEDIA_CAPTURE_'+verdict)
    return 0 if verdict == 'PASS' else 1


def finalize(output: Path, job_status: str) -> int:
    output.mkdir(parents=True, exist_ok=True)
    target = output/'live-status.json'
    if target.exists():
        return 1  # No overwrite or replay of a finalized generation.
    if (output/'matrix_receipt.json').is_file():
        plan, receipt = strict_load(output/'manifest.json'), strict_load(output/'matrix_receipt.json')
        result = verify(plan, receipt)
        require(receipt['raw_evidence_sha256'] == sha(output/'test-results.json'), 'raw_digest')
        raw = strict_load(output/'test-results.json')
        for name in ('stdout', 'stderr'):
            require(raw[name+'_sha256'] == sha(output/(name+'.bin')), 'output_digest')
        if result == 'PASS' and plan['boundary'] == 'T069-G2D-retained-public-file':
            try:
                readback = verify_retained_commons(output/'retained-file', plan['identity']['source_sha'])
                require(readback == raw.get('retained_readback') and receipt.get('retained_verified_file') is True,
                        'retained_closeout_changed')
            except (ValueError, OSError, KeyError, TypeError):
                result = 'ERROR'
        if job_status != 'success' and result == 'PASS':
            result = 'ERROR'
    else:
        result = 'NOT_TESTED'
    write_exclusive(target, dict(scope=plan['boundary'] if (output/'matrix_receipt.json').is_file() else BOUNDARY, result=result,
                                 retained_verified_file=bool(result == 'PASS' and receipt.get('retained_verified_file')) if (output/'matrix_receipt.json').is_file() else False,
                                 reason='see immutable receipt' if result != 'NOT_TESTED' else 'preparation_or_capture_not_completed'))
    return 0 if result == 'PASS' else 1


def main() -> int:
    try:
        if len(sys.argv) == 4 and sys.argv[1] == 'finalize':
            return finalize(Path(sys.argv[2]), sys.argv[3])
        require(len(sys.argv) == 7 and sys.argv[1] in ('capture', 'capture-retained'), 'usage')
        return run_fixture(ROOT, Path(sys.argv[2]), Path(sys.argv[3]),
                           Path(sys.argv[4]), Path(sys.argv[5]), Path(sys.argv[6]), dict(os.environ), retained=sys.argv[1] == 'capture-retained')
    except (ValueError, OSError, KeyError, TypeError, subprocess.SubprocessError):
        print('PULQVA_MEDIA_CAPTURE_BLOCKED')  # Never include raw paths or arbitrary input.
        return 2


if __name__ == '__main__':
    raise SystemExit(main())
