use pulqva_intent_server::{IntentServer,IntentServerPlan};
use std::{net::TcpListener,path::PathBuf,time::Duration};
fn free_port()->u16{let l=TcpListener::bind("127.0.0.1:0").unwrap();l.local_addr().unwrap().port()}
fn fixture()->PathBuf{PathBuf::from(env!("CARGO_BIN_EXE_server_fixture"))}
#[test] fn starts_only_on_loopback_and_stops_on_drop(){
 let p=IntentServerPlan::new(fixture(),"model.gguf",include_str!("../../../sidecars/llama.cpp/intent.schema.json"),free_port(),Duration::from_secs(1)).unwrap();
 let port=p.port(); let s=IntentServer::spawn(&p).unwrap(); assert_eq!(s.port(),port); drop(s);
 std::thread::sleep(Duration::from_millis(30));
 assert!(std::net::TcpStream::connect(("127.0.0.1",port)).is_err());
}

#[test] fn http_200_loading_is_not_ready(){
 let p=IntentServerPlan::new(fixture(),"loading-model.gguf",include_str!("../../../sidecars/llama.cpp/intent.schema.json"),free_port(),Duration::from_millis(80)).unwrap();
 assert!(IntentServer::spawn(&p).is_err());
}
