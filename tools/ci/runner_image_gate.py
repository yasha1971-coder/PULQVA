"""Exact hosted image identity gate using the runner's own Set up job metadata."""
import argparse,json,os,pathlib,re
def image_identity(data):
 if not isinstance(data,list):raise ValueError("metadata must be a list")
 groups=[r for r in data if isinstance(r,dict) and r.get("group")=="Runner Image"]
 if len(groups)!=1 or not isinstance(groups[0].get("detail"),str):raise ValueError("missing or ambiguous Runner Image metadata")
 result={}
 for key in ("Image","Version"):
  values=re.findall(r"(?m)^\s*"+key+r":\s*(\S+)\s*$",groups[0]["detail"])
  if len(values)!=1:raise ValueError("missing or ambiguous "+key)
  result[key]=values[0]
 return result
def require_match(data,image,version):
 actual=image_identity(data)
 if actual!={"Image":image,"Version":version}:raise ValueError("runner image mismatch; no GUI launch")
 return actual
def require_consistent_identity(data,environment_version):
 actual=image_identity(data)
 if not environment_version or environment_version!=actual["Version"]:
  raise ValueError("contradictory or missing runner image identity; no GUI launch")
 return actual
def main():
 p=argparse.ArgumentParser();p.add_argument("--metadata",required=True);p.add_argument("--output",required=True);p.add_argument("--expected-image",required=True);p.add_argument("--expected-version",required=True);a=p.parse_args()
 receipt={"expected":{"Image":a.expected_image,"Version":a.expected_version},"ImageOS":os.environ.get("ImageOS"),"ImageVersion":os.environ.get("ImageVersion"),"result":"FAIL","gui_started":False}
 try:
  data=json.loads(pathlib.Path(a.metadata).read_text());receipt["actual"]=image_identity(data)
  require_consistent_identity(data,receipt["ImageVersion"])
  require_match(data,a.expected_image,a.expected_version);receipt["result"]="PASS"
 except Exception as e:receipt["error_type"]=type(e).__name__;raise
 finally:
  with pathlib.Path(a.output).open("x",encoding="utf-8") as f:json.dump(receipt,f,indent=2)
  print(json.dumps(receipt))
if __name__=="__main__":main()
