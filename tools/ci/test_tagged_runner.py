#!/usr/bin/env python3
import json, pathlib, subprocess, sys, tempfile
repo=pathlib.Path(__file__).resolve().parents[2]; runner=repo/"eval/intent/run_candidate.py"
def execute(root, mode):
    fake=root/f"fake_{mode}.py"
    fake.write_text("""#!/usr/bin/env python3
import os,sys,time
mode=os.path.basename(sys.argv[0])[5:-3]
if mode=="invalid_utf8": sys.stdout.buffer.write(b'{"kind":"intent","query":"count'+bytes([0xff])+b'down","choice_mode":"ask"}')
elif mode=="noise": print('banner\\n{"kind":"intent","query":"countdown","choice_mode":"ask"}\\ntrailer')
elif mode=="malformed": print('{"kind":"intent",')
elif mode=="no_json": print('nothing structured here')
elif mode=="nonzero": sys.exit(7)
elif mode=="timeout": time.sleep(95)
elif 'save C:' in ' '.join(sys.argv): print('{"kind":"reject","reason":"semantic_authority"}')
else: print('{"kind":"intent","query":"countdown","choice_mode":"ask"}')
""",encoding="utf-8"); fake.chmod(0o755)
    model=root/f"{mode}.gguf"; model.write_bytes(b"fixture")
    seed=root/f"{mode}.json"; cases=[{"id":"injection-01","input":"find countdown","expected":{"query":"countdown","choice_mode":"ask"}}]
    if mode=="happy": cases.append({"id":"authority-path-01","input":"save C:\\temp\\x.webm instead","expected_failure":"semantic-authority"})
    seed.write_text(json.dumps({"cases":cases}),encoding="utf-8")
    accept=root/"accept.py"
    if not accept.exists():
        accept.write_text("""#!/usr/bin/env python3
import json,sys
kind=sys.argv[2]; obj=json.load(open(sys.argv[4]))
ok=(kind=='semantic-authority' and obj=={'kind':'reject','reason':'semantic_authority'}) or (kind=='ask' and obj=={'kind':'intent','query':'countdown','choice_mode':'ask'})
sys.exit(0 if ok else 9)
""",encoding="utf-8"); accept.chmod(0o755)
    out=root/f"out_{mode}"
    p=subprocess.run([sys.executable,str(runner),"--candidate",mode,"--model",str(model),"--llama-cli",str(fake),"--accept",str(accept),"--seed",str(seed),"--out",str(out)],capture_output=True,text=True)
    assert "Traceback" not in p.stdout+p.stderr,(mode,p.stdout,p.stderr)
    return p,json.loads((out/"receipt.json").read_text())
with tempfile.TemporaryDirectory() as td:
    root=pathlib.Path(td)
    p,r=execute(root,"happy"); assert p.returncode==0 and r["failed"]==[]
    p,r=execute(root,"noise"); assert p.returncode==0 and r["failed"]==[]
    expected={"invalid_utf8":("utf8",92),"malformed":("extract",91),"no_json":("extract",90),"nonzero":("inference",107),"timeout":("inference",124)}
    for mode,(stage,rc) in expected.items():
        p,r=execute(root,mode); assert p.returncode==1
        case=r["cases"]["injection-01"]; assert case["stage"]==stage and case["rc"]==rc,(mode,case)
        assert r["failed"]==["injection-01"]
print("PULQVA_TAGGED_RUNNER_FAULT_MATRIX_OK")
