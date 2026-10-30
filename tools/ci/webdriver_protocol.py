"""Fail-closed W3C command evidence; no command replay and no response payload logging."""
import json,time,urllib.request
class ProtocolFailure(RuntimeError):pass
class RecordedProtocol:
 def __init__(self,trace,open_url=urllib.request.urlopen,clock=time.monotonic,health=lambda:None):
  self.trace=trace;self.open_url=open_url;self.clock=clock;self.health=health;self.sequence=0
 def call(self,method,path,value=None):
  self.sequence+=1;start=self.clock()
  row={"sequence":self.sequence,"method":method,"endpoint":path,"result":"ERROR","duration_ms":None}
  try:
   data=None if value is None else json.dumps(value).encode()
   req=urllib.request.Request("http://127.0.0.1:4444"+path,data=data,method=method,headers={"Content-Type":"application/json"})
   with self.open_url(req,timeout=12) as response:
    payload=json.load(response)
   if not isinstance(payload,dict) or "value" not in payload:raise ProtocolFailure("missing W3C value")
   answer=payload["value"]
   if isinstance(answer,dict) and "error" in answer:raise ProtocolFailure(str(answer["error"]))
   row["result"]="PASS";return answer
  except Exception as error:
   row["error_type"]=type(error).__name__
   # Native HTTP failure details only; no request/response payload, query or title.
   raise
  finally:
   row["duration_ms"]=(self.clock()-start)*1000
   try:row["native_health"]=self.health()
   except Exception as error:row["health_error_type"]=type(error).__name__
   with open(self.trace,"a",encoding="utf-8") as stream:stream.write(json.dumps(row)+"\n")
def verify_second_choice(expected,selected,displayed,download_enabled):
 if not expected or expected!=selected or not displayed or not download_enabled:
  raise ProtocolFailure("second GUI choice not independently verified")
 return True
