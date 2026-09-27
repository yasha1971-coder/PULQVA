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
