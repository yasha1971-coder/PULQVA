//! C1 deterministic caller-binding tests. No model, Tor or external traffic.
use pulqva_core::{CandidateRetrieval, CandidateSearch, ChoiceMode, FileReceipt,
    Interpretation, InterpretedIntent, JourneyError, PendingInterpretedJourney,
    RejectReason, SearchCandidate, SearchIntent, SelectedCandidate};
use std::{cell::RefCell, fs, io::Write, path::{Path, PathBuf}, rc::Rc,
    sync::atomic::{AtomicU64, Ordering}};

#[derive(Default)]
struct Calls { queries: Vec<String>, selected: Vec<(usize, String)> }
struct Backend {
    calls: Rc<RefCell<Calls>>,
    candidates: Vec<SearchCandidate>,
    search_error: bool,
    retrieval_error: bool,
}
impl Backend {
    fn new() -> (Self, Rc<RefCell<Calls>>) {
        let calls = Rc::new(RefCell::new(Calls::default()));
        (Self { calls: calls.clone(), search_error: false, retrieval_error: false,
            candidates: vec![SearchCandidate::new("First", "fixture:a").unwrap(),
                SearchCandidate::new("Second", "fixture:b").unwrap()] }, calls)
    }
}
impl CandidateSearch for Backend {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
        self.calls.borrow_mut().queries.push(intent.query().to_owned());
        if self.search_error { return Err(JourneyError::SearchFailed); }
        Ok(self.candidates.clone())
    }
}
impl CandidateRetrieval for Backend {
    fn retrieve(&mut self, selected: &SelectedCandidate, output: &Path) -> Result<FileReceipt, JourneyError> {
        self.calls.borrow_mut().selected.push((selected.index(), selected.candidate().title().to_owned()));
        if self.retrieval_error { return Err(JourneyError::RetrievalFailed); }
        let payload = selected.candidate().locator().as_bytes();
        let path = output.join("selected.bin");
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path)
            .map_err(|_| JourneyError::RetrievalFailed)?;
        file.write_all(payload).map_err(|_| JourneyError::RetrievalFailed)?;
        file.sync_all().map_err(|_| JourneyError::RetrievalFailed)?;
        FileReceipt::new(selected, path, payload.len() as u64)
    }
}
fn interpreted(mode: ChoiceMode) -> Interpretation {
    Interpretation::Intent(InterpretedIntent::new("найди обратный отсчёт", mode).unwrap())
}
struct OwnedOutput(PathBuf);
impl OwnedOutput {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!("pulqva-c1-{}-{}",
            std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&p).expect("must acquire fresh test output, never reuse it");
        Self(p)
    }
}
impl Drop for OwnedOutput {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("selected.bin"));
        let _ = fs::remove_dir(&self.0);
    }
}

#[test]
fn c1_ask_presents_validated_query_and_waits_without_retrieval() {
    let (mut backend, calls) = Backend::new();
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    assert_eq!(pending.choice_mode(), ChoiceMode::Ask);
    assert_eq!(pending.choices().candidates().len(), 2);
    assert_eq!(pending.choices().intent().query(), "найди обратный отсчёт");
    assert_eq!(calls.borrow().queries, ["найди обратный отсчёт"]);
    assert!(calls.borrow().selected.is_empty());
    pending.cancel();
}

#[test]
fn c1_model_autopilot_does_not_authorize_an_automatic_download() {
    let (mut backend, calls) = Backend::new();
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Autopilot)).unwrap();
    assert_eq!(pending.choice_mode(), ChoiceMode::Autopilot);
    assert_eq!(calls.borrow().queries.len(), 1);
    assert!(calls.borrow().selected.is_empty());
    pending.cancel();
    assert!(calls.borrow().selected.is_empty());
}

