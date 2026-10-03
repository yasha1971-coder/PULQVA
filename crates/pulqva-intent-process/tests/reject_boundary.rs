use pulqva_core::{Interpretation, RejectReason};
use pulqva_intent_process::{interpret_with_local_process, LocalIntentProcessPlan};
use std::{path::PathBuf, time::Duration};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_intent_fixture"))
}

#[test]
fn child_intent_crosses_supervised_boundary() {
    let plan = LocalIntentProcessPlan::new(fixture(), Duration::from_secs(2)).unwrap()
        .with_argument("valid");
    let result = interpret_with_local_process(&plan, "find countdown video").unwrap();
    match result {
        Interpretation::Intent(intent) => {
            assert_eq!(intent.query(), "countdown video");
            assert_eq!(intent.into_search_intent().unwrap().query(), "countdown video");
        }
        Interpretation::Reject(_) => panic!("valid fixture unexpectedly rejected"),
    }
}

#[test]
fn child_reject_stops_before_search_capability() {
    let plan = LocalIntentProcessPlan::new(fixture(), Duration::from_secs(2)).unwrap()
        .with_argument("reject");
    let result = interpret_with_local_process(&plan, "save C:\\temp\\x.webm instead").unwrap();
    assert_eq!(result, Interpretation::Reject(RejectReason::SemanticAuthority));
    assert!(result.into_search_intent().is_err());
}
