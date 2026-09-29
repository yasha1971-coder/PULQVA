//! Typed Commons discovery and endpoint-private, supervised Tor HTTPS execution.
//! Provider/parser contracts remain separate from transport and the T067 core.
mod artifact;
mod commons;
mod https;

pub use artifact::{ArtifactVerificationError, VerifiedCommonsFile};
pub use commons::{CommonsSearch, CommonsSearchPlan, CommonsTransport, DiscoveredMedia,
                  CandidateFailure, DiscoveryError, NetworkFailure, ReadinessFailure, MAX_MEDIA_BYTES, MAX_RESPONSE_BYTES, MAX_RESULTS, parse_response};
pub use https::{CommonsHttpsTransport, CommonsVerifiedTorTransport, DiscoveryCancellation};
