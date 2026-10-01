use std::{env,io::{Read,Write},net::TcpListener,thread,time::Duration};
fn main(){
 let a:Vec<String>=env::args().collect();
 let port=a.windows(2).find(|w|w[0]=="--port").unwrap()[1].parse::<u16>().unwrap();
 let model=a.windows(2).find(|w|w[0]=="-m").unwrap()[1].clone();
 let l=TcpListener::bind(("127.0.0.1",port)).unwrap();
 let mut probes=0usize;
 loop{
   let (mut s,_)=l.accept().unwrap();
   let mut request_bytes=Vec::new(); let mut chunk=[0u8;1024];
   loop {
     let n=s.read(&mut chunk).unwrap(); if n==0 { break; }
     request_bytes.extend_from_slice(&chunk[..n]);
     if let Some(p)=request_bytes.windows(4).position(|w|w==b"\r\n\r\n") {
       let head=String::from_utf8_lossy(&request_bytes[..p]);
       let len=head.lines().find_map(|line|line.strip_prefix("Content-Length: ")).and_then(|x|x.parse::<usize>().ok()).unwrap_or(0);
       if request_bytes.len() >= p+4+len { break; }
     }
     if request_bytes.len()>16*1024 { break; }
   }
   probes+=1;
   let request=String::from_utf8_lossy(&request_bytes);
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
