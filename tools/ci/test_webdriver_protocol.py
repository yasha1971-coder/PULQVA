import unittest,tempfile,pathlib,json,http.client,io
from webdriver_protocol import RecordedProtocol,ProtocolFailure,verify_second_choice
class Reply(io.BytesIO):
 pass
class MethodTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.trace=pathlib.Path(self.temp.name)/"commands.jsonl";self.calls=[]
 def tearDown(self):self.temp.cleanup()
 def opener(self,body):
  def run(req,timeout):self.calls.append(req);return Reply(body)
  return run
 def test_valid_w3c_response_and_no_payload_logging(self):
  p=RecordedProtocol(self.trace,self.opener(b'{"value":"private-query"}'))
  self.assertEqual(p.call('POST','/session',{'secret':'never-record'}),'private-query')
  r=json.loads(self.trace.read_text());self.assertEqual(r['result'],'PASS');self.assertNotIn('private-query',self.trace.read_text());self.assertNotIn('never-record',self.trace.read_text())
 def test_w3c_error_fails(self):
  with self.assertRaises(ProtocolFailure):RecordedProtocol(self.trace,self.opener(b'{"value":{"error":"invalid session id"}}')).call('GET','/session/x/title')
 def test_malformed_fails(self):
  with self.assertRaises(ProtocolFailure):RecordedProtocol(self.trace,self.opener(b'{}')).call('GET','/status')
 def test_disconnect_is_not_retried_and_retains_original_error(self):
  def fail(req,timeout):self.calls.append(req);raise http.client.RemoteDisconnected('closed')
  with self.assertRaises(http.client.RemoteDisconnected):RecordedProtocol(self.trace,fail,health=lambda:{'driver_returncode':1}).call('POST','/session')
  self.assertEqual(len(self.calls),1);r=json.loads(self.trace.read_text());self.assertEqual(r['error_type'],'RemoteDisconnected');self.assertEqual(r['native_health']['driver_returncode'],1)
 def test_timeout_fails_without_replay(self):
  def fail(req,timeout):self.calls.append(req);raise TimeoutError()
  with self.assertRaises(TimeoutError):RecordedProtocol(self.trace,fail).call('POST','/session/x/element/a/click',{})
  self.assertEqual(len(self.calls),1)
 def test_second_choice_requires_exact_visible_title_and_enabled_download(self):
  self.assertTrue(verify_second_choice('second','second',True,True))
  for args in [('second','first',True,True),('second','second',False,True),('second','second',True,False),('','',True,True)]:
   with self.assertRaises(ProtocolFailure):verify_second_choice(*args)

class CollectorManifestTests(unittest.TestCase):
 def make_actual_plan(self,receipt):
  import ast,os,hashlib,platform,uuid,subprocess
  from unittest.mock import patch
  from matrix_receipt import PROTOCOL,MatrixRecorder
  source=pathlib.Path(__file__).with_name("tauri_webdriver_smoke.py")
  tree=ast.parse(source.read_text())
  nodes=[n for n in tree.body if (isinstance(n,ast.FunctionDef) and n.name=="object_identity") or (isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id in ("identity","plan") for t in n.targets))]
  with tempfile.TemporaryDirectory() as tmp:
   root=pathlib.Path(tmp);app=root/"bin"/"PULQVA";app.parent.mkdir();app.write_bytes(b"unit-fixture-not-a-GUI-candidate")
   (root/"prebuild-plan.json").write_text(json.dumps(receipt))
   from types import SimpleNamespace
   linux_context=SimpleNamespace(platform=lambda:"Linux unit context",machine=lambda:"x86_64")
   scope=dict(app=app,pathlib=pathlib,json=json,hashlib=hashlib,platform=linux_context,uuid=uuid,PROTOCOL=PROTOCOL,os=os,subprocess=subprocess,__file__=str(source))
   with patch.dict(os.environ,{},clear=True),patch.object(subprocess,"check_output",return_value="3532dbed169139dba81a755aff23575f98745dc8"):
    exec(compile(ast.Module(body=nodes,type_ignores=[]),str(source),"exec"),scope)
    return MatrixRecorder(scope["plan"]).manifest()
 def valid(self):
  return {"ci_provenance":{"checkout_sha":"3532dbed169139dba81a755aff23575f98745dc8","run_id":"41030429934","attempt":"1"}}
 def test_actual_manifest_survives_sanitized_environment(self):
  p=self.make_actual_plan(self.valid());self.assertEqual(p["identity"]["run_id"],"41030429934");self.assertEqual(p["identity"]["attempt"],"1")
 def test_missing_receipt_rejected(self):
  with self.assertRaises((KeyError,ValueError)):self.make_actual_plan({})
 def test_missing_run_id_rejected(self):
  d=self.valid();del d["ci_provenance"]["run_id"]
  with self.assertRaises((KeyError,ValueError)):self.make_actual_plan(d)
 def test_invalid_attempt_rejected(self):
  d=self.valid();d["ci_provenance"]["attempt"]="invalid"
  with self.assertRaises(ValueError):self.make_actual_plan(d)

if __name__=='__main__':unittest.main()
