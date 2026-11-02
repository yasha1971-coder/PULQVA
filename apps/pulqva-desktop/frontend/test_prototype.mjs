import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { test } from "node:test";
const source = readFileSync(new URL("./prototype.js", import.meta.url), "utf8");
function node(tag="div") {
  return { tag, children:[], handlers:{}, disabled:false, hidden:false, value:"", textContent:"",
    append(...values){this.children.push(...values);}, replaceChildren(){this.children=[];},
    addEventListener(event,fn){this.handlers[event]=fn;},
    querySelectorAll(tag){return this.children.flatMap(child=>[...(child.tag===tag?[child]:[]),...child.querySelectorAll(tag)]);}
  };
}
function harness() {
  const elements=new Map();const calls=[];const pending=[];
  const el=id=>{if(!elements.has(id))elements.set(id,node());return elements.get(id);};
  const invoke=(command,args)=>{calls.push({command,args});let resolve,reject;const promise=new Promise((a,b)=>{resolve=a;reject=b;});pending.push({command,args,resolve,reject});return promise;};
  vm.runInNewContext(source,{window:{__TAURI__:{core:{invoke}}}, document:{querySelector:s=>el(s.slice(1)),createElement:node},TextEncoder,console});
  const take=command=>{const i=pending.findIndex(p=>p.command===command);assert.notEqual(i,-1,command);return pending.splice(i,1)[0];};
  el('prototype-file').hidden=true;el('prototype-selection').hidden=true;el('prototype-interpretation').hidden=true;
  return {el,take,calls};
}
const nonce='a'.repeat(64);
const results=()=>({sessionId:nonce,query:'countdown',source:'Wikimedia Commons',interpretation:'keyword-search-not-AI',candidates:[{index:0,title:'zero',byteSize:200},{index:1,title:'one',byteSize:300}]});
const submit=h=>{h.el('prototype-query').value='countdown';return h.el('prototype-form').handlers.submit({preventDefault(){}});};
async function searched(h){const work=submit(h);h.take('prototype_search').resolve(results());await work;}
const choose=(h,index=1)=>h.el('prototype-choices').querySelectorAll('button')[index].handlers.click();
async function selected(h){await searched(h);const work=choose(h);h.take('prototype_select').resolve({sessionId:nonce,index:1,title:'one'});await work;}

test('PT-UI-01 real command chain retains selected nonce and index',async()=>{
 const h=harness();await selected(h);const work=h.el('prototype-download').handlers.click();const request=h.take('prototype_download');
 assert.equal(request.args.sessionId,nonce);assert.equal(request.args.index,1);assert.deepEqual(Object.keys(request.args).sort(),['index','sessionId']);
 request.resolve({title:'one',filePath:'Downloads/PULQVA/selected.webm',byteSize:300,sha256:'b'.repeat(64)});await work;
 assert.equal(h.el('prototype-file').hidden,false);assert.equal(h.el('prototype-file-size').textContent,'300');assert.equal(h.el('prototype-download').disabled,true);
 assert.equal(h.calls.some(c=>c.command.includes('local')),false);
});
test('PT-UI-02 duplicate search while busy starts no second backend operation',async()=>{
 const h=harness();const work=submit(h);await submit(h);assert.equal(h.calls.length,1);h.take('prototype_search').resolve(results());await work;
});
test('PT-UI-03 failed search offers no fabricated candidates or download',async()=>{
 const h=harness();const work=submit(h);h.take('prototype_search').reject({message:'Tor failed'});await work;
 assert.equal(h.el('prototype-choices').children.length,0);assert.equal(h.el('prototype-download').disabled,true);assert.equal(h.el('prototype-status').textContent,'Tor failed');
});
test('PT-UI-04 invalid result indices are rejected before presentation',async()=>{
 const h=harness();const work=submit(h);const r=results();r.candidates[1].index=9;h.take('prototype_search').resolve(r);await work;
 assert.equal(h.el('prototype-choices').children.length,0);assert.equal(h.el('prototype-download').disabled,true);
});
test('PT-UI-05 foreign selection response cannot enable download',async()=>{
 const h=harness();await searched(h);const work=choose(h);h.take('prototype_select').resolve({sessionId:'c'.repeat(64),index:1,title:'one'});await work;
 assert.equal(h.el('prototype-selection').hidden,true);assert.equal(h.el('prototype-download').disabled,true);
});
test('PT-UI-06 malformed file receipt cannot announce saved file',async()=>{
 const h=harness();await selected(h);const work=h.el('prototype-download').handlers.click();h.take('prototype_download').resolve({title:'one',filePath:'a',byteSize:300,sha256:'wrong'});await work;
 assert.equal(h.el('prototype-file').hidden,true);assert.equal(h.el('prototype-download').disabled,true);
});
test('PT-UI-07 clearing session invalidates all old result handlers',async()=>{
 const h=harness();await searched(h);const old=h.el('prototype-choices').querySelectorAll('button')[1];
 const work=h.el('prototype-clear').handlers.click();h.take('prototype_clear').resolve();await work;const count=h.calls.length;await old.handlers.click();assert.equal(h.calls.length,count);
 assert.equal(h.el('prototype-choices').children.length,0);assert.equal(h.el('prototype-download').disabled,true);
});
test('PT-UI-08 download cannot start before explicit selection',async()=>{
 const h=harness();await searched(h);const count=h.calls.length;await h.el('prototype-download').handlers.click();assert.equal(h.calls.length,count);
});

