import copy,hashlib,json,pathlib,subprocess,sys,time,platform
from matrix_receipt import PROTOCOL,PersistedMatrix,verify
from runtime_linux_job import _identity
root=pathlib.Path(sys.argv[1]);root.mkdir(exist_ok=False)
here=pathlib.Path(__file__).parent
captured=lambda data:{"status":"captured","sha256":hashlib.sha256(data).hexdigest()}
runtime=b"".join((here/p).read_bytes() for p in ["runtime_linux_job.py","capture_linux_child.py","linux_guardian.py"])
plan=dict(protocol=PROTOCOL,boundary="Linux bounded collector and independent guardian native admission",
 generation_id=root.name,identity=dict(source_sha="4618e013f55a5430ccba06f32b4a8f3b3e18eb83",
 checkout_sha="4618e013f55a5430ccba06f32b4a8f3b3e18eb83",run_id=None,run_id_reason="local WSL fixture",
 job_id=None,job_id_reason="local WSL fixture",attempt=None,attempt_reason="local native admission",
 components={n:captured(runtime if n=="runtime" else pathlib.Path(__file__).read_bytes() if n in ["fixtures","evaluator"] else b"not-used-local-native-fixture") for n in ["runtime","model","prompt","schema","flags","fixtures","evaluator"]}),
 environment=dict(os=platform.platform(),arch=platform.machine(),toolchain=sys.version),
 isolation=dict(status="verified",strategy="fresh namespace and dedicated subreaper guardian per fixture; product source base is reference only, no product launch",
 evidence_refs=["launch.json","guard-result.json"]),
 probes=[dict(id=n,hypothesis=h,invariant_set=["owned descendants only","bounded stdout/stderr","failclosed cleanup"],
 expected={"accepted":True},depends_on=[]) for n,h in [
 ("NORMAL","Both output streams fully retained on ordinary child exit"),
 ("CAP","Output overflow stops collector and independently cleans child"),
 ("DEADLINE","Unresponsive child is cleaned by native deadline guardian"),
 ("CRASH","Abrupt collector death still cleans its adopted child"),
 ("CLEANUP","Every observed identity gone; every guardian reports empty owned tree")]])
cases=[
 ("NORMAL","import sys;print('OUT');print('ERR',file=sys.stderr)",False,2,128),
 ("CAP","import os,time;os.write(1,b'X'*8192);time.sleep(60)",False,2,128),
 ("DEADLINE","import time;time.sleep(60)",False,.5,128),
 ("CRASH","import time;time.sleep(60)",True,2,128)]
results=[];boundary=PersistedMatrix(plan,root)
with boundary as rec:
 for name,script,crash,deadline,limit in cases:
  d=root/name;d.mkdir();m=dict(directory=str(d),argv=[sys.executable,"-c",script],cwd=str(d),log_limit=limit,deadline_seconds=deadline,fixture_collector_crash=crash)
  path=d/"launch.json";path.open("x").write(json.dumps(m))
  start=time.monotonic();p=subprocess.run([sys.executable,"-B",str(here/"linux_guardian.py"),str(path)],timeout=8,capture_output=True)
  guard=json.loads((d/"guard-result.json").read_text());ok=p.returncode==0 and guard["cleanup_pass"] and guard["identities_gone"] and not guard["members_after"]
  if name=="NORMAL":ok=ok and (d/"stdout.log").read_bytes()==b"OUT\n" and (d/"stderr.log").read_bytes()==b"ERR\n"
  if name=="CAP":ok=ok and (d/"stdout.log").stat().st_size==limit and json.loads((d/"capture-result.json").read_text())["log_cap_or_sink_error"]
  if name=="DEADLINE":ok=ok and guard["reason"]=="deadline"
  if name=="CRASH":ok=ok and not (d/"capture-result.json").exists()
  results.append(guard)
  rec.record(name,result="PASS" if ok else "FAIL",duration_ms=(time.monotonic()-start)*1000,
   observed={"accepted":ok,"guardian":guard},evidence_refs=[name+"/launch.json",name+"/guard-result.json"])
 cleanup_start=time.monotonic()
 identities=[tuple(i) for g in results for i in g["observed_identities"]]
 independent_gone=all(_identity(pid)!=(pid,ticks) for pid,ticks in identities)
 clean=independent_gone and all(x["cleanup_pass"] and x["identities_gone"] and not x["members_after"] for x in results)
 cleanup_ms=(time.monotonic()-cleanup_start)*1000
 rec.record("CLEANUP",result="PASS" if clean else "FAIL",
 duration_ms=cleanup_ms,observed={"all_owned_trees_empty":all(not x["members_after"] for x in results),
 "independent_identity_recheck":independent_gone,"identities_checked":len(identities)},
 evidence_refs=[n+"/guard-result.json" for n,_,_,_,_ in cases])

verdict=verify(plan,boundary.receipt);print(json.dumps({"verdict":verdict,"probes":[{"id":x["id"],"result":x["result"]} for x in boundary.receipt["probes"]],"application_launched":False}))
assert verdict=="PASS"
