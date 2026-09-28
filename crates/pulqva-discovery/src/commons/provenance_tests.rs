//! Synthetic fixtures derived from upstream source, NOT a recorded live response.
//! MediaWiki File::getUrl/appendRequestProvenance + ApiQueryImageInfo::getInfo:
//! https://doc.wikimedia.org/mediawiki-core/master/php/File_8php_source.html
//! https://github.com/wikimedia/mediawiki/blob/master/includes/Api/ApiQueryImageInfo.php
//! Reviewed 2026-09-27. Upstream adds format=original and generator=imageinfo;
//! the site is commons.wikimedia.org for this endpoint. No arbitrary UTM values.
use super::*;
use pulqva_core::request_choices;
use serde_json::{json, Value};
use std::collections::VecDeque;

const PROVENANCE: &str = "utm_source=commons.wikimedia.org&utm_campaign=imageinfo&utm_content=original";
fn url(leaf: &str) -> String { format!("{MEDIA_PREFIX}a/ab/{leaf}.webm") }
fn page(id: u64, locator: &str) -> Value {
    json!({"pageid":id,"ns":6,"title":format!("File:fixture-{id}.webm"),"imageinfo":[{
        "url":locator,"size":32,"sha1":"a".repeat(40),"mime":"video/webm"}]})
}
fn body(first: &str, second: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"query":{"pages":[page(1,first),page(2,second)]}})).unwrap()
}

#[test]
fn upstream_original_provenance_is_normalized_before_existing_strict_validation() {
    let plain = url("A");
    let tagged = format!("{plain}?{PROVENANCE}");
    // Minimal reproducer: the prior strict validator rejects this source-derived
    // provider form. That validator remains strict after normalization is added.
    assert_eq!(validate_media_url(&tagged), Err(CandidateFailure::UrlQueryOrFragment));
    assert_eq!(normalize_media_url(&tagged), Ok(plain.as_str()));
    assert_eq!(normalize_media_url(&plain), Ok(plain.as_str()));
    let expected = parse_response(&body(&plain, &url("B"))).unwrap();
    let actual = parse_response(&body(&tagged, &format!("{}?{PROVENANCE}",url("B")))).unwrap();
    assert_eq!(actual, expected);
    let escaped = url("A%3FB%23C");
    assert_eq!(normalize_media_url(&format!("{escaped}?{PROVENANCE}")), Ok(escaped.as_str()));
}

#[test]
fn exactly_three_unique_pairs_are_order_independent_and_idempotent() {
    let pairs = ["utm_source=commons.wikimedia.org", "utm_campaign=imageinfo", "utm_content=original"];
    let plain = url("A");
    let mut count = 0;
    for a in 0..3 { for b in 0..3 { for c in 0..3 {
        if a == b || b == c || a == c { continue; }
        let tagged = format!("{plain}?{}&{}&{}",pairs[a],pairs[b],pairs[c]);
        let normalized = normalize_media_url(&tagged).unwrap();
        assert_eq!(normalized, plain);
        assert_eq!(normalize_media_url(normalized), Ok(normalized));
        count += 1;
    } } }
    assert_eq!(count, 6);
}

#[test]
fn unknown_ambiguous_or_nonoriginal_queries_and_all_fragments_are_rejected() {
    let plain = url("A");
    for suffix in [
        "?".to_owned(), "#".to_owned(), "#t=1".to_owned(), "?download=1".to_owned(),
        format!("?{PROVENANCE}#"), format!("?{PROVENANCE}#t=1"),
        format!("?{PROVENANCE}&redirect=https://evil.invalid"),
        format!("?{PROVENANCE}&utm_source=commons.wikimedia.org"),
        format!("?{PROVENANCE}&utm_campaign=imageinfo"),
        format!("?{PROVENANCE}&utm_content=original"),
        format!("?{PROVENANCE}&"), format!("?&{PROVENANCE}"),
        format!("?{}", PROVENANCE.replace('&',"&amp;")),
        format!("?{}", PROVENANCE.replace('&',";")),
        format!("?{}", PROVENANCE.replace("commons.wikimedia.org","evil.invalid")),
        format!("?{}", PROVENANCE.replace("imageinfo","api")),
        format!("?{}", PROVENANCE.replace("original","thumbnail")),
        format!("?{}", PROVENANCE.replace("utm_source","UTM_SOURCE")),
        format!("?{}", PROVENANCE.replace("utm_source","%75tm_source")),
        format!("?{}", PROVENANCE.replace("original","%6friginal")),
        format!("?{}", PROVENANCE.replace("original","original%00")),
        "?utm_source=commons.wikimedia.org&utm_campaign=imageinfo".to_owned(),
        "?utm_content=original".to_owned(), format!("?{PROVENANCE}?x=1"),
    ] {
        let invalid = format!("{plain}{suffix}");
        assert_eq!(normalize_media_url(&invalid), Err(CandidateFailure::UrlQueryOrFragment));
        assert_eq!(parse_response(&body(&url("B"), &invalid)), Err(DiscoveryError::UntrustedCandidate));
    }
}

