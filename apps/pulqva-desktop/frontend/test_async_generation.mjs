import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import { test } from "node:test";

const source = readFileSync(new URL("./app.js", import.meta.url), "utf8");
function deferred() { let resolve, reject; const promise = new Promise((a,b) => { resolve=a; reject=b; }); return {promise,resolve,reject}; }
function harness() {
  const elements = new Map();
  function element(id) {
    if (!elements.has(id)) {
      elements.set(id, { id, textContent:"", hidden:false, disabled:false, value:"",
        dataset:{}, children:[], handlers:{}, replaceChildren(){this.children=[];},
        append(...nodes){this.children.push(...nodes);},
        addEventListener(name, fn){this.handlers[name]=fn;} });
    }
    return elements.get(id);
  }
  const calls=[];
  const pending=[];
  const document={
    querySelector(sel){ return element(sel.slice(1)); },
    createElement(tag){ return {tag,textContent:"",className:"",type:"",disabled:false,children:[],handlers:{},
      append(...nodes){this.children.push(...nodes);},
      addEventListener(name,fn){this.handlers[name]=fn;} }; }
  };
  const invoke=(name,args)=>{calls.push({name,args}); if(name==="app_status") return Promise.resolve({appVersion:"test",kernelVersion:"test",privacyMode:"Tor",coreReady:true});
    const d=deferred(); pending.push({name,args,...d}); return d.promise; };
  vm.runInNewContext(source,{window:{__TAURI__:{core:{invoke}}},document,console});
  const take=(name)=>{let i=pending.findIndex(x=>x.name===name);assert.notEqual(i,-1,"expected "+name);return pending.splice(i,1)[0];};
  const tick=async()=>{for(let i=0;i<4;i++)await Promise.resolve();};
  return {element,calls,take,tick};
}
function submit(h,value){h.element("intent-query").value=value;return h.element("intent-form").handlers.submit({preventDefault(){}});}
function candidate(title,locator){return {title,locator};}
async function ready(h,query,title,locator) {
  const work=submit(h,query);
  h.take("submit_intent").resolve({query,stage:"validated"});await h.tick();
  h.take("list_local_candidates").resolve({intentQuery:query,stage:"local",candidates:[candidate(title,locator)]});
  await work;
}
async function select(h){const b=h.element("candidate-list").children[0].children[1];return b.handlers.click();}
test("late old intent must not replace latest candidate list",async()=>{
  const h=harness();const old=submit(h,"old");const oldCall=h.take("submit_intent");
  const newest=submit(h,"new");h.take("submit_intent").resolve({query:"new",stage:"validated"});await h.tick();
  h.take("list_local_candidates").resolve({intentQuery:"new",stage:"local",candidates:[candidate("new title","new-loc")]});await newest;
  oldCall.resolve({query:"old",stage:"validated"});await h.tick();
  h.take("list_local_candidates").resolve({intentQuery:"old",stage:"local",candidates:[candidate("old title","old-loc")]});await old;
  assert.equal(h.element("candidate-list").children[0].children[0].children[0].textContent,"new title");
});
test("late selection cannot restore stale selection",async()=>{
  const h=harness();await ready(h,"old","old title","old-loc");
  const choosing=select(h);const pending=h.take("select_local_candidate");
  await ready(h,"new","new title","new-loc");
  pending.resolve({intentQuery:"old",title:"old title",locator:"old-loc",stage:"selected"});await choosing;
  assert.equal(h.element("selection-panel").hidden,true);
});
test("late download plan cannot cross request boundary",async()=>{
  const h=harness();await ready(h,"old","old title","old-loc");
  const choosing=select(h);h.take("select_local_candidate").resolve({intentQuery:"old",title:"old title",locator:"old-loc",stage:"selected"});await choosing;
  const downloading=h.element("download-action").handlers.click();const pending=h.take("plan_local_download");
  await ready(h,"new","new title","new-loc");
  pending.resolve({intentQuery:"old",title:"old title",locator:"old-loc",action:"download",stage:"planned",mediaSourceReady:true,mediaSourceStage:"ready"});await downloading;
  assert.equal(h.element("download-plan-panel").hidden,true);
});
