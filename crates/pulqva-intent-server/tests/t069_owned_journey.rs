//! The existing supervised server calls the real loopback-to-journey adapter.
//! server_fixture simulates inference; it is not an LLM or a Tor backend.
use pulqva_core::{CandidateSearch, CandidateRetrieval, ChoiceMode, FileReceipt,
    JourneyError, SearchCandidate, SearchIntent, SelectedCandidate};
use pulqva_intent_http::LoopbackJourneyError;
use pulqva_intent_server::{IntentServer, IntentServerPlan};
use std::{fs, io::Write, net::TcpListener, path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering}, time::Duration};

struct Backend { searches: usize, retrievals: usize }
impl CandidateSearch for Backend {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
        self.searches += 1;
        assert_eq!(intent.query(), "countdown"); // fixture response, not raw user text
        Ok(vec![SearchCandidate::new("First", "fixture:first").unwrap(),
                SearchCandidate::new("Second", "fixture:second").unwrap()])
    }
}
impl CandidateRetrieval for Backend {
    fn retrieve(&mut self, selected: &SelectedCandidate, output: &Path) -> Result<FileReceipt, JourneyError> {
        self.retrievals += 1;
        let path = output.join("selected.bin");
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        file.write_all(b"selected-second").unwrap(); file.sync_all().unwrap();
        FileReceipt::new(selected, path, 15)
    }
}
fn server() -> IntentServer {
    // Preserve the existing supervised-child port API. Kernel reserves the port
    // during selection; bind/spawn failure is a test failure, never a retry.
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port(); drop(listener);
    let plan = IntentServerPlan::new(env!("CARGO_BIN_EXE_server_fixture"), "model.gguf",
        include_str!("../../../sidecars/llama.cpp/intent.schema.json"), port, Duration::from_secs(3)).unwrap();
    IntentServer::spawn(&plan).unwrap()
}
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!("pulqva-c2-owner-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); Self(path)
    }
}
impl Drop for Output { fn drop(&mut self) { let _ = fs::remove_file(self.0.join("selected.bin")); let _ = fs::remove_dir(&self.0); } }

#[test]
fn c2_owned_server_connects_human_request_to_explicit_selected_file() {
    let mut server = server(); let mut backend = Backend { searches: 0, retrievals: 0 }; let output = Output::new();
    let pending = server.request_choices("найди видео обратного отсчёта", &mut backend, Duration::from_secs(2)).unwrap();
    assert_eq!(pending.choice_mode(), ChoiceMode::Ask);
    assert_eq!(pending.choices().candidates().len(), 2);
    assert!(!output.0.join("selected.bin").exists());
    let receipt = pending.retrieve_selected(1, &output.0).unwrap();
    assert_eq!(receipt.title(), "Second"); assert_eq!(receipt.selected_index(), 1);
    assert_eq!(receipt.byte_size(), 15); assert_eq!(fs::read(receipt.path()).unwrap(), b"selected-second");
    assert_eq!(backend.searches, 1); assert_eq!(backend.retrievals, 1);
    drop(server); // existing process owner, no second launcher
}
#[test]
fn c2_owned_server_zero_budget_is_not_search_permission() {
    let mut server = server(); let mut backend = Backend { searches: 0, retrievals: 0 };
    let error = server.request_choices("find birds", &mut backend, Duration::ZERO).unwrap_err();
    assert_eq!(error, LoopbackJourneyError::InvalidRequest);
    assert_eq!(backend.searches, 0); assert_eq!(backend.retrievals, 0);
}