#[test]
fn c1_reject_calls_neither_search_nor_retrieval() {
    let (mut backend, calls) = Backend::new();
    let result = PendingInterpretedJourney::start(&mut backend, Interpretation::Reject(RejectReason::SemanticAuthority));
    assert_eq!(result.unwrap_err(), JourneyError::InvalidRequest);
    assert!(calls.borrow().queries.is_empty());
    assert!(calls.borrow().selected.is_empty());
}

#[test]
fn c1_selected_file_uses_exact_displayed_candidate_and_original_backend() {
    let output = OwnedOutput::new();
    let (mut backend, calls) = Backend::new();
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    let title = pending.choices().candidates()[1].title().to_owned();
    let receipt = pending.retrieve_selected(1, &output.0).unwrap();
    assert_eq!(receipt.selected_index(), 1);
    assert_eq!(receipt.title(), title);
    assert_eq!(receipt.path(), output.0.join("selected.bin"));
    assert_eq!(receipt.byte_size(), 9);
    assert_eq!(fs::read(receipt.path()).unwrap(), b"fixture:b");
    assert_eq!(calls.borrow().queries.len(), 1);
    assert_eq!(calls.borrow().selected, [(1, "Second".to_owned())]);
}

#[test]
fn c1_invalid_selection_never_enters_retrieval() {
    let output = OwnedOutput::new();
    let (mut backend, calls) = Backend::new();
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    assert_eq!(pending.retrieve_selected(2, &output.0), Err(JourneyError::InvalidSelection));
    assert!(calls.borrow().selected.is_empty());
    assert!(!output.0.join("selected.bin").exists());
}

#[test]
fn c1_search_failure_has_no_raw_request_fallback_or_retrieval() {
    let (mut backend, calls) = Backend::new(); backend.search_error = true;
    let result = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask));
    assert_eq!(result.unwrap_err(), JourneyError::SearchFailed);
    assert_eq!(calls.borrow().queries.len(), 1);
    assert!(calls.borrow().selected.is_empty());
}

#[test]
fn c1_too_few_choices_never_becomes_a_pending_download() {
    let (mut backend, calls) = Backend::new(); backend.candidates.truncate(1);
    let result = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask));
    assert_eq!(result.unwrap_err(), JourneyError::TooFewChoices);
    assert_eq!(calls.borrow().queries.len(), 1);
    assert!(calls.borrow().selected.is_empty());
}

#[test]
fn c1_retrieval_failure_is_returned_once_without_hidden_retry() {
    let output = OwnedOutput::new();
    let (mut backend, calls) = Backend::new(); backend.retrieval_error = true;
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    assert_eq!(pending.retrieve_selected(1, &output.0), Err(JourneyError::RetrievalFailed));
    assert_eq!(calls.borrow().selected.len(), 1);
    assert!(!output.0.join("selected.bin").exists());
}

#[test]
fn c1_cancel_releases_backend_for_a_new_explicit_request() {
    let (mut backend, calls) = Backend::new();
    PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap().cancel();
    backend.candidates[1] = SearchCandidate::new("New second", "fixture:new").unwrap();
    let next = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    assert_eq!(next.choices().candidates()[1].title(), "New second");
    next.cancel();
    assert_eq!(calls.borrow().queries.len(), 2);
    assert!(calls.borrow().selected.is_empty());
}

#[test]
fn c1_dropping_pending_choices_does_not_download() {
    let (mut backend, calls) = Backend::new();
    {
        let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
        assert_eq!(pending.choices().candidates().len(), 2);
    }
    assert!(calls.borrow().selected.is_empty());
    backend.search_error = true; // the exclusive borrow has ended
}

#[test]
fn c1_default_debug_contains_no_query_titles_locators_or_backend() {
    let (mut backend, _) = Backend::new();
    let pending = PendingInterpretedJourney::start(&mut backend, interpreted(ChoiceMode::Ask)).unwrap();
    assert_eq!(format!("{pending:?}"), "PendingInterpretedJourney { .. }");
    pending.cancel();
}
