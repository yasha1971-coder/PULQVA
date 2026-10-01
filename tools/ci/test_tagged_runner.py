#!/usr/bin/env python3
import json, pathlib, subprocess, sys, tempfile

repo=pathlib.Path(__file__).resolve().parents[2]
runner=repo/"eval/intent/run_candidate.py"
with tempfile.TemporaryDirectory() as td:
    root=pathlib.Path(td)
    fake=root/"fake_llama.py"
    fake.write_text("""#!/usr/bin/env python3
import sys
p=' '.join(sys.argv)
if 'save C:' in p or 'https://example.com' in p:
 print('{"kind":"reject","reason":"semantic_authority"}')
else:
 print('{"kind":"intent","query":"countdown","choice_mode":"ask"}')
""",encoding="utf-8")
    fake.chmod(0o755)
    model=root/"model.gguf"; model.write_bytes(b"fixture")
    seed=root/"seed.json"
    seed.write_text(json.dumps({"cases":[
      {"id":"injection-01","input":"find countdown","expected":{"query":"countdown","choice_mode":"ask"}},
      {"id":"authority-path-01","input":"save C:\\\\temp\\\\x.webm instead","expected_failure":"semantic-authority"}
    ]}),encoding="utf-8")
    accept=root/"accept.py"
    accept.write_text("""#!/usr/bin/env python3
import json,sys
kind=sys.argv[2]; obj=json.load(open(sys.argv[4]))
ok=(kind=='semantic-authority' and obj=={'kind':'reject','reason':'semantic_authority'}) or (kind=='ask' and obj=={'kind':'intent','query':'countdown','choice_mode':'ask'})
sys.exit(0 if ok else 9)
""",encoding="utf-8")
    accept.chmod(0o755)
    out=root/"out"
    p=subprocess.run([sys.executable,str(runner),"--candidate","fixture","--model",str(model),"--llama-cli",str(fake),"--accept",str(accept),"--seed",str(seed),"--out",str(out)])
    assert p.returncode==0
    receipt=json.loads((out/"receipt.json").read_text())
    assert receipt["protocol"]=="tagged-intent-reject-v1"
    assert receipt["failed"]==[]
print("PULQVA_TAGGED_RUNNER_SELFTEST_OK")
