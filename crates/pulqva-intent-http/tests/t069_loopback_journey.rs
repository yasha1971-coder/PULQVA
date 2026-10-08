//! Actual loopback HTTP and production parser/journey; synthetic model/backend.
//! Every case owns its listener and output. No external traffic or shared env.
use pulqva_core::{CandidateRetrieval, CandidateSearch, ChoiceMode, FileReceipt,
    JourneyError, SearchCandidate, SearchIntent, SelectedCandidate};
use pulqva_intent_http::{request_loopback_choices, LoopbackIntentEndpoint, LoopbackJourneyError};
use serde_json::{json, Value};
use std::{cell::RefCell, fs, io::{Read, Write}, net::{TcpListener, TcpStream},
    path::{Path, PathBuf}, rc::Rc, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU64, Ordering}},
    thread::{self, JoinHandle}, time::{Duration, Instant}};

struct Server {
    port: u16,
    stop: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<Vec<u8>>>>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    fn raw(status: &str, body: String, delay: Duration) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let (flag, captured) = (stop.clone(), requests.clone());
        let reply = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        let worker = thread::spawn(move || {
            let end = Instant::now() + Duration::from_secs(5);
            while !flag.load(Ordering::Acquire) && Instant::now() < end {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                        stream.set_write_timeout(Some(Duration::from_secs(2))).unwrap();
                        let request = read_request(&mut stream);
                        captured.lock().unwrap().push(request);
                        thread::sleep(delay);
                        let _ = stream.write_all(reply.as_bytes());
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(1)),
                    Err(e) => panic!("fixture accept: {e}"),
                }
            }
        });
        Self { port, stop, requests, worker: Some(worker) }
    }
    fn content(content: Value) -> Self {
        Self::raw("200 OK", json!({"choices":[{"message":{"content":content.to_string()}}]}).to_string(), Duration::ZERO)
    }
    fn intent(mode: &str) -> Self {
        Self::content(json!({"kind":"intent","query":"обратный отсчёт","choice_mode":mode}))
    }
    fn endpoint(&self) -> LoopbackIntentEndpoint {
        LoopbackIntentEndpoint::new(self.port, Duration::from_secs(2)).unwrap()
    }
    fn count(&self) -> usize { self.requests.lock().unwrap().len() }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let outcome = self.worker.take().unwrap().join();
        if !thread::panicking() { assert!(outcome.is_ok(), "fixture thread failed"); }
    }
}
fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 1024];
    loop {
        let n = stream.read(&mut chunk).unwrap();
        assert!(n > 0, "incomplete HTTP request");
        bytes.extend_from_slice(&chunk[..n]);
        assert!(bytes.len() <= 16384);
        if let Some(p) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = std::str::from_utf8(&bytes[..p]).unwrap();
            let len: usize = head.lines().find_map(|line| line.strip_prefix("Content-Length: ")).unwrap().parse().unwrap();
            if bytes.len() == p + 4 + len { return bytes; }
            assert!(bytes.len() < p + 4 + len);
        }
    }
}
#[derive(Default)]
struct Calls { queries: Vec<String>, selections: Vec<usize> }
struct Backend { calls: Rc<RefCell<Calls>>, fail_search: bool, fail_retrieval: bool }
impl Backend {
    fn new() -> (Self, Rc<RefCell<Calls>>) {
        let calls = Rc::new(RefCell::new(Calls::default()));
        (Self { calls: calls.clone(), fail_search: false, fail_retrieval: false }, calls)
    }
}
impl CandidateSearch for Backend {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
        self.calls.borrow_mut().queries.push(intent.query().to_owned());
        if self.fail_search { return Err(JourneyError::SearchFailed); }
        Ok(vec![SearchCandidate::new("First", "fixture:a").unwrap(), SearchCandidate::new("Second", "fixture:b").unwrap()])
    }
}
impl CandidateRetrieval for Backend {
    fn retrieve(&mut self, selected: &SelectedCandidate, output: &Path) -> Result<FileReceipt, JourneyError> {
        self.calls.borrow_mut().selections.push(selected.index());
        if self.fail_retrieval { return Err(JourneyError::RetrievalFailed); }
        let path = output.join("selected.bin");
        let bytes = selected.candidate().locator().as_bytes();
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        file.write_all(bytes).unwrap(); file.sync_all().unwrap();
        FileReceipt::new(selected, path, bytes.len() as u64)
    }
}
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("pulqva-c2-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); Self(path)
    }
}
impl Drop for Output { fn drop(&mut self) { let _ = fs::remove_file(self.0.join("selected.bin")); let _ = fs::remove_dir(&self.0); } }
fn assert_no_backend(calls: &Rc<RefCell<Calls>>) {
    assert!(calls.borrow().queries.is_empty()); assert!(calls.borrow().selections.is_empty());
}

