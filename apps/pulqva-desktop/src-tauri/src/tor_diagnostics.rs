//! Explicitly opted-in, bounded diagnostics; never grants Tor readiness.
use pulqva_privacy::{TorReadinessError, TorReadinessTrace};
use std::{ffi::OsStr, io::{self, Write}};

pub(crate) fn enabled(value: Option<&OsStr>) -> bool {
    value == Some(OsStr::new("1"))
}

fn category(error: &TorReadinessError) -> &'static str {
    match error {
        TorReadinessError::Timeout(_) => "timeout",
        TorReadinessError::BootstrapActivation(_) => "bootstrap-activation",
        TorReadinessError::ChildExited(_) => "child-exited",
        TorReadinessError::Protocol(_) => "protocol",
        TorReadinessError::Io { .. } => "io",
    }
}

/// Closed diagnostic vocabulary only: never format raw errors, queries or paths.
pub(crate) fn write_failure(
    out: &mut impl Write,
    error: &TorReadinessError,
    trace: &TorReadinessTrace,
    cleanup_ok: bool,
) -> io::Result<()> {
    writeln!(out, "PULQVA_TOR_DIAG v=1 error={} cleanup_ok={} omitted={}",
        category(error), cleanup_ok, trace.omitted())?;
    if let TorReadinessError::Timeout(timeout) = error {
        writeln!(out, "timeout_stage={:?}", timeout.stage)?;
    }
    if let TorReadinessError::Io { source, .. } = error {
        writeln!(out, "io_kind={:?}", source.kind())?;
    }
    for item in trace.observations().iter().take(64) {
        writeln!(out, "family={:?} stage={:?} operation={:?} io={:?} socks={:?} class={:?} duration_ms={}",
            item.family(), item.stage(), item.operation(), item.io_kind(),
            item.socks_reply(), item.socks_reply_class(), item.duration_ms())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulqva_privacy::{TorReadinessStage, TorReadinessTimeout};

    #[test]
    fn exact_opt_in_only() {
        assert!(!enabled(None));
        for value in ["", "0", "true", "01", "1 "] {
            assert!(!enabled(Some(OsStr::new(value))));
        }
        assert!(enabled(Some(OsStr::new("1"))));
    }

    #[test]
    fn distinguishes_timeout_stage_and_cleanup_failure() {
        let mut out = Vec::new();
        write_failure(&mut out, &TorReadinessError::Timeout(TorReadinessTimeout {
            stage: TorReadinessStage::Destination,
        }), &TorReadinessTrace::default(), false).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("error=timeout cleanup_ok=false"));
        assert!(text.contains("timeout_stage=Destination"));
    }

    #[test]
    fn never_exports_raw_io_or_protocol_details() {
        for error in [
            TorReadinessError::Io { operation: "PRIVATE_OPERATION", source:
                io::Error::new(io::ErrorKind::PermissionDenied, "PRIVATE_PATH_QUERY") },
            TorReadinessError::Protocol("PRIVATE_PROTOCOL_PAYLOAD"),
        ] {
            let mut out = Vec::new();
            write_failure(&mut out, &error, &TorReadinessTrace::default(), true).unwrap();
            let text = String::from_utf8(out).unwrap();
            assert!(!text.contains("PRIVATE"));
            assert!(text.contains("cleanup_ok=true"));
        }
    }

    #[test]
    fn sink_failure_is_returned_without_panicking() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::ErrorKind::BrokenPipe.into()) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        assert!(write_failure(&mut Broken, &TorReadinessError::Protocol("hidden"),
            &TorReadinessTrace::default(), true).is_err());
    }
}

/// Only DiscoveryError's closed, data-free enum crosses this receipt boundary.
/// No request, response, URL, process error or arbitrary string is accepted.
pub(crate) fn write_search_failure(
    out: &mut impl Write,
    error: Option<pulqva_discovery::DiscoveryError>,
    cleanup_ok: bool,
) -> io::Result<()> {
    writeln!(out, "PULQVA_SEARCH_DIAG v=1 category={error:?} cleanup_ok={cleanup_ok}")
}

