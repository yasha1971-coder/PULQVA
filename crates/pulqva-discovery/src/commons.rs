//! Commons discovery boundary for the existing T067 coordinator.
//!
//! T068-A: typed request + bounded untrusted-response adapter only. There is no
//! HTTP implementation here yet. Injected fixture bytes are NOT live discovery.
//! The future Tor-only executor must enforce the plan's limits while streaming,
//! verify HTTPS certificates, reject redirects, and never use ambient proxies.

use pulqva_core::{CandidateRetrieval, CandidateSearch, FileReceipt, JourneyError, SearchCandidate, SearchIntent, SelectedCandidate};
use pulqva_privacy::{ReadyTorTransport, YtDlpMediaRequestPlan, YtDlpMediaSourceUrl, launch_ytdlp_request};
use serde::Deserialize;
use std::{collections::HashSet, fmt, path::Path, time::Duration};

pub const MAX_RESPONSE_BYTES: usize = 256 * 1024;
pub const MAX_RESULTS: usize = 10;
pub const MAX_MEDIA_BYTES: u64 = 8 * 1024 * 1024;
const ENDPOINT: &str = "https://commons.wikimedia.org/w/api.php";
const MEDIA_PREFIX: &str = "https://upload.wikimedia.org/wikipedia/commons/";

/// Data-only policy. The public constructor cannot take an unverified raw port.
/// No URL, proxy or arbitrary-header override is exposed.
///
/// ```compile_fail
/// use pulqva_core::SearchIntent;
/// use pulqva_privacy::TorSocksEndpoint;
/// use pulqva_discovery::CommonsSearchPlan;
/// let raw = TorSocksEndpoint::new(19050).unwrap();
/// CommonsSearchPlan::new(&SearchIntent::new("countdown").unwrap(), raw);
/// ```
#[derive(Clone)]
pub struct CommonsSearchPlan { url: String, proxy: String }
impl CommonsSearchPlan {
    pub fn new(intent: &SearchIntent, ready: ReadyTorTransport) -> Result<Self, DiscoveryError> {
        Self::from_verified_proxy(intent, ready.proxy_url())
    }
    fn from_verified_proxy(intent: &SearchIntent, proxy: String) -> Result<Self, DiscoveryError> {
        let query = intent.query();
        if query.trim().is_empty() || query.len() > 512 || query.chars().any(char::is_control) {
            return Err(DiscoveryError::InvalidRequest);
        }
        // CirrusSearch filesize uses 1024-byte units. Restrict the discovery
        // window before gsrlimit is applied, rather than filling it with files
        // the parser must discard. These are availability hints, not security
        // controls: responses and eventually downloaded bytes still need checks.
        let max_kib = MAX_MEDIA_BYTES / 1024;
        let query = encode_query(&format!(
            "{query} filetype:video filemime:\"video/webm\" filesize:<{max_kib}"
        ));
        let url = format!("{ENDPOINT}?action=query&format=json&formatversion=2&generator=search&gsrnamespace=6&gsrlimit=10&prop=imageinfo&iilimit=1&iiprop=url%7Csize%7Csha1%7Cmime&gsrsearch={query}");
        Ok(Self { url, proxy })
    }
    pub fn url(&self) -> &str { &self.url }
    pub fn proxy_url(&self) -> &str { &self.proxy }
    pub fn timeout(&self) -> Duration { Duration::from_secs(60) }
    pub fn max_response_bytes(&self) -> usize { MAX_RESPONSE_BYTES }
    pub fn max_redirects(&self) -> u8 { 0 }
    pub fn user_agent(&self) -> &'static str {
        "PULQVA/0.0.0 (https://github.com/yasha1971-coder/PULQVA)"
    }
}

fn encode_query(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len() * 3);
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) { encoded.push(b as char); }
        else { encoded.push('%'); encoded.push(HEX[(b >> 4) as usize] as char); encoded.push(HEX[(b & 15) as usize] as char); }
    }
    encoded
}