#[test]
fn c2_wire_request_uses_compiled_policy_and_saves_only_explicit_selected_bytes() {
    let server = Server::intent("ask"); let (mut backend, calls) = Backend::new(); let output = Output::new();
    let user = "  найди видео обратного отсчёта\nбез музыки  ";
    let pending = request_loopback_choices(server.endpoint(), user, &mut backend).unwrap();
    assert_eq!(pending.choice_mode(), ChoiceMode::Ask);
    assert_eq!(calls.borrow().queries, ["обратный отсчёт"]);
    assert!(calls.borrow().selections.is_empty());
    assert!(!output.0.join("selected.bin").exists());
    let requests = server.requests.lock().unwrap(); assert_eq!(requests.len(), 1);
    let wire = std::str::from_utf8(&requests[0]).unwrap();
    assert!(wire.starts_with("POST /v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1:"));
    let value: Value = serde_json::from_str(wire.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(value["messages"], json!([
        {"role":"system","content":include_str!("../../../sidecars/llama.cpp/intent.system.txt")},
        {"role":"user","content":user}]));
    assert_eq!(value["response_format"]["json_schema"]["schema"], serde_json::from_str::<Value>(include_str!("../../../sidecars/llama.cpp/intent.schema.json")).unwrap());
    assert_eq!(value["temperature"], 0); assert_eq!(value["max_tokens"], 96);
    assert_eq!(value["response_format"]["json_schema"]["strict"], true);
    assert_eq!(value["chat_template_kwargs"]["enable_thinking"], false);
    drop(requests);
    let receipt = pending.retrieve_selected(1, &output.0).unwrap();
    assert_eq!(receipt.selected_index(), 1); assert_eq!(receipt.title(), "Second");
    assert_eq!(receipt.byte_size(), 9); assert_eq!(fs::read(receipt.path()).unwrap(), b"fixture:b");
    assert_eq!(calls.borrow().selections, [1]); assert_eq!(server.count(), 1);
}
#[test]
fn c2_model_autopilot_still_waits_and_cancel_does_not_download() {
    let server = Server::intent("autopilot"); let (mut backend, calls) = Backend::new();
    let pending = request_loopback_choices(server.endpoint(), "choose for me", &mut backend).unwrap();
    assert_eq!(pending.choice_mode(), ChoiceMode::Autopilot);
    assert!(calls.borrow().selections.is_empty()); pending.cancel();
    assert!(calls.borrow().selections.is_empty()); assert_eq!(server.count(), 1);
}
#[test]
fn c2_semantic_reject_never_reaches_backend() {
    let server = Server::content(json!({"kind":"reject","reason":"semantic_authority"}));
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "a request", &mut backend).unwrap_err(), LoopbackJourneyError::Rejected);
    assert_no_backend(&calls); assert_eq!(server.count(), 1);
}
#[test]
fn c2_malformed_model_text_cannot_fall_back_to_raw_request() {
    let server = Server::raw("200 OK", json!({"choices":[{"message":{"content":"not json"}}]}).to_string(), Duration::ZERO);
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "find something", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationFailed);
    assert_no_backend(&calls); assert_eq!(server.count(), 1);
}
#[test]
fn c2_model_authority_payload_is_rejected_before_search() {
    let server = Server::content(json!({"kind":"intent","query":"birds","choice_mode":"ask","shell":"do not execute"}));
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationFailed);
    assert_no_backend(&calls);
}
#[test]
fn c2_http_failure_is_terminal_without_retry_or_search() {
    let server = Server::raw("503 Service Unavailable", "{}".into(), Duration::ZERO);
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationFailed);
    assert_eq!(server.count(), 1); assert_no_backend(&calls);
}
#[test]
fn c2_empty_response_choices_are_not_search_choices() {
    let server = Server::raw("200 OK", "{\"choices\":[]}".into(), Duration::ZERO);
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationFailed);
    assert_no_backend(&calls);
}
#[test]
fn c2_interpretation_deadline_never_reaches_backend() {
    let server = Server::raw("200 OK", "{}".into(), Duration::from_millis(600));
    let endpoint = LoopbackIntentEndpoint::new(server.port, Duration::from_millis(150)).unwrap();
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(endpoint, "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationTimedOut);
    assert_no_backend(&calls); assert!(server.count() <= 1);
}
#[test]
fn c2_invalid_human_input_is_rejected_before_http() {
    let server = Server::intent("ask"); let (mut backend, calls) = Backend::new();
    for user in ["".to_owned(), " \n\t".to_owned(), "a\0b".to_owned(), "x".repeat(4097), "я".repeat(2049)] {
        assert_eq!(request_loopback_choices(server.endpoint(), &user, &mut backend).unwrap_err(), LoopbackJourneyError::InvalidRequest);
    }
    assert_eq!(server.count(), 0); assert_no_backend(&calls);
}
#[test]
fn c2_locator_disguised_as_query_is_rejected_before_search() {
    let server = Server::content(json!({"kind":"intent","query":"https://example.invalid/file","choice_mode":"ask"}));
    let (mut backend, calls) = Backend::new();
    assert_eq!(request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::InterpretationFailed);
    assert_no_backend(&calls);
}
#[test]
fn c2_search_failure_does_not_repeat_interpretation() {
    let server = Server::intent("ask"); let (mut backend, calls) = Backend::new(); backend.fail_search = true;
    assert_eq!(request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap_err(), LoopbackJourneyError::Search(JourneyError::SearchFailed));
    assert_eq!(server.count(), 1); assert_eq!(calls.borrow().queries.len(), 1); assert!(calls.borrow().selections.is_empty());
}
#[test]
fn c2_invalid_selection_cannot_reach_retrieval() {
    let server = Server::intent("ask"); let (mut backend, calls) = Backend::new(); let output = Output::new();
    let pending = request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap();
    assert_eq!(pending.retrieve_selected(2, &output.0).unwrap_err(), JourneyError::InvalidSelection);
    assert!(calls.borrow().selections.is_empty()); assert!(!output.0.join("selected.bin").exists());
}
#[test]
fn c2_retrieval_failure_has_no_second_inference_or_retry() {
    let server = Server::intent("ask"); let (mut backend, calls) = Backend::new(); backend.fail_retrieval = true;
    let output = Output::new();
    let pending = request_loopback_choices(server.endpoint(), "find birds", &mut backend).unwrap();
    assert_eq!(pending.retrieve_selected(1, &output.0).unwrap_err(), JourneyError::RetrievalFailed);
    assert_eq!(server.count(), 1); assert_eq!(calls.borrow().selections, [1]); assert!(!output.0.join("selected.bin").exists());
}
#[test]
fn c2_diagnostics_do_not_echo_untrusted_model_or_user_text() {
    let server = Server::raw("200 OK", json!({"choices":[{"message":{"content":"private-model-secret"}}]}).to_string(), Duration::ZERO);
    let (mut backend, calls) = Backend::new();
    let error = request_loopback_choices(server.endpoint(), "private-user-secret", &mut backend).unwrap_err();
    assert_eq!(format!("{error:?}"), "InterpretationFailed");
    assert_eq!(error.to_string(), "interpretation failed validation or transport"); assert_no_backend(&calls);
}
