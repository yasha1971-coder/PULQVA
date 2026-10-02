use pulqva_core::Interpretation;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt, io::{self, Read, Write}, net::{Ipv4Addr, SocketAddrV4, TcpStream}, time::{Duration, Instant}};

const MAX_HTTP_BYTES: usize = 64 * 1024;

// Production policy derives from the T069-E interpreter rules in eval/intent.
// Compile it in: user input cannot select a policy file or add message roles.
// This is model guidance, not a replacement for strict Rust validation.
const INTENT_SYSTEM_POLICY: &str = include_str!("../../../sidecars/llama.cpp/intent.system.txt");
const _: () = assert!(!INTENT_SYSTEM_POLICY.is_empty() && INTENT_SYSTEM_POLICY.len() <= 2048);

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
    InvalidPlan, Io(std::io::Error), TooLarge, HttpStatus, Envelope, Interpretation(pulqva_intent_json::InterpretationDiagnostic), Deadline,
}
impl fmt::Display for IntentHttpError {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for IntentHttpError {}
impl From<std::io::Error> for IntentHttpError { fn from(e:std::io::Error)->Self{Self::Io(e)} }

#[derive(Serialize)]
struct ChatRequest<'a> {
    messages: [ChatMessage<'a>; 2],
    temperature: u8,
    max_tokens: u16,
    response_format: ResponseFormat<'a>,
    chat_template_kwargs: ChatTemplateKwargs,
}
#[derive(Serialize)]
struct ChatMessage<'a> { role: &'static str, content: &'a str }
#[derive(Serialize)]
struct ResponseFormat<'a> { r#type: &'static str, json_schema: JsonSchemaFormat<'a> }
#[derive(Serialize)]
struct JsonSchemaFormat<'a> { name: &'static str, strict: bool, schema: &'a Value }
#[derive(Serialize)]
struct ChatTemplateKwargs { enable_thinking: bool }

pub fn build_interpretation_request(user_request: &str, schema_json: &str)
    -> Result<String, IntentHttpError>
{
    if user_request.is_empty() || user_request.len()>4096 { return Err(IntentHttpError::InvalidPlan); }
    let schema:Value=serde_json::from_str(schema_json).map_err(|_|IntentHttpError::InvalidPlan)?;
    let req=ChatRequest {
        messages:[
            ChatMessage{role:"system",content:INTENT_SYSTEM_POLICY},
            ChatMessage{role:"user",content:user_request},
        ],
        temperature:0, max_tokens:96,
        // Pinned llama-server reads response_format.json_schema.schema.
        // A top-level response_format.schema silently becomes a generic object.
        response_format:ResponseFormat{r#type:"json_schema",json_schema:JsonSchemaFormat{
            name:"pulqva_intent",strict:true,schema:&schema,
        }},
        chat_template_kwargs:ChatTemplateKwargs{enable_thinking:false},
    };
    serde_json::to_string(&req).map_err(|_|IntentHttpError::InvalidPlan)
}

#[derive(Deserialize)]
struct ResponseEnvelope { choices: Vec<Choice> }
#[derive(Deserialize)]
struct Choice { message: Message }
#[derive(Deserialize)]
struct Message { content: String }

fn remaining(deadline: Instant) -> Result<Duration, IntentHttpError> {
    deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero())
        .ok_or(IntentHttpError::Deadline)
}
fn io_failure(error: io::Error) -> IntentHttpError {
    match error.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => IntentHttpError::Deadline,
        _ => IntentHttpError::Io(error),
    }
}

/// One request; endpoint.timeout is a total budget, never a fresh allowance per read.
pub fn interpret_via_loopback(endpoint: LoopbackIntentEndpoint, request_json: &str)
    -> Result<Interpretation, IntentHttpError>
{
    let deadline=Instant::now().checked_add(endpoint.timeout).ok_or(IntentHttpError::InvalidPlan)?;
    interpret_via_loopback_until(endpoint, request_json, deadline)
}

/// The caller's existing startup deadline can only shorten this request's budget.
/// No retry, redirect, DNS resolution, or external destination is introduced.
pub fn interpret_via_loopback_until(endpoint: LoopbackIntentEndpoint, request_json: &str, outer_deadline: Instant)
    -> Result<Interpretation, IntentHttpError>
{
    let own_deadline=Instant::now().checked_add(endpoint.timeout).ok_or(IntentHttpError::InvalidPlan)?;
    let deadline=outer_deadline.min(own_deadline);
    let addr=SocketAddrV4::new(Ipv4Addr::LOCALHOST,endpoint.port);
    let mut s=TcpStream::connect_timeout(&addr.into(),remaining(deadline)?).map_err(io_failure)?;
    let req=format!(
        "POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        endpoint.port,request_json.len(),request_json);
    let mut pending=req.as_bytes();
    while !pending.is_empty() {
        s.set_write_timeout(Some(remaining(deadline)?))?;
        match s.write(pending) {
            Ok(0) => return Err(IntentHttpError::Io(io::Error::from(io::ErrorKind::WriteZero))),
            Ok(n) => pending=&pending[n..],
            Err(e) if e.kind()==io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(io_failure(e)),
        }
    }
    let mut bytes=Vec::new();
    let mut chunk=[0u8;4096];
    loop {
        s.set_read_timeout(Some(remaining(deadline)?))?;
        let capacity=(MAX_HTTP_BYTES+1-bytes.len()).min(chunk.len());
        match s.read(&mut chunk[..capacity]) {
            Ok(0) => break,
            Ok(n) => {
                bytes.extend_from_slice(&chunk[..n]);
                if bytes.len()>MAX_HTTP_BYTES { return Err(IntentHttpError::TooLarge); }
            }
            Err(e) if e.kind()==io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(io_failure(e)),
        }
    }
    remaining(deadline)?;
    let text=std::str::from_utf8(&bytes).map_err(|_|IntentHttpError::Envelope)?;
    let (head,body)=text.split_once("\r\n\r\n").ok_or(IntentHttpError::Envelope)?;
    let status=head.lines().next().ok_or(IntentHttpError::Envelope)?;
    if !status.starts_with("HTTP/1.1 200 ") { return Err(IntentHttpError::HttpStatus); }
    let env:ResponseEnvelope=serde_json::from_str(body).map_err(|_|IntentHttpError::Envelope)?;
    let content=env.choices.first().ok_or(IntentHttpError::Envelope)?.message.content.as_str();
    let intent=pulqva_intent_json::parse_interpretation_json(content).map_err(|_|IntentHttpError::Interpretation(pulqva_intent_json::diagnose_interpretation_json(content)))?;
    remaining(deadline)?;
    Ok(intent)
}
