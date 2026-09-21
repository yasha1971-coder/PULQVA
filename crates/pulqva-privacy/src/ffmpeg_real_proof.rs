use crate::{
    CompletedDownloadResult, FfmpegRemuxContainer, FfmpegRemuxPlan, YtDlpMediaSourceUrl,
    ffmpeg_process::launch_ffmpeg_remux,
    ytdlp_artifact::validate_completed_media_artifact,
};
use std::{
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const PINNED_SOURCE: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/4d21a042353022326071acb0251ab75cd6bae114/big_buck_bunny.mp4";
const REMUX_TIMEOUT: Duration = Duration::from_secs(30);

#[test]
#[ignore = "requires pinned real FFmpeg/ffprobe sidecars and pinned local media fixture"]
fn real_pinned_ffmpeg_remuxes_validated_local_media() {
    let ffmpeg = required_env_path("PULQVA_FFMPEG_BIN");
    let ffprobe = required_env_path("PULQVA_FFPROBE_BIN");
    let fixture = required_env_path("PULQVA_MEDIA_FIXTURE");

    assert!(ffmpeg.is_file(), "pinned FFmpeg executable must exist");
    assert!(ffprobe.is_file(), "pinned ffprobe executable must exist");
    assert!(fixture.is_file(), "pinned media fixture must exist");

    let root = std::env::temp_dir().join(format!(
        "pulqva-real-ffmpeg-remux-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("stale test root cleanup succeeds");
    }

    let input_root = root.join("input");
    let remux_root = root.join("remux");
    fs::create_dir_all(&input_root).expect("input root creation succeeds");
    fs::create_dir_all(&remux_root).expect("remux root creation succeeds");

    let local_input = input_root.join("big_buck_bunny.mp4");
    fs::copy(&fixture, &local_input).expect("pinned media fixture copy succeeds");

    let receipt =
        validate_completed_media_artifact(&input_root).expect("local media fixture validates");
    let source = YtDlpMediaSourceUrl::parse(PINNED_SOURCE).expect("pinned source URL is valid");
    let completed = CompletedDownloadResult::from_validated_completion(source, receipt);

    let output = remux_root.join("big_buck_bunny.mkv");
    let plan = FfmpegRemuxPlan::new(
        &ffmpeg,
        &completed,
        &output,
        FfmpegRemuxContainer::Matroska,
    )
    .expect("typed local remux plan is valid");

    assert_eq!(plan.input(), completed.artifact_path());
    assert_ne!(plan.input(), plan.output());

    let arguments = plan.arguments();
    let whitelist = arguments
        .windows(2)
        .any(|pair| pair[0] == OsStr::new("-protocol_whitelist") && pair[1] == OsStr::new("file"));
    assert!(whitelist, "real remux must keep the local file-only protocol whitelist");
    assert!(
        !arguments.iter().any(|arg| {
            let value = arg.to_string_lossy();
            value.starts_with("http://")
                || value.starts_with("https://")
                || value.starts_with("tcp://")
                || value.starts_with("udp://")
        }),
        "FFmpeg argv must contain no network input"
    );

    let mut running = launch_ffmpeg_remux(plan).expect("real FFmpeg child launches directly");
    let deadline = Instant::now() + REMUX_TIMEOUT;

    let status = loop {
        if let Some(status) = running.try_wait().expect("FFmpeg child status query succeeds") {
            break status;
        }

        if Instant::now() >= deadline {
            let _ = running.stop_and_wait();
            panic!("real FFmpeg remux exceeded bounded timeout");
        }

        thread::sleep(Duration::from_millis(50));
    };

    assert!(status.success(), "real FFmpeg remux must succeed: {status}");

    let output_meta = fs::symlink_metadata(&output).expect("remuxed output metadata exists");
    assert!(!output_meta.file_type().is_symlink(), "remux output must not be a symlink");
    assert!(output_meta.is_file(), "remux output must be a regular file");
    assert!(output_meta.len() > 0, "remux output must be non-empty");
    assert_ne!(
        fs::canonicalize(&output).expect("output canonicalization succeeds"),
        fs::canonicalize(completed.artifact_path()).expect("input canonicalization succeeds")
    );

    let probe = Command::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file",
            "-show_entries",
            "format=format_name,size",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(&output)
        .stdin(Stdio::null())
        .output()
        .expect("local ffprobe process launches");

    assert!(
        probe.status.success(),
        "ffprobe must validate local remuxed output: {}",
        String::from_utf8_lossy(&probe.stderr)
    );

    let probe_stdout = String::from_utf8(probe.stdout).expect("ffprobe output is UTF-8");
    assert!(
        probe_stdout.contains("format_name=matroska"),
        "ffprobe must identify Matroska output: {probe_stdout}"
    );
    assert!(
        probe_stdout.lines().any(|line| {
            line.strip_prefix("size=")
                .and_then(|value| value.parse::<u64>().ok())
                .is_some_and(|size| size > 0)
        }),
        "ffprobe must report a non-zero output size: {probe_stdout}"
    );

    fs::remove_dir_all(root).expect("real remux test cleanup succeeds");
}

fn required_env_path(name: &str) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{name} must be set by the dedicated real-remux workflow"))
}
