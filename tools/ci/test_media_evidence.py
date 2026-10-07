"""Synthetic G2C2 controls only; fixtures never contact Tor or an Internet provider."""
from __future__ import annotations
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch
import uuid

import media_evidence as media
import readiness_contract_matrix as native
from matrix_receipt import MatrixRecorder, canonical, digest, strict_load, verify, write_exclusive
from readiness_contract_matrix import PROBES

ROOT = Path(__file__).resolve().parents[2]
POSIX = unittest.skipUnless(platform.system() == 'Linux', 'Linux capture implementation; Windows admission-only contracts')


class MediaEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='pulqva-media-synthetic-')
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.root = self.base/'repo'; self.root.mkdir()
        files = ['tools/ci/readiness_contract_matrix.py', 'tools/ci/matrix_receipt.py',
                 'crates/pulqva-privacy/src/arti_readiness/tests.rs',
                 'crates/pulqva-privacy/examples/real_ytdlp_tor_media.rs',
                 '.github/workflows/ytdlp-tor-media-check.yml', 'sidecars/arti/VERSION', 'Cargo.lock']
        for name in files:
            dest = self.root/name; dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes((ROOT/name).read_bytes())
        self.ytdlp = self.base/'yt-dlp'; self.ytdlp.write_bytes(b'synthetic pinned sidecar')
        self.arti = self.base/'arti'; self.arti.write_bytes(b'synthetic Arti; never executed')
        self.binary = self.base/'media-fixture'
        self.binary.write_text('#!'+sys.executable+'\nimport os\nassert not any(k in os.environ for k in ("GITHUB_TOKEN","HTTP_PROXY","LD_PRELOAD"))\nprint("PULQVA_YTDLP_TOR_MEDIA_OK")\n')
        self.binary.chmod(0o700)
        p=self.root/'sidecars/yt-dlp/SHA256SUMS'; p.parent.mkdir(parents=True)
        p.write_text(media.sha(self.ytdlp)+'  yt-dlp_linux\n')
        def git(*args):
            return subprocess.check_output(['git',*args],cwd=self.root,stderr=subprocess.DEVNULL,text=True).strip()
        git('init','-q'); git('add','.')
        git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid',
            '-c','commit.gpgsign=false','commit','-qm','synthetic input only')
        self.source = git('rev-parse','HEAD')
        self.env = dict(GITHUB_EVENT_NAME='workflow_dispatch', GITHUB_RUN_ID='12345', GITHUB_RUN_ATTEMPT='1',
                        GITHUB_SHA=self.source, PULQVA_SOURCE_SHA=self.source, PULQVA_REQUESTED_SOURCE=self.source,
                        PULQVA_CONTRACTS_RESULT='success', RUNNER_OS='Linux')
        self.prereq = self.base/'prereq'; self.prereq.mkdir()
        captured=lambda data: dict(status='captured',sha256=hashlib.sha256(data).hexdigest())
        components={name:captured(b'synthetic') for name in ('runtime','flags')}
        for key, rel in [('evaluator',files[0]),('codec',files[1]),('fixtures',files[2])]:
            components[key]=captured((self.root/rel).read_bytes())
        for key in ('model','prompt','schema'):
            components[key]=dict(status='not_applicable',sha256=None,reason='synthetic prerequisite fixture')
        self.plan=dict(protocol='pulqva-evidence-boundary-v1',boundary='T069-G2B-readiness-diagnostics-native-contract',
                       generation_id='synthetic-prerequisite',
                       identity=dict(source_sha=self.source,checkout_sha=self.source,run_id='12345',attempt='1',
                                     job_id=None,job_id_reason='synthetic prerequisite only',components=components),
                       environment=dict(os='Linux',arch='synthetic',toolchain='synthetic native-output fixture'),
                       isolation=dict(status='verified',strategy='fabricated input for validator unit tests, NOT real native evidence',
                                      evidence_refs=['test_media_evidence.py']),
                       probes=[{k:v for k,v in p.items() if k!='test'}|dict(expected=dict(contract=p['expected'],test=p['test'],executed=1,passed=1),depends_on=[]) for p in PROBES])
        recorder=MatrixRecorder(self.plan)
        raw=[]
        for p in PROBES:
            stdout=('running 1 test\ntest '+p['test']+' ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s\n').encode()
            raw.append(dict(id=p['id'],test=p['test'],result='PASS',returncode=0,output_truncated=False,
                            stdout_hex=stdout.hex(),stderr_hex='',stdout_sha256=hashlib.sha256(stdout).hexdigest(),
                            stderr_sha256=hashlib.sha256(b'').hexdigest()))
            recorder.record(p['id'],result='PASS',duration_ms=0.0,observed='synthetic input fixture',evidence_refs=['test-results.json'])
        self.raw=dict(fresh_process_per_probe=True,probes=raw)
        self.receipt=recorder.close(self.plan['identity'])
        self.save_prereq()

    def save_prereq(self):
        # Deliberate adversarial fixture rewrites are NOT production evidence mutation.
        for name, value in [('manifest.json',self.plan),('test-results.json',self.raw)]:
            (self.prereq/name).write_bytes(canonical(value)+b'\n')
        self.receipt['raw_evidence_sha256']=media.sha(self.prereq/'test-results.json')
        (self.prereq/'matrix_receipt.json').write_bytes(canonical(self.receipt)+b'\n')

    def admit(self):
        return media.admit(self.env,self.root,self.prereq)

    def capture(self, out=None):
        return media.run_fixture(self.root,out or self.base/'output',self.prereq,self.binary,self.arti,self.ytdlp,self.env)

    def prepare_retained(self):
        """Build explicit synthetic native-prerequisite input; not real native evidence."""
        fixture = self.root/'crates/pulqva-discovery/examples/real_commons_file.rs'
        fixture.parent.mkdir(parents=True, exist_ok=True); fixture.write_bytes((ROOT/'crates/pulqva-discovery/examples/real_commons_file.rs').read_bytes())
        for rel in ('crates/pulqva-core/src/journey.rs', 'crates/pulqva-discovery/src/artifact.rs',
                    'crates/pulqva-discovery/src/artifact/tests.rs', 'crates/pulqva-discovery/src/commons.rs',
                    'crates/pulqva-discovery/src/https.rs'):
            dest = self.root/rel; dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes((ROOT/rel).read_bytes())
        subprocess.run(['git','add','.'],cwd=self.root,check=True)
        subprocess.run(['git','-c','user.name=Fixture','-c','user.email=fixture@example.invalid',
                        '-c','commit.gpgsign=false','commit','-qm','retained synthetic fixture'],cwd=self.root,check=True)
        self.source=media.checkout(self.root)
        for key in ('GITHUB_SHA','PULQVA_SOURCE_SHA','PULQVA_REQUESTED_SOURCE'): self.env[key]=self.source
        for obj in (self.plan['identity'], self.receipt['identity']):
            obj['source_sha']=obj['checkout_sha']=self.source
        self.receipt['manifest_sha256']=digest(self.plan)
        self.save_prereq()
        self.env['PULQVA_MEDIA_PROFILE'] = media.RETAINED_PROFILE
        self.extra_dir = self.prereq/media.RETAINED_PROFILE
        self.extra_dir.mkdir()
        self.extra_plan = copy.deepcopy(self.plan)
        self.extra_plan['generation_id'] = 'synthetic-retained-prerequisite'
        self.extra_plan['boundary'] = native.profile_spec(media.RETAINED_PROFILE)['boundary']
        self.extra_plan['identity']['components']['fixtures'] = media.captured(media.sha(
            self.root/native.profile_spec(media.RETAINED_PROFILE)['fixtures']))
        self.extra_plan['probes'] = [
            {k: copy.deepcopy(v) for k, v in p.items() if k != 'test'} |
            dict(expected=dict(contract=p['expected'], test=p['test'], executed=1, passed=1), depends_on=[])
            for p in native.RETAINED_PROBES]
        recorder = MatrixRecorder(self.extra_plan)
        rows = []
        for spec in native.RETAINED_PROBES:
            data = ('running 1 test\ntest '+spec['test']+' ... ok\n\n'
                    'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s\n').encode()
            rows.append(dict(id=spec['id'], test=spec['test'], result='PASS', returncode=0,
                             stdout_hex=data.hex(), stderr_hex='', output_truncated=False,
                             stdout_sha256=hashlib.sha256(data).hexdigest(),
                             stderr_sha256=hashlib.sha256(b'').hexdigest()))
            recorder.record(spec['id'], result='PASS', duration_ms=0.0, observed='synthetic prerequisite input',
                            evidence_refs=['test-results.json'])
        self.extra_raw = dict(fresh_process_per_probe=True, probes=rows)
        self.extra_receipt = recorder.close(self.extra_plan['identity'])
        self.save_extra()

    def save_extra(self):
        for name, value in [('manifest.json', self.extra_plan), ('test-results.json', self.extra_raw)]:
            (self.extra_dir/name).write_bytes(canonical(value)+b'\n')
        self.extra_receipt['raw_evidence_sha256'] = media.sha(self.extra_dir/'test-results.json')
        (self.extra_dir/'matrix_receipt.json').write_bytes(canonical(self.extra_receipt)+b'\n')

    def retained_capture(self, corrupt=False, missing=False, cleanup=True):
        # Synthetic child and owned output; no Tor/model/network execution.
        self.prepare_retained()
        out=self.base/'retained-output'
        def child(argv,cwd,env):
            self.assertEqual(env['PULQVA_SOURCE_SHA'], self.source)
            directory=Path(argv[3]); self.assertFalse(directory.exists()); directory.mkdir()
            h1=hashlib.sha1(b'abc').hexdigest()
            receipt=dict(schema=2,scope='linux-live-request-choice-file',source_sha=self.source,
                request='countdown',choice_count=2,selected_index=1,selected_locator='public-fixture',
                choices=[dict(index=0),dict(index=1,title='fixture',locator='public-fixture',declared_size=3,declared_sha1=h1)],
                file=dict(path='selected.webm',byte_size=3,selected_index=1,title='fixture',sha1=h1,sha256=hashlib.sha256(b'abc').hexdigest()),
                core_retrieval_used=True,file_downloaded=True,cancelled_search_rejected=True,stale_results_cleared=True,
                publisher_authenticated=False,windows_e2e_verified=False)
            (directory/'receipt.json').write_text(json.dumps(receipt))
            if not missing: (directory/'selected.webm').write_bytes(b'abd' if corrupt else b'abc')
            return dict(stdout=b'PULQVA_COMMONS_FILE_E2E_OK\n',stderr=b'',reason=None,cleanup=cleanup,returncode=0,duration_ms=1.0)
        with patch.object(media,'bounded_process',side_effect=child):
            result=media.run_fixture(self.root,out,self.prereq,self.binary,self.arti,self.ytdlp,self.env,retained=True)
        return result,out

    @POSIX
    def test_retained_profile_requires_readback_and_keeps_selected_bytes(self):
        """Matching selected bytes pass independent readback and remain after closeout."""
        result,out=self.retained_capture()
        self.assertEqual(result,0)
        self.assertEqual((out/'retained-file/selected.webm').read_bytes(),b'abc')
        self.assertTrue(strict_load(out/'matrix_receipt.json')['retained_verified_file'])
        self.assertEqual(media.finalize(out,'success'),0)
        self.assertTrue(strict_load(out/'live-status.json')['retained_verified_file'])

    @POSIX
    def test_retained_success_marker_cannot_accept_same_size_corruption(self):
        """An exit-zero success marker cannot accept altered bytes of identical size."""
        result,out=self.retained_capture(corrupt=True)
        self.assertEqual(result,1)
        self.assertFalse(strict_load(out/'matrix_receipt.json')['retained_verified_file'])

    @POSIX
    def test_retained_success_marker_cannot_accept_missing_file(self):
        """An exit-zero success marker cannot accept a missing selected file."""
        result,out=self.retained_capture(missing=True)
        self.assertEqual(result,1)
        self.assertFalse(strict_load(out/'matrix_receipt.json')['retained_verified_file'])

    @POSIX
    def test_retained_closeout_rejects_file_changed_after_capture(self):
        """Changing retained bytes after capture invalidates final acceptance."""
        result,out=self.retained_capture()
        self.assertEqual(result,0)
        (out/'retained-file/selected.webm').write_bytes(b'abd')
        self.assertEqual(media.finalize(out,'success'),1)
        self.assertFalse(strict_load(out/'live-status.json')['retained_verified_file'])

    def test_exact_same_run_native_evidence_admits_one_generation(self):
        """Exact source/run/attempt and six exact prerequisite outcomes admit budget one."""
        status=self.admit()
        self.assertTrue(status['admitted']);self.assertEqual(status['launch_budget'],1)
        self.assertEqual(status['live_result'],'NOT_TESTED')

    def test_automatic_event_cannot_admit_even_with_valid_artifact(self):
        """Supplying a valid-looking artifact on an automatic event cannot launch media."""
        for event in ('push','pull_request','pull_request_target','schedule',''):
            self.env['GITHUB_EVENT_NAME']=event
            with self.assertRaises(ValueError): self.admit()

    def test_rerun_attempt_is_denied(self):
        """GitHub reruns cannot reuse the original attempt's generation."""
        for attempt in ('2','0','01','', '1\ntrue'):
            self.env['GITHUB_RUN_ATTEMPT']=attempt
            with self.assertRaises(ValueError): self.admit()

    def test_wrong_source_or_selected_dispatch_ref_is_denied(self):
        """Input source, dispatch source and actual checkout must all match."""
        for key in ('GITHUB_SHA','PULQVA_SOURCE_SHA','PULQVA_REQUESTED_SOURCE'):
            original=self.env[key];self.env[key]='a'*40
            with self.assertRaises(ValueError): self.admit()
            self.env[key]=original

    def test_failed_incomplete_or_absent_contracts_are_denied(self):
        """Failure, cancellation and missing contracts cannot be replaced by a native artifact."""
        for value in ('failure','cancelled','skipped','in_progress',''):
            self.env['PULQVA_CONTRACTS_RESULT']=value
            with self.assertRaises(ValueError): self.admit()

    def test_foreign_run_or_platform_is_denied(self):
        """A different workflow run or unimplemented Windows live path is not admitted."""
        self.env['GITHUB_RUN_ID']='999'
        with self.assertRaises(ValueError): self.admit()
        self.env['GITHUB_RUN_ID']='12345';self.env['RUNNER_OS']='Windows'
        with self.assertRaises(ValueError): self.admit()

    def test_missing_prerequisite_denies_before_process(self):
        """Absent native evidence fails before any runtime invocation."""
        (self.prereq/'manifest.json').unlink()
        with patch.object(media,'bounded_process') as process:
            with self.assertRaises(OSError): self.capture()
            process.assert_not_called()

    def test_modified_evaluator_or_fixture_is_denied(self):
        """Prerequisite digests must match actual current evaluator bytes."""
        (self.root/'tools/ci/readiness_contract_matrix.py').write_text('changed')
        with self.assertRaises(ValueError): self.admit()

    def test_missing_native_test_is_denied_despite_consistent_raw_hashes(self):
        """Zero executed tests never become a prerequisite PASS, even with matching hashes."""
        item=self.raw['probes'][0];data=b'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n'
        item['stdout_hex']=data.hex();item['stdout_sha256']=hashlib.sha256(data).hexdigest();self.save_prereq()
        with self.assertRaises(ValueError): self.admit()

    def test_changed_probe_declaration_is_denied(self):
        """A self-consistent but altered prerequisite hypothesis is not the expected contract."""
        self.plan['probes'][0]['hypothesis']='different hypothesis'
        self.receipt['probes'][0]['hypothesis']='different hypothesis'
        self.receipt['manifest_sha256']=digest(self.plan);self.save_prereq()
        with self.assertRaises(ValueError): self.admit()

    def test_failed_or_incomplete_native_receipt_denied(self):
        """Closed is not enough: the actual prerequisite aggregate must be accepted."""
        self.receipt['closed']=False;self.receipt['accepted']=False;self.receipt['overall']='INCOMPLETE';self.save_prereq()
        with self.assertRaises(ValueError): self.admit()

    @POSIX
    def test_manifest_precedes_single_launch_and_success_is_scoped(self):
        """The real adapter freezes its manifest before invoking one synthetic executable."""
        out=self.base/'output';real=media.bounded_process
        def observed(*args,**kwargs):
            self.assertTrue((out/'manifest.json').is_file())
            return real(*args,**kwargs)
        with patch.object(media,'bounded_process',side_effect=observed) as call:
            self.assertEqual(self.capture(out),0);self.assertEqual(call.call_count,1)
        self.assertEqual(media.finalize(out,'success'),0)
        self.assertFalse(strict_load(out/'live-status.json')['retained_verified_file'])
        self.assertEqual(media.finalize(out,'success'),1)

    @POSIX
    def test_failed_fixture_has_receipt_and_no_retry(self):
        """One nonzero fixture exit is captured as FAIL without another launch."""
        self.binary.write_text('#!'+sys.executable+'\nimport sys\nsys.exit(7)\n')
        self.assertEqual(self.capture(),1)
        receipt=strict_load(self.base/'output/matrix_receipt.json')
        self.assertEqual(receipt['overall'],'FAIL');self.assertEqual(len(receipt['probes']),1)
        self.assertEqual(strict_load(self.base/'output/test-results.json')['returncode'],7)

    @POSIX
    def test_empty_or_fail_closed_output_does_not_prove_media_success(self):
        """An exit-zero failure marker cannot be counted as a successful media fixture."""
        self.binary.write_text('#!'+sys.executable+'\nprint("PULQVA_YTDLP_TOR_MEDIA_WINDOWS_FAIL_CLOSED_OK")\n')
        self.assertEqual(self.capture(),1)
        self.assertEqual(strict_load(self.base/'output/matrix_receipt.json')['overall'],'FAIL')

    @POSIX
    def test_identity_drift_closes_evidence_as_error(self):
        """Changed bytes after execution invalidate the generation rather than relaxing pins."""
        original=media.bounded_process
        def change(*args,**kwargs):
            result=original(*args,**kwargs);self.arti.write_bytes(b'changed');return result
        with patch.object(media,'bounded_process',side_effect=change): self.assertEqual(self.capture(),1)
        self.assertFalse(strict_load(self.base/'output/matrix_receipt.json')['intact'])

    @POSIX
    def test_duplicate_output_namespace_cannot_launch_or_overwrite(self):
        """An occupied generation directory blocks execution without replacing prior output."""
        out=self.base/'output';out.mkdir();(out/'manifest.json').write_bytes(b'original')
        with patch.object(media,'bounded_process') as call:
            with self.assertRaises(FileExistsError): self.capture(out)
            call.assert_not_called()
        self.assertEqual((out/'manifest.json').read_bytes(),b'original')

    @POSIX
    def test_timeout_and_output_cap_are_bounded_and_cleanup_owned_group(self):
        """Hanging and flooding synthetic children are killed without unbounded pipe buffering."""
        for code, expected, timeout in [('import time; time.sleep(30)','outer_deadline',0.2),
                                        ('import os; os.write(1,b"x"*100000)','output_limit',3)]:
            result=media.bounded_process([sys.executable,'-c',code],self.base,{'PATH':'/usr/bin:/bin'},timeout=timeout,cap=1024)
            self.assertEqual(result['reason'],expected);self.assertTrue(result['cleanup'])
            self.assertLessEqual(len(result['stdout']),1024);self.assertLess(result['duration_ms'],7000)

    @POSIX
    def test_child_does_not_inherit_token_or_proxy(self):
        """Fresh home/temp and explicit child env omit inherited CI secrets and proxies."""
        with patch.dict(os.environ,{'GITHUB_TOKEN':'private-fixture','HTTP_PROXY':'private-fixture'}):
            self.assertEqual(self.capture(),0)
        launch=strict_load(self.base/'output/launch.json')
        self.assertNotIn('GITHUB_TOKEN',launch['isolated_env_keys'])

    def test_preparation_failure_is_not_tested_not_pass(self):
        """Failure before a runtime manifest is explicit NOT_TESTED and nonzero."""
        out=self.base/'output'
        self.assertEqual(media.finalize(out,'failure'),1)
        self.assertEqual(strict_load(out/'live-status.json')['result'],'NOT_TESTED')

    def test_media_wiring_has_prior_gate_exact_checkout_and_always_capture(self):
        """The existing media job is the sole live path and keeps failure evidence uploads."""
        text=(ROOT/'.github/workflows/ytdlp-tor-media-check.yml').read_text()
        self.assertIn('needs: contracts',text)
        self.assertIn("github.event_name == 'workflow_dispatch' && needs.admission.outputs.admitted == 'true'",text)
        self.assertIn('ref: ${{ github.sha }}',text)
        self.assertIn('media-native-prerequisite-${{ github.run_id }}-${{ github.run_attempt }}',text)
        self.assertIn('media_evidence.py finalize',text)
        self.assertIn('media-evidence/*.json',text)
        self.assertNotIn('continue-on-error:',text)
        self.assertEqual(text.count('media_evidence.py capture'),2)

    def test_explicit_retained_admission_binds_both_native_boundaries(self):
        """Retained admission requires both same-run native manifests and records each digest."""
        self.prepare_retained()
        status = self.admit()
        self.assertEqual(status['profile'], media.RETAINED_PROFILE)
        self.assertEqual(status['prerequisite_manifest_sha256'], digest(self.plan))
        self.assertEqual(status['retained_prerequisite_manifest_sha256'], digest(self.extra_plan))

    def test_retained_profile_without_native_supplement_cannot_launch(self):
        """Selecting retained mode alone never authorizes a new runtime invocation."""
        self.env['PULQVA_MEDIA_PROFILE'] = media.RETAINED_PROFILE
        with patch.object(media, 'bounded_process') as called:
            with self.assertRaises((ValueError, OSError)):
                media.run_fixture(self.root, self.base/'out', self.prereq, self.binary,
                                  self.arti, self.ytdlp, self.env, retained=True)
            called.assert_not_called()
        self.assertFalse((self.base/'out').exists())

    def test_profile_mismatch_cannot_switch_capture_after_admission(self):
        """A legacy admission cannot be silently used to run the retained-file profile."""
        with patch.object(media, 'bounded_process') as called:
            with self.assertRaisesRegex(ValueError, 'capture_profile_mismatch'):
                media.run_fixture(self.root, self.base/'out', self.prereq, self.binary,
                                  self.arti, self.ytdlp, self.env, retained=True)
            called.assert_not_called()

    def test_unknown_profile_denied_before_any_prerequisite_or_launch(self):
        """Arbitrary profile strings never become commands, packages or test filters."""
        self.env['PULQVA_MEDIA_PROFILE'] = 'retained-commons; arbitrary-command'
        with patch.object(media, 'verify_prerequisite') as called:
            with self.assertRaisesRegex(ValueError, 'unknown_media_profile'): self.admit()
            called.assert_not_called()

    def test_retained_prerequisite_cannot_reuse_readiness_boundary(self):
        """A valid readiness receipt is not evidence for the retained-file verifier."""
        self.prepare_retained()
        for name in ('manifest.json', 'matrix_receipt.json', 'test-results.json'):
            (self.extra_dir/name).write_bytes((self.prereq/name).read_bytes())
        with self.assertRaisesRegex(ValueError, 'wrong_boundary'): self.admit()

    def test_retained_prerequisite_zero_tests_rejected_despite_fresh_raw_digest(self):
        """Zero executed native tests cannot authorize retention even with recomputed raw hashes."""
        self.prepare_retained()
        row = self.extra_raw['probes'][0]
        data = b'running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s\n'
        row['stdout_hex'] = data.hex(); row['stdout_sha256'] = hashlib.sha256(data).hexdigest()
        self.save_extra()
        with self.assertRaisesRegex(ValueError, 'exact_native_test_not_executed'): self.admit()

    def test_retained_prerequisite_foreign_source_denied(self):
        """A self-consistent retained prerequisite from another source is refused."""
        self.prepare_retained()
        for obj in (self.extra_plan['identity'], self.extra_receipt['identity']):
            obj['source_sha'] = obj['checkout_sha'] = 'b'*40
        self.extra_receipt['manifest_sha256'] = digest(self.extra_plan); self.save_extra()
        with self.assertRaisesRegex(ValueError, 'foreign_or_stale_prerequisite'): self.admit()

    @POSIX
    def test_retained_cleanup_failure_cannot_mint_verified_file(self):
        """Correct marker and file bytes do not compensate for incomplete process cleanup."""
        result, out = self.retained_capture(cleanup=False)
        self.assertEqual(result, 1)
        self.assertFalse(strict_load(out/'matrix_receipt.json')['retained_verified_file'])

    def test_retained_native_compilation_selects_only_fixed_discovery_libtest(self):
        """The shared collector selects the declared discovery libtest from Cargo JSON."""
        item = dict(reason='compiler-artifact', profile=dict(test=True),
                    target=dict(name='pulqva_discovery'), executable=str(self.binary))
        proc = subprocess.CompletedProcess([], 0, json.dumps(item), '')
        with patch.object(native.subprocess, 'run', return_value=proc) as called:
            self.assertEqual(native.compile_test_binary(media.RETAINED_PROFILE), self.binary)
            cmd = called.call_args.args[0]
            self.assertIn('pulqva-discovery', cmd); self.assertIn('--no-run', cmd)
            self.assertIn('--message-format=json', cmd)
            self.assertNotIn('pulqva-privacy', cmd)
        with self.assertRaises(ValueError): native.profile_spec('arbitrary')

    def test_retained_native_collector_executes_expected_named_tests_only(self):
        """The shared collector records all six retained-verifier tests after its manifest exists."""
        self.prepare_retained()
        plan = copy.deepcopy(self.extra_plan)
        plan['identity'] = native.measured_identity(plan, self.binary, media.RETAINED_PROFILE)
        out = self.base/'native-output'; out.mkdir()
        def execute(argv, **kwargs):
            self.assertEqual(strict_load(out/'manifest.json'), plan)
            self.assertIn(argv[1], [p['test'] for p in native.RETAINED_PROBES])
            data = ('running 1 test\ntest '+argv[1]+' ... ok\n\n'
                    'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s\n').encode()
            return subprocess.CompletedProcess(argv, 0, data, b'')
        with patch.object(native.subprocess, 'run', side_effect=execute) as called:
            self.assertEqual(native.collect(self.binary, out, plan, media.RETAINED_PROFILE), 0)
            self.assertEqual(called.call_count, 6)
        self.assertEqual(verify(plan, strict_load(out/'matrix_receipt.json')), 'PASS')
        self.assertEqual([p['id'] for p in strict_load(out/'test-results.json')['probes']],
                         [p['id'] for p in native.RETAINED_PROBES])

    def test_workflow_retained_profile_has_fixed_selection_prerequisites_and_upload(self):
        """The retained profile uses fixed paths, prior native gates and two-file-only retention."""
        text = (ROOT/'.github/workflows/ytdlp-tor-media-check.yml').read_text()
        self.assertIn('type: choice', text)
        self.assertIn('default: fixed-public-media', text)
        self.assertIn('PULQVA_MEDIA_PROFILE:', text)
        self.assertIn('cargo +1.91.0 test --locked -p pulqva-core -p pulqva-discovery --all-targets', text)
        self.assertIn('media-native-prerequisite/retained-commons" --retained-commons', text)
        retained = text.split('      - name: Capture one selected retained Commons journey', 1)[1].split('      - name:', 1)[0]
        legacy = text.split('      - name: Capture one immutable existing Tor media fixture', 1)[1].split('      - name:', 1)[0]
        self.assertIn("env.PULQVA_MEDIA_PROFILE == 'retained-commons'", retained)
        self.assertIn("env.PULQVA_MEDIA_PROFILE == 'fixed-public-media'", legacy)
        self.assertIn('media_evidence.py capture-retained', retained)
        self.assertIn('target/debug/examples/real_commons_file', retained)
        self.assertNotIn('${{ inputs.', retained)
        upload = text.split('      - name: Retain bounded public evidence', 1)[1]
        self.assertIn('media-evidence/retained-file/receipt.json', upload)
        self.assertIn('media-evidence/retained-file/selected.webm', upload)
        self.assertNotIn('retained-file/**', upload)
        self.assertNotIn('owned-home', upload); self.assertNotIn('owned-tmp', upload)


