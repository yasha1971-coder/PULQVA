//! PULQVA core domain boundaries.
//!
//! The core crate stays independent from UI, networking, Tor, AI providers,
//! media backends, and shell execution.

mod candidate;
mod intent;

pub use candidate::{SearchCandidate, SearchCandidateError};
pub use intent::{SearchIntent, SearchIntentError};

/// Marker proving the core crate is linked and available.
pub const CORE_CRATE_READY: bool = true;

#[cfg(test)]
mod tests {
    use super::CORE_CRATE_READY;

    #[test]
    fn core_crate_smoke_test() {
        assert!(CORE_CRATE_READY);
    }
}
