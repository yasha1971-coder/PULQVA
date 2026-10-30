"""Real Tauri/WebKit window/input smoke; never mocks IPC or search results."""
import base64,json,pathlib,sys,time,urllib.request,subprocess
from runtime_linux_job import LinuxOwnedJob
app=pathlib.Path(sys.argv[1]).resolve();out=pathlib.Path(sys.argv[2]);out.mkdir(exist_ok=False)
job=LinuxOwnedJob();session=None;receipt={"gui_window_input":"FAIL","real_results":"NOT_TESTED","explicit_choice":"NOT_TESTED","real_tor_e2e":"NOT_TESTED"}
def call(method,path,value=None):
 data=None if value is None else json.dumps(value).encode()
 req=urllib.request.Request("http://127.0.0.1:4444"+path,data=data,method=method,headers={"Content-Type":"application/json"})
 with urllib.request.urlopen(req,timeout=12) as r:return json.load(r)["value"]
try:
 with (out/"driver.log").open("xb") as log:
  driver=job.launch(["tauri-driver","--port","4444","--native-port","4445"],stdout=log,stderr=subprocess.STDOUT)
  deadline=time.monotonic()+20
  while True:
   try:
    value=call("POST","/session",{"capabilities":{"alwaysMatch":{"browserName":"wry","tauri:options":{"application":str(app)}}}})
    session=value["sessionId"];break
   except urllib.error.URLError:
    if time.monotonic()>=deadline:raise
    time.sleep(.25)
  prefix="/session/"+session
  element=call("POST",prefix+"/element",{"using":"css selector","value":"#prototype-query"})
  eid=element["element-6066-11e4-a52e-4f735466cecf"]
  assert call("GET",prefix+"/element/"+eid+"/displayed") is True
  call("POST",prefix+"/element/"+eid+"/value",{"text":"countdown","value":list("countdown")})
  assert call("GET",prefix+"/element/"+eid+"/property/value")=="countdown"
  (out/"window-input.png").write_bytes(base64.b64decode(call("GET",prefix+"/screenshot")))
  receipt["gui_window_input"]="PASS";receipt["query_submitted"]=False
finally:
 if session:
  try:call("DELETE","/session/"+session)
  except Exception:pass
 clean=job.close();receipt["cleanup"]="PASS" if clean and not job.members() else "FAIL"
 receipt["owned_members_after"]=job.members()
 (out/"smoke-receipt.json").write_text(json.dumps(receipt,indent=2))
print(json.dumps(receipt));assert receipt["gui_window_input"]=="PASS" and receipt["cleanup"]=="PASS"