/// Endpoint-specific transport seam, not a generic HTTP escape hatch.
/// Implementations must return only a successful, TLS-verified response from the
/// plan endpoint through its socks5h route, enforcing byte/time/redirect limits.
/// The trait alone is NOT proof of egress confinement. T068-B must implement and
/// test the real executor; only unit tests provide an implementation in T068-A.
pub trait CommonsTransport {
    fn fetch(&mut self, plan: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError>;
}

/// Metadata from an untrusted API reply, NOT an authenticated download receipt.
/// Verify actual downloaded bytes independently at the retrieval boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredMedia {
    page_id: u64,
    candidate: SearchCandidate,
    declared_size: u64,
    declared_sha1: String,
}
impl DiscoveredMedia {
    pub fn page_id(&self) -> u64 { self.page_id }
    pub fn candidate(&self) -> &SearchCandidate { &self.candidate }
    pub fn declared_size(&self) -> u64 { self.declared_size }
    pub fn declared_sha1(&self) -> &str { &self.declared_sha1 }
    pub(crate) fn matches_selection(&self, selection: &SelectedCandidate) -> bool {
        self.candidate == *selection.candidate()
    }
    pub fn description_url(&self) -> String {
        format!("https://commons.wikimedia.org/?curid={}", self.page_id)
    }
}

