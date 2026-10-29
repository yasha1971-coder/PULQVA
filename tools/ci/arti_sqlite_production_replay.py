"""Isolated production SQLite recipe replay; no full Arti or Tor."""
import hashlib,io,json,os,pathlib,re,subprocess,sys,tarfile,time,tomllib,urllib.request
from runtime_windows_job import WindowsJob
ROOT=pathlib.Path(os.environ['RUNNER_TEMP'])/'pulqva-sqlite-production-replay'
EXPECTED='78baee9501df02656726f9d37fb8af49ee6465c8589d46c94c502bbb20a7800e'
def sha(b):return hashlib.sha256(b).hexdigest()
def archive(name,version,expected):
 with urllib.request.urlopen(f'https://static.crates.io/crates/{name}/{name}-{version}.crate',timeout=30) as response:b=response.read(16*1024*1024+1)
 if len(b)>16*1024*1024 or sha(b)!=expected:raise ValueError('source mismatch')
 return tarfile.open(fileobj=io.BytesIO(b),mode='r:gz')
def owned(args,label,timeout=180):
 with (ROOT/(label+'-args.json')).open('x') as f:json.dump(args,f)
 code="import json,pathlib,subprocess,sys;r=pathlib.Path(sys.argv[1]);n=sys.argv[2];a=json.loads((r/(n+'-args.json')).read_text());o=(r/(n+'.stdout')).open('xb');e=(r/(n+'.stderr')).open('xb');p=subprocess.run(a,cwd=r,stdout=o,stderr=e);o.close();e.close();(r/(n+'-exit.json')).write_text(json.dumps({'exit':p.returncode}))"
 marker=ROOT/(label+'-exit.json');start=time.monotonic()
 with WindowsJob() as job:
  job.arm_deadline(time.monotonic()+timeout)
  job.launch([sys.executable,'-B','-c',code,str(ROOT),label],ROOT)
  while not marker.exists():
   if time.monotonic()-start>timeout:raise TimeoutError(label)
   if not job.members():raise RuntimeError('missing wrapper exit')
   time.sleep(.05)
  job.cleanup()
 if json.loads(marker.read_text())['exit']!=0:raise RuntimeError(label+' failed')
 return {'duration_ms':(time.monotonic()-start)*1000,'owned_job_cleanup':True}
