"""Method canary only: compile verified SQLite C to objects, never run output."""
import hashlib,io,json,os,pathlib,subprocess,sys,tarfile,time,urllib.request
from runtime_windows_job import WindowsJob
ROOT=pathlib.Path(os.environ['RUNNER_TEMP'])/'pulqva-sqlite-canary'
def digest(b):return hashlib.sha256(b).hexdigest()
def download(name,version,sha):
 with urllib.request.urlopen(f'https://static.crates.io/crates/{name}/{name}-{version}.crate',timeout=30) as response:
  data=response.read(16*1024*1024+1)
 if len(data)>16*1024*1024 or digest(data)!=sha:raise ValueError('source identity mismatch')
 return tarfile.open(fileobj=io.BytesIO(data),mode='r:gz')
def main():
 ROOT.mkdir(exist_ok=False)
 receipt={'protocol':'arti-sqlite-method-canary-v1','status':'ERROR','source_sha':os.environ['GITHUB_SHA'],'image':os.environ.get('ImageVersion'),'rows':[],
 'limitations':['Method control only, not production-feature equivalence or historical SDK identity.','Existing selected compiler files authenticated; actual loaded DLL list not comprehensively captured.','No Tor or executable output launched.']}
 try:
  archive=download('libsqlite3-sys','0.37.0','b1f111c8c41e7c61a49cd34e44c7619462967221a6443b0ec299e0ac30cfb9b1')
  sources={}
  for name in ('build.rs','sqlite3/sqlite3.c','sqlite3/sqlite3.h'):
   data=archive.extractfile('libsqlite3-sys-0.37.0/'+name).read()
   sources[name]=digest(data)
   with (ROOT/pathlib.PurePosixPath(name).name).open('xb') as f:f.write(data)
  if sources['sqlite3/sqlite3.c']!='9512509b1bccb7461f79bea8aad6280ae4699e925fa4804381b71f59e7efb0c5':raise ValueError('C source mismatch')
  receipt['source_files']=sources
  build=(ROOT/'build.rs').read_text()
  # Unconditional upstream bundled flags only; optional Arti feature parity remains open.
  block=build[build.index('let mut cfg = cc::Build::new();'):]
  block=block[:block.index('.warnings(false)')]
  import re
  defines=re.findall(r'\.flag\("(-D[^"]+)"\)',block)
  if len(defines)<15:raise ValueError('unexpected bundled build script')
  compiler=os.environ['CC_x86_64_pc_windows_msvc']
  receipt['compiler_sha256']=digest(pathlib.Path(compiler).read_bytes())
  receipt['defines']=defines
  receipt['environment']={k:os.environ.get(k) for k in ['CL','_CL_','CFLAGS','CFLAGS_x86_64_pc_windows_msvc','INCLUDE','LIB','VCToolsVersion','WindowsSDKVersion']}
  # Remove uncontrolled compiler option variables for these declared method controls.
  for key in ('CL','_CL_','CFLAGS','CFLAGS_x86_64_pc_windows_msvc'):os.environ.pop(key,None)
  for arm in ('unchanged_a','unchanged_b','positive_define'):
   out=ROOT/'sqlite3.obj'
   if out.exists():raise ValueError('canonical object path unexpectedly occupied')
   args=[compiler,'/nologo','/MD','/O2','/Brepro','/W0','/showIncludes','/c','sqlite3.c','/Fosqlite3.obj']+defines
   if arm=='positive_define':args+=['-DSQLITE_DEFAULT_CACHE_SIZE=-12345']
   spec=ROOT/(arm+'-command.json')
   with spec.open('x') as f:json.dump(args,f)
   # Wrapper is created in an owned job before it may spawn the compiler.
   code="import json,pathlib,subprocess,sys; r=pathlib.Path(sys.argv[1]); a=sys.argv[2]; args=json.loads((r/(a+'-command.json')).read_text()); f=(r/(a+'-compiler.log')).open('xb'); p=subprocess.run(args,cwd=r,stdout=f,stderr=subprocess.STDOUT); f.close(); (r/(a+'-exit.json')).write_text(json.dumps({'exit':p.returncode}))"
   start=time.monotonic();marker=ROOT/(arm+'-exit.json')
   with WindowsJob() as job:
    job.arm_deadline(time.monotonic()+120)
    job.launch([sys.executable,'-B','-c',code,str(ROOT),arm],ROOT)
    while not marker.exists():
     if time.monotonic()-start>120:raise TimeoutError('compiler canary timeout')
     if not job.members():raise RuntimeError('wrapper ended without receipt')
     time.sleep(.05)
    if json.loads(marker.read_text())['exit']!=0:raise RuntimeError('compiler canary failed')
    job.cleanup()
    if job.members():raise RuntimeError('owned compiler job remains')
   data=out.read_bytes()
   receipt['rows'].append({'arm':arm,'sha256':digest(data),'bytes':len(data),'duration_ms':(time.monotonic()-start)*1000,'owned_job_cleanup':True})
   # Preserve each object; canonical source/output location remains identical per compile.
   out.rename(ROOT/(arm+'.obj'))
  rows=receipt['rows']
  receipt['negative_control_identical']=rows[0]['sha256']==rows[1]['sha256']
  receipt['positive_control_detected']=rows[0]['sha256']!=rows[2]['sha256']
  receipt['status']='PASS' if receipt['negative_control_identical'] and receipt['positive_control_detected'] else 'FAIL'
 except Exception as exc:
  receipt['error_class']=type(exc).__name__
  raise
 finally:
  with (ROOT/'receipt.json').open('x') as f:json.dump(receipt,f,indent=2)
 if receipt['status']!='PASS':raise RuntimeError('method controls failed')
if __name__=='__main__':main()
