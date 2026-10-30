"""Real Tauri/WebKit window/input smoke; never mocks IPC or search results."""
import base64,json,pathlib,sys,time,urllib.request,subprocess,hashlib,socket,os
from runtime_linux_job import LinuxOwnedJob
app=pathlib.Path(sys.argv[1]).resolve();out=pathlib.Path(sys.argv[2]);out.mkdir(exist_ok=False)
job=LinuxOwnedJob();session=None;receipt={"gui_window_input":"FAIL","real_results":"NOT_TESTED","explicit_choice":"NOT_TESTED","real_tor_e2e":"NOT_TESTED"}
from webdriver_protocol import RecordedProtocol,verify_second_choice
def native_health():
 from runtime_linux_job import _identity
 rows=[]
 for pid,tick in job.members():
  try:
   raw=pathlib.Path("/proc/"+str(pid)+"/stat").read_text()
   fields=raw[raw.rfind(")")+2:].split()
   rows.append({"pid":pid,"starttime":tick,"comm":pathlib.Path("/proc/"+str(pid)+"/comm").read_text().strip(),"state":fields[0],"exit_code":int(fields[49]) if len(fields)>49 else None})
  except (FileNotFoundError,ProcessLookupError):pass
 return {"driver_returncode":None if job.process is None else job.process.poll(),"owned":rows}
protocol=RecordedProtocol(out/"commands.jsonl",health=native_health)
call=protocol.call
from matrix_receipt import MatrixRecorder,PROTOCOL,write_exclusive
import platform,uuid
def object_identity():
 components={k:{"status":"not_applicable","sha256":None,"reason":"No LLM in this public GUI test"} for k in ["model","prompt","schema"]}
 for k,b in {"runtime":app.read_bytes(),"evaluator":pathlib.Path(__file__).read_bytes()+pathlib.Path(__file__).with_name("webdriver_protocol.py").read_bytes(),"fixtures":b"countdown;second GUI result;public WebM <=8388608","flags":b"native wry;readiness95;download70;guardian220;socks5h;no fallback"}.items():components[k]={"status":"captured","sha256":hashlib.sha256(b).hexdigest()}
 return {"source_sha":"ef9f2bd10374a109e40d9e21bc729d73e06221de","checkout_sha":subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip(),"run_id":os.environ.get("GITHUB_RUN_ID"),"job_id":None,"job_id_reason":"GitHub provides job name, numeric job ID independently recorded by observer","attempt":os.environ.get("GITHUB_RUN_ATTEMPT"),"components":components}
identity=object_identity()
plan={"protocol":PROTOCOL,"boundary":"one frozen Linux GUI request-choice-file plus owned cleanup","generation_id":str(uuid.uuid4()),"identity":identity,"environment":{"os":platform.platform(),"arch":platform.machine(),"toolchain":"Tauri WebDriver2.0.6/WebKit2.52.6"},"isolation":{"status":"verified","strategy":"new hosted runner and exclusive empty HOME/XDG/Downloads before single GUI launch","evidence_refs":["../launch.json","../resource-layout.json"]},"probes":[{"id":"GUI_E2E","hypothesis":"exact app delivers new public file through real GUI and Tor","invariant_set":["Tor-only","fail-closed","exact-second-choice","file-readback"],"expected":"real GUI query >=2 results then second choice and new <=8MiB verified file","depends_on":[]},{"id":"CLEANUP","hypothesis":"owned processes and listening/bound sockets absent after run","invariant_set":["owned-only","no-live-owned-pid","no-listener-or-bound-udp"],"expected":"all current owned PID identities gone and no listener/bound UDP4444/4445/19050","depends_on":[]}]}
write_exclusive(out/"manifest.json",plan);matrix=MatrixRecorder(plan);boundary_started=time.monotonic()
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
  expected_title=text("#prototype-choices li:nth-child(2) strong")
  click("#prototype-choices li:nth-child(2) button")
  choice_deadline=time.monotonic()+8
  while not call("GET",prefix+"/element/"+find("#prototype-download")+"/enabled"):
   if time.monotonic()>choice_deadline:raise TimeoutError("GUI choice not accepted")
   time.sleep(.1)
  selected_title=text("#prototype-selected-title")
  verify_second_choice(expected_title,selected_title,call("GET",prefix+"/element/"+find("#prototype-selection")+"/displayed"),call("GET",prefix+"/element/"+find("#prototype-download")+"/enabled"))
  receipt["selected_title"]=selected_title
  receipt["explicit_choice"]="PASS";receipt["selected_index"]=1
  picture("03-explicit-choice.png")
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

except Exception as error:
 receipt["original_error_type"]=type(error).__name__
 receipt["failure_stage"]="native_GUI_command_or_acceptance"
 (out/"failure-health.json").write_text(json.dumps(native_health(),indent=2))
 raise
finally:
 e2e_duration=(time.monotonic()-boundary_started)*1000
 cleanup_started=time.monotonic()
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
 matrix.record("GUI_E2E",result="PASS" if receipt["real_tor_e2e"]=="PASS" else "ERROR",duration_ms=e2e_duration,observed=receipt.copy(),evidence_refs=["smoke-receipt.json","commands.jsonl"],reason=None if receipt["real_tor_e2e"]=="PASS" else receipt.get("original_error_type","early_exit"))
 matrix.record("CLEANUP",result=receipt["cleanup"],duration_ms=(time.monotonic()-cleanup_started)*1000,observed={"identity_absence":receipt["independent_identity_absence"],"live_ports":live_ports},evidence_refs=["smoke-receipt.json"])
 write_exclusive(out/"matrix_receipt.json",matrix.close(object_identity(),early_exit_reason="GUI_failure"))
print(json.dumps(receipt));assert receipt["gui_window_input"]=="PASS" and receipt["real_tor_e2e"]=="PASS" and receipt["cleanup"]=="PASS"