pub struct CommonsSearch<T> {
    transport: T,
    proxy: String,
    ready: Option<ReadyTorTransport>,
    ytdlp_executable: Option<std::path::PathBuf>,
    last_results: Vec<DiscoveredMedia>,
    last_error: Option<DiscoveryError>,
}
impl<T: CommonsTransport> CommonsSearch<T> {
    pub fn new(transport: T, ready: ReadyTorTransport) -> Self {
        Self { transport, proxy: ready.proxy_url(), ready: Some(ready), ytdlp_executable: None, last_results: Vec::new(), last_error: None }
    }
    /// Only the latest successful response is retained; errors clear it.
    pub fn last_results(&self) -> &[DiscoveredMedia] { &self.last_results }
    /// Same opaque readiness capability that authorized this production route.
    /// Fixture-only searches have None and therefore cannot authorize retrieval.
    pub fn ready_transport(&self) -> Option<ReadyTorTransport> { self.ready }
    pub fn with_ytdlp_executable(mut self, executable: impl Into<std::path::PathBuf>) -> Self { self.ytdlp_executable = Some(executable.into()); self }
    /// Safe category of the latest invocation of discover/search, not raw network data.
    /// A request rejected by the outer coordinator before search does not update this.
    pub fn last_error(&self) -> Option<DiscoveryError> { self.last_error }
    pub fn discover(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, DiscoveryError> {
        self.last_results.clear();
        self.last_error = None;
        let outcome = (|| {
            let plan = CommonsSearchPlan::from_verified_proxy(intent, self.proxy.clone())?;
            let body = self.transport.fetch(&plan)?;
            let results = parse_response_classified(&body)?;
            let candidates = results.iter().map(|r| r.candidate.clone()).collect();
            self.last_results = results;
            Ok(candidates)
        })();
        self.last_error = outcome.as_ref().err().copied();
        outcome
    }
}

impl<T: CommonsTransport> CandidateRetrieval for CommonsSearch<T> {
    fn retrieve(&mut self, selection: &SelectedCandidate, output_root: &Path)
        -> Result<FileReceipt, JourneyError>
    {
        // C2 fail-closed binding: retrieval is permitted only for the exact
        // candidate/metadata pair retained by the latest successful discovery.
        // Network/file execution is deliberately not introduced by this commit.
        let Some(media) = self.last_results.get(selection.index()) else {
            return Err(JourneyError::RetrievalFailed);
        };
        if !media.matches_selection(selection) { return Err(JourneyError::RetrievalFailed); }
        let ready = self.ready.ok_or(JourneyError::RetrievalFailed)?;
        let executable = self.ytdlp_executable.clone().ok_or(JourneyError::RetrievalFailed)?;
        let source = YtDlpMediaSourceUrl::parse(media.candidate().locator().to_owned()).map_err(|_| JourneyError::RetrievalFailed)?;
        let plan = YtDlpMediaRequestPlan::new_tor_gated(executable, ready, source, output_root).map_err(|_| JourneyError::RetrievalFailed)?;
        let running = launch_ytdlp_request(plan).map_err(|_| JourneyError::RetrievalFailed)?;
        let completed = running.complete_download().map_err(|_| JourneyError::RetrievalFailed)?;
        self.verify_completed_download(selection, &completed).map(|verified| verified.into_receipt()).map_err(|_| JourneyError::RetrievalFailed)
    }
}

impl<T: CommonsTransport> CandidateSearch for CommonsSearch<T> {
    fn search(&mut self, intent: &SearchIntent) -> Result<Vec<SearchCandidate>, JourneyError> {
        self.discover(intent).map_err(|error| match error {
            DiscoveryError::InvalidRequest => JourneyError::InvalidRequest,
            DiscoveryError::TooFewChoices => JourneyError::TooFewChoices,
            _ => JourneyError::SearchFailed,
        })
    }
}

#[derive(Deserialize)]
struct Reply {
    error: Option<serde_json::Value>,
    errors: Option<serde_json::Value>,
    warnings: Option<serde_json::Value>,
    query: Option<Query>,
}
#[derive(Deserialize)]
struct Query { pages: Vec<Page> }
#[derive(Deserialize)]
struct Page {
    pageid: u64,
    ns: u32,
    title: String,
    #[serde(default)]
    imageinfo: Vec<ImageInfo>,
}
#[derive(Deserialize)]
struct ImageInfo { url: String, size: u64, sha1: String, mime: String }

/// No I/O, pagination, canned fallback results, or implicit file fetching.
/// Unknown additional metadata is ignored, but typed fields and limits are strict.
pub fn parse_response(bytes: &[u8]) -> Result<Vec<DiscoveredMedia>, DiscoveryError> {
    // Preserve the original public parser error contract. The coordinator uses
    // the same implementation directly so it retains the safe rejection reason.
    parse_response_classified(bytes).map_err(|error| match error {
        DiscoveryError::CandidateRejected(_) => DiscoveryError::UntrustedCandidate,
        other => other,
    })
}

fn parse_response_classified(bytes: &[u8]) -> Result<Vec<DiscoveredMedia>, DiscoveryError> {
    if bytes.len() > MAX_RESPONSE_BYTES { return Err(DiscoveryError::ResponseTooLarge); }
    let reply: Reply = serde_json::from_slice(bytes).map_err(|_| DiscoveryError::InvalidResponse)?;
    if reply.error.is_some() || reply.errors.is_some() || reply.warnings.is_some() {
        return Err(DiscoveryError::RemoteRejected);
    }
    let pages = reply.query.ok_or(DiscoveryError::TooFewChoices)?.pages;
    if pages.len() > MAX_RESULTS { return Err(DiscoveryError::InvalidResponse); }
    let mut ids = HashSet::new();
    let mut urls = HashSet::new();
    let mut results = Vec::new();
    for page in pages {
        // Same predicates and evaluation order; only the error value changes.
        if page.pageid == 0 { return Err(rejected(CandidateFailure::PageId)); }
        if page.ns != 6 { return Err(rejected(CandidateFailure::Namespace)); }
        if !page.title.starts_with("File:") { return Err(rejected(CandidateFailure::TitlePrefix)); }
        if page.title.len() > 512 { return Err(rejected(CandidateFailure::TitleLength)); }
        if page.title.chars().any(char::is_control) { return Err(rejected(CandidateFailure::TitleControl)); }
        if !ids.insert(page.pageid) { return Err(rejected(CandidateFailure::DuplicatePageId)); }
        // Missing/deleted media contributes no fabricated candidate.
        if page.imageinfo.is_empty() { continue; }
        if page.imageinfo.len() != 1 { return Err(DiscoveryError::InvalidResponse); }
        let info = page.imageinfo.into_iter().next().ok_or(DiscoveryError::InvalidResponse)?;
        if info.mime != "video/webm" || info.size == 0 || info.size > MAX_MEDIA_BYTES { continue; }
        let media_url = normalize_media_url(&info.url).map_err(rejected)?;
        if info.sha1.len() != 40 { return Err(rejected(CandidateFailure::DigestLength)); }
        if !info.sha1.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(rejected(CandidateFailure::DigestEncoding)); }
        if !urls.insert(media_url.to_owned()) { return Err(rejected(CandidateFailure::DuplicateMediaUrl)); }
        let candidate = SearchCandidate::new(page.title, media_url)
            .map_err(|_| rejected(CandidateFailure::CoreContract))?;
        results.push(DiscoveredMedia { page_id: page.pageid, candidate,
            declared_size: info.size, declared_sha1: info.sha1.to_ascii_lowercase() });
    }
    if results.len() < 2 { return Err(DiscoveryError::TooFewChoices); }
    Ok(results)
}