test('UI01-09 English product window is truthful about AI and size',()=>{
 const html=readFileSync(new URL('./prototype.html',import.meta.url),'utf8');
 assert.match(html,/<html lang="en">/);assert.match(html,/Find what matters\./);
 assert.match(html,/Keyword search · Local AI development mode/);assert.match(html,/up to 8 MiB/);
 assert.match(html,/Saved and verified/);assert.doesNotMatch(html,/AI Ready|Tor Connected|100 MB|sample results/i);
});
test('UI01-10 selected styling requires matching backend confirmation',async()=>{
 const h=harness();await searched(h);const work=choose(h);
 assert.equal(h.el('prototype-choices').children[1].className,'candidate-item');
 h.take('prototype_select').resolve({sessionId:nonce,index:1,title:'one'});await work;
 assert.equal(h.el('prototype-choices').children[1].className,'candidate-item is-selected');
 assert.equal(h.el('prototype-choices').querySelectorAll('button')[1].textContent,'Selected');
});
test('UI01-11 failed reselection clears the old selected marker',async()=>{
 const h=harness();await selected(h);const work=choose(h,0);
 h.take('prototype_select').resolve({sessionId:'c'.repeat(64),index:0,title:'zero'});await work;
 assert.equal(h.el('prototype-choices').children.some(r=>r.className.includes('is-selected')),false);
 assert.equal(h.el('prototype-download').disabled,true);
});
test('UI01-12 product assets contain no frontend external network entry points',()=>{
 const html=readFileSync(new URL('./prototype.html',import.meta.url),'utf8');
 const css=readFileSync(new URL('./style.css',import.meta.url),'utf8');
 const networkFixture=JSON.parse(readFileSync(new URL('../tests/prototype_ui_security.json',import.meta.url),'utf8'));
 assert.doesNotMatch(source,new RegExp(networkFixture.forbidden_runtime_pattern));
 assert.doesNotMatch(html,/(?:src|href)\s*=\s*["'](?:https?:|\/\/)/i);
 assert.doesNotMatch(css,/@import|url\s*\(/i);
 const ids=[...html.matchAll(/id="(prototype-[^"]+)"/g)].map(m=>m[1]);
 const required=['prototype-form','prototype-query','prototype-search','prototype-status','prototype-choices','prototype-selection','prototype-selected-title','prototype-download','prototype-file','prototype-file-title','prototype-file-path','prototype-file-size','prototype-file-sha','prototype-clear','prototype-interpret','prototype-ai-status','prototype-interpretation','prototype-interpretation-query','prototype-interpretation-mode'];
 assert.deepEqual(ids.slice().sort(),required.slice().sort());
});
test('UI01-13 displayed size is derived from validated backend bytes',async()=>{
 const h=harness();const work=submit(h);const r=results();r.candidates[1].byteSize=2131934;
 h.take('prototype_search').resolve(r);await work;
 assert.equal(h.el('prototype-choices').children[1].children[1].textContent,'2.03 MiB');
});
test('UI01-14 dynamic result titles remain text rather than HTML',async()=>{
 const h=harness();const work=submit(h);const r=results();r.candidates[1].title=JSON.parse(readFileSync(new URL('../tests/prototype_ui_security.json',import.meta.url),'utf8')).hostile_title;
 h.take('prototype_search').resolve(r);await work;
 assert.equal(h.el('prototype-choices').children[1].children[0].textContent,r.candidates[1].title);
 assert.doesNotMatch(source,/innerHTML|insertAdjacentHTML/);
});

const interpret=h=>h.el('prototype-interpret').handlers.click();
const intent=()=>({kind:'intent',query:'Big Buck Bunny',choice_mode:'ask'});
async function interpreted(h) {
 h.el('prototype-query').value='Big Buck Bunny';
 const work=interpret(h);h.take('interpret_local_request').resolve(intent());await work;
}
test('INTENT-UI-15 typed Intent is displayed without starting search or changing input',async()=>{
 const h=harness();h.el('prototype-query').value='Find Big Buck Bunny';
 const work=interpret(h);const req=h.take('interpret_local_request');
 assert.deepEqual(Object.keys(req.args),['query']);assert.equal(req.args.query,'Find Big Buck Bunny');
 assert.equal(h.el('prototype-ai-status').textContent,'Interpreting locally…');
 req.resolve(intent());await work;
 assert.equal(h.el('prototype-interpretation').hidden,false);
 assert.equal(h.el('prototype-interpretation-query').textContent,'Big Buck Bunny');
 assert.equal(h.el('prototype-interpretation-mode').textContent,'ask');
 assert.equal(h.el('prototype-query').value,'Find Big Buck Bunny');
 assert.deepEqual(h.calls.map(c=>c.command),['interpret_local_request']);
});
test('INTENT-UI-16 Reject displays an English explanation and never starts search',async()=>{
 const h=harness();h.el('prototype-query').value='an unclear request';
 const work=interpret(h);h.take('interpret_local_request').resolve({kind:'reject',reason:'semantic_authority'});await work;
 assert.equal(h.el('prototype-interpretation').hidden,true);
 assert.match(h.el('prototype-ai-status').textContent,/Please rephrase/);
 assert.equal(h.el('prototype-query').value,'an unclear request');assert.equal(h.calls.length,1);
});
test('INTENT-UI-17 server errors preserve input, hide old Intent and restore controls',async()=>{
 for(const code of ['unavailable','timeout','invalid_response']) {
  const h=harness();await interpreted(h);const work=interpret(h);
  h.take('interpret_local_request').reject({code,message:'private error text'});await work;
  assert.equal(h.el('prototype-interpretation').hidden,true);
  assert.equal(h.el('prototype-interpretation-query').textContent,'');
  assert.equal(h.el('prototype-query').value,'Big Buck Bunny');
  assert.equal(h.el('prototype-query').disabled,false);assert.equal(h.el('prototype-interpret').disabled,false);
  assert.equal(h.calls.length,2);assert.doesNotMatch(h.el('prototype-ai-status').textContent,/private/);
 }
});
test('INTENT-UI-18 duplicate and competing actions while interpreting issue no extra IPC',async()=>{
 const h=harness();h.el('prototype-query').value='Big Buck Bunny';const work=interpret(h);
 await interpret(h);await submit(h);await h.el('prototype-clear').handlers.click();await h.el('prototype-download').handlers.click();
 assert.equal(h.calls.length,1);assert.equal(h.el('prototype-interpret').disabled,true);
 assert.equal(h.el('prototype-search').disabled,true);
 h.take('interpret_local_request').resolve(intent());await work;
 assert.equal(h.el('prototype-search').disabled,false);
});
test('INTENT-UI-19 changed input invalidates stale success and failure without locking the UI',async()=>{
 for(const failed of [false,true]) {
  const h=harness();h.el('prototype-query').value='Big Buck Bunny';const work=interpret(h);
  h.el('prototype-query').value='new subject';h.el('prototype-query').handlers.input();
  const changedStatus=h.el('prototype-ai-status').textContent;
  const request=h.take('interpret_local_request');failed?request.reject({code:'timeout'}):request.resolve(intent());
  await work;
  assert.equal(h.el('prototype-interpretation').hidden,true);assert.equal(h.el('prototype-query').value,'new subject');
  assert.equal(h.el('prototype-ai-status').textContent,changedStatus);
  assert.equal(h.el('prototype-query').disabled,false);assert.equal(h.el('prototype-interpret').disabled,false);
 }
});
test('INTENT-UI-20 interpretation preserves the existing selection session and download receipt flow',async()=>{
 const h=harness();await selected(h);await interpreted(h);
 assert.equal(h.el('prototype-download').disabled,false);assert.equal(h.el('prototype-selection').hidden,false);
 const work=h.el('prototype-download').handlers.click();const request=h.take('prototype_download');
 assert.equal(request.args.sessionId,nonce);assert.equal(request.args.index,1);
 request.resolve({title:'one',filePath:'Downloads/PULQVA/new.webm',byteSize:300,sha256:'b'.repeat(64)});await work;
 assert.equal(h.el('prototype-file').hidden,false);
});
test('INTENT-UI-21 malformed or substituted IPC shapes cannot show a successful Intent',async()=>{
 const bad=[null,'raw model text',{kind:'other'},{...intent(),choice_mode:'shell'},
  {...intent(),query:''},{...intent(),query:'я'.repeat(257)},{...intent(),command:'run'},
  {kind:'reject',reason:'other'},{kind:'reject',reason:'semantic_authority',query:'injected'}];
 for(const value of bad) {
  const h=harness();h.el('prototype-query').value='Big Buck Bunny';const work=interpret(h);
  h.take('interpret_local_request').resolve(value);await work;
  assert.equal(h.el('prototype-interpretation').hidden,true);
  assert.match(h.el('prototype-ai-status').textContent,/failed validation/);assert.equal(h.calls.length,1);
 }
});
test('INTENT-UI-22 Clear and editing remove the prior local interpretation',async()=>{
 const h=harness();await interpreted(h);h.el('prototype-query').handlers.input();
 assert.equal(h.el('prototype-interpretation').hidden,true);
 await interpreted(h);const work=h.el('prototype-clear').handlers.click();
 h.take('prototype_clear').resolve();await work;
 assert.equal(h.el('prototype-interpretation').hidden,true);assert.equal(h.el('prototype-interpretation-query').textContent,'');
});
test('INTENT-UI-23 an in-flight Tor search also blocks interpretation',async()=>{
 const h=harness();const work=submit(h);await interpret(h);
 assert.deepEqual(h.calls.map(c=>c.command),['prototype_search']);
 h.take('prototype_search').resolve(results());await work;
 assert.equal(h.el('prototype-interpret').disabled,false);
});
