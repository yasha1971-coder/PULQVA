use core::fmt;

/// Maximum UTF-8 byte length accepted from the local intent interpreter.
pub const MAX_INTERPRETED_QUERY_BYTES: usize = 512;

/// Choice policy requested by the local intent interpreter.
///
/// This is product policy data only. It cannot select a network route, command,
/// executable, URL, filesystem path, or provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceMode {
    Ask,
    Autopilot,
}

/// Fixed fail-closed reason emitted when no permissible semantic search intent exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    SemanticAuthority,
}

/// Complete typed result of local intent interpretation.
///
/// Reject carries no query, locator, path, command, provider or transport data
/// and therefore cannot be converted into SearchIntent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Interpretation {
    Intent(InterpretedIntent),
    Reject(RejectReason),
}

impl Interpretation {
    pub fn into_search_intent(self) -> Result<crate::SearchIntent, InterpretationRejected> {
        match self {
            Self::Intent(intent) => intent.into_search_intent().map_err(|_| InterpretationRejected),
            Self::Reject(_) => Err(InterpretationRejected),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterpretationRejected;

impl fmt::Display for InterpretationRejected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("interpretation was rejected")
    }
}
impl std::error::Error for InterpretationRejected {}

/// Validated output of an AI/local intent interpreter.
///
/// Construction is deliberately narrower than deserializing arbitrary model
/// output into application state. Only a search query and a choice policy cross
/// this boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterpretedIntent {
    query: String,
    choice_mode: ChoiceMode,
}

impl InterpretedIntent {
    pub fn new(query: impl Into<String>, choice_mode: ChoiceMode)
        -> Result<Self, InterpretedIntentError>
    {
        let query = query.into();
        validate_query(&query)?;
        Ok(Self { query, choice_mode })
    }

    pub fn query(&self) -> &str { &self.query }
    pub fn choice_mode(&self) -> ChoiceMode { self.choice_mode }

    /// Converts only validated semantic data into the existing core intent.
    pub fn into_search_intent(self) -> Result<crate::SearchIntent, crate::SearchIntentError> {
        crate::SearchIntent::new(self.query)
    }
}

fn validate_query(query: &str) -> Result<(), InterpretedIntentError> {
    if query.trim().is_empty() { return Err(InterpretedIntentError::EmptyQuery); }
    if query.len() > MAX_INTERPRETED_QUERY_BYTES {
        return Err(InterpretedIntentError::QueryTooLong);
    }
    if query.chars().any(char::is_control) {
        return Err(InterpretedIntentError::ControlCharacter);
    }

    // The interpreter is allowed to describe what to search for, not smuggle a
    // locator or local execution primitive across the semantic boundary.
    let lower = query.to_ascii_lowercase();
    if lower.contains("://") || lower.starts_with("www.") {
        return Err(InterpretedIntentError::LocatorLikeQuery);
    }
    let bytes = query.as_bytes();
    let embedded_drive_path = bytes.windows(3).any(|w| {
        w[0].is_ascii_alphabetic() && w[1] == b':' && matches!(w[2], b'\\' | b'/')
    });
    let embedded_unc = query.contains(r"\\");
    if query.starts_with('/') || embedded_unc || embedded_drive_path {
        return Err(InterpretedIntentError::PathLikeQuery);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretedIntentError {
    EmptyQuery,
    QueryTooLong,
    ControlCharacter,
    LocatorLikeQuery,
    PathLikeQuery,
}

impl fmt::Display for InterpretedIntentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyQuery => "interpreted query must not be empty",
            Self::QueryTooLong => "interpreted query exceeded its byte limit",
            Self::ControlCharacter => "interpreted query contained a control character",
            Self::LocatorLikeQuery => "interpreted query must not be a locator",
            Self::PathLikeQuery => "interpreted query must not be a filesystem path",
        })
    }
}

impl std::error::Error for InterpretedIntentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_human_search_language_and_unicode() {
        for query in [
            "countdown video",
            "найди видео с обратным отсчётом",
            "動画のカウントダウン",
        ] {
            let value = InterpretedIntent::new(query, ChoiceMode::Ask).unwrap();
            assert_eq!(value.query(), query);
            assert_eq!(value.choice_mode(), ChoiceMode::Ask);
        }
    }

    #[test]
    fn choice_policy_is_typed_not_free_text() {
        assert_eq!(
            InterpretedIntent::new("countdown", ChoiceMode::Autopilot).unwrap().choice_mode(),
            ChoiceMode::Autopilot
        );
    }

    #[test]
    fn rejects_empty_control_and_oversized_queries() {
        assert_eq!(InterpretedIntent::new("   ", ChoiceMode::Ask).unwrap_err(),
                   InterpretedIntentError::EmptyQuery);
        assert_eq!(InterpretedIntent::new("line\nbreak", ChoiceMode::Ask).unwrap_err(),
                   InterpretedIntentError::ControlCharacter);
        assert_eq!(InterpretedIntent::new("x".repeat(MAX_INTERPRETED_QUERY_BYTES + 1), ChoiceMode::Ask).unwrap_err(),
                   InterpretedIntentError::QueryTooLong);
    }

    #[test]
    fn rejects_urls_and_filesystem_paths() {
        for query in [
            "https://example.com/file.webm",
            "socks5h://127.0.0.1:19050",
            "C:\\Users\\person\\file",
            r"\\server\share\file",
            "/etc/passwd",
        ] {
            assert!(matches!(
                InterpretedIntent::new(query, ChoiceMode::Ask),
                Err(InterpretedIntentError::LocatorLikeQuery | InterpretedIntentError::PathLikeQuery)
            ));
        }
    }

    #[test]
    fn rejects_embedded_path_authority_not_only_prefix_paths() {
        for query in [
            r"save C:\\temp\\x.webm instead",
            "save C:/temp/x.webm instead",
            r"please use \\\\server\\share\\x.webm",
            "search then save D:/Downloads/result.mp4",
        ] {
            assert_eq!(
                InterpretedIntent::new(query, ChoiceMode::Ask).unwrap_err(),
                InterpretedIntentError::PathLikeQuery
            );
        }
    }

    #[test]
    fn reject_has_no_search_capability() {
        let rejected = Interpretation::Reject(RejectReason::SemanticAuthority);
        assert_eq!(rejected.into_search_intent().unwrap_err(), InterpretationRejected);
    }

    #[test]
    fn validated_value_enters_existing_search_intent_without_extra_authority() {
        let interpreted = InterpretedIntent::new("countdown video", ChoiceMode::Ask).unwrap();
        let core = interpreted.into_search_intent().unwrap();
        assert_eq!(core.query(), "countdown video");
    }
}
