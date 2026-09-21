use pulqva_privacy::{
    ArtiRuntimePlan, TorSocksEndpoint, YtDlpLaunchPlan, YtDlpMediaRequestPlan,
    YtDlpMediaSourceUrl, launch_prepared_arti, launch_ytdlp_request,
    prepare_arti_runtime, verify_tor_readiness,
};
use std::{
    fs,
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn test_root(label: &str) -> PathBuf {
    let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "pulqva-{label}-{}-{sequence}",
        std::process::id()
    ))
}

fn ytdlp_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-ytdlp-fixture"))
}

fn arti_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-ready-arti-fixture"))
}

fn free_loopback_port() -> u16 {
    let listener =
        TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("ephemeral loopback bind succeeds");
    listener
        .local_addr()
        .expect("local address is available")
        .port()
}

fn arti_plan(root: &Path, port: u16) -> ArtiRuntimePlan {
    ArtiRuntimePlan::new(
        arti_fixture(),
        root.join("arti/config/pulqva.toml"),
        root.join("arti/cache"),
        root.join("arti/state"),
        TorSocksEndpoint::new(port).expect("ephemeral port is non-zero"),
    )
}

#[test]
fn typed_request_launches_exact_argv_and_stops_cleanly_without_external_network() {
    let root = test_root("ytdlp-launch");
    let port = free_loopback_port();

    let prepared =
        prepare_arti_runtime(arti_plan(&root, port)).expect("fixture Arti preparation succeeds");
    let mut arti = launch_prepared_arti(prepared).expect("fixture Arti launch succeeds");
    let ready = verify_tor_readiness(&mut arti, Duration::from_secs(5))
        .expect("local fixture certifies Tor transport");

    let output_root = root.join("downloads");
    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source = YtDlpMediaSourceUrl::parse("https://example.invalid/media")
        .expect("test URL is syntactically valid");
    let request = YtDlpMediaRequestPlan::new(launch, source, &output_root)
        .expect("typed request is valid");

    let expected = request
        .arguments()
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n");

    let running = launch_ytdlp_request(request).expect("direct yt-dlp fixture launch succeeds");
    assert!(running.id() > 0);

    let marker = output_root.join("pulqva-ytdlp-fixture.argv");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }

    assert_eq!(
        fs::read_to_string(&marker).expect("fixture recorded argv"),
        expected
    );

    let _ = running.stop_and_wait().expect("yt-dlp fixture shutdown succeeds");
    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");

    fs::remove_dir_all(root).expect("test tree cleanup succeeds");
}

#[test]
fn launcher_uses_only_explicit_fixture_executable_and_no_shell() {
    let root = test_root("ytdlp-explicit-exe");
    let port = free_loopback_port();

    let prepared =
        prepare_arti_runtime(arti_plan(&root, port)).expect("fixture Arti preparation succeeds");
    let mut arti = launch_prepared_arti(prepared).expect("fixture Arti launch succeeds");
    let ready = verify_tor_readiness(&mut arti, Duration::from_secs(5))
        .expect("local fixture certifies Tor transport");

    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source = YtDlpMediaSourceUrl::parse("https://example.invalid/no-network")
        .expect("test URL is syntactically valid");
    let request = YtDlpMediaRequestPlan::new(launch, source, root.join("out"))
        .expect("typed request is valid");

    assert_eq!(request.executable(), ytdlp_fixture());

    let running = launch_ytdlp_request(request).expect("direct fixture launch succeeds");
    let _ = running.stop_and_wait().expect("yt-dlp fixture shutdown succeeds");
    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");

    fs::remove_dir_all(root).expect("test tree cleanup succeeds");
}