def main():
 ROOT.mkdir(exist_ok=False)
 receipt={'protocol':'arti-sqlite-production-replay-v1','status':'ERROR','source_sha':os.environ['GITHUB_SHA'],'expected_object_sha256':EXPECTED,'rows':[],'limits':['Restores inferred bundled features, not historical SDK identity.','Original Arti build argv not retained; exact equality is tested, never assumed.']}
 try:
  a=archive('arti','2.6.0','6c2d88fb27c29401cac3d384ef7c3998915192b9e08fda9823e02ca7d578be2e')
  original_lock=a.extractfile('arti-2.6.0/Cargo.lock').read()
  if sha(original_lock)!='939d35880fcb517cce8de9588d9f4b1cc2eb1aa548b88f0a1569da67ba1a0567':raise ValueError('lock mismatch')
  packages=tomllib.loads(original_lock.decode())['package']
  z=archive('libsqlite3-sys','0.37.0','b1f111c8c41e7c61a49cd34e44c7619462967221a6443b0ec299e0ac30cfb9b1')
  source=ROOT/'source';(source/'sqlite3').mkdir(parents=True)
  for name in ('build.rs','sqlite3/sqlite3.c','sqlite3/sqlite3.h'):
   with (source/name).open('xb') as f:f.write(z.extractfile('libsqlite3-sys-0.37.0/'+name).read())
  build=(source/'build.rs').read_text();block=build[build.index('let mut cfg = cc::Build::new();'):];block=block[:block.index('.warnings(false)')]
  defines=re.findall(r'\.flag\("(-D[^"]+)"\)',block)
  if len(defines)!=19:raise ValueError('unexpected defines')
  helper=ROOT/'helper';(helper/'src').mkdir(parents=True)
  manifest='[package]\nname="sqlite-replay-helper"\nversion="0.0.0"\nedition="2021"\n[dependencies]\ncc="=1.4.0"\nfind-msvc-tools="=0.1.9"\nshlex="=2.0.1"\n'
  (helper/'Cargo.toml').write_text(manifest)
  rust='fn main(){let src=std::env::var("SQLITE_REPLAY_SOURCE").unwrap();std::env::set_current_dir(src).unwrap();let mut c=cc::Build::new();c.file("sqlite3/sqlite3.c").out_dir(std::env::var("SQLITE_REPLAY_OUT").unwrap()).opt_level(3).debug(false).target("x86_64-pc-windows-msvc").host("x86_64-pc-windows-msvc").warnings(false);c.flags('+json.dumps(defines)+');println!("REPLAY_TOOL={:?}",c.get_compiler().to_command());c.compile("sqlite3");}'
  (helper/'src/main.rs').write_text(rust)
  cargo=['cargo','+1.91.0']
  receipt['rows'].append(owned(cargo+['metadata','--manifest-path',str(helper/'Cargo.toml'),'--format-version','1'],'metadata'))
  metadata=json.loads((ROOT/'metadata.stdout').read_text())
  active={n['id'] for n in metadata['resolve']['nodes']}
  helper_lock=tomllib.loads((helper/'Cargo.lock').read_text())
  resolved=[]
  for p in metadata['packages']:
   if p['id'] not in active or p['name']=='sqlite-replay-helper':continue
   old=next(x for x in packages if x['name']==p['name'] and x['version']==p['version'])
   new=next(x for x in helper_lock['package'] if x['name']==p['name'] and x['version']==p['version'])
   if old.get('checksum')!=new.get('checksum'):raise ValueError('helper dependency drift')
   resolved.append({'name':p['name'],'version':p['version'],'checksum':new.get('checksum')})
  receipt['verified_helper_dependencies']=resolved
  out=pathlib.Path(os.environ['RUNNER_TEMP'])/'pulqva-arti-target/release/build/libsqlite3-sys-2afacf9793ab054e/out';out.mkdir(parents=True,exist_ok=False)
  os.environ['SQLITE_REPLAY_SOURCE']=str(source);os.environ['SQLITE_REPLAY_OUT']=str(out)
  os.environ['OPT_LEVEL']='3';os.environ['DEBUG']='false';os.environ['CARGO_CFG_TARGET_FEATURE']='fxsr,sse,sse2'
  os.environ['CC_ENABLE_DEBUG_OUTPUT']='1'
  receipt['defines']=defines
  receipt['compiler_environment']={k:os.environ.get(k) for k in ('CC_x86_64_pc_windows_msvc','CL','_CL_','CFLAGS','TARGET_CFLAGS','INCLUDE','LIB','WindowsSDKVersion')}
  receipt['rows'].append(owned(cargo+['run','--manifest-path',str(helper/'Cargo.toml'),'--release','--locked'],'replay',240))
  objects=list(out.glob('*sqlite3.o'))
  if len(objects)!=1:raise ValueError('object inventory mismatch')
  b=objects[0].read_bytes()
  receipt.update(object_name=objects[0].name,object_bytes=len(b),object_sha256=sha(b),status='PASS' if sha(b)==EXPECTED else 'REPLAY_HASH_MISMATCH')
  (ROOT/'sqlite3-replayed.obj').write_bytes(b)
 except Exception as exc:
  receipt['error_class']=type(exc).__name__;raise
 finally:
  with (ROOT/'receipt.json').open('x') as f:json.dump(receipt,f,indent=2)
 if receipt['status']!='PASS':raise RuntimeError('isolated object differs; no full Arti admission')
if __name__=='__main__':main()
