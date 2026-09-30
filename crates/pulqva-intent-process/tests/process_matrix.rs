use pulqva_intent_process::{
    interpret_with_local_process, LocalIntentProcessError, LocalIntentProcessPlan,
};
use std::{path::PathBuf, time::{Duration, Instant}};

fn fixture_plan(mode: &str, timeout: Duration) -> LocalIntentProcessPlan {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_intent_fixture"));
    LocalIntentProcessPlan::new(executable, timeout).unwrap().with_argument(mode)
}

#[test]
fn valid_json_crosses_the_strict_boundary() {
    let interpretation = interpret_with_local_process(
        &fixture_plan("valid", Duration::from_secs(2)), "find a countdown video"
    ).unwrap();
    let pulqva_core::Interpretation::Intent(intent) = interpretation else {
        panic!("valid fixture must produce Intent");
    };
    assert_eq!(intent.query(), "countdown video");
    assert_eq!(intent.choice_mode(), pulqva_core::ChoiceMode::Ask);
}

#[test]
fn malformed_output_fails_closed() {
    assert_eq!(
        interpret_with_local_process(
            &fixture_plan("malformed", Duration::from_secs(2)), "countdown"
        ).unwrap_err(),
        LocalIntentProcessError::InvalidOutput
    );
}

#[test]
fn oversized_output_is_rejected() {
    assert_eq!(
        interpret_with_local_process(
            &fixture_plan("oversized", Duration::from_secs(2)), "countdown"
        ).unwrap_err(),
        LocalIntentProcessError::OutputTooLarge
    );
}

#[test]
fn nonzero_exit_is_rejected() {
    assert_eq!(
        interpret_with_local_process(
            &fixture_plan("crash", Duration::from_secs(2)), "countdown"
        ).unwrap_err(),
        LocalIntentProcessError::ChildFailed
    );
}

#[test]
fn timeout_kills_and_reaps_child() {
    let started = Instant::now();
    assert_eq!(
        interpret_with_local_process(
            &fixture_plan("timeout", Duration::from_millis(100)), "countdown"
        ).unwrap_err(),
        LocalIntentProcessError::Timeout
    );
    assert!(started.elapsed() < Duration::from_secs(3));
}
