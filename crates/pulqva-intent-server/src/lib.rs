//! Supervised lifecycle boundary for the local PULQVA intent server.
use std::{fmt, io::{Read,Write}, net::{Ipv4Addr, SocketAddrV4, TcpStream}, path::PathBuf,
          process::{Child,Command,Stdio}, thread, time::{Duration,Instant}};

#[derive(Debug,Clone)]
pub struct IntentServerPlan {
    executable: PathBuf,
    model: PathBuf,
    schema_json: String,
    port: u16,
    readiness_timeout: Duration,
}
impl IntentServerPlan {
    pub fn new(executable: impl Into<PathBuf>, model: impl Into<PathBuf>,
        schema_json: impl Into<String>, port:u16, readiness_timeout:Duration)
        -> Result<Self,IntentServerError> {
        if port==0 || readiness_timeout.is_zero() { return Err(IntentServerError::InvalidPlan); }
        Ok(Self{executable:executable.into(),model:model.into(),schema_json:schema_json.into(),port,readiness_timeout})
    }
    pub fn port(&self)->u16{self.port}
}
#[derive(Debug)] pub enum IntentServerError { InvalidPlan, Spawn(std::io::Error), ExitedEarly, ReadinessTimeout }
impl fmt::Display for IntentServerError { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{self:?}")}}
impl std::error::Error for IntentServerError {}

pub struct IntentServer { child: Child, port:u16 }
impl IntentServer {
    pub fn spawn(plan:&IntentServerPlan)->Result<Self,IntentServerError>{
        let mut cmd=Command::new(&plan.executable);
        cmd.arg("-m").arg(&plan.model)
           .arg("--host").arg("127.0.0.1")
           .arg("--port").arg(plan.port.to_string())
           .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
           .env_clear();
        let mut child=cmd.spawn().map_err(IntentServerError::Spawn)?;
        let deadline=Instant::now()+plan.readiness_timeout;
        let addr=SocketAddrV4::new(Ipv4Addr::LOCALHOST,plan.port);
        loop {
            if child.try_wait().map_err(IntentServerError::Spawn)?.is_some(){
                return Err(IntentServerError::ExitedEarly);
            }
            if let Ok(mut stream)=TcpStream::connect_timeout(&addr.into(),Duration::from_millis(50)){
                let _=stream.set_read_timeout(Some(Duration::from_millis(100)));
                let _=stream.set_write_timeout(Some(Duration::from_millis(100)));
                if stream.write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").is_ok(){
                    let mut response=[0u8;256];
                    if let Ok(n)=stream.read(&mut response){
                        if n>0 && response[..n].starts_with(b"HTTP/1.1 200 ") && response[..n].ends_with(b"\r\n\r\n{\"status\":\"ok\"}"){
                            let endpoint=pulqva_intent_http::LoopbackIntentEndpoint::new(plan.port,Duration::from_millis(500)).map_err(|_|IntentServerError::InvalidPlan)?;
                            if let Ok(request)=pulqva_intent_http::build_interpretation_request("find countdown", &plan.schema_json) {
                                if pulqva_intent_http::interpret_via_loopback(endpoint,&request).is_ok(){ return Ok(Self{child,port:plan.port}); }
                            }
                        }
                    }
                }
            }
            if Instant::now()>=deadline {
                let _=child.kill(); let _=child.wait();
                return Err(IntentServerError::ReadinessTimeout);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn port(&self)->u16{self.port}
}
impl Drop for IntentServer {
    fn drop(&mut self){ let _=self.child.kill(); let _=self.child.wait(); }
}
