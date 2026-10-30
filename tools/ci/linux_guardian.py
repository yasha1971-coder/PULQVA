"""Dedicated deadline owner, independent from the agent and capture child."""
import json,pathlib,sys,time
from runtime_linux_job import LinuxOwnedJob,_identity
m=json.loads(pathlib.Path(sys.argv[1]).read_text());root=pathlib.Path(m["directory"])
job=LinuxOwnedJob();start=time.monotonic();deadline=start+m["deadline_seconds"];seen=set()
reason="collector_exit";clean=False
try:
 child=job.launch([sys.executable,"-B",str(pathlib.Path(__file__).with_name("capture_linux_child.py")),sys.argv[1]],
   stdin=-3,stdout=-3,stderr=-3)
 (root/"owner.json").open("x").write(json.dumps({"owner_pid":job.owner,"collector_pid":child.pid,"deadline_seconds":m["deadline_seconds"]}))
 while child.poll() is None:
  seen.update(job.members())
  if (root/"abort.request").exists():reason="abort";break
  if time.monotonic()>=deadline:reason="deadline";break
  time.sleep(.02)
finally:
 seen.update(job.members());clean=job.close()
 result={"reason":reason,"cleanup_pass":clean,"members_after":job.members(),
 "observed_identities":sorted(seen),"identities_gone":all(_identity(p)!= (p,t) for p,t in seen),
 "duration_ms":(time.monotonic()-start)*1000}
 (root/"guard-result.json").open("x").write(json.dumps(result))
