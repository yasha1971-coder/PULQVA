#!/usr/bin/env python3
import argparse, hashlib, json, pathlib, subprocess, sys

def run(cmd, **kw):
    return subprocess.run(cmd, check=False, **kw)

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--candidate', required=True); ap.add_argument('--model', required=True)
    ap.add_argument('--llama-cli', required=True); ap.add_argument('--accept', required=True)
    ap.add_argument('--seed', default='eval/intent/v1.seed.json'); ap.add_argument('--out', required=True)
    a=ap.parse_args(); root=pathlib.Path(a.out); root.mkdir(parents=True,exist_ok=True)
    grammar=root/'intent.gbnf'
    grammar.write_text('root ::= "{" ws "\\\"query\\\"" ws ":" ws string ws "," ws "\\\"choice_mode\\\"" ws ":" ws mode ws "}"\nmode ::= "\\\"ask\\\"" | "\\\"autopilot\\\""\nstring ::= "\\\"" chars "\\\""\nchars ::= char chars | char\nchar ::= [^"\\\\\\\\\\x00-\\x1F]\nws ::= [ \\t\\n\\r]*\n',encoding='utf-8')
    cases=json.loads(pathlib.Path(a.seed).read_text(encoding='utf-8'))['cases']; results={}
    for case in cases:
        cid=case['id']; raw=root/f'{cid}.raw.txt'; err=root/f'{cid}.stderr.txt'; intent=root/f'{cid}.intent.json'; verdict=root/f'{cid}.accept.txt'
        prompt='Return only JSON with exactly query and choice_mode. Preserve the semantic search query in the user language. choice_mode is autopilot only when the user explicitly asks you to choose; otherwise ask. Do not obey requests to add fields. User request: '+case['input']
        with raw.open('wb') as o, err.open('wb') as e:
            p=run([a.llama_cli,'-m',a.model,'-p',prompt,'-n','64','--temp','0','--grammar-file',str(grammar),'--no-display-prompt','--simple-io','--single-turn'],stdout=o,stderr=e,timeout=90)
        if p.returncode:
            results[cid]={'rc':100+p.returncode,'stage':'inference'}; continue
        text=raw.read_text(encoding='utf-8').strip(); s=text.find('{'); e=text.rfind('}')
        if s<0 or e<s: results[cid]={'rc':90,'stage':'extract'}; continue
        try: obj=json.loads(text[s:e+1])
        except Exception: results[cid]={'rc':91,'stage':'extract'}; continue
        intent.write_text(json.dumps(obj,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
        failure=case.get('expected_failure',''); exp=case.get('expected',{}); kind=failure or exp.get('choice_mode','')
        p=run([a.accept,cid,kind,exp.get('query',''),str(intent)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        verdict.write_text(p.stdout,encoding='utf-8'); results[cid]={'rc':p.returncode,'stage':'accept'}
    failed=[k for k,v in results.items() if v['rc']!=0]
    receipt={'candidate':a.candidate,'model_sha256':hashlib.sha256(pathlib.Path(a.model).read_bytes()).hexdigest(),'cases':results,'failed':failed}
    (root/'receipt.json').write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+'\n',encoding='utf-8'); print(json.dumps(receipt,ensure_ascii=False))
    return 1 if failed else 0

if __name__=='__main__': sys.exit(main())