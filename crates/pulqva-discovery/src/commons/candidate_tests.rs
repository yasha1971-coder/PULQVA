//! Synthetic, network-free regression cases. No live response is recorded here.
use super::*;
use pulqva_core::request_choices;
use serde_json::{json, Value};
use std::collections::VecDeque;

fn page(id: u64) -> Value {
    json!({"pageid":id,"ns":6,"title":format!("File:PRIVATE-{id}.webm"),"imageinfo":[{
        "url":format!("{MEDIA_PREFIX}a/ab/{id}.webm"),"size":32,
        "sha1":"a".repeat(40),"mime":"video/webm"}]})
}
fn response(second: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"query":{"pages":[page(1),second]}})).unwrap()
}
struct Scripted(VecDeque<Vec<u8>>);
impl CommonsTransport for Scripted {
    fn fetch(&mut self, _: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
        Ok(self.0.pop_front().expect("no additional fetch is allowed"))
    }
}
fn expect_rejection(body: Vec<u8>, kind: CandidateFailure) {
    let expected = DiscoveryError::CandidateRejected(kind);
    assert_eq!(parse_response_classified(&body), Err(expected));
    // Existing standalone parser callers retain their original error contract.
    assert_eq!(parse_response(&body), Err(DiscoveryError::UntrustedCandidate));
    let mut search = CommonsSearch {
        transport: Scripted(VecDeque::from([response(page(2)),body,response(page(2))])),
        proxy:"socks5h://127.0.0.1:19050".into(),last_results:Vec::new(),last_error:None,
    };
    request_choices(&mut search,"fixture").unwrap();
    assert_eq!(request_choices(&mut search,"fixture"),Err(JourneyError::SearchFailed));
    assert_eq!(search.last_error(),Some(expected));
    assert!(search.last_results().is_empty());
    let safe=format!("{expected:?}: {expected}");
    assert!(safe.len()<160 && !safe.contains("PRIVATE") && !safe.contains("://"));
    request_choices(&mut search,"fixture").unwrap();
    assert_eq!(search.last_error(),None);
    assert_eq!(search.last_results().len(),2);
    assert!(search.transport.0.is_empty());
}
#[test]
fn page_digest_and_duplicate_predicates_survive_coordinator() {
    use CandidateFailure::*;
    for (field,value,kind) in [
        ("pageid",json!(0),PageId),("ns",json!(0),Namespace),
        ("title",json!("PRIVATE"),TitlePrefix),
        ("title",json!(format!("File:{}","x".repeat(508))),TitleLength),
        ("title",json!("File:PRIVATE\n"),TitleControl),
        ("pageid",json!(1),DuplicatePageId),
    ] {
        let mut bad=page(2);bad[field]=value;expect_rejection(response(bad),kind);
    }
    for (field,value,kind) in [
        ("sha1",json!("a".repeat(39)),DigestLength),
        ("sha1",json!("g".repeat(40)),DigestEncoding),
        ("url",json!(format!("{MEDIA_PREFIX}a/ab/1.webm")),DuplicateMediaUrl),
    ] {
        let mut bad=page(2);bad["imageinfo"][0][field]=value;expect_rejection(response(bad),kind);
    }
}
#[test]
fn url_predicates_survive_coordinator_without_relaxing_acceptance() {
    use CandidateFailure::*;
    for (url,kind) in [
        (format!("{MEDIA_PREFIX}{}.webm","x".repeat(2048)),UrlLength),
        (format!("{MEDIA_PREFIX}кино.webm"),UrlNonAscii),
        (format!("{MEDIA_PREFIX}A B.webm"),UrlControlOrSpace),
        (format!("{MEDIA_PREFIX}A.webm?PRIVATE"),UrlQueryOrFragment),
        (format!("{MEDIA_PREFIX}A\\B.webm"),UrlBackslash),
        (format!("{MEDIA_PREFIX}A.WEBM"),UrlExtension),
        ("https://upload.wikimedia.org.evil.invalid/a.webm".into(),UrlAuthority),
        ("https://upload.wikimedia.org/other/A.webm".into(),UrlPathPrefix),
        (format!("{MEDIA_PREFIX}a/%xy/A.webm"),UrlEscape),
        (format!("{MEDIA_PREFIX}a//A.webm"),UrlEmptySegment),
        (format!("{MEDIA_PREFIX}%2e%2e/A.webm"),UrlTraversal),
        (format!("{MEDIA_PREFIX}a/%2fA.webm"),UrlDecodedControlOrSeparator),
    ] {
        let mut bad=page(2);bad["imageinfo"][0]["url"]=json!(url);
        expect_rejection(response(bad),kind);
    }
}
#[test]
fn accepted_metadata_and_digest_normalization_are_unchanged() {
    let mut second=page(2);second["imageinfo"][0]["sha1"]=json!("ABCDEF0123".repeat(4));
    second["imageinfo"][0]["url"]=json!(format!("{MEDIA_PREFIX}a/ab/A%20B.webm"));
    let bytes=response(second);
    let results=parse_response_classified(&bytes).unwrap();
    assert_eq!(parse_response(&bytes).unwrap(),results);
    assert_eq!(results[1].declared_sha1(),"abcdef0123".repeat(4));
}

// Test-only snapshot of the exact previous URL predicate, used as an independent
// acceptance oracle. The production path does not call or compile this snapshot.
fn legacy_valid_media_url(value: &str) -> bool {
    if value.len() > 2048 || !value.is_ascii() || value.bytes().any(|b| b <= 32 || b == 127)
        || value.contains(['?', '#', '\\']) || !value.ends_with(".webm") { return false; }
    let Some(path) = value.strip_prefix(MEDIA_PREFIX) else { return false; };
    for part in path.split('/') {
        let mut decoded = Vec::new();
        let mut bytes = part.bytes();
        while let Some(b) = bytes.next() {
            if b == b'%' {
                let (Some(a), Some(c)) = (bytes.next(), bytes.next()) else { return false; };
                let (Some(a), Some(c)) = ((a as char).to_digit(16), (c as char).to_digit(16)) else { return false; };
                decoded.push((a * 16 + c) as u8);
            } else { decoded.push(b); }
        }
        if decoded.is_empty() || decoded == b"." || decoded == b".."
            || decoded.iter().any(|b| *b < 32 || *b == 127 || b"/\\".contains(b)) {
            return false;
        }
    }
    true
}

#[test]
fn url_acceptance_matches_previous_predicate_on_boundary_corpus() {
    let mut corpus=vec![String::new(),MEDIA_PREFIX.into(),
        format!("{MEDIA_PREFIX}A.webm"),format!("{MEDIA_PREFIX}A.WEBM")];
    for authority in ["https://upload.wikimedia.org/","http://upload.wikimedia.org/",
        "https://upload.wikimedia.org:443/","https://upload.wikimedia.org.evil.invalid/",
        "https://upload.wikimedia.org@evil.invalid/","file:///"] {
        for path in ["wikipedia/commons/","wikipedia/commons//","wikipedia/commons/../","other/"] {
            for leaf in ["A.webm","A%20B.webm","A.webm?x","A.webm#x","A\\B.webm","кино.webm"] {
                corpus.push(format!("{authority}{path}{leaf}"));
            }
        }
    }
    for byte in 0..=255_u16 {
        corpus.push(format!("{MEDIA_PREFIX}%{byte:02X}/A.webm"));
        corpus.push(format!("{MEDIA_PREFIX}A%{byte:02X}B.webm"));
    }
    for part in ["%","%0","%gg",".","..","%2e","%2E%2e","%252e%252e",""] {
        corpus.push(format!("{MEDIA_PREFIX}{part}/A.webm"));
    }
    for length in [2047_usize,2048,2049] {
        corpus.push(format!("{MEDIA_PREFIX}{}.webm","x".repeat(length-MEDIA_PREFIX.len()-5)));
    }
    for url in corpus {
        assert_eq!(validate_media_url(&url).is_ok(),legacy_valid_media_url(&url),
            "candidate URL acceptance changed");
    }
}
