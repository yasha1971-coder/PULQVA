use core::fmt;

/// A validated natural-language search request.
///
/// This is deliberately data-only. It does not contain executable commands,
/// provider details, URLs, or transport configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchIntent {
    query: String,
}

impl SearchIntent {
    /// Builds a validated search intent.
    ///
    /// The original non-blank query is preserved. Empty and whitespace-only
    /// input is rejected at the core boundary.
    pub fn new(query: impl Into<String>) -> Result<Self, SearchIntentError> {
        let query = query.into();

        if query.trim().is_empty() {
            return Err(SearchIntentError::EmptyQuery);
        }

        Ok(Self { query })
    }

    /// Returns the user's natural-language query.
    pub fn query(&self) -> &str {
        &self.query
    }
}

/// Validation errors produced while constructing a SearchIntent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchIntentError {
    /// The query contained no non-whitespace characters.
    EmptyQuery,
}

impl fmt::Display for SearchIntentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyQuery => f.write_str("search query must not be empty"),
        }
    }
}

impl std::error::Error for SearchIntentError {}

#[cfg(test)]
mod tests {
    use super::{SearchIntent, SearchIntentError};

    #[test]
    fn accepts_non_empty_query() {
        let intent = SearchIntent::new("find the official live performance")
            .expect("non-empty query should be accepted");

        assert_eq!(intent.query(), "find the official live performance");
    }

    #[test]
    fn rejects_empty_query() {
        assert_eq!(
            SearchIntent::new("").unwrap_err(),
            SearchIntentError::EmptyQuery
        );
    }

    #[test]
    fn rejects_whitespace_only_query() {
        assert_eq!(
            SearchIntent::new(" \t\n ").unwrap_err(),
            SearchIntentError::EmptyQuery
        );
    }
}
