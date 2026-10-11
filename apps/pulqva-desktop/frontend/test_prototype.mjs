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
  vm.runInNewContext(source,{window:{__TAURI__:{core:{invoke}}}, document:{querySelector:s=>el(s.slice(1)),createElement:node},console});
  const take=command=>{const i=pending.findIndex(p=>p.command===command);assert.notEqual(i,-1,command);return pending.splice(i,1)[0];};
  el('prototype-file').hidden=true;el('prototype-selection').hidden=true;
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