#[test]
fn known_provenance_never_legitimizes_an_unsafe_base_or_bypasses_full_input_cap() {
    for base in [
        "http://upload.wikimedia.org/wikipedia/commons/a/A.webm",
        "https://upload.wikimedia.org.evil.invalid/wikipedia/commons/a/A.webm",
        "https://upload.wikimedia.org@evil.invalid/wikipedia/commons/a/A.webm",
        "https://upload.wikimedia.org:443/wikipedia/commons/a/A.webm",
        "https://upload.wikimedia.org/wikipedia/en/a/A.webm",
        "https://upload.wikimedia.org/wikipedia/commons/%2e%2e/A.webm",
        "https://upload.wikimedia.org/wikipedia/commons/a/%2fA.webm",
        "https://upload.wikimedia.org/wikipedia/commons/a/%5cA.webm",
        "https://upload.wikimedia.org/wikipedia/commons/a/%00A.webm",
        "https://upload.wikimedia.org/wikipedia/commons/a/%zzA.webm",
        "https://upload.wikimedia.org/wikipedia/commons//A.webm",
        "https://upload.wikimedia.org/wikipedia/commons/a/A.mp4",
        "file:///tmp/A.webm",
    ] {
        let expected = validate_media_url(base).unwrap_err();
        let tagged = format!("{base}?{PROVENANCE}");
        assert_eq!(normalize_media_url(&tagged), Err(expected));
        assert_eq!(parse_response(&body(&url("B"), &tagged)), Err(DiscoveryError::UntrustedCandidate));
    }
    let base = format!("{MEDIA_PREFIX}a/ab/{}.webm", "x".repeat(2000 - MEDIA_PREFIX.len() - 10));
    assert!(base.len() < 2048 && validate_media_url(&base).is_ok());
    assert_eq!(normalize_media_url(&format!("{base}?{PROVENANCE}")), Err(CandidateFailure::UrlLength));
    for (suffix, error) in [("\n", CandidateFailure::UrlControlOrSpace),
        ("\u{7f}", CandidateFailure::UrlControlOrSpace), ("кино", CandidateFailure::UrlNonAscii)] {
        assert_eq!(normalize_media_url(&format!("{}?{PROVENANCE}{suffix}",url("A"))), Err(error));
    }
}

#[test]
fn duplicates_are_checked_after_normalization() {
    let plain = url("A");
    let tagged = format!("{plain}?{PROVENANCE}");
    for (first, second) in [(&plain,&tagged),(&tagged,&plain),(&tagged,&tagged)] {
        assert_eq!(parse_response_classified(&body(first,second)),
            Err(DiscoveryError::CandidateRejected(CandidateFailure::DuplicateMediaUrl)));
    }
}

struct Scripted { replies: VecDeque<Vec<u8>>, calls: usize }
impl CommonsTransport for Scripted {
    fn fetch(&mut self, _: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
        self.calls += 1;
        Ok(self.replies.pop_front().expect("unexpected extra fetch"))
    }
}
#[test]
fn coordinator_exposes_only_canonical_locators_and_clears_results_on_rejection() {
    let good = body(&format!("{}?{PROVENANCE}",url("A")), &url("B"));
    let bad = body(&url("A"), &format!("{}?{PROVENANCE}&private=SECRET",url("B")));
    let mut search = CommonsSearch {
        transport: Scripted { replies: VecDeque::from([good.clone(),bad,good]), calls:0 },
        proxy:"socks5h://127.0.0.1:19050".into(),ready:None,last_results:Vec::new(),last_error:None,
    };
    let choices = request_choices(&mut search,"fixture").unwrap();
    assert_eq!(choices.select(0).unwrap().candidate().locator(),url("A"));
    assert_eq!(search.last_results()[0].candidate().locator(),url("A"));
    assert_eq!(search.transport.calls,1);
    assert_eq!(request_choices(&mut search,"fixture"),Err(JourneyError::SearchFailed));
    assert!(search.last_results().is_empty());
    assert_eq!(search.last_error(),Some(DiscoveryError::CandidateRejected(CandidateFailure::UrlQueryOrFragment)));
    assert!(!format!("{:?}",search.last_error()).contains("SECRET"));
    request_choices(&mut search,"fixture").unwrap();
    assert_eq!(search.last_error(),None);
    assert_eq!(search.transport.calls,3);
    assert!(search.transport.replies.is_empty());
}
