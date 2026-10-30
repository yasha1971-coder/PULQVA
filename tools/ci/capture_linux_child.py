"""Linux adaptation of the accepted bounded capture_child collector."""
import json,os,pathlib,subprocess,sys,threading,time
m=json.loads(pathlib.Path(sys.argv[1]).read_text());root=pathlib.Path(m["directory"])
allowed={"PATH","LANG","LC_ALL","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS"}
env={k:v for k,v in os.environ.items() if k in allowed}
env.update(m.get("isolated_environment",{}));env["PULQVA_TOR_DIAGNOSTICS"]="1"
child=subprocess.Popen(m["argv"],cwd=m["cwd"],env=env,stdin=subprocess.DEVNULL,
 stdout=subprocess.PIPE,stderr=subprocess.PIPE,close_fds=True,start_new_session=True)
(root/"child.json").open("x").write(json.dumps({"pid":child.pid,"environment_keys":sorted(env)}))
if m.get("fixture_collector_crash"):os._exit(23)
stop=threading.Event();errors=[]
def pump(pipe,name):
 count=0
 try:
  with (root/name).open("xb") as out:
   while True:
    data=os.read(pipe.fileno(),4096)
    if not data:break
    room=max(0,m["log_limit"]-count);out.write(data[:room]);out.flush();count+=len(data)
    if count>m["log_limit"]:stop.set();break
 except Exception as exc:errors.append(type(exc).__name__);stop.set()
threads=[threading.Thread(target=pump,args=(p,n),daemon=True) for p,n in
 [(child.stdout,"stdout.log"),(child.stderr,"stderr.log")]]
for t in threads:t.start()
while child.poll() is None and not stop.wait(.02):pass
for t in threads:t.join(.3)
result={"returncode":child.poll(),"log_cap_or_sink_error":stop.is_set(),"collector_errors":errors}
(root/"capture-result.json").open("x").write(json.dumps(result))
sys.exit(2 if stop.is_set() else 0)
