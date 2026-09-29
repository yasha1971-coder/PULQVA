//! Strict JSON decoding for PULQVA intent data.
//!
//! This crate is a narrow adapter. It decodes JSON into validated core types
//! and intentionally contains no provider, network, transport, or execution logic.

use pulqva_core::{ChoiceMode, InterpretedIntent, InterpretedIntentError, SearchIntent, SearchIntentError};
use serde::Deserialize;
use std::{error::Error, fmt};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSearchIntent {
    query: String,
}


#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireInterpretedIntent {
    query: String,
    choice_mode: WireChoiceMode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WireChoiceMode {
    Ask,
    Autopilot,
}

/// Parses the only structured output shape future local inference may emit.
///
/// Accepted shape:
/// {"query":"semantic search terms","choice_mode":"ask|autopilot"}
///
/// Unknown fields are rejected before core validation. The decoder has no
/// network, filesystem, process or provider authority.
pub fn parse_interpreted_intent_json(input: &str)
    -> Result<InterpretedIntent, InterpretedIntentJsonError>
{
    let wire: WireInterpretedIntent =
        serde_json::from_str(input).map_err(InterpretedIntentJsonError::Decode)?;
    let mode = match wire.choice_mode {
        WireChoiceMode::Ask => ChoiceMode::Ask,
        WireChoiceMode::Autopilot => ChoiceMode::Autopilot,
    };
    InterpretedIntent::new(wire.query, mode)
        .map_err(InterpretedIntentJsonError::Validation)
}

#[derive(Debug)]
pub enum InterpretedIntentJsonError {
    Decode(serde_json::Error),
    Validation(InterpretedIntentError),
}

impl fmt::Display for InterpretedIntentJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(_) => f.write_str("invalid interpreted-intent JSON"),
            Self::Validation(error) => write!(f, "invalid interpreted intent: {error}"),
        }
    }
}

impl Error for InterpretedIntentJsonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode(error) => Some(error),
            Self::Validation(error) => Some(error),
        }
    }
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

    #[test]
    fn parses_strict_interpreted_intent() {
        let intent = super::parse_interpreted_intent_json(
            r#"{"query":"countdown video","choice_mode":"ask"}"#
        ).unwrap();
        assert_eq!(intent.query(), "countdown video");
        assert_eq!(intent.choice_mode(), pulqva_core::ChoiceMode::Ask);
    }

    #[test]
    fn parses_typed_autopilot_policy() {
        let intent = super::parse_interpreted_intent_json(
            r#"{"query":"rain ambience","choice_mode":"autopilot"}"#
        ).unwrap();
        assert_eq!(intent.choice_mode(), pulqva_core::ChoiceMode::Autopilot);
    }

    #[test]
    fn rejects_unknown_model_authority_fields() {
        for input in [
            r#"{"query":"countdown","choice_mode":"ask","url":"https://example.com"}"#,
            r#"{"query":"countdown","choice_mode":"ask","proxy":"socks5h://127.0.0.1:1"}"#,
            r#"{"query":"countdown","choice_mode":"ask","command":"curl"}"#,
            r#"{"query":"countdown","choice_mode":"ask","path":"C:\\temp"}"#,
        ] {
            assert!(matches!(
                super::parse_interpreted_intent_json(input),
                Err(super::InterpretedIntentJsonError::Decode(_))
            ));
        }
    }

    #[test]
    fn rejects_missing_invalid_or_free_text_choice_policy() {
        for input in [
            r#"{"query":"countdown"}"#,
            r#"{"query":"countdown","choice_mode":"yes"}"#,
            r#"{"query":"countdown","choice_mode":1}"#,
            r#"{"query":"countdown","choice_mode":{"mode":"ask"}}"#,
        ] {
            assert!(matches!(
                super::parse_interpreted_intent_json(input),
                Err(super::InterpretedIntentJsonError::Decode(_))
            ));
        }
    }

    #[test]
    fn core_rejects_locator_path_control_and_oversize_after_decode() {
        let cases = [
            r#"{"query":"https://example.com/a.webm","choice_mode":"ask"}"#.to_owned(),
            r#"{"query":"C:\\Users\\person\\file","choice_mode":"ask"}"#.to_owned(),
            r#"{"query":"line\nbreak","choice_mode":"ask"}"#.to_owned(),
            format!(r#"{{"query":"{}","choice_mode":"ask"}}"#, "x".repeat(pulqva_core::MAX_INTERPRETED_QUERY_BYTES + 1)),
        ];
        for input in cases {
            assert!(matches!(
                super::parse_interpreted_intent_json(&input),
                Err(super::InterpretedIntentJsonError::Validation(_))
            ));
        }
    }

    #[test]
    fn trailing_or_multiple_json_values_are_rejected() {
        for input in [
            r#"{"query":"countdown","choice_mode":"ask"} garbage"#,
            r#"{"query":"countdown","choice_mode":"ask"} {"query":"other","choice_mode":"ask"}"#,
            r#"[{"query":"countdown","choice_mode":"ask"}]"#,
        ] {
            assert!(matches!(
                super::parse_interpreted_intent_json(input),
                Err(super::InterpretedIntentJsonError::Decode(_))
            ));
        }
    }

}
