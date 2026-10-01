use pulqva_intent_http::{build_interpretation_request,interpret_via_loopback,LoopbackIntentEndpoint};
use pulqva_intent_server::{IntentServer,IntentServerPlan};
use std::{env,fs,time::Duration};

fn main(){
 let mut a=env::args().skip(1);
 let server=a.next().expect("llama-server"); let model=a.next().expect("model"); let schema_path=a.next().expect("schema");
 let out=a.next().expect("receipt"); let port=a.next().unwrap_or_else(||"19081".into()).parse().expect("port");
 let schema=fs::read_to_string(&schema_path).expect("schema");
 let plan=IntentServerPlan::new(server,model,schema.clone(),port,Duration::from_secs(120)).expect("plan");
 let _owner=IntentServer::spawn(&plan).expect("real llama-server readiness");
 let endpoint=LoopbackIntentEndpoint::new(port,Duration::from_secs(60)).expect("endpoint");
 let cases=[
   ("en","find a countdown video"),
   ("ru","найди видео обратного отсчёта"),
   ("uk","знайди відео зворотного відліку"),
 ];
 let mut rows=Vec::new();
 for (id,input) in cases {
   let request=build_interpretation_request(input,&schema).expect("request");
   match interpret_via_loopback(endpoint,&request) {
     Ok(x)=>rows.push(serde_json::json!({"id":id,"ok":true,"interpretation":format!("{x:?}")})),
     Err(e)=>rows.push(serde_json::json!({"id":id,"ok":false,"error":format!("{e:?}")})),
   }
 }
 let ok=rows.iter().all(|x|x["ok"]==true);
 let receipt=serde_json::json!({"protocol":"t069f-real-server-v1","cases":rows,"all_ok":ok});
 fs::write(out,serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
 println!("{receipt}");
 if !ok {std::process::exit(1)}
}
