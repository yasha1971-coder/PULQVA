use pulqva_core::Interpretation;
use serde::Deserialize;
use std::{fmt, io::{Read, Write}, net::{Ipv4Addr, SocketAddrV4, TcpStream}, time::Duration};

const MAX_HTTP_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy)]
pub struct LoopbackIntentEndpoint {
    port: u16,
    timeout: Duration,
}
impl LoopbackIntentEndpoint {
    pub fn new(port: u16, timeout: Duration) -> Result<Self, IntentHttpError> {
        if port == 0 || timeout.is_zero() { return Err(IntentHttpError::InvalidPlan); }
        Ok(Self { port, timeout })
    }
}

#[derive(Debug)]
pub enum IntentHttpError {
    InvalidPlan, Io(std::io::Error), TooLarge, HttpStatus, Envelope, Interpretation,
}
impl fmt::Display for IntentHttpError {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for IntentHttpError {}
impl From<std::io::Error> for IntentHttpError { fn from(e:std::io::Error)->Self{Self::Io(e)} }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseEnvelope { content: String }

pub fn interpret_via_loopback(endpoint: LoopbackIntentEndpoint, request_json: &str)
    -> Result<Interpretation, IntentHttpError>
{
    let addr=SocketAddrV4::new(Ipv4Addr::LOCALHOST,endpoint.port);
    let mut s=TcpStream::connect_timeout(&addr.into(),endpoint.timeout)?;
    s.set_read_timeout(Some(endpoint.timeout))?;
    s.set_write_timeout(Some(endpoint.timeout))?;
    let req=format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        endpoint.port,request_json.len(),request_json);
    s.write_all(req.as_bytes())?;
    let mut bytes=Vec::new();
    s.take((MAX_HTTP_BYTES+1) as u64).read_to_end(&mut bytes)?;
    if bytes.len()>MAX_HTTP_BYTES { return Err(IntentHttpError::TooLarge); }
    let text=std::str::from_utf8(&bytes).map_err(|_|IntentHttpError::Envelope)?;
    let (head,body)=text.split_once("\r\n\r\n").ok_or(IntentHttpError::Envelope)?;
    let status=head.lines().next().ok_or(IntentHttpError::Envelope)?;
    if !status.starts_with("HTTP/1.1 200 ") { return Err(IntentHttpError::HttpStatus); }
    let env:ResponseEnvelope=serde_json::from_str(body).map_err(|_|IntentHttpError::Envelope)?;
    pulqva_intent_json::parse_interpretation_json(&env.content).map_err(|_|IntentHttpError::Interpretation)
}
