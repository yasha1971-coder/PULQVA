//! Deterministic action-trace regressions; these do not claim live Tor readiness.
use super::*;

#[derive(Clone, Copy)]
enum Outcome { Success, Retry(TorReadinessStage), Protocol }

fn response(outcome: Outcome) -> Result<(), SocksProbeError> {
    match outcome {
        Outcome::Success => Ok(()),
        Outcome::Retry(stage) => Err(SocksProbeError::Retryable(stage)),
        Outcome::Protocol => Err(SocksProbeError::Protocol("fixed test error")),
    }
}

fn addresses() -> [SocketAddr; 2] {
    [SocketAddr::from((Ipv4Addr::LOCALHOST, 19050)),
     SocketAddr::from((Ipv6Addr::LOCALHOST, 19050))]
}

#[test]
fn every_retry_stage_still_attempts_ipv6_and_retains_verified_address() {
    for stage in [TorReadinessStage::Listener, TorReadinessStage::Negotiation,
                  TorReadinessStage::Destination] {
        let mut visited = Vec::new();
        let result = probe_loopbacks(addresses(), |address| {
            visited.push(address);
            if address.is_ipv4() { Err(SocksProbeError::Retryable(stage)) }
            else { Ok(()) }
        });
        assert_eq!(result, Ok(IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert_eq!(visited, addresses().to_vec());
    }
}

#[test]
fn retry_pair_retains_furthest_progress_instead_of_last_listener_failure() {
    let stages = [TorReadinessStage::Listener, TorReadinessStage::Negotiation,
                  TorReadinessStage::Destination];
    for first in stages { for second in stages {
        let mut calls = 0;
        let result = probe_loopbacks(addresses(), |_| {
            let stage = if calls == 0 { first } else { second };
            calls += 1;
            Err(SocksProbeError::Retryable(stage))
        });
        assert_eq!(calls, 2);
        assert_eq!(result, Err(SocksProbeError::Retryable(first.max(second))));
    } }
}

#[test]
fn action_trace_matches_pre_diagnostics_policy_for_all_25_outcome_pairs() {
    let outcomes = [Outcome::Success, Outcome::Retry(TorReadinessStage::Listener),
        Outcome::Retry(TorReadinessStage::Negotiation),
        Outcome::Retry(TorReadinessStage::Destination), Outcome::Protocol];
    for first in outcomes { for second in outcomes {
        let script = [first, second];
        // Independent small oracle for the original coarse Retryable semantics.
        let mut expected_trace = Vec::new();
        let mut expected_result = 2; // 0/1: success address; 2: retry; 3: protocol.
        for (index, outcome) in script.iter().enumerate() {
            expected_trace.push(addresses()[index]);
            match outcome {
                Outcome::Success => { expected_result = index; break; }
                Outcome::Protocol => { expected_result = 3; break; }
                Outcome::Retry(_) => (),
            }
        }
        let mut actual_trace = Vec::new();
        let result = probe_loopbacks(addresses(), |address| {
            let index = actual_trace.len();
            actual_trace.push(address);
            response(script[index])
        });
        let actual_result = match result {
            Ok(IpAddr::V4(_)) => 0,
            Ok(IpAddr::V6(_)) => 1,
            Err(SocksProbeError::Retryable(_)) => 2,
            Err(SocksProbeError::Protocol(_)) => 3,
        };
        assert_eq!(actual_trace, expected_trace);
        assert_eq!(actual_result, expected_result);
        assert!(actual_trace.iter().all(|address| address.ip().is_loopback()));
    } }
}

#[test]
fn timeout_stage_display_contains_only_fixed_stage_information() {
    for (stage, name) in [(TorReadinessStage::Listener, "Listener"),
                         (TorReadinessStage::Negotiation, "Negotiation"),
                         (TorReadinessStage::Destination, "Destination")] {
        let error = TorReadinessError::Timeout(TorReadinessTimeout { stage });
        assert_eq!(error.to_string(), format!("Tor readiness verification timed out at {name}"));
        assert!(error.source().is_none());
    }
}


#[test]
fn rd_wire_request_is_fixed_remote_domain_connect() {
    let request = readiness_request().expect("fixed host fits SOCKS5 domain length");
    let host = READINESS_HOST.as_bytes();
    let mut expected = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    expected.extend_from_slice(host);
    expected.extend_from_slice(&READINESS_PORT.to_be_bytes());
    assert_eq!(request, expected);
}

#[test]
fn rd_reply_classifies_all_256_codes() {
    for reply in 0_u8..=u8::MAX {
        let actual = classify_socks_reply(reply);
        let expected = match reply {
            0x00 => TorSocksReplyClass::Succeeded,
            0x01 => TorSocksReplyClass::GeneralFailure,
            0x02 => TorSocksReplyClass::RulesetDenied,
            0x03 => TorSocksReplyClass::NetworkUnreachable,
            0x04 => TorSocksReplyClass::HostUnreachable,
            0x05 => TorSocksReplyClass::ConnectionRefused,
            0x06 => TorSocksReplyClass::TtlExpired,
            0x07 => TorSocksReplyClass::CommandUnsupported,
            0x08 => TorSocksReplyClass::AddressTypeUnsupported,
            _ => TorSocksReplyClass::Unassigned,
        };
        assert_eq!(actual, expected, "REP={reply}");
    }
}

#[test]
fn rd_io_classification_distinguishes_timeout_kinds() {
    for (kind, expected) in [
        (io::ErrorKind::ConnectionRefused, TorReadinessIoKind::ConnectionRefused),
        (io::ErrorKind::ConnectionReset, TorReadinessIoKind::ConnectionReset),
        (io::ErrorKind::ConnectionAborted, TorReadinessIoKind::ConnectionAborted),
        (io::ErrorKind::TimedOut, TorReadinessIoKind::TimedOut),
        (io::ErrorKind::WouldBlock, TorReadinessIoKind::WouldBlock),
        (io::ErrorKind::NotConnected, TorReadinessIoKind::NotConnected),
        (io::ErrorKind::PermissionDenied, TorReadinessIoKind::Other),
    ] {
        assert_eq!(classify_io_kind(&io::Error::from(kind)), expected);
    }
}

#[test]
fn rd_trace_is_bounded_and_counts_omissions() {
    let mut trace = TorReadinessTrace::default();
    for _ in 0..(MAX_READINESS_OBSERVATIONS + 7) {
        trace.push(TorReadinessObservation {
            family: TorReadinessLoopbackFamily::Ipv4,
            stage: TorReadinessStage::Destination,
            operation: TorReadinessOperation::ReadDestination,
            io_kind: Some(TorReadinessIoKind::TimedOut),
            socks_reply: None,
            socks_reply_class: None,
            duration_ms: 1,
        });
    }
    assert_eq!(trace.observations().len(), MAX_READINESS_OBSERVATIONS);
    assert_eq!(trace.omitted(), 7);
}

#[test]
fn rd_trace_contains_only_fixed_privacy_safe_fields() {
    let mut trace = TorReadinessTrace::default();
    trace.push(TorReadinessObservation {
        family: TorReadinessLoopbackFamily::Ipv6,
        stage: TorReadinessStage::Destination,
        operation: TorReadinessOperation::ClassifyReply,
        io_kind: None,
        socks_reply: Some(0x05),
        socks_reply_class: Some(TorSocksReplyClass::ConnectionRefused),
        duration_ms: 3,
    });
    let rendered = format!("{trace:?}");
    for forbidden in [READINESS_HOST, "19050", "/tmp/", "C:\\", "private-error-text"] {
        assert!(!rendered.contains(forbidden), "leaked forbidden diagnostic text: {forbidden}");
    }
}
