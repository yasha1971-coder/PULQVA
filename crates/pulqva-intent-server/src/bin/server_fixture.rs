use std::{env,net::TcpListener,thread,time::Duration};
fn main(){
 let a:Vec<String>=env::args().collect();
 if env::var("PULQVA_FIXTURE_EXIT").is_ok(){std::process::exit(7)}
 let port=a.windows(2).find(|w|w[0]=="--port").unwrap()[1].parse::<u16>().unwrap();
 if env::var("PULQVA_FIXTURE_NEVER_READY").is_ok(){thread::sleep(Duration::from_secs(5));return}
 let _l=TcpListener::bind(("127.0.0.1",port)).unwrap();
 loop{thread::sleep(Duration::from_secs(1))}
}
