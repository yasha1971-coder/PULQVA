//! Real loopback interpretation joined to the existing explicit-choice journey.
use pulqva_core::{CandidateRetrieval, CandidateSearch, Interpretation, JourneyError,
    PendingInterpretedJourney};
use crate::{build_interpretation_request, interpret_via_loopback, IntentHttpError,
    LoopbackIntentEndpoint};

const SCHEMA: &str = include_str!("../../../sidecars/llama.cpp/intent.schema.json");

/// Fixed categories only: no user/model text, paths, URLs or raw I/O diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopbackJourneyError {
    InvalidRequest,
    InterpreterUnavailable,
    InterpretationFailed,
    InterpretationTimedOut,
    Rejected,
    Search(JourneyError),
}
impl core::fmt::Display for LoopbackJourneyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidRequest => f.write_str("invalid interpretation request"),
            Self::InterpreterUnavailable => f.write_str("owned interpreter is unavailable"),
            Self::InterpretationFailed => f.write_str("interpretation failed validation or transport"),
            Self::InterpretationTimedOut => f.write_str("interpretation timed out"),
            Self::Rejected => f.write_str("request was rejected by the interpreter"),
            Self::Search(error) => write!(f, "interpreted search: {error}"),
        }
    }
}
impl std::error::Error for LoopbackJourneyError {}

/// Send human text through the existing local HTTP client and strict parser, then
/// search on the original backend. No raw-input fallback or implicit selection.
///
/// The request always uses the compiled production schema/policy. The caller owns
/// the local server and the privacy-gated search backend; a loopback address alone
/// is not authentication. A future GUI must separately authorize the selected
/// index/output directory. Model-proposed Autopilot is policy data, not consent.
pub fn request_loopback_choices<'a, B: CandidateSearch + CandidateRetrieval>(
    endpoint: LoopbackIntentEndpoint,
    user_request: &str,
    backend: &'a mut B,
) -> Result<PendingInterpretedJourney<'a, B>, LoopbackJourneyError> {
    if user_request.trim().is_empty() || user_request.len() > 4096 || user_request.contains('\0') {
        return Err(LoopbackJourneyError::InvalidRequest);
    }
    let request = build_interpretation_request(user_request, SCHEMA)
        .map_err(|_| LoopbackJourneyError::InvalidRequest)?;
    let interpretation = interpret_via_loopback(endpoint, &request).map_err(|error| match error {
        IntentHttpError::Deadline => LoopbackJourneyError::InterpretationTimedOut,
        _ => LoopbackJourneyError::InterpretationFailed,
    })?;
    if matches!(interpretation, Interpretation::Reject(_)) {
        return Err(LoopbackJourneyError::Rejected);
    }
    PendingInterpretedJourney::start(backend, interpretation).map_err(LoopbackJourneyError::Search)
}
