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
if __name__=='__main__':unittest.main()
