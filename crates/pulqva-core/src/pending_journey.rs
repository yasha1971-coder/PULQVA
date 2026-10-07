use core::fmt;
use std::path::Path;

use crate::{CandidateRetrieval, CandidateSearch, ChoiceMode, ChoiceSet, FileReceipt,
    Interpretation, InterpretedChoices, JourneyError, request_interpreted_choices, retrieve_choice};

/// A waiting interpreted request bound to the backend that produced its choices.
///
/// This adapter composes the existing search and retrieval functions. It neither
/// interprets raw model text nor implements a second search/download path. Starting
/// either Ask or Autopilot only searches: the caller must separately authorize and
/// supply a selected index. Policy data from a model is never that authorization.
///
/// The exclusive borrow prevents ordinary callers from replacing this backend's
/// search results while these choices are pending. Retrieval consumes the pending
/// request, so a second click cannot reuse it. This is not protection against a
/// malicious backend or shared interior state, and does not add publisher/file
/// verification beyond that provided by the selected backend.
///
/// A caller cannot search again on this backend and then reuse the old choices:
/// ```compile_fail,E0499
/// use pulqva_core::{CandidateSearch, CandidateRetrieval, Interpretation,
///     PendingInterpretedJourney, request_choices};
/// fn stale<B: CandidateSearch + CandidateRetrieval>(backend: &mut B, value: Interpretation) {
///     let pending = PendingInterpretedJourney::start(backend, value).unwrap();
///     let _ = request_choices(backend, "a different request");
///     let _ = pending.choices();
/// }
/// ```
///
/// A completed or failed retrieval attempt cannot be replayed:
/// ```compile_fail,E0382
/// use pulqva_core::{CandidateSearch, CandidateRetrieval, Interpretation,
///     PendingInterpretedJourney};
/// use std::path::Path;
/// fn twice<B: CandidateSearch + CandidateRetrieval>(backend: &mut B, value: Interpretation) {
///     let pending = PendingInterpretedJourney::start(backend, value).unwrap();
///     let _ = pending.retrieve_selected(0, Path::new("downloads"));
///     let _ = pending.retrieve_selected(1, Path::new("downloads"));
/// }
/// ```
#[must_use = "present the choices, then explicitly select or cancel the pending request"]
pub struct PendingInterpretedJourney<'a, B: CandidateSearch + CandidateRetrieval> {
    backend: &'a mut B,
    presented: InterpretedChoices,
}

impl<'a, B: CandidateSearch + CandidateRetrieval> PendingInterpretedJourney<'a, B> {
    pub fn start(backend: &'a mut B, interpretation: Interpretation) -> Result<Self, JourneyError> {
        let presented = request_interpreted_choices(backend, interpretation)?;
        Ok(Self { backend, presented })
    }

    /// Read-only presentation data; no mutable backend or foreign selection input.
    pub fn choices(&self) -> &ChoiceSet { self.presented.choices() }
    pub fn choice_mode(&self) -> ChoiceMode { self.presented.choice_mode() }

    /// One explicit caller-selected retrieval using the original backend/choices.
    ///
    /// Even an invalid index or a backend failure consumes this request. A retry
    /// needs a new explicitly started request, not a hidden loop. UI consent,
    /// authorization and trusted output-directory ownership remain caller duties.
    pub fn retrieve_selected(self, index: usize, output_root: &Path) -> Result<FileReceipt, JourneyError> {
        retrieve_choice(self.backend, self.presented.choices(), index, output_root)
    }

    /// Cancel while waiting for a choice. Does not start retrieval or stop a Tor
    /// process owned elsewhere; this is not in-flight download cancellation.
    pub fn cancel(self) {}
}

impl<B: CandidateSearch + CandidateRetrieval> fmt::Debug for PendingInterpretedJourney<'_, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PendingInterpretedJourney { .. }")
    }
}