// MediaWiki File::appendRequestProvenance and ApiQueryImageInfo add these
// public provenance fields to original URLs. Strip ONLY this exact vocabulary;
// never reinterpret arbitrary query data or fragments as a canonical file URL.
// See commons/provenance_tests.rs for independent provider-contract fixtures.
fn normalize_media_url(value: &str) -> Result<&str, CandidateFailure> {
    use CandidateFailure::*;
    // Apply the envelope to the original input, not just the stripped prefix.
    if value.len() > 2048 { return Err(UrlLength); }
    if !value.is_ascii() { return Err(UrlNonAscii); }
    if value.bytes().any(|b| b <= 32 || b == 127) { return Err(UrlControlOrSpace); }
    if value.contains('#') { return Err(UrlQueryOrFragment); }
    let canonical = if let Some((base, query)) = value.split_once('?') {
        let mut seen = 0u8;
        for pair in query.split('&') {
            let bit = match pair {
                "utm_source=commons.wikimedia.org" => 1u8,
                "utm_campaign=imageinfo" => 2u8,
                "utm_content=original" => 4u8,
                _ => return Err(UrlQueryOrFragment),
            };
            if seen & bit != 0 { return Err(UrlQueryOrFragment); }
            seen |= bit;
        }
        if seen != 7 { return Err(UrlQueryOrFragment); }
        base
    } else { value };
    // Origin, path, percent escapes, traversal and extension checks are unchanged.
    validate_media_url(canonical)?;
    Ok(canonical)
}

// Deliberately narrower than general URL parsing: one HTTPS authority, a fixed
// media path prefix, no query/fragment/credentials/backslashes or decoded traversal.
fn validate_media_url(value: &str) -> Result<(), CandidateFailure> {
    use CandidateFailure::*;
    if value.len() > 2048 { return Err(UrlLength); }
    if !value.is_ascii() { return Err(UrlNonAscii); }
    if value.bytes().any(|b| b <= 32 || b == 127) { return Err(UrlControlOrSpace); }
    if value.contains(['?', '#']) { return Err(UrlQueryOrFragment); }
    if value.contains('\\') { return Err(UrlBackslash); }
    if !value.ends_with(".webm") { return Err(UrlExtension); }
    // Splitting the existing literal prefix is diagnostic, not normalization.
    // Neither alternate schemes/authorities nor alternate path prefixes pass.
    let Some(path) = value.strip_prefix(MEDIA_PREFIX) else {
        return Err(if value.starts_with("https://upload.wikimedia.org/") {
            UrlPathPrefix
        } else { UrlAuthority });
    };
    for part in path.split('/') {
        let mut decoded = Vec::new();
        let mut bytes = part.bytes();
        while let Some(b) = bytes.next() {
            if b == b'%' {
                let (Some(a), Some(c)) = (bytes.next(), bytes.next()) else { return Err(UrlEscape); };
                let (Some(a), Some(c)) = ((a as char).to_digit(16), (c as char).to_digit(16)) else { return Err(UrlEscape); };
                decoded.push((a * 16 + c) as u8);
            } else { decoded.push(b); }
        }
        if decoded.is_empty() { return Err(UrlEmptySegment); }
        if decoded == b"." || decoded == b".." { return Err(UrlTraversal); }
        if decoded.iter().any(|b| *b < 32 || *b == 127 || b"/\\".contains(b)) {
            return Err(UrlDecodedControlOrSeparator);
        }
    }
    Ok(())
}

