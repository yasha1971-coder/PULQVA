//! PULQVA core domain boundaries.
//!
//! The core crate stays independent from UI, networking, Tor, AI providers,
//! media backends, and shell execution.

mod candidate;
mod intent;
mod interpreted_intent;
mod journey;

pub use candidate::{SearchCandidate, SearchCandidateError};
pub use intent::{SearchIntent, SearchIntentError};
pub use interpreted_intent::{ChoiceMode, Interpretation, InterpretationRejected, InterpretedIntent, InterpretedIntentError, RejectReason, MAX_INTERPRETED_QUERY_BYTES};
pub use journey::{
    CandidateRetrieval, CandidateSearch, ChoiceSet, FileReceipt, InterpretedChoices, JourneyError,
    SelectedCandidate, request_choices, request_interpreted_choices, retrieve_choice,
};

pub const CORE_CRATE_READY: bool = true;

#[cfg(test)]
mod tests {
    use super::CORE_CRATE_READY;
    #[test]
    fn core_crate_smoke_test() { assert!(CORE_CRATE_READY); }
}
