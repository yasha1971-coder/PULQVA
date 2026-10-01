//! Supervised lifecycle boundary for the local PULQVA intent server.
mod diagnostics;
pub use diagnostics::{ReadinessDiagnostics, STDERR_RETAINED_BYTES};
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
        Self::spawn_internal(plan, &mut ReadinessDiagnostics::default(), false)
    }
    /// Opt-in capture for a fixed-public-fixture diagnostic run. No persistent logs by default.
    pub fn spawn_diagnostic(plan:&IntentServerPlan, trace:&mut ReadinessDiagnostics)->Result<Self,IntentServerError>{
        Self::spawn_internal(plan, trace, true)
    }
    fn spawn_internal(plan:&IntentServerPlan, trace:&mut ReadinessDiagnostics, capture:bool)->Result<Self,IntentServerError>{
        *trace = ReadinessDiagnostics::default();
        trace.phase = "process";
        trace.capture_enabled = capture;
        let started = Instant::now();
        let mut cmd=Command::new(&plan.executable);
        cmd.arg("-m").arg(&plan.model)
           .arg("--host").arg("127.0.0.1")
           .arg("--port").arg(plan.port.to_string())
           .stdin(Stdio::null()).stdout(Stdio::null())
           .stderr(if capture { Stdio::piped() } else { Stdio::null() })
           .env_clear();
        let child=cmd.spawn().map_err(|e| {
            trace.process_error = Some(format!("{:?}",e.kind()));
            IntentServerError::Spawn(e)
        })?;
        // Establish cleanup ownership immediately, including all subsequent error returns.
        let mut owner=Self{child,port:plan.port};
        if capture {
            if let Some(stderr)=owner.child.stderr.take(){
                trace.capture(stderr).map_err(IntentServerError::Spawn)?;
            }
        }
        let deadline=Instant::now()+plan.readiness_timeout;
        let addr=SocketAddrV4::new(Ipv4Addr::LOCALHOST,plan.port);
        loop {
            trace.elapsed_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            if owner.child.try_wait().map_err(IntentServerError::Spawn)?.is_some(){
                trace.phase = "process";
                trace.process_error = Some("exited_early".into());
                return Err(IntentServerError::ExitedEarly);
            }
            trace.phase = "health";
            trace.health_attempts += 1;
            match TcpStream::connect_timeout(&addr.into(),Duration::from_millis(50)) {
                Ok(mut stream) => {
                    let _=stream.set_read_timeout(Some(Duration::from_millis(100)));
                    let _=stream.set_write_timeout(Some(Duration::from_millis(100)));
                    match stream.write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n") {
                        Ok(()) => {
                            let mut response=Vec::new();
                            let read=stream.take(512).read_to_end(&mut response);
                            trace.health_response(&response);
                            match read {
                                Ok(_) => {
                                    if response.starts_with(b"HTTP/1.1 200 ") && response.ends_with(b"\r\n\r\n{\"status\":\"ok\"}"){
                                        trace.health_ok_seen = true;
                                        trace.health_error = None;
                                        trace.phase = "completion";
                                        trace.completion_attempts += 1;
                                        let endpoint=pulqva_intent_http::LoopbackIntentEndpoint::new(plan.port,Duration::from_millis(500)).map_err(|_|IntentServerError::InvalidPlan)?;
                                        match pulqva_intent_http::build_interpretation_request("find countdown", &plan.schema_json) {
                                            Ok(request) => match pulqva_intent_http::interpret_via_loopback(endpoint,&request) {
                                                Ok(_) => {
                                                    trace.ready=true;
                                                    trace.phase="ready";
                                                    trace.completion_error=None;
                                                    trace.elapsed_ms=started.elapsed().as_millis().min(u64::MAX as u128) as u64;
                                                    return Ok(owner);
                                                }
                                                Err(e) => trace.completion_error=Some(format!("{e:?}")),
                                            },
                                            Err(_) => trace.completion_error=Some("request_builder_invalid_plan".into()),
                                        }
                                    } else { trace.health_error=Some("health_predicate_mismatch".into()); }
                                }
                                Err(e) => trace.health_error=Some(format!("read:{:?}",e.kind())),
                            }
                        }
                        Err(e) => trace.health_error=Some(format!("write:{:?}",e.kind())),
                    }
                }
                Err(e) => trace.health_error=Some(format!("connect:{:?}",e.kind())),
            }
            trace.elapsed_ms=started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            if Instant::now()>=deadline { return Err(IntentServerError::ReadinessTimeout); }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn port(&self)->u16{self.port}
}
impl Drop for IntentServer {
    fn drop(&mut self){ let _=self.child.kill(); let _=self.child.wait(); }
}
