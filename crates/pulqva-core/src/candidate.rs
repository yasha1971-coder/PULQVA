use core::fmt;

/// A provider-neutral search result presented to the user.
///
/// The locator is intentionally opaque to the core. It is data that a future
/// backend adapter may understand; it is not a URL contract or executable text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCandidate {
    title: String,
    locator: String,
}

impl SearchCandidate {
    /// Builds a validated search candidate.
    pub fn new(
        title: impl Into<String>,
        locator: impl Into<String>,
    ) -> Result<Self, SearchCandidateError> {
        let title = title.into();
        let locator = locator.into();

        if title.trim().is_empty() {
            return Err(SearchCandidateError::EmptyTitle);
        }

        if locator.trim().is_empty() {
            return Err(SearchCandidateError::EmptyLocator);
        }

        Ok(Self { title, locator })
    }

    /// Human-visible title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Opaque backend locator.
    pub fn locator(&self) -> &str {
        &self.locator
    }
}

/// Validation errors produced while constructing a SearchCandidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchCandidateError {
    EmptyTitle,
    EmptyLocator,
}

impl fmt::Display for SearchCandidateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => f.write_str("candidate title must not be empty"),
            Self::EmptyLocator => f.write_str("candidate locator must not be empty"),
        }
    }
}

impl std::error::Error for SearchCandidateError {}

#[cfg(test)]
mod tests {
    use super::{SearchCandidate, SearchCandidateError};

    #[test]
    fn accepts_valid_candidate() {
        let candidate = SearchCandidate::new(
            "Official live performance",
            "backend:opaque:123",
        )
        .expect("valid candidate should be accepted");

        assert_eq!(candidate.title(), "Official live performance");
        assert_eq!(candidate.locator(), "backend:opaque:123");
    }

    #[test]
    fn rejects_blank_title() {
        assert_eq!(
            SearchCandidate::new(" \t ", "backend:opaque:123").unwrap_err(),
            SearchCandidateError::EmptyTitle
        );
    }

    #[test]
    fn rejects_blank_locator() {
        assert_eq!(
            SearchCandidate::new("Official live performance", "\n ").unwrap_err(),
            SearchCandidateError::EmptyLocator
        );
    }
}
