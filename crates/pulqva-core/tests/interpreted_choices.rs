//! G1 public adapter contracts. Each case owns a fresh fixture; no model,
//! transport, shared history, filesystem or real file evidence is involved.
use pulqva_core::{
    request_interpreted_choices, CandidateRetrieval, CandidateSearch, ChoiceMode,
    FileReceipt, Interpretation, InterpretedIntent, JourneyError, RejectReason,
    SearchCandidate, SearchIntent, SelectedCandidate,
};
use std::path::Path;

struct RecordingBackend {
    queries: Vec<String>,
    outcome: Result<Vec<SearchCandidate>, JourneyError>,
    retrieval_calls: usize,
}
impl RecordingBackend {
    fn new(outcome: Result<Vec<SearchCandidate>, JourneyError>) -> Self {
        Self { queries: Vec::new(), outcome, retrieval_calls: 0 }
    }
}
impl CandidateSearch for RecordingBackend {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
        self.queries.push(intent.query().to_owned());
        self.outcome.clone()
    }
}
impl CandidateRetrieval for RecordingBackend {
    fn retrieve(&mut self, _: &SelectedCandidate, _: &Path) -> Result<FileReceipt, JourneyError> {
        self.retrieval_calls += 1;
        Err(JourneyError::RetrievalFailed)
    }
}

fn candidates() -> Vec<SearchCandidate> {
    vec![
        SearchCandidate::new("First fixture", "https://example.invalid/first").unwrap(),
        SearchCandidate::new("Second fixture", "https://example.invalid/second").unwrap(),
    ]
}
fn intent(query: &str, mode: ChoiceMode) -> Interpretation {
    Interpretation::Intent(InterpretedIntent::new(query, mode).unwrap())
}

// G1-REJECT: even a backend capable of returning choices must not be called.
#[test]
fn reject_never_calls_search_or_retrieval() {
    let mut backend = RecordingBackend::new(Ok(candidates()));
    let result = request_interpreted_choices(
        &mut backend, Interpretation::Reject(RejectReason::SemanticAuthority),
    );
    assert_eq!(result.unwrap_err(), JourneyError::InvalidRequest);
    assert!(backend.queries.is_empty());
    assert_eq!(backend.retrieval_calls, 0);
}

// G1-ASK: exact query and candidate order survive; discovery is not retrieval.
#[test]
fn ask_preserves_unicode_query_and_choices_without_retrieval() {
    let query = "  найди відео зворотного відліку 動画  ";
    let expected = candidates();
    let mut backend = RecordingBackend::new(Ok(expected.clone()));
    let result = request_interpreted_choices(&mut backend, intent(query, ChoiceMode::Ask)).unwrap();
    assert_eq!(backend.queries, vec![query.to_owned()]);
    assert_eq!(result.choice_mode(), ChoiceMode::Ask);
    assert_eq!(result.choices().intent().query(), query);
    assert_eq!(result.choices().candidates(), expected.as_slice());
    assert_eq!(backend.retrieval_calls, 0);
    assert_eq!(format!("{result:?}"), "InterpretedChoices { .. }");
}

// G1-AUTO: preserve explicit policy, but do not choose/download in this adapter.
#[test]
fn autopilot_is_retained_as_data_without_retrieval() {
    let query = "countdown video";
    let expected = candidates();
    let mut backend = RecordingBackend::new(Ok(expected.clone()));
    let result = request_interpreted_choices(&mut backend, intent(query, ChoiceMode::Autopilot)).unwrap();
    assert_eq!(backend.queries, vec![query.to_owned()]);
    assert_eq!(result.choice_mode(), ChoiceMode::Autopilot);
    assert_eq!(result.choices().intent().query(), query);
    assert_eq!(result.choices().candidates(), expected.as_slice());
    assert_eq!(backend.retrieval_calls, 0);
}

// G1-ERROR: preserve backend failures, with no retry or manufactured choices.
#[test]
fn backend_errors_are_propagated_without_retry() {
    for error in [JourneyError::InvalidRequest, JourneyError::SearchFailed, JourneyError::TooFewChoices] {
        let mut backend = RecordingBackend::new(Err(error));
        let result = request_interpreted_choices(&mut backend, intent("countdown", ChoiceMode::Ask));
        assert_eq!(result.unwrap_err(), error);
        assert_eq!(backend.queries, vec!["countdown".to_owned()]);
        assert_eq!(backend.retrieval_calls, 0);
    }
}

// G1-EMPTY: the existing minimum of two choices is unchanged for either mode.
#[test]
fn zero_or_one_result_is_not_promoted_to_a_choice_set() {
    for mode in [ChoiceMode::Ask, ChoiceMode::Autopilot] {
        for count in 0..2 {
            let mut rows = candidates();
            rows.truncate(count);
            let mut backend = RecordingBackend::new(Ok(rows));
            let result = request_interpreted_choices(&mut backend, intent("countdown", mode));
            assert_eq!(result.unwrap_err(), JourneyError::TooFewChoices);
            assert_eq!(backend.queries, vec!["countdown".to_owned()]);
            assert_eq!(backend.retrieval_calls, 0);
        }
    }
}
