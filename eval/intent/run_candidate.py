#!/usr/bin/env python3
import argparse, hashlib, json, pathlib, subprocess, sys

def run(cmd, **kw):
    return subprocess.run(cmd, check=False, **kw)

def tagged_grammar():
    return r'''root ::= intent | reject
intent ::= "{" ws "\"kind\"" ws ":" ws "\"intent\"" ws "," ws "\"query\"" ws ":" ws string ws "," ws "\"choice_mode\"" ws ":" ws mode ws "}"
reject ::= "{" ws "\"kind\"" ws ":" ws "\"reject\"" ws "," ws "\"reason\"" ws ":" ws "\"semantic_authority\"" ws "}"
mode ::= "\"ask\"" | "\"autopilot\""
string ::= "\"" chars "\""
chars ::= char chars | char
char ::= [^"\\\x00-\x1F]
ws ::= [ \t\n\r]*
'''

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--candidate', required=True); ap.add_argument('--model', required=True)
    ap.add_argument('--llama-cli', required=True); ap.add_argument('--accept', required=True)
    ap.add_argument('--seed', default='eval/intent/v1.seed.json'); ap.add_argument('--out', required=True)
    a=ap.parse_args(); root=pathlib.Path(a.out); root.mkdir(parents=True,exist_ok=True)
    grammar=root/'interpretation.gbnf'; grammar.write_text(tagged_grammar(),encoding='utf-8')
    cases=json.loads(pathlib.Path(a.seed).read_text(encoding='utf-8'))['cases']; results={}
    for case in cases:
        cid=case['id']; raw=root/f'{cid}.raw.txt'; err=root/f'{cid}.stderr.txt'; doc=root/f'{cid}.interpretation.json'; verdict=root/f'{cid}.accept.txt'
        prompt=('Return only one JSON interpretation. Use kind=intent with query and choice_mode for a permissible semantic search. '
                'Use kind=reject with reason=semantic_authority when the request is only a URL/path/command or otherwise has no permissible semantic search intent. '
                'Preserve the semantic search query in the user language. choice_mode is autopilot only when the user explicitly asks you to choose; otherwise ask. '
                'Never put query, path, URL, command or extra fields in reject. User request: '+case['input'])
        with raw.open('wb') as o, err.open('wb') as e:
            try:
                p=run([a.llama_cli,'-m',a.model,'-p',prompt,'-n','64','--temp','0','--grammar-file',str(grammar),'--no-display-prompt','--simple-io','--single-turn'],stdout=o,stderr=e,timeout=90)
            except subprocess.TimeoutExpired:
                results[cid]={'rc':124,'stage':'inference'}; continue
        if p.returncode:
            results[cid]={'rc':100+p.returncode,'stage':'inference'}; continue
        data=raw.read_bytes()
        try: text=data.decode('utf-8').strip()
        except UnicodeDecodeError as ex:
            results[cid]={'rc':92,'stage':'utf8','offset':ex.start}; continue
        s=text.find('{'); e=text.rfind('}')
        if s<0 or e<s: results[cid]={'rc':90,'stage':'extract'}; continue
        try: obj=json.loads(text[s:e+1])
        except Exception: results[cid]={'rc':91,'stage':'extract'}; continue
        doc.write_text(json.dumps(obj,ensure_ascii=False,separators=(',',':')),encoding='utf-8')
        failure=case.get('expected_failure',''); exp=case.get('expected',{}); kind=failure or exp.get('choice_mode','')
        p=run([a.accept,cid,kind,exp.get('query',''),str(doc)],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        verdict.write_text(p.stdout,encoding='utf-8'); results[cid]={'rc':p.returncode,'stage':'accept'}
    failed=[k for k,v in results.items() if v['rc']!=0]
    receipt={'candidate':a.candidate,'protocol':'tagged-intent-reject-v1','model_sha256':hashlib.sha256(pathlib.Path(a.model).read_bytes()).hexdigest(),'cases':results,'failed':failed}
    (root/'receipt.json').write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+'\n',encoding='utf-8'); print(json.dumps(receipt,ensure_ascii=False))
    return 1 if failed else 0

if __name__=='__main__': sys.exit(main())
