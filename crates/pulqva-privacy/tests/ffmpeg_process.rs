use pulqva_privacy::{
    ArtiRuntimePlan, FfmpegRemuxContainer, FfmpegRemuxPlan, TorSocksEndpoint,
    YtDlpLaunchPlan, YtDlpMediaRequestPlan, YtDlpMediaSourceUrl, launch_ffmpeg_remux,
    launch_prepared_arti, launch_ytdlp_request, prepare_arti_runtime, verify_tor_readiness,
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
        "pulqva-ffmpeg-process-{label}-{}-{sequence}",
        std::process::id()
    ))
}

fn ffmpeg_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-ffmpeg-fixture"))
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

fn completed_download(root: &Path) -> (pulqva_privacy::RunningArti, pulqva_privacy::CompletedDownloadResult) {
    let port = free_loopback_port();
    let arti_plan = ArtiRuntimePlan::new(
        arti_fixture(),
        root.join("arti/config/pulqva.toml"),
        root.join("arti/cache"),
        root.join("arti/state"),
        TorSocksEndpoint::new(port).expect("port is non-zero"),
    );

    let prepared = prepare_arti_runtime(arti_plan).expect("Arti preparation succeeds");
    let mut arti = launch_prepared_arti(prepared).expect("Arti fixture launch succeeds");
    let ready = verify_tor_readiness(&mut arti, Duration::from_secs(5))
        .expect("local SOCKS fixture becomes ready");

    let output_root = root.join("download");
    let launch = YtDlpLaunchPlan::new(ytdlp_fixture(), ready);
    let source =
        YtDlpMediaSourceUrl::parse("https://example.invalid/success").expect("valid source");
    let request =
        YtDlpMediaRequestPlan::new(launch, source, &output_root).expect("valid request");

    let running = launch_ytdlp_request(request).expect("yt-dlp fixture launch succeeds");
    let completed = running
        .complete_download()
        .expect("yt-dlp fixture yields validated completed download");

    (arti, completed)
}

#[test]
fn controlled_ffmpeg_launcher_passes_exact_local_only_argv_and_stops_cleanly() {
    let root = test_root("argv");
    let (arti, completed) = completed_download(&root);
    let output = root.join("remux").join("output.mp4");

    let plan = FfmpegRemuxPlan::new(
        ffmpeg_fixture(),
        &completed,
        &output,
        FfmpegRemuxContainer::Mp4,
    )
    .expect("remux plan is valid");

    let expected = plan
        .arguments()
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("\n");

    let running = launch_ffmpeg_remux(plan).expect("direct FFmpeg fixture launch succeeds");
    assert!(running.id() > 0);

    let mut marker_os = output.as_os_str().to_os_string();
    marker_os.push(".pulqva-ffmpeg-fixture.argv");
    let marker = PathBuf::from(marker_os);

    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }

    assert_eq!(
        fs::read_to_string(&marker).expect("fixture recorded argv"),
        expected
    );

    assert!(!output.exists(), "fixture must not perform media transformation");

    let _ = running
        .stop_and_wait()
        .expect("FFmpeg fixture shutdown succeeds");
    let _ = arti.stop_and_wait().expect("Arti fixture shutdown succeeds");

    fs::remove_dir_all(root).expect("cleanup succeeds");
}
