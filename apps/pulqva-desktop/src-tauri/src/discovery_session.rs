//! Bounded typed session provenance for Tor-derived candidate lists.
//! A session is created only after the trusted backend receives validated
//! discovery results; this module does not initiate network access.
use pulqva_core::SearchCandidate;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub const SESSION_TTL: Duration = Duration::from_secs(600);
pub const MAX_SESSIONS: usize = 8;
pub const MAX_CHOICES: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionChoice {
    pub title: String,
    pub locator: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError { Empty, TooMany, Duplicate, Expired, Unknown, ForeignChoice, QueryMismatch }

struct Entry {
    query: String,
    choices: Vec<SessionChoice>,
    created: Instant,
}
pub struct DiscoverySessions {
    entries: HashMap<String, Entry>,
    next: u64,
}
impl Default for DiscoverySessions {
    fn default() -> Self { Self { entries: HashMap::new(), next: 0 } }
}
impl DiscoverySessions {
    pub fn insert(&mut self, query: &str, candidates: &[SearchCandidate]) -> Result<String, SessionError> {
        if candidates.is_empty() { return Err(SessionError::Empty); }
        if candidates.len() > MAX_CHOICES { return Err(SessionError::TooMany); }
        let mut choices = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let choice = SessionChoice {
                title: candidate.title().to_owned(),
                locator: candidate.locator().to_owned(),
            };
            if choices.iter().any(|c: &SessionChoice| c.locator == choice.locator) {
                return Err(SessionError::Duplicate);
            }
            choices.push(choice);
        }
        self.entries.retain(|_, entry| entry.created.elapsed() < SESSION_TTL);
        if self.entries.len() >= MAX_SESSIONS {
            if let Some(oldest) = self.entries.iter().min_by_key(|(_, e)| e.created).map(|(k, _)| k.clone()) {
                self.entries.remove(&oldest);
            }
        }
        self.next = self.next.checked_add(1).expect("session counter exhausted");
        let id = format!("session-{:016x}", self.next);
        self.entries.insert(id.clone(), Entry { query: query.to_owned(), choices, created: Instant::now() });
        Ok(id)
    }
    pub fn select(&self, id: &str, query: &str, locator: &str) -> Result<&SessionChoice, SessionError> {
        let entry = self.entries.get(id).ok_or(SessionError::Unknown)?;
        if entry.created.elapsed() >= SESSION_TTL { return Err(SessionError::Expired); }
        if entry.query != query { return Err(SessionError::QueryMismatch); }
        entry.choices.iter().find(|c| c.locator == locator).ok_or(SessionError::ForeignChoice)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_rejected() {
        assert_eq!(DiscoverySessions::default().insert("query", &[]), Err(SessionError::Empty));
    }
    fn candidate(title: &str, locator: &str) -> SearchCandidate {
        SearchCandidate::new(title, locator).expect("valid fixture")
    }
    #[test]
    fn exact_session_selection_and_foreign_choice() {
        let mut sessions = DiscoverySessions::default();
        let id = sessions.insert("bird song", &[candidate("bird", "commons:1")]).unwrap();
        assert_eq!(sessions.select(&id, "bird song", "commons:1").unwrap().title, "bird");
        assert_eq!(sessions.select(&id, "other", "commons:1"), Err(SessionError::QueryMismatch));
        assert_eq!(sessions.select(&id, "bird song", "commons:2"), Err(SessionError::ForeignChoice));
    }
    #[test]
    fn duplicate_and_oversize_rejected() {
        let mut sessions = DiscoverySessions::default();
        assert_eq!(sessions.insert("q", &[candidate("one","a"),candidate("two","a")]), Err(SessionError::Duplicate));
        let many: Vec<_> = (0..=MAX_CHOICES).map(|i| candidate("valid", &format!("loc:{i}"))).collect();
        assert_eq!(sessions.insert("q", &many), Err(SessionError::TooMany));
    }
    #[test]
    fn expired_session_rejected() {
        let mut sessions = DiscoverySessions::default();
        let id = sessions.insert("q", &[candidate("valid", "loc")]).unwrap();
        sessions.entries.get_mut(&id).unwrap().created = Instant::now().checked_sub(SESSION_TTL).expect("test clock supports TTL");
        assert_eq!(sessions.select(&id, "q", "loc"), Err(SessionError::Expired));
    }
    #[test]
    fn oldest_session_evicted_at_capacity() {
        let mut sessions = DiscoverySessions::default();
        let first = sessions.insert("q", &[candidate("valid","loc")]).unwrap();
        for i in 1..=MAX_SESSIONS {
            sessions.insert("q", &[candidate("valid", &format!("loc:{i}"))]).unwrap();
        }
        assert_eq!(sessions.entries.len(), MAX_SESSIONS);
        assert_eq!(sessions.select(&first, "q", "loc"), Err(SessionError::Unknown));
    }
    #[test]
    fn unknown_session_rejected() {
        assert_eq!(DiscoverySessions::default().select("session-1", "q", "l"), Err(SessionError::Unknown));
    }
}