/// First failed candidate predicate; fixed vocabulary, no provider-controlled data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateFailure {
    PageId, Namespace, TitlePrefix, TitleLength, TitleControl, DuplicatePageId,
    UrlLength, UrlNonAscii, UrlControlOrSpace, UrlQueryOrFragment, UrlBackslash,
    UrlExtension, UrlAuthority, UrlPathPrefix, UrlEscape, UrlEmptySegment,
    UrlTraversal, UrlDecodedControlOrSeparator, DigestLength, DigestEncoding,
    DuplicateMediaUrl, CoreContract,
}
fn rejected(kind: CandidateFailure) -> DiscoveryError {
    DiscoveryError::CandidateRejected(kind)
}

/// Fixed vocabulary only: never store error messages, URLs, bodies or identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadinessFailure { Timeout, ListenerTimeout, NegotiationTimeout, DestinationTimeout, BootstrapActivation, ChildExited, Protocol, Io }

/// ConnectOrTls does NOT claim to distinguish a SOCKS circuit from a TLS failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFailure { Timeout, ConnectOrTls, Body, Other }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryError {
    InvalidRequest, ResponseTooLarge, InvalidResponse, RemoteRejected,
    UntrustedCandidate, TooFewChoices, Transport,
    Readiness(ReadinessFailure), Network(NetworkFailure), HttpStatus(u16),
    ContentType, ContentEncoding, CandidateRejected(CandidateFailure),
}
impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRequest => "discovery request is invalid",
            Self::ResponseTooLarge => "discovery response exceeded its byte budget",
            Self::InvalidResponse => "discovery response has an invalid schema",
            Self::RemoteRejected => "discovery provider returned an error or warning",
            Self::UntrustedCandidate => "discovery candidate failed validation",
            Self::CandidateRejected(kind) => return write!(f, "discovery candidate: {kind:?}"),
            Self::TooFewChoices => "fewer than two supported media choices",
            Self::Transport => "discovery transport failed",
            Self::ContentType => "discovery response content type was rejected",
            Self::ContentEncoding => "discovery response content encoding was rejected",
            Self::Readiness(kind) => return write!(f, "discovery readiness: {kind:?}"),
            Self::Network(kind) => return write!(f, "discovery network: {kind:?}"),
            Self::HttpStatus(status) => return write!(f, "discovery HTTP status: {status}"),
        })
    }
}
impl std::error::Error for DiscoveryError {}

#[cfg(test)]
mod tests {
    use super::*;
    use pulqva_core::request_choices;
    use serde_json::{json, Value};

