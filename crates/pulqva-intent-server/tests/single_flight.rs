use pulqva_intent_server::{IntentServer, IntentServerError, IntentServerPlan, ReadinessDiagnostics};
use std::{net::{TcpListener, TcpStream}, time::{Duration, Instant}};

fn plan(mode: &str, budget: Duration) -> IntentServerPlan {
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();
    let port=listener.local_addr().unwrap().port();
    drop(listener);
    IntentServerPlan::new(env!("CARGO_BIN_EXE_server_fixture"),mode,
        include_str!("../../../sidecars/llama.cpp/intent.schema.json"),port,budget).unwrap()
}
fn assert_one_request(trace: &ReadinessDiagnostics) {
    trace.settle_stderr();
    assert_eq!(trace.report()["completion_attempts"],1);
    let stderr=trace.stderr_bytes();
    let text=std::str::from_utf8(&stderr).unwrap();
    assert_eq!(text.matches("PULQVA_FIXTURE_COMPLETION").count(),1,"{text}");
}
fn assert_closed(port:u16) {
    assert!(TcpStream::connect(("127.0.0.1",port)).is_err());
}
#[test]
fn delayed_completion_over_500ms_succeeds_once() {
    let p=plan("delayed-completion",Duration::from_secs(3));
    let mut trace=ReadinessDiagnostics::default();
    let started=Instant::now();
    let owner=IntentServer::spawn_diagnostic(&p,&mut trace).unwrap();
    assert!(started.elapsed()>=Duration::from_millis(750));
    assert_eq!(trace.report()["ready"],true);
    assert_eq!(trace.report()["completion_timeout_scope"],"total");
    drop(owner);
    assert_one_request(&trace);
    assert_closed(p.port());
}
#[test]
fn completion_error_is_terminal_not_a_retry_loop() {
    for (mode,expected) in [("bad-completion","HttpStatus"),("invalid-interpretation","Interpretation")] {
        let p=plan(mode,Duration::from_secs(3));
        let mut trace=ReadinessDiagnostics::default();
        assert!(matches!(IntentServer::spawn_diagnostic(&p,&mut trace),Err(IntentServerError::CompletionFailed)));
        assert_one_request(&trace);
        assert_eq!(trace.report()["ready"],false);
        assert_eq!(trace.report()["phase"],"completion");
        let actual=trace.report()["completion_error"].as_str().unwrap().to_owned();
        assert!(actual.starts_with(expected),"{actual}");
        assert_closed(p.port());
    }
}
#[test]
fn completion_budget_is_capped_by_remaining_startup_time() {
    let p=plan("delayed-completion",Duration::from_millis(300));
    let mut trace=ReadinessDiagnostics::default();
    let started=Instant::now();
    assert!(matches!(IntentServer::spawn_diagnostic(&p,&mut trace),Err(IntentServerError::CompletionFailed)));
    assert!(started.elapsed()<Duration::from_secs(2));
    assert_one_request(&trace);
    let report=trace.report();
    assert_eq!(report["completion_error"],"Deadline");
    let budget=report["completion_budget_ms"].as_u64().unwrap();
    assert!(budget>0 && budget<=300);
    assert_eq!(report["ready"],false);
    assert_closed(p.port());
}
#[test]
fn trickled_bytes_do_not_renew_completion_deadline() {
    let p=plan("trickle-completion",Duration::from_millis(400));
    let mut trace=ReadinessDiagnostics::default();
    let started=Instant::now();
    assert!(matches!(IntentServer::spawn_diagnostic(&p,&mut trace),Err(IntentServerError::CompletionFailed)));
    assert!(started.elapsed()<Duration::from_secs(2));
    assert_one_request(&trace);
    assert_eq!(trace.report()["completion_error"],"Deadline");
    assert_eq!(trace.report()["ready"],false);
    assert_closed(p.port());
}
