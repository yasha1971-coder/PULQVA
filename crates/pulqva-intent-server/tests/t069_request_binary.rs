//! Executes the actual caller binary without starting Tor or a model.
use std::process::{Command, Stdio};

#[test]
fn c3_binary_help_is_honest_and_requires_no_runtime() {
    let out = Command::new(env!("CARGO_BIN_EXE_pulqva_request"))
        .arg("--help").stdin(Stdio::null()).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("not the consumer package"));
    assert!(text.contains("No automatic installs"));
    assert!(out.stderr.is_empty());
}

#[test]
fn c3_binary_missing_configuration_is_a_safe_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_pulqva_request"))
        .stdin(Stdio::null()).output().unwrap();
    assert!(!out.status.success());
    assert!(!String::from_utf8(out.stdout).unwrap().contains("Saved"));
    assert!(String::from_utf8(out.stderr).unwrap().starts_with("PULQVA:"));
}

#[cfg(target_os = "linux")]
#[test]
fn c3_binary_invalid_input_stops_before_runtime_creation_or_child_launch() {
    use std::{fs, io::Write};
    let root = std::env::temp_dir().join(format!("pulqva-c3-binary-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let component = root.join("not-executable");
    fs::write(&component, b"must never execute").unwrap();
    let output = root.join("output"); let runtime = root.join("runtime");
    let mut child = Command::new(env!("CARGO_BIN_EXE_pulqva_request"))
        .args([&component, &component, &component, &component, &output, &runtime])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(b"bad\0request\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("request is empty or contains controls"));
    assert!(!runtime.exists()); assert!(!output.exists());
    assert_eq!(fs::read(&component).unwrap(), b"must never execute");
    fs::remove_file(component).unwrap(); fs::remove_dir(root).unwrap();
}
