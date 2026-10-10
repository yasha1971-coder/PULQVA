import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
import {test} from 'node:test';
const page=readFileSync(new URL('./index.html',import.meta.url),'utf8');
const source=page.split('<script>')[1].split('</script>')[0];
const token='a'.repeat(64);
function setup(initial={phase:'idle'}) {
 const elements=new Map(),calls=[];
 const element=()=>({textContent:'',hidden:false,disabled:false,value:'',handlers:{},children:[],addEventListener(n,f){this.handlers[n]=f;},append(...xs){this.children.push(...xs);},replaceChildren(){this.children=[];}});
 const get=id=>{if(!elements.has(id))elements.set(id,element());return elements.get(id);};
 const fetch=async (path,options={})=>{calls.push({path,options});assert.match(path,new RegExp(`^/${token}/(status|command)$`));return {ok:true,json:async()=>path.endsWith('/status')?initial:{accepted:true}};};
 vm.runInNewContext(source,{document:{getElementById:get,createElement:element},location:{pathname:`/${token}/`},fetch,setTimeout(){}});
 return {get,calls,tick:async()=>{for(let i=0;i<8;i++)await Promise.resolve();}};
}
test('query uses local JSON command, not a URL or executable instruction',async()=>{const h=setup();await h.tick();h.get('query').value='birds';h.get('form').handlers.submit({preventDefault(){}});await h.tick();const c=h.calls.find(c=>c.path.endsWith('/command'));assert.equal(c.options.method,'POST');assert.equal(c.options.redirect,'error');assert.deepEqual(JSON.parse(c.options.body),{action:'search',query:'birds'});});
test('real choice sends exactly its session and selected index',async()=>{const h=setup({phase:'choosing',session:'independent-session',candidates:[{index:1,title:'Birds',bytes:4096,source:'Wikimedia Commons'}]});await h.tick();h.get('choices').children[0].children[1].handlers.click();await h.tick();assert.deepEqual(JSON.parse(h.calls.find(c=>c.path.endsWith('/command')).options.body),{action:'select',session:'independent-session',index:1});});
test('untrusted title is text; no HTML rendering or direct remote fetch',async()=>{const h=setup({phase:'choosing',session:'s',candidates:[{index:0,title:'<img onerror=evil()>',bytes:16,source:'Commons'}]});await h.tick();assert.equal(h.get('choices').children[0].children[0].children[0].textContent,'<img onerror=evil()>');assert.equal(/innerHTML|outerHTML|document\.write|eval\s*\(/.test(source),false);});
test('completion displays saved path and identity without opening content',async()=>{const h=setup({phase:'complete',path:'C:\\Downloads\\selected.webm',sha256:'b'.repeat(64),bytes:42});await h.tick();assert.equal(h.get('file').hidden,false);assert.equal(h.get('path').textContent,'C:\\Downloads\\selected.webm');assert.match(h.get('digest').textContent,/42/);assert.equal(h.calls.filter(c=>c.path.endsWith('/command')).length,0);});
