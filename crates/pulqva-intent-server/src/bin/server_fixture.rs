use std::{env,io::{Read,Write},net::TcpListener,thread,time::Duration};
fn main(){
 let a:Vec<String>=env::args().collect();
 let port=a.windows(2).find(|w|w[0]=="--port").unwrap()[1].parse::<u16>().unwrap();
 let model=a.windows(2).find(|w|w[0]=="-m").unwrap()[1].clone();
 let l=TcpListener::bind(("127.0.0.1",port)).unwrap();
 let mut probes=0usize;
 loop{
   let (mut s,_)=l.accept().unwrap(); let mut b=[0u8;512]; let _=s.read(&mut b);
   probes+=1;
   let request=String::from_utf8_lossy(&b);
   let (status,body)=if request.starts_with("GET /health") {
     ("200 OK", if model.contains("loading") {"{\"status\":\"loading\"}"} else {"{\"status\":\"ok\"}"})
   } else if request.starts_with("POST /v1/chat/completions") && model.contains("bad-completion") {
     ("500 Internal Server Error","{\"error\":\"backend unavailable\"}")
   } else {
     ("200 OK","{\"choices\":[{\"message\":{\"content\":\"{\\\"kind\\\":\\\"intent\\\",\\\"query\\\":\\\"countdown\\\",\\\"choice_mode\\\":\\\"ask\\\"}\"}}]}")
   };
   let r=format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
   let _=s.write_all(r.as_bytes());
   if env::var("PULQVA_FIXTURE_ONCE").is_ok(){thread::sleep(Duration::from_millis(20));}
 }
}
