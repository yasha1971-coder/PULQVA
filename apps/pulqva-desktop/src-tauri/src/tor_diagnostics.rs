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