def run_matrix(out: Path) -> int:
    tests=list(unittest.defaultTestLoader.loadTestsFromTestCase(MediaEvidenceTests))
    checkout=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    paths=[Path(__file__),ROOT/'tools/ci/media_evidence.py',ROOT/'tools/ci/legacy_live_guard.py',
           ROOT/'tools/ci/readiness_contract_matrix.py',ROOT/'tools/ci/matrix_receipt.py',
           ROOT/'.github/workflows/ytdlp-tor-media-check.yml']
    def identity():
        comp=dict(runtime=media.captured(media.sha(Path(sys.executable))),evaluator=media.captured(media.sha(paths[1])),
                  fixtures=media.captured(media.sha(paths[0])),flags=media.captured(digest(['fresh TestCase','synthetic fixtures','one generation'])))
        comp.update({str(p.relative_to(ROOT)):media.captured(media.sha(p)) for p in paths[2:]})
        for k in ('model','prompt','schema'):
            comp[k]=dict(status='not_applicable',sha256=None,reason='synthetic controls; no model or Tor')
        return dict(source_sha=os.environ.get('PULQVA_SOURCE_SHA',checkout),checkout_sha=checkout,
                    run_id=os.environ.get('GITHUB_RUN_ID'),run_id_reason='not supplied locally',attempt=os.environ.get('GITHUB_RUN_ATTEMPT'),
                    attempt_reason='not supplied locally',job_id=None,job_id_reason='numeric ID externally bound',components=comp)
    plan=dict(protocol='pulqva-evidence-boundary-v1',boundary='T069-G2C2-media-controls-synthetic',generation_id=str(uuid.uuid4()),
              identity=identity(),environment=dict(os=platform.system(),arch=platform.machine(),toolchain=sys.version),
              isolation=dict(status='verified',strategy='fresh TestCase and owned synthetic repository per probe; no Tor/network',evidence_refs=['test_media_evidence.py:setUp']),
              probes=[dict(id=t._testMethodName,hypothesis=t.shortDescription(),invariant_set=['immutable source/interpreter','no native Tor execution','fresh fixture'],
                           expected='assertions pass',depends_on=[]) for t in tests])
    out.mkdir(parents=True,exist_ok=False);write_exclusive(out/'manifest.json',plan)
    recorder=MatrixRecorder(plan);raw=[]
    for test in tests:
        if identity()!=plan['identity']:
            recorder.fail_closed('source_changed');break
        result=unittest.TestResult();started=time.monotonic_ns();test.run(result)
        duration=(time.monotonic_ns()-started)/1_000_000
        state='ERROR' if result.errors else 'FAIL' if result.failures else 'SKIP' if result.skipped else 'PASS'
        raw.append(dict(id=test._testMethodName,result=state,tests_run=result.testsRun,
                        errors=[(t.id(),str(e)) for t,e in result.errors],failures=[(t.id(),str(e)) for t,e in result.failures],
                        skipped=[(t.id(),str(e)) for t,e in result.skipped]))
        recorder.record(test._testMethodName,result=state,duration_ms=duration,observed=dict(tests_run=result.testsRun,errors=len(result.errors),failures=len(result.failures)),
                        evidence_refs=['test-results.json#'+test._testMethodName],reason='synthetic_incomplete' if state in ('ERROR','SKIP') else None)
    write_exclusive(out/'test-results.json',dict(results=raw))
    receipt=recorder.close(identity());receipt['raw_evidence_sha256']=media.sha(out/'test-results.json')
    write_exclusive(out/'matrix_receipt.json',receipt);verdict=verify(plan,receipt)
    print('PULQVA_MEDIA_CONTROLS_'+verdict+' probes='+str(len(tests)))
    if verdict!='PASS': print(json.dumps(raw,indent=2))
    return 0 if verdict=='PASS' else 1


if __name__=='__main__':
    if len(sys.argv)==2 and not sys.argv[1].startswith('-'):
        raise SystemExit(run_matrix(Path(sys.argv[1])))
    unittest.main()