    // SYNTHETIC API schema fixtures, not recorded live responses or a Tor proof.
    fn page(id: u64, leaf: &str) -> Value {
        json!({"pageid":id,"ns":6,"title":format!("File:{leaf}"),"imageinfo":[{
            "url":format!("{MEDIA_PREFIX}a/ab/{leaf}"),"size":32,
            "sha1":"a".repeat(40),"mime":"video/webm"}]})
    }
    fn body(pages: Vec<Value>) -> Vec<u8> {
        serde_json::to_vec(&json!({"batchcomplete":true,"query":{"pages":pages}})).unwrap()
    }
    fn good_body() -> Vec<u8> { body(vec![page(1,"A.webm"),page(2,"B.webm")]) }
    struct Fixture { body: Vec<u8>, calls: usize, fail: bool, urls: Vec<String> }
    impl CommonsTransport for Fixture {
        fn fetch(&mut self, plan: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
            self.calls += 1;
            self.urls.push(plan.url().to_owned());
            assert_eq!(plan.proxy_url(), "socks5h://127.0.0.1:19050");
            assert_eq!(plan.timeout(), Duration::from_secs(60));
            assert_eq!(plan.max_response_bytes(), MAX_RESPONSE_BYTES);
            assert_eq!(plan.max_redirects(), 0);
            assert!(plan.user_agent().starts_with("PULQVA/"));
            if self.fail { Err(DiscoveryError::Transport) } else { Ok(self.body.clone()) }
        }
    }
    fn fixture() -> CommonsSearch<Fixture> {
        // Private struct initialization simulates route data only. No public
        // readiness token constructor is introduced and no network is opened.
        CommonsSearch { transport: Fixture { body: good_body(), calls:0, fail:false, urls:Vec::new() },
            proxy:"socks5h://127.0.0.1:19050".into(), ready: None, ytdlp_executable: None, last_results:Vec::new(), last_error:None }
    }
    #[test]
    fn request_to_response_choices_uses_existing_coordinator() {
        let mut search = fixture();
        let choices = request_choices(&mut search, "countdown").unwrap();
        assert_eq!(choices.intent().query(), "countdown");
        assert_eq!(choices.candidates().len(), 2);
        let selected = choices.select(1).unwrap();
        assert_eq!(selected.candidate().locator(), format!("{MEDIA_PREFIX}a/ab/B.webm"));
        assert_eq!(search.last_results()[1].page_id(), 2);
        assert_eq!(search.last_results()[1].declared_size(), 32);
        assert_eq!(search.last_results()[1].declared_sha1(), "a".repeat(40));
        assert_eq!(search.transport.calls, 1);
        assert!(search.transport.urls[0].ends_with("gsrsearch=countdown%20filetype%3Avideo%20filemime%3A%22video%2Fwebm%22%20filesize%3A%3C8192"));
        assert_eq!(choices.select(2), Err(JourneyError::InvalidSelection));
    }
    #[test]
    fn query_is_encoded_as_data_and_not_replaced_by_a_canned_locator() {
        let mut search = fixture();
        request_choices(&mut search, "snow &format=xml").unwrap();
        request_choices(&mut search, "кино").unwrap();
        assert_ne!(search.transport.urls[0], search.transport.urls[1]);
        assert!(search.transport.urls[0].ends_with("snow%20%26format%3Dxml%20filetype%3Avideo%20filemime%3A%22video%2Fwebm%22%20filesize%3A%3C8192"));
        assert!(search.transport.urls[1].contains("%D0%BA%D0%B8%D0%BD%D0%BE"));
        assert_eq!(search.transport.urls[0].matches("format=").count(), 1);
    }
    #[test]
    fn invalid_requests_do_not_call_transport() {
        let mut search = fixture();
        for request in [" ".to_owned(), "x".repeat(513), "line\nbreak".to_owned()] {
            assert_eq!(request_choices(&mut search, request), Err(JourneyError::InvalidRequest));
        }
        assert_eq!(search.transport.calls, 0);
    }
    #[test]
    fn failed_search_never_reuses_previous_choices() {
        let mut search = fixture();
        request_choices(&mut search, "countdown").unwrap();
        search.transport.fail = true;
        assert_eq!(request_choices(&mut search, "snow"), Err(JourneyError::SearchFailed));
        assert!(search.last_results().is_empty());
        assert_eq!(search.transport.calls, 2);
    }
    #[test]
    fn refuses_size_schema_error_warning_and_duplicate_fields() {
        assert_eq!(parse_response(&vec![b' '; MAX_RESPONSE_BYTES + 1]), Err(DiscoveryError::ResponseTooLarge));
        for bytes in [b"not json".as_slice(), b"{\"query\":{\"pages\":{}}}", b"{\"query\":null,\"query\":null}"] {
            assert_eq!(parse_response(bytes), Err(DiscoveryError::InvalidResponse));
        }
        for key in ["error", "errors", "warnings"] {
            let mut value: Value = serde_json::from_slice(&good_body()).unwrap();
            value[key] = json!({"code":"test-only-rejection"});
            assert_eq!(parse_response(&serde_json::to_vec(&value).unwrap()), Err(DiscoveryError::RemoteRejected));
        }
    }
    #[test]
    fn rejects_duplicate_candidates_and_unsafe_download_targets() {
        assert_eq!(parse_response(&body(vec![page(1,"A.webm"),page(1,"B.webm")])), Err(DiscoveryError::UntrustedCandidate));
        assert_eq!(parse_response(&body(vec![page(1,"A.webm"),page(2,"A.webm")])), Err(DiscoveryError::UntrustedCandidate));
        for url in ["http://upload.wikimedia.org/wikipedia/commons/a/A.webm",
            "https://upload.wikimedia.org.evil.invalid/wikipedia/commons/a/A.webm",
            "https://upload.wikimedia.org@evil.invalid/wikipedia/commons/a/A.webm",
            "https://upload.wikimedia.org/wikipedia/commons/%2e%2e/A.webm",
            "https://upload.wikimedia.org/wikipedia/commons/a/%2fA.webm",
            "https://upload.wikimedia.org/wikipedia/commons/a/A.webm?redirect=1",
            "file:///tmp/A.webm"] {
            let mut bad = page(2,"B.webm"); bad["imageinfo"][0]["url"] = json!(url);
            assert_eq!(parse_response(&body(vec![page(1,"A.webm"),bad])), Err(DiscoveryError::UntrustedCandidate));
        }
    }
    #[test]
    fn unsupported_missing_or_oversized_media_do_not_become_choices() {
        assert_eq!(parse_response(&body(vec![])), Err(DiscoveryError::TooFewChoices));
        for (key, value) in [("mime", json!("text/html")), ("size", json!(0)), ("size", json!(MAX_MEDIA_BYTES+1))] {
            let mut bad = page(2,"B.webm"); bad["imageinfo"][0][key] = value;
            assert_eq!(parse_response(&body(vec![page(1,"A.webm"),bad])), Err(DiscoveryError::TooFewChoices));
        }
        let mut bad = page(2,"B.webm"); bad["imageinfo"] = json!([]);
        assert_eq!(parse_response(&body(vec![page(1,"A.webm"),bad])), Err(DiscoveryError::TooFewChoices));
        let pages = (1..=11).map(|id| page(id,&format!("{id}.webm"))).collect();
        assert_eq!(parse_response(&body(pages)), Err(DiscoveryError::InvalidResponse));
    }
    #[test]
    fn provider_query_matches_supported_media_and_one_revision_envelope() {
        let plan = CommonsSearchPlan::from_verified_proxy(
            &SearchIntent::new("countdown").unwrap(), "socks5h://127.0.0.1:19050".into()
        ).unwrap();
        assert_eq!(MAX_MEDIA_BYTES, 8192 * 1024);
        assert_eq!(plan.url(), concat!(
            "https://commons.wikimedia.org/w/api.php?action=query&format=json&formatversion=2",
            "&generator=search&gsrnamespace=6&gsrlimit=10&prop=imageinfo&iilimit=1",
            "&iiprop=url%7Csize%7Csha1%7Cmime&gsrsearch=countdown%20filetype%3Avideo",
            "%20filemime%3A%22video%2Fwebm%22%20filesize%3A%3C8192"
        ));
    }
    #[test]
    fn local_byte_cap_is_enforced_even_when_provider_ignores_search_filters() {
        let mut at_limit = page(2, "B.webm");
        at_limit["imageinfo"][0]["size"] = json!(MAX_MEDIA_BYTES);
        let accepted = parse_response(&body(vec![page(1, "A.webm"), at_limit.clone()])).unwrap();
        assert_eq!(accepted[1].declared_size(), MAX_MEDIA_BYTES);
        at_limit["imageinfo"][0]["size"] = json!(MAX_MEDIA_BYTES + 1);
        assert_eq!(parse_response(&body(vec![page(1, "A.webm"), at_limit])), Err(DiscoveryError::TooFewChoices));
    }
}

