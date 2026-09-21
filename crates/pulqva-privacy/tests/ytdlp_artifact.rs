use pulqva_privacy::{
    ArtiRuntimePlan, TorSocksEndpoint, YtDlpCompletionError, YtDlpLaunchPlan,
    YtDlpMediaRequestPlan, YtDlpMediaSourceUrl, launch_prepared_arti,
    launch_ytdlp_request, prepare_arti_runtime, verify_tor_readiness,
};
use std::{
    fs,
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root(label: &str) -> PathBuf {
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "pulqva-complete-{label}-{}-{sequence}",
        std::process::id()
    ))
}

fn ytdlp_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-ytdlp-complete-fixture"))
}

fn arti_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-ready-arti-fixture"))
}

fn free_loopback_port() -> u16 {
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("ephemeral loopback bind succeeds");
    listener.local_addr().expect("local address exists").port()
}

fn ready_transport(root: &Path) -> (pulqva_privacy::RunningArti, pulqva_privacy::ReadyTorTransport) {
    let port = free_loopback_port();
    let plan = ArtiRuntimePlan::new(
        arti_fixture(),
        root.join("arti/config/pulqva.toml"),
        root.join("arti/cache"),
        root.join("arti/state"),
        TorSocksEndpoint::new(port).expect("port is non-zero"),
    );

    let prepared = prepare_arti_runtime(plan).expect("Arti preparation succeeds");
    let mut arti = launch_prepared_arti(prepared).expect("Arti fixture launch succeeds");
    let ready = verify_tor_readiness(&mut arti, Duration::from_secs(5))
        .expect("local SOCKS fixture becomes ready");

    (arti, ready)
}

#[test]
fn receipt_is_created_only_after_successful_child_completion() {
    let root = test_root("success");
    let output_root = root.join("downloads");
    let (arti, ready) = ready_transport(&root);

    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source =
        YtDlpMediaSourceUrl::parse("https://example.invalid/success").expect("valid URL");
    let request =
        YtDlpMediaRequestPlan::new(launch, source, &output_root).expect("valid request");

    let running = launch_ytdlp_request(request).expect("fixture launch succeeds");
    let receipt = running.complete().expect("successful child yields receipt");

    assert_eq!(receipt.byte_size(), 6);
    assert_eq!(
        receipt.path(),
        fs::canonicalize(output_root.join("artifact.bin"))
            .expect("artifact canonicalization succeeds")
    );

    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");
    fs::remove_dir_all(root).expect("cleanup succeeds");
}

#[test]
fn completed_download_result_retains_typed_source_and_validated_artifact_only() {
    let root = test_root("download-result");
    let output_root = root.join("downloads");
    let (arti, ready) = ready_transport(&root);

    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source_url = "https://example.invalid/success";
    let source = YtDlpMediaSourceUrl::parse(source_url).expect("valid URL");
    let request =
        YtDlpMediaRequestPlan::new(launch, source, &output_root).expect("valid request");

    let running = launch_ytdlp_request(request).expect("fixture launch succeeds");
    let result = running
        .complete_download()
        .expect("successful child yields completed download result");

    let canonical_artifact = fs::canonicalize(output_root.join("artifact.bin"))
        .expect("artifact canonicalization succeeds");

    assert_eq!(result.source_url(), source_url);
    assert_eq!(result.artifact_path(), canonical_artifact);
    assert_eq!(result.byte_size(), 6);

    let fields = result.display_fields();
    assert_eq!(fields.source_url(), source_url);
    assert_eq!(fields.artifact_path(), canonical_artifact);
    assert_eq!(fields.byte_size(), 6);

    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");
    fs::remove_dir_all(root).expect("cleanup succeeds");
}

#[test]
fn failed_child_never_yields_receipt() {
    let root = test_root("failure");
    let output_root = root.join("downloads");
    let (arti, ready) = ready_transport(&root);

    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source =
        YtDlpMediaSourceUrl::parse("https://example.invalid/fail").expect("valid URL");
    let request =
        YtDlpMediaRequestPlan::new(launch, source, &output_root).expect("valid request");

    let running = launch_ytdlp_request(request).expect("fixture launch succeeds");

    assert!(matches!(
        running.complete(),
        Err(YtDlpCompletionError::ProcessFailed(_))
    ));

    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");
    if root.exists() {
        fs::remove_dir_all(root).expect("cleanup succeeds");
    }
}
