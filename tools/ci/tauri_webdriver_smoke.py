"""Real Tauri/WebKit window/input smoke; never mocks IPC or search results."""
import base64,json,pathlib,sys,time,urllib.request,subprocess,hashlib,socket,os
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
  # Retry only idempotent readiness, never duplicate NewSession.
  while True:
   try:
    if call("GET","/status").get("ready") is True:break
   except (urllib.error.URLError,ConnectionError,OSError):pass
   if time.monotonic()>=deadline:raise TimeoutError("Native WebDriver status not ready")
   time.sleep(.1)
  value=call("POST","/session",{"capabilities":{"alwaysMatch":{"browserName":"wry","tauri:options":{"application":str(app)}}}})
  session=value["sessionId"]
  prefix="/session/"+session
  element=call("POST",prefix+"/element",{"using":"css selector","value":"#prototype-query"})
  eid=element["element-6066-11e4-a52e-4f735466cecf"]
  assert call("GET",prefix+"/element/"+eid+"/displayed") is True
  call("POST",prefix+"/element/"+eid+"/value",{"text":"countdown","value":list("countdown")})
  assert call("GET",prefix+"/element/"+eid+"/property/value")=="countdown"
  (out/"window-input.png").write_bytes(base64.b64decode(call("GET",prefix+"/screenshot")))
  receipt["gui_window_input"]="PASS";receipt["query_submitted"]=False
  def find(css):
   return call("POST",prefix+"/element",{"using":"css selector","value":css})["element-6066-11e4-a52e-4f735466cecf"]
  def text(css):return call("GET",prefix+"/element/"+find(css)+"/text")
  def click(css):call("POST",prefix+"/element/"+find(css)+"/click",{})
  def picture(name):
   time.sleep(.15)
   (out/name).write_bytes(base64.b64decode(call("GET",prefix+"/screenshot")))
  picture("01-query-entered.png")
  receipt["query_submitted_at_monotonic"]=time.monotonic()
  click("#prototype-search");receipt["query_submitted"]=True
  results_deadline=time.monotonic()+95
  while True:
   choices=call("POST",prefix+"/elements",{"using":"css selector","value":"#prototype-choices li"})
   if len(choices)>=2:break
   state=text("#prototype-status")
   if call("GET",prefix+"/element/"+find("#prototype-search")+"/enabled"):
    raise RuntimeError("Real search failed: "+state)
   if time.monotonic()>results_deadline:raise TimeoutError("Real Tor/results window expired: "+state)
   time.sleep(.3)
  receipt["real_results"]="PASS";receipt["result_count"]=len(choices)
  receipt["results_text"]=text("#prototype-choices");picture("02-real-results.png")
  click("#prototype-choices li:nth-child(2) button")
  choice_deadline=time.monotonic()+8
  while not call("GET",prefix+"/element/"+find("#prototype-download")+"/enabled"):
   if time.monotonic()>choice_deadline:raise TimeoutError("GUI choice not accepted")
   time.sleep(.1)
  receipt["explicit_choice"]="PASS";receipt["selected_index"]=1
  receipt["selected_title"]=text("#prototype-selected-title");picture("03-explicit-choice.png")
  click("#prototype-download");download_deadline=time.monotonic()+70
  while not call("GET",prefix+"/element/"+find("#prototype-file")+"/displayed"):
   if time.monotonic()>download_deadline:raise TimeoutError("GUI file delivery deadline")
   if call("GET",prefix+"/element/"+find("#prototype-query")+"/enabled"):
    raise RuntimeError("File delivery failed: "+text("#prototype-status"))
   time.sleep(.3)
  path=pathlib.Path(text("#prototype-file-path"))
  expected_root=pathlib.Path(os.environ["HOME"])/"Downloads"
  assert path.resolve().is_relative_to(expected_root.resolve()) and not path.is_symlink()
  size=int(text("#prototype-file-size"));assert 0<size<=8388608 and path.stat().st_size==size
  with path.open("rb") as stream:actual=hashlib.file_digest(stream,"sha256").hexdigest()
  assert actual==text("#prototype-file-sha")
  receipt["file"]={"path":str(path),"bytes":size,"sha256":actual}
  receipt["real_tor_e2e"]="PASS";picture("04-file-sha.png")

finally:
 if session:
  try:call("DELETE","/session/"+session)
  except Exception:pass
 identities=job.members()
 clean=job.close()
 from runtime_linux_job import _identity
 receipt["independent_identity_absence"]=all(_identity(p)!=(p,t) for p,t in identities)
 receipt["port_states"]=[]
 for table in ["tcp","tcp6","udp","udp6"]:
  for row in pathlib.Path("/proc/net/"+table).read_text().splitlines()[1:]:
   fields=row.split();port=int(fields[1].rsplit(":",1)[1],16)
   if port in [4444,4445,19050]:
    receipt["port_states"].append({"protocol":table,"port":port,"state":fields[3]})
 live_ports=[r for r in receipt["port_states"] if r["protocol"].startswith("udp") or r["state"]=="0A"]
 receipt["cleanup"]="PASS" if clean and not job.members() and receipt["independent_identity_absence"] and not live_ports else "FAIL"
 receipt["cleanup_note"]="TCP TIME_WAIT retained explicitly; acceptance requires no live owned PID/listener or bound UDP socket."
 (out/"smoke-receipt.json").write_text(json.dumps(receipt,indent=2))
print(json.dumps(receipt));assert receipt["gui_window_input"]=="PASS" and receipt["real_tor_e2e"]=="PASS" and receipt["cleanup"]=="PASS"