#[cfg(test)]
mod diagnostic_tests {
    use super::*;
    use pulqva_core::request_choices;
    use std::collections::VecDeque;
    struct Scripted(VecDeque<Result<Vec<u8>, DiscoveryError>>);
    impl CommonsTransport for Scripted {
        fn fetch(&mut self, _: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
            self.0.pop_front().expect("unexpected additional request")
        }
    }
    fn good() -> Vec<u8> {
        let pages: Vec<_> = (1..=2).map(|id| serde_json::json!({
            "pageid": id, "ns": 6, "title": format!("File:{id}.webm"),
            "imageinfo": [{"url": format!("{MEDIA_PREFIX}a/ab/{id}.webm"),
                "size": 32, "sha1": "a".repeat(40), "mime": "video/webm"}]
        })).collect();
        serde_json::to_vec(&serde_json::json!({"query":{"pages":pages}})).unwrap()
    }
    #[test]
    fn safe_cause_survives_coordinator_and_resets_after_success() {
        let cases = [
            Err(DiscoveryError::HttpStatus(403)), Err(DiscoveryError::HttpStatus(429)),
            Err(DiscoveryError::Network(NetworkFailure::ConnectOrTls)),
            Err(DiscoveryError::Network(NetworkFailure::Timeout)),
            Err(DiscoveryError::Readiness(ReadinessFailure::Timeout)),
            Err(DiscoveryError::ContentType), Err(DiscoveryError::ContentEncoding),
            Ok(b"not json".to_vec()), Ok(br#"{"error":{"code":"maxlag","info":"PRIVATE"}}"#.to_vec()),
            Ok(br#"{"warnings":{"query":{"warnings":"PRIVATE"}}}"#.to_vec()),
        ];
        for response in cases {
            let expected = match &response {
                Err(error) => *error,
                Ok(body) => parse_response(body).unwrap_err(),
            };
            let mut search = CommonsSearch {
                transport: Scripted(VecDeque::from([Ok(good()), response, Ok(good())])),
                proxy: "socks5h://127.0.0.1:19050".into(), ready: None, ytdlp_executable: None,
                last_results: Vec::new(), last_error: None,
            };
            request_choices(&mut search, "fixture").unwrap();
            assert_eq!(search.last_error(), None);
            assert_eq!(request_choices(&mut search, "fixture"), Err(JourneyError::SearchFailed));
            assert_eq!(search.last_error(), Some(expected));
            assert!(search.last_results().is_empty());
            let diagnostic = format!("{expected:?}: {expected}");
            assert!(!diagnostic.contains("PRIVATE") && !diagnostic.contains("://"));
            assert!(diagnostic.len() < 160);
            request_choices(&mut search, "fixture").unwrap();
            assert_eq!(search.last_error(), None);
            assert_eq!(search.last_results().len(), 2);
            assert!(search.transport.0.is_empty());
        }
    }
    #[test]
    fn local_validation_error_replaces_old_cause_without_fetch() {
        let mut search = CommonsSearch {
            transport: Scripted(VecDeque::new()), proxy: "socks5h://127.0.0.1:19050".into(), ready: None, ytdlp_executable: None,
            last_results: Vec::new(), last_error: Some(DiscoveryError::HttpStatus(403)),
        };
        let intent = SearchIntent::new("x".repeat(513)).unwrap();
        assert_eq!(search.discover(&intent), Err(DiscoveryError::InvalidRequest));
        assert_eq!(search.last_error(), Some(DiscoveryError::InvalidRequest));
    }
    #[test]
    fn retrieval_binding_rejects_stale_foreign_and_unimplemented_downloads_without_receipt() {
        use pulqva_core::{CandidateRetrieval, ChoiceSet};
        let mut search = CommonsSearch {
            transport: Scripted(VecDeque::from([Ok(good())])),
            proxy: "socks5h://127.0.0.1:19050".into(), ready: None, ytdlp_executable: None,
            last_results: Vec::new(), last_error: None,
        };
        let choices = request_choices(&mut search, "countdown").unwrap();
        let selected = choices.select(1).unwrap();
        let root = std::path::Path::new("unused-c2-root");
        assert_eq!(CandidateRetrieval::retrieve(&mut search, &selected, root),
                   Err(JourneyError::RetrievalFailed));

        let foreign_choices = ChoiceSet::new(
            SearchIntent::new("foreign").unwrap(),
            vec![
                SearchCandidate::new("Foreign A", "https://example.invalid/a").unwrap(),
                SearchCandidate::new("Foreign B", "https://example.invalid/b").unwrap(),
            ],
        ).unwrap();
        let foreign = foreign_choices.select(1).unwrap();
        assert_eq!(CandidateRetrieval::retrieve(&mut search, &foreign, root),
                   Err(JourneyError::RetrievalFailed));

        search.last_results.clear();
        assert_eq!(CandidateRetrieval::retrieve(&mut search, &selected, root),
                   Err(JourneyError::RetrievalFailed));
    }


}

#[cfg(test)]
mod candidate_tests;


#[cfg(test)]
mod provenance_tests;