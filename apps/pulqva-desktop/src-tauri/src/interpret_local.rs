//! Development-only, manually started local model. This does not qualify model egress isolation.
use pulqva_core::{ChoiceMode, Interpretation};
use pulqva_intent_http::{
    IntentHttpError, LoopbackIntentEndpoint, build_interpretation_request, interpret_via_loopback,
};
use serde::Serialize;
use std::{sync::atomic::{AtomicBool, Ordering}, time::Duration};

const SCHEMA: &str = include_str!("../../../../sidecars/llama.cpp/intent.schema.json");
static INTERPRETING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalChoiceMode { Ask, Autopilot }

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LocalInterpretation {
    Intent { query: String, choice_mode: LocalChoiceMode },
    Reject { reason: &'static str },
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalIntentErrorCode {
    Busy, InvalidRequest, Unavailable, Timeout, InvalidResponse, WorkerFailed,
}

#[derive(Debug, Serialize)]
pub struct LocalIntentError {
    code: LocalIntentErrorCode,
    message: &'static str,
}
impl LocalIntentError {
    fn new(code: LocalIntentErrorCode, message: &'static str) -> Self { Self { code, message } }
}
impl From<IntentHttpError> for LocalIntentError {
    fn from(error: IntentHttpError) -> Self {
        match error {
            IntentHttpError::InvalidPlan => Self::new(LocalIntentErrorCode::InvalidRequest, "Enter a nonempty request of at most 4096 UTF-8 bytes."),
            IntentHttpError::Io(_) => Self::new(LocalIntentErrorCode::Unavailable, "The local model is unavailable. Start the development model server and try again."),
            IntentHttpError::Deadline => Self::new(LocalIntentErrorCode::Timeout, "The local model timed out. No search or download was started."),
            IntentHttpError::HttpStatus | IntentHttpError::TooLarge | IntentHttpError::Envelope | IntentHttpError::Interpretation(_) =>
                Self::new(LocalIntentErrorCode::InvalidResponse, "The local model response failed strict validation. No search or download was started."),
        }
    }
}

struct InferenceGuard;
impl InferenceGuard {
    fn acquire() -> Result<Self, LocalIntentError> {
        INTERPRETING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| LocalIntentError::new(LocalIntentErrorCode::Busy, "A local interpretation is already running."))
    }
}
impl Drop for InferenceGuard {
    fn drop(&mut self) { INTERPRETING.store(false, Ordering::Release); }
}

fn view(value: Interpretation) -> LocalInterpretation {
    match value {
        Interpretation::Intent(intent) => LocalInterpretation::Intent {
            query: intent.query().to_owned(),
            choice_mode: match intent.choice_mode() {
                ChoiceMode::Ask => LocalChoiceMode::Ask,
                ChoiceMode::Autopilot => LocalChoiceMode::Autopilot,
            },
        },
        Interpretation::Reject(_) => LocalInterpretation::Reject { reason: "semantic_authority" },
    }
}

fn interpret_at(query: &str, endpoint: LoopbackIntentEndpoint) -> Result<LocalInterpretation, LocalIntentError> {
    if query.trim().is_empty() || query.len() > 4096 {
        return Err(LocalIntentError::from(IntentHttpError::InvalidPlan));
    }
    let request = build_interpretation_request(query, SCHEMA)?;
    // This existing adapter invokes parse_interpretation_json before returning core types.
    interpret_via_loopback(endpoint, &request).map(view).map_err(Into::into)
}

/// One fixed IPv4 loopback endpoint; no URL/port, prompt/schema, proxy or route from IPC.
/// Returns typed data only. It never starts Tor, searches, downloads or launches a model.
#[tauri::command]
pub async fn interpret_local_request(query: String) -> Result<LocalInterpretation, LocalIntentError> {
    let guard = InferenceGuard::acquire()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let endpoint = LoopbackIntentEndpoint::new(18081, Duration::from_secs(60))?;
        interpret_at(&query, endpoint)
    }).await.map_err(|_| LocalIntentError::new(LocalIntentErrorCode::WorkerFailed, "The local interpretation worker failed."))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::{Ipv4Addr, TcpListener}, thread, time::Instant};

    const INTENT: &str = r#"{"choices":[{"message":{"content":"{\"kind\":\"intent\",\"query\":\"Big Buck Bunny\",\"choice_mode\":\"ask\"}"}}]}"#;
    const REJECT: &str = r#"{"choices":[{"message":{"content":"{\"kind\":\"reject\",\"reason\":\"semantic_authority\"}"}}]}"#;
    const MALFORMED: &str = r#"{"choices":[{"message":{"content":"not json"}}]}"#;
    const UNKNOWN: &str = r#"{"choices":[{"message":{"content":"{\"kind\":\"intent\",\"query\":\"Big Buck Bunny\",\"choice_mode\":\"ask\",\"command\":\"shell\"}"}}]}"#;

    fn fixture(body: &'static str, delay: Duration) -> (u16, thread::JoinHandle<()>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let worker = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(2);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                        thread::sleep(Duration::from_millis(2)),
                    _ => return,
                }
            };
            stream.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            stream.set_write_timeout(Some(Duration::from_secs(1))).unwrap();
            let mut bytes = Vec::new();
            let mut chunk = [0; 1024];
            while bytes.len() < 16384 {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                }
                if let Some(start) = bytes.windows(4).position(|x| x == b"\r\n\r\n") {
                    let headers = std::str::from_utf8(&bytes[..start]).unwrap();
                    let length: usize = headers.lines().find_map(|line| line.strip_prefix("Content-Length: "))
                        .unwrap().parse().unwrap();
                    if bytes.len() >= start + 4 + length { break; }
                }
            }
            thread::sleep(delay);
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(response.as_bytes());
        });
        (port, worker)
    }
    fn response(body: &'static str, delay: Duration, timeout: Duration) -> Result<LocalInterpretation, LocalIntentError> {
        let (port, worker) = fixture(body, delay);
        let result = interpret_at("Big Buck Bunny", LoopbackIntentEndpoint::new(port, timeout).unwrap());
        worker.join().unwrap();
        result
    }

    #[test]
    fn real_http_fixture_preserves_valid_intent_and_reject() {
        let timeout = Duration::from_secs(1);
        assert_eq!(response(INTENT, Duration::ZERO, timeout).unwrap(), LocalInterpretation::Intent {
            query: "Big Buck Bunny".to_owned(), choice_mode: LocalChoiceMode::Ask,
        });
        assert_eq!(response(REJECT, Duration::ZERO, timeout).unwrap(), LocalInterpretation::Reject { reason: "semantic_authority" });
    }
    #[test]
    fn malformed_and_unknown_field_output_never_become_intent() {
        for body in [MALFORMED, UNKNOWN] {
            assert_eq!(response(body, Duration::ZERO, Duration::from_secs(1)).unwrap_err().code,
                LocalIntentErrorCode::InvalidResponse);
        }
    }
    #[test]
    fn timeout_is_a_structured_failure_and_fixture_is_joined() {
        assert_eq!(response(INTENT, Duration::from_millis(150), Duration::from_millis(40)).unwrap_err().code,
            LocalIntentErrorCode::Timeout);
    }
    #[test]
    fn missing_local_server_has_no_fallback() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        assert_eq!(interpret_at("Big Buck Bunny", LoopbackIntentEndpoint::new(port, Duration::from_millis(200)).unwrap())
            .unwrap_err().code, LocalIntentErrorCode::Unavailable);
    }
    #[test]
    fn invalid_input_fails_before_connection_and_errors_do_not_echo_sensitive_data() {
        let endpoint = LoopbackIntentEndpoint::new(1, Duration::from_millis(50)).unwrap();
        for text in [" \t", &"x".repeat(4097)] {
            assert_eq!(interpret_at(text, endpoint).unwrap_err().code, LocalIntentErrorCode::InvalidRequest);
        }
        let error = LocalIntentError::from(IntentHttpError::Io(std::io::Error::other("private model output")));
        assert!(!error.message.contains("private"));
        assert_eq!(error.code, LocalIntentErrorCode::Unavailable);
    }
    #[test]
    fn singleflight_is_nonblocking_and_released_on_drop() {
        let guard = InferenceGuard::acquire().unwrap();
        assert!(matches!(InferenceGuard::acquire(), Err(LocalIntentError { code: LocalIntentErrorCode::Busy, .. })));
        drop(guard);
        drop(InferenceGuard::acquire().unwrap());
    }
}