#[cfg(test)]
mod search_receipt_tests {
    use super::*;
    use pulqva_discovery::{DiscoveryError as E, ReadinessFailure as R, NetworkFailure as N, CandidateFailure as C};
    #[test]
    fn preserves_every_existing_category_without_sensitive_fields() {
        let mut cases = vec![
            (None, "None"), (Some(E::InvalidRequest), "Some(InvalidRequest)"),
            (Some(E::ResponseTooLarge), "Some(ResponseTooLarge)"),
            (Some(E::InvalidResponse), "Some(InvalidResponse)"),
            (Some(E::RemoteRejected), "Some(RemoteRejected)"),
            (Some(E::UntrustedCandidate), "Some(UntrustedCandidate)"),
            (Some(E::TooFewChoices), "Some(TooFewChoices)"),
            (Some(E::Transport), "Some(Transport)"),
            (Some(E::ContentType), "Some(ContentType)"),
            (Some(E::ContentEncoding), "Some(ContentEncoding)"),
        ];
        cases.extend([
            (Some(E::Readiness(R::Timeout)), "Some(Readiness(Timeout))"),
            (Some(E::Readiness(R::ListenerTimeout)), "Some(Readiness(ListenerTimeout))"),
            (Some(E::Readiness(R::NegotiationTimeout)), "Some(Readiness(NegotiationTimeout))"),
            (Some(E::Readiness(R::DestinationTimeout)), "Some(Readiness(DestinationTimeout))"),
            (Some(E::Readiness(R::BootstrapActivation)), "Some(Readiness(BootstrapActivation))"),
            (Some(E::Readiness(R::ChildExited)), "Some(Readiness(ChildExited))"),
            (Some(E::Readiness(R::Protocol)), "Some(Readiness(Protocol))"),
            (Some(E::Readiness(R::Io)), "Some(Readiness(Io))"),
            (Some(E::Network(N::Timeout)), "Some(Network(Timeout))"),
            (Some(E::Network(N::ConnectOrTls)), "Some(Network(ConnectOrTls))"),
            (Some(E::Network(N::Body)), "Some(Network(Body))"),
            (Some(E::Network(N::Other)), "Some(Network(Other))"),
            (Some(E::CandidateRejected(C::PageId)), "Some(CandidateRejected(PageId))"),
            (Some(E::CandidateRejected(C::Namespace)), "Some(CandidateRejected(Namespace))"),
            (Some(E::CandidateRejected(C::TitlePrefix)), "Some(CandidateRejected(TitlePrefix))"),
            (Some(E::CandidateRejected(C::TitleLength)), "Some(CandidateRejected(TitleLength))"),
            (Some(E::CandidateRejected(C::TitleControl)), "Some(CandidateRejected(TitleControl))"),
            (Some(E::CandidateRejected(C::DuplicatePageId)), "Some(CandidateRejected(DuplicatePageId))"),
            (Some(E::CandidateRejected(C::UrlLength)), "Some(CandidateRejected(UrlLength))"),
            (Some(E::CandidateRejected(C::UrlNonAscii)), "Some(CandidateRejected(UrlNonAscii))"),
            (Some(E::CandidateRejected(C::UrlControlOrSpace)), "Some(CandidateRejected(UrlControlOrSpace))"),
            (Some(E::CandidateRejected(C::UrlQueryOrFragment)), "Some(CandidateRejected(UrlQueryOrFragment))"),
            (Some(E::CandidateRejected(C::UrlBackslash)), "Some(CandidateRejected(UrlBackslash))"),
            (Some(E::CandidateRejected(C::UrlExtension)), "Some(CandidateRejected(UrlExtension))"),
            (Some(E::CandidateRejected(C::UrlAuthority)), "Some(CandidateRejected(UrlAuthority))"),
            (Some(E::CandidateRejected(C::UrlPathPrefix)), "Some(CandidateRejected(UrlPathPrefix))"),
            (Some(E::CandidateRejected(C::UrlEscape)), "Some(CandidateRejected(UrlEscape))"),
            (Some(E::CandidateRejected(C::UrlEmptySegment)), "Some(CandidateRejected(UrlEmptySegment))"),
            (Some(E::CandidateRejected(C::UrlTraversal)), "Some(CandidateRejected(UrlTraversal))"),
            (Some(E::CandidateRejected(C::UrlDecodedControlOrSeparator)), "Some(CandidateRejected(UrlDecodedControlOrSeparator))"),
            (Some(E::CandidateRejected(C::DigestLength)), "Some(CandidateRejected(DigestLength))"),
            (Some(E::CandidateRejected(C::DigestEncoding)), "Some(CandidateRejected(DigestEncoding))"),
            (Some(E::CandidateRejected(C::DuplicateMediaUrl)), "Some(CandidateRejected(DuplicateMediaUrl))"),
            (Some(E::CandidateRejected(C::CoreContract)), "Some(CandidateRejected(CoreContract))"),
        ]);
        for (error, expected) in cases {
            for cleanup in [true, false] {
                let mut out = Vec::new();
                write_search_failure(&mut out, error, cleanup).unwrap();
                let text = String::from_utf8(out).unwrap();
                assert_eq!(text, format!("PULQVA_SEARCH_DIAG v=1 category={expected} cleanup_ok={cleanup}\n"));
                for secret in ["PRIVATE_QUERY", "https://", "127.0.0.1", "cookie=", "token=", "response="] {
                    assert!(!text.contains(secret));
                }
            }
        }
        for status in 0..=u16::MAX {
            let mut out=Vec::new();
            write_search_failure(&mut out, Some(E::HttpStatus(status)), true).unwrap();
            assert_eq!(String::from_utf8(out).unwrap(),
                format!("PULQVA_SEARCH_DIAG v=1 category=Some(HttpStatus({status})) cleanup_ok=true\n"));
        }
    }
    #[test]
    fn broken_receipt_sink_returns_error() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::ErrorKind::BrokenPipe.into()) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        assert!(write_search_failure(&mut Broken, Some(E::Transport), false).is_err());
    }
}
