use core::fmt;
use std::path::{Path, PathBuf};

use crate::{SearchCandidate, SearchIntent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceSet {
    intent: SearchIntent,
    candidates: Vec<SearchCandidate>,
}
impl ChoiceSet {
    pub fn new(intent: SearchIntent, candidates: Vec<SearchCandidate>) -> Result<Self, JourneyError> {
        if candidates.len() < 2 { return Err(JourneyError::TooFewChoices); }
        Ok(Self { intent, candidates })
    }
    pub fn intent(&self) -> &SearchIntent { &self.intent }
    pub fn candidates(&self) -> &[SearchCandidate] { &self.candidates }
    pub fn select(&self, index: usize) -> Result<SelectedCandidate, JourneyError> {
        let candidate = self.candidates.get(index).ok_or(JourneyError::InvalidSelection)?.clone();
        Ok(SelectedCandidate { index, candidate })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedCandidate { index: usize, candidate: SearchCandidate }
impl SelectedCandidate {
    pub fn index(&self) -> usize { self.index }
    pub fn candidate(&self) -> &SearchCandidate { &self.candidate }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReceipt {
    selected_index: usize,
    title: String,
    path: PathBuf,
    byte_size: u64,
}
impl FileReceipt {
    pub fn new(selection: &SelectedCandidate, path: impl Into<PathBuf>, byte_size: u64)
        -> Result<Self, JourneyError>
    {
        let path = path.into();
        if path.as_os_str().is_empty() { return Err(JourneyError::MissingFilePath); }
        Ok(Self { selected_index: selection.index, title: selection.candidate.title().to_owned(), path, byte_size })
    }
    pub fn selected_index(&self) -> usize { self.selected_index }
    pub fn title(&self) -> &str { &self.title }
    pub fn path(&self) -> &Path { &self.path }
    pub fn byte_size(&self) -> u64 { self.byte_size }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyError { TooFewChoices, InvalidSelection, MissingFilePath, RetrievalFailed }
impl fmt::Display for JourneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooFewChoices => "journey requires at least two choices",
            Self::InvalidSelection => "selection is not in the presented choice set",
            Self::MissingFilePath => "retrieval receipt requires a file path",
            Self::RetrievalFailed => "retrieval failed",
        })
    }
}
impl std::error::Error for JourneyError {}

pub trait CandidateSearch {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError>;
}
pub trait CandidateRetrieval {
    fn retrieve(&mut self, selection: &SelectedCandidate, output_root: &Path)
        -> Result<FileReceipt, JourneyError>;
}

pub fn request_choices(search: &mut impl CandidateSearch, request: impl Into<String>)
    -> Result<ChoiceSet, JourneyError>
{
    let intent = SearchIntent::new(request).map_err(|_| JourneyError::TooFewChoices)?;
    ChoiceSet::new(intent.clone(), search.search(&intent)?)
}

pub fn retrieve_choice(retrieval: &mut impl CandidateRetrieval, choices: &ChoiceSet,
    index: usize, output_root: &Path) -> Result<FileReceipt, JourneyError>
{
    let selected = choices.select(index)?;
    retrieval.retrieve(&selected, output_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::atomic::{AtomicU64, Ordering}};
    static N: AtomicU64 = AtomicU64::new(0);

    struct SearchFixture;
    impl CandidateSearch for SearchFixture {
        fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
            assert_eq!(intent.query(), "find the blue fixture");
            Ok(vec![
                SearchCandidate::new("Blue fixture A", "https://example.invalid/a").unwrap(),
                SearchCandidate::new("Blue fixture B", "https://example.invalid/b").unwrap(),
            ])
        }
    }
    struct RetrievalFixture { fail: bool }
    impl CandidateRetrieval for RetrievalFixture {
        fn retrieve(&mut self, selected: &SelectedCandidate, root: &Path) -> Result<FileReceipt, JourneyError> {
            if self.fail { return Err(JourneyError::RetrievalFailed); }
            fs::create_dir_all(root).map_err(|_| JourneyError::RetrievalFailed)?;
            let path = root.join("selected.bin");
            let payload = b"PULQVA deterministic vertical slice\n";
            fs::write(&path, payload).map_err(|_| JourneyError::RetrievalFailed)?;
            FileReceipt::new(selected, path, payload.len() as u64)
        }
    }
    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("pulqva-t067-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)))
    }

    #[test]
    fn request_choice_file_vertical_slice_is_reproducible() {
        let root = root();
        let choices = request_choices(&mut SearchFixture, "find the blue fixture").unwrap();
        assert_eq!(choices.candidates().len(), 2);
        assert_eq!(choices.candidates()[1].title(), "Blue fixture B");
        let receipt = retrieve_choice(&mut RetrievalFixture { fail: false }, &choices, 1, &root).unwrap();
        assert_eq!(receipt.selected_index(), 1);
        assert_eq!(receipt.title(), "Blue fixture B");
        assert_eq!(receipt.byte_size(), 36);
        assert_eq!(fs::read(receipt.path()).unwrap(), b"PULQVA deterministic vertical slice\n");
        fs::remove_file(receipt.path()).unwrap(); fs::remove_dir(root).unwrap();
    }

    #[test]
    fn invalid_selection_and_retrieval_failure_do_not_create_success_receipt() {
        let root = root();
        let choices = request_choices(&mut SearchFixture, "find the blue fixture").unwrap();
        assert_eq!(retrieve_choice(&mut RetrievalFixture { fail: false }, &choices, 2, &root), Err(JourneyError::InvalidSelection));
        assert!(!root.exists());
        assert_eq!(retrieve_choice(&mut RetrievalFixture { fail: true }, &choices, 0, &root), Err(JourneyError::RetrievalFailed));
        assert!(!root.exists());
    }
}
