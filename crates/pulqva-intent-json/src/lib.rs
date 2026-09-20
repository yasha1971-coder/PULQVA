//! Strict JSON decoding for PULQVA intent data.
//!
//! This crate is a narrow adapter. It decodes JSON into validated core types
//! and intentionally contains no provider, network, transport, or execution logic.

use pulqva_core::{SearchIntent, SearchIntentError};
use serde::Deserialize;
use std::{error::Error, fmt};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSearchIntent {
    query: String,
}

/// Parses a strict JSON object into a validated SearchIntent.
///
/// Accepted shape:
/// {"query":"natural-language request"}
pub fn parse_search_intent_json(input: &str) -> Result<SearchIntent, IntentJsonError> {
    let wire: WireSearchIntent =
        serde_json::from_str(input).map_err(IntentJsonError::Decode)?;

    SearchIntent::new(wire.query).map_err(IntentJsonError::Validation)
}

/// Errors at the JSON -> core-intent boundary.
#[derive(Debug)]
pub enum IntentJsonError {
    /// The input was not valid JSON matching the strict wire contract.
    Decode(serde_json::Error),
    /// JSON decoded correctly, but core intent validation rejected the data.
    Validation(SearchIntentError),
}

impl fmt::Display for IntentJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(f, "invalid intent JSON: {error}"),
            Self::Validation(error) => write!(f, "invalid search intent: {error}"),
        }
    }
}

impl Error for IntentJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IntentJsonError, parse_search_intent_json};

    #[test]
    fn parses_valid_search_intent() {
        let intent = parse_search_intent_json(
            r#"{"query":"find the official live performance"}"#,
        )
        .expect("valid strict intent JSON should parse");

        assert_eq!(intent.query(), "find the official live performance");
    }

    #[test]
    fn rejects_malformed_json() {
        let error = parse_search_intent_json(r#"{"query":"broken""#)
            .expect_err("malformed JSON must be rejected");

        assert!(matches!(error, IntentJsonError::Decode(_)));
    }

    #[test]
    fn rejects_missing_query() {
        let error = parse_search_intent_json("{}")
            .expect_err("missing query must be rejected");

        assert!(matches!(error, IntentJsonError::Decode(_)));
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = parse_search_intent_json(
            r#"{"query":"concert","command":"run this"}"#,
        )
        .expect_err("unknown fields must be rejected");

        assert!(matches!(error, IntentJsonError::Decode(_)));
    }

    #[test]
    fn rejects_blank_query_through_core_validation() {
        let error = parse_search_intent_json(r#"{"query":"   \t  "}"#)
            .expect_err("blank query must be rejected");

        assert!(matches!(error, IntentJsonError::Validation(_)));
    }
}
