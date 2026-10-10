"""Finite, offline checks. Uses the existing project evidence codec; no Tor launch."""
from __future__ import annotations
import argparse, copy, hashlib, json, os, platform, re, shutil, subprocess, sys, tempfile, time, uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools/ci'))
from matrix_receipt import MatrixRecorder, PROTOCOL, canonical, verify, strict_load

RUST_TESTS=['inline_policy_hash_encoding','tokens_and_query_boundary','session_and_index_are_bound',
 'foreign_host_origin_and_smuggling_fail_closed','local_fixture_or_foreign_selection_never_dispatches',
 'closed_channel_does_not_mint_success']
def hash_file(path):
    with Path(path).open('rb') as stream: return hashlib.file_digest(stream,'sha256').hexdigest()
def durable(path, data, mode='xb'):
    with path.open(mode) as stream: stream.write(data);stream.flush();os.fsync(stream.fileno())
def refresh(path,value):
    tmp=path.with_suffix('.pending'); durable(tmp,canonical(value)+b'\n','wb');os.replace(tmp,path)
def main():
    p=argparse.ArgumentParser();p.add_argument('--build-events',type=Path,required=True);p.add_argument('--binary',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
    a.out.mkdir(parents=True,exist_ok=False)
    events=[json.loads(line) for line in a.build_events.read_text(encoding='utf-8').splitlines() if line.startswith('{')]
    exes={e['executable'] for e in events if e.get('reason')=='compiler-artifact' and e.get('profile',{}).get('test') and e.get('target',{}).get('name')=='pulqva-prototype' and e.get('executable')}
    if len(exes)!=1: raise ValueError('Expected exactly one native test executable')
    test_exe=Path(next(iter(exes))).resolve();release_exe=a.binary.resolve();node=Path(shutil.which('node') or '')
    frozen=[test_exe,release_exe,Path(sys.executable),node,Path(__file__),ROOT/'apps/pulqva-prototype/index.html',ROOT/'apps/pulqva-prototype/test_ui.mjs',ROOT/'apps/pulqva-prototype/package.py',ROOT/'Cargo.lock',ROOT/'tools/ci/matrix_receipt.py']
    before={str(path):hash_file(path) for path in frozen}
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    captured=lambda value:{'status':'captured','sha256':hashlib.sha256(canonical(value)).hexdigest()}
    na={'status':'not_applicable','sha256':None,'reason':'No model or model prompt in this offline prototype gate'}
    identity={'source_sha':source,'checkout_sha':source,'run_id':os.getenv('GITHUB_RUN_ID'),'job_id':None,'job_id_reason':'GitHub exposes a job label, not a numeric job ID, in runner environment','attempt':os.getenv('GITHUB_RUN_ATTEMPT'),
      'components':{'runtime':captured({str(x):before[str(x)] for x in frozen[:4]}),'evaluator':captured({str(x):before[str(x)] for x in frozen[4:]}),'model':na,'prompt':na,'schema':captured({'tests':RUST_TESTS}),'flags':captured({'timeout':60,'network':'no live Tor or provider tests'}),'fixtures':captured({'test_source':before[str(ROOT/'apps/pulqva-prototype/test_ui.mjs')],'packager':before[str(ROOT/'apps/pulqva-prototype/package.py')]})}}
    for key in ('run_id','attempt'):
        if identity[key] is None: identity[key+'_reason']='Local execution, not a GitHub job'
    commands=[([str(test_exe),'--exact','tests::'+name,'--nocapture','--color','never'],name,'rust') for name in RUST_TESTS]
    commands += [([str(node),'--test',str(ROOT/'apps/pulqva-prototype/test_ui.mjs')],'local_ui_contracts','node'),([sys.executable,'-B',str(ROOT/'apps/pulqva-prototype/package.py'),'--self-test'],'package_identity_controls','python'),([str(release_exe),'--version'],'release_binary_version','version')]
    probes=[{'id':f'ALPHA-{i+1:02}','hypothesis':name,'invariant_set':['same frozen source and binaries','fresh child process and owned temporary home','no external network or Tor launch','no hot fixes'],'expected':{'contract_passed':True},'depends_on':[]} for i,(_,name,_) in enumerate(commands)]
    plan={'protocol':PROTOCOL,'boundary':'v01-alpha-offline-package-contracts-not-live-e2e','generation_id':str(uuid.uuid4()),'identity':identity,'environment':{'os':platform.platform(),'arch':platform.machine(),'toolchain':platform.python_version()},'isolation':{'status':'verified','strategy':'Fresh process and private HOME/TEMP for each bounded probe; source IDs and exact native test counts checked','evidence_refs':['collector-inputs.json']},'probes':probes}
    durable(a.out/'manifest.json',canonical(plan)+b'\n');durable(a.out/'collector-inputs.json',canonical({'hashes':before,'commands':commands})+b'\n')
    recorder=MatrixRecorder(plan)
    for spec,(cmd,name,kind) in zip(probes,commands):
        start=time.monotonic_ns(); result='ERROR'; reason=None
        with tempfile.TemporaryDirectory(prefix='pulqva-alpha-probe-') as temp:
            env={k:v for k,v in os.environ.items() if k in ('PATH','SystemRoot','SYSTEMROOT','WINDIR','COMSPEC','PATHEXT','LANG','LC_ALL')}
            env.update(HOME=temp,USERPROFILE=temp,TMPDIR=temp,TMP=temp,TEMP=temp,PYTHONDONTWRITEBYTECODE='1',RUST_TEST_THREADS='1')
            try:
                run=subprocess.run(cmd,cwd=ROOT,env=env,capture_output=True,timeout=60)
                text=(run.stdout+run.stderr).decode('utf-8',errors='replace')
                exact=(kind=='rust' and f'test tests::{name} ... ok' in text and '1 passed; 0 failed;' in text) or (kind=='node' and re.search(r'# pass 4\b',text) and re.search(r'# fail 0\b',text)) or (kind=='python' and 'Ran 3 tests' in text and re.search(r'\bOK\b',text)) or (kind=='version' and text.strip()=='PULQVA 0.1.0-alpha.1')
                result='PASS' if run.returncode==0 and exact else 'FAIL'
                observed={'returncode':run.returncode,'exact_contract_observed':bool(exact)};raw=run.stdout+b'\n--- stderr ---\n'+run.stderr
            except subprocess.TimeoutExpired as exc:
                raw=(exc.stdout or b'')+b'\n--- timeout ---\n'+(exc.stderr or b'');observed={'timeout':True};reason='probe_deadline'
        duration=(time.monotonic_ns()-start)/1e6
        filename=spec['id']+'.txt';durable(a.out/filename,raw)
        observed['raw_sha256']=hashlib.sha256(raw).hexdigest()
        durable(a.out/(spec['id']+'.json'),canonical({'id':spec['id'],'result':result,'duration_ms':duration,'observed':observed})+b'\n')
        recorder.record(spec['id'],result=result,duration_ms=duration,observed=observed,evidence_refs=[filename],reason=reason)
        durable(a.out/'journal.jsonl',canonical({'id':spec['id'],'result':result,'duration_ms':duration})+b'\n','ab')
        refresh(a.out/'matrix_receipt.json',recorder._snapshot())
    after={str(path):hash_file(path) for path in frozen}
    if after!=before: recorder.fail_closed('frozen_files_changed')
    receipt=recorder.close(copy.deepcopy(identity));refresh(a.out/'matrix_receipt.json',receipt)
    outcome=verify(strict_load(a.out/'manifest.json'),strict_load(a.out/'matrix_receipt.json'))
    durable(a.out/'independent-readback.json',canonical({'codec':outcome,'frozen_hashes_match':after==before,'matrix_sha256':hash_file(a.out/'matrix_receipt.json'),'scope':'offline alpha contracts only; live user and privacy acceptance NOT TESTED'})+b'\n')
    print('PULQVA_ALPHA_OFFLINE_'+outcome)
    return 0 if outcome=='PASS' else 1
if __name__=='__main__': raise SystemExit(main())
