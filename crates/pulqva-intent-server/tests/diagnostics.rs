use pulqva_intent_server::{IntentServer,IntentServerPlan,ReadinessDiagnostics};
use std::{fs,net::TcpListener,process::Command,time::{Duration,SystemTime,UNIX_EPOCH}};

#[test]
fn diagnostic_reports_completion_failure_without_marking_ready(){
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();
    let port=listener.local_addr().unwrap().port(); drop(listener);
    let plan=IntentServerPlan::new(env!("CARGO_BIN_EXE_server_fixture"),"bad-completion-model.gguf",
        include_str!("../../../sidecars/llama.cpp/intent.schema.json"),port,Duration::from_millis(500)).unwrap();
    let mut trace=ReadinessDiagnostics::default();
    assert!(IntentServer::spawn_diagnostic(&plan,&mut trace).is_err());
    trace.settle_stderr();
    let report=trace.report();
    assert_eq!(report["ready"],false);
    assert_eq!(report["phase"],"completion");
    assert_eq!(report["health_ok_seen"],true);
    assert!(report["completion_attempts"].as_u64().unwrap()>0);
    assert_eq!(report["completion_error"],"HttpStatus");
    assert_eq!(report["completion_io_timeout_ms"],500);
    assert!(std::net::TcpStream::connect(("127.0.0.1",port)).is_err());
}

#[test]
fn smoke_preserves_failure_receipt_when_child_cannot_start(){
    let root=std::env::temp_dir().join(format!("pulqva-smoke-diagnostic-{}-{}",std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&root).unwrap();
    let schema=root.join("schema.json");
    fs::write(&schema,include_str!("../../../sidecars/llama.cpp/intent.schema.json")).unwrap();
    let out=root.join("receipt.json");
    let output=Command::new(env!("CARGO_BIN_EXE_real_server_smoke"))
        .arg(root.join("missing-executable")).arg("unused-model.gguf").arg(schema).arg(&out).arg("19081")
        .output().unwrap();
    assert_eq!(output.status.code(),Some(1));
    let receipt:serde_json::Value=serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert_eq!(receipt["all_ok"],false);
    assert_eq!(receipt["stage"],"readiness_failed");
    assert_eq!(receipt["readiness"]["phase"],"process");
    assert_eq!(receipt["readiness"]["health_attempts"],0);
    assert_eq!(receipt["cases"].as_array().unwrap().len(),0);
    assert_eq!(fs::metadata(out.with_extension("stderr.bin")).unwrap().len(),0);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
    fs::remove_dir_all(root).unwrap();
}
