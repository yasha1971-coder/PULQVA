#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
import sys
ROOT = Path(__file__).resolve().parents[1]
REQUIRED=[
 'AGENTS.md','PROJECT_STATE.json','NEXT.md','kernel/CORE_CONTRACT.md','kernel/PRIVACY_INVARIANTS.md','kernel/UX_CONTRACT.md','kernel/KERNEL_VERSION',
 'decisions/ADR-0001-repository-is-source-of-truth.md','decisions/ADR-0002-kernel-vs-adapters.md','decisions/ADR-0003-atomic-work-green-checkpoints.md']
errors=[]
for rel in REQUIRED:
    if not (ROOT/rel).is_file(): errors.append(f'missing required file: {rel}')
try: state=json.loads((ROOT/'PROJECT_STATE.json').read_text(encoding='utf-8'))
except Exception as exc: errors.append(f'PROJECT_STATE.json is invalid: {exc}'); state={}
try: kv=(ROOT/'kernel/KERNEL_VERSION').read_text(encoding='utf-8').strip()
except Exception: kv=''
if state.get('project')!='PULQVA': errors.append('PROJECT_STATE.json project must be PULQVA')
if state.get('kernel_version')!=kv: errors.append(f'kernel version mismatch: state={state.get("kernel_version")!r}, file={kv!r}')
next_task=state.get('next_task')
if not next_task: errors.append('PROJECT_STATE.json next_task must be non-empty')
else:
    try: nt=(ROOT/'NEXT.md').read_text(encoding='utf-8')
    except Exception: nt=''
    if next_task not in nt: errors.append(f'NEXT.md does not mention next task {next_task}')
task_ids=set()
for bucket in ('READY','ACTIVE','DONE'):
    p=ROOT/'tasks'/bucket
    if not p.is_dir(): errors.append(f'missing task bucket: tasks/{bucket}'); continue
    for f in p.glob('T*.md'): task_ids.add(f.name.split('-',1)[0])
for key in ('last_completed_task','active_task','next_task'):
    v=state.get(key)
    if v and v not in task_ids: errors.append(f'{key} points to unknown task: {v}')
core=(ROOT/'kernel/CORE_CONTRACT.md').read_text(encoding='utf-8') if (ROOT/'kernel/CORE_CONTRACT.md').exists() else ''
privacy=(ROOT/'kernel/PRIVACY_INVARIANTS.md').read_text(encoding='utf-8') if (ROOT/'kernel/PRIVACY_INVARIANTS.md').exists() else ''
for needle in ('ZERO CONFIG','TOR BY DEFAULT','FAIL CLOSED','NO API KEY REQUIRED','NATURAL LANGUAGE FIRST'):
    if needle not in core: errors.append(f'kernel missing invariant marker: {needle}')
for needle in ('Tor failure is fail-closed','AI output is data, never code','Frontend does not own Internet access'):
    if needle not in privacy: errors.append(f'privacy contract missing marker: {needle}')
if errors:
    print('PULQVA continuity guard: FAIL')
    for e in errors: print('-',e)
    sys.exit(1)
print('PULQVA continuity guard: PASS')
print(f'kernel={kv} next={next_task}')
