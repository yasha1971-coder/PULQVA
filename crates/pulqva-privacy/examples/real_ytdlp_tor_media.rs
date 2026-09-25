use pulqva_privacy::{
    ArtiRuntimePlan, TorReadinessError, TorSocksEndpoint, YtDlpLaunchPlan,
    YtDlpMediaRequestPlan, YtDlpMediaSourceUrl, launch_prepared_arti,
    prepare_arti_runtime, verify_tor_readiness,
};
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const MEDIA_SOURCE: &str =
    "https://upload.wikimedia.org/wikipedia/commons/5/55/Five-second_counter.webm";
const EXPECTED_MEDIA_BYTES: u64 = 239_482;
const TOR_READY_TIMEOUT: Duration = Duration::from_secs(90);
const YTDLP_TIMEOUT: Duration = Duration::from_secs(180);

fn main() -> Result<(), Box<dyn Error>> {
    let arti = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("expected path to pinned arti executable")?;
    let ytdlp = env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .ok_or("expected path to pinned yt-dlp executable")?;

    if !arti.is_file() {
        return Err(format!("Arti executable does not exist: {}", arti.display()).into());
    }
    if !ytdlp.is_file() {
        return Err(format!("yt-dlp executable does not exist: {}", ytdlp.display()).into());
    }

    let root = env::temp_dir().join(format!(
        "pulqva-real-ytdlp-tor-media-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }

    let plan = ArtiRuntimePlan::new(
        arti,
        root.join("arti/config/pulqva.toml"),
        root.join("arti/cache"),
        root.join("arti/state"),
        TorSocksEndpoint::new(19050)?,
    );

    let prepared = prepare_arti_runtime(plan)?;
    let mut running_arti = launch_prepared_arti(prepared)?;

    let ready = match verify_tor_readiness(&mut running_arti, TOR_READY_TIMEOUT) {
        Ok(ready) => ready,
        Err(TorReadinessError::Timeout) if cfg!(windows) => {
            let _ = running_arti.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            println!("PULQVA_YTDLP_TOR_MEDIA_WINDOWS_FAIL_CLOSED_OK");
            return Ok(());
        }
        Err(error) => {
            let _ = running_arti.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            return Err(Box::new(error));
        }
    };

    let output_root = root.join("downloads");
    let launch = YtDlpLaunchPlan::new(ytdlp, ready);
    let source = YtDlpMediaSourceUrl::parse(MEDIA_SOURCE)?;
    let request = YtDlpMediaRequestPlan::new(launch, source, &output_root)?;

    if request.is_metadata_only() {
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err("media proof constructed a metadata-only request".into());
    }

    // Diagnostics are opt-in and confined to this fixed public CI fixture.
    let diagnostics = env::args_os().nth(3).is_some_and(|arg| arg == "--diagnostics");
    let mut command = std::process::Command::new(request.executable());
    command.args(request.arguments())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(if diagnostics { std::process::Stdio::piped() } else { std::process::Stdio::null() });
    // Same typed argv and Tor gate as the production launcher; no extra request or fallback.
    let mut running_ytdlp = command.spawn()?;
    let diagnostic_rx = running_ytdlp.stderr.take().map(|stderr| {
        let (tx, rx) = std::sync::mpsc::channel();
        thread::spawn(move || { let _ = tx.send(bounded_diagnostic(stderr)); });
        rx
    });
    let deadline = Instant::now() + YTDLP_TIMEOUT;

    let status = loop {
        if let Some(status) = running_ytdlp.try_wait()? {
            break status;
        }

        if Instant::now() >= deadline {
            let _ = running_ytdlp.kill();
            let _ = running_ytdlp.wait();
            report_diagnostic(diagnostic_rx);
            let _ = running_arti.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            return Err("real yt-dlp media request exceeded its bounded timeout".into());
        }

        thread::sleep(Duration::from_millis(100));
    };

    if !status.success() {
        report_diagnostic(diagnostic_rx);
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err(format!("real yt-dlp media request failed: {status}").into());
    }

    let artifacts = collect_regular_files(&output_root)?;
    let exact = artifacts
        .iter()
        .filter(|(_, size)| *size == EXPECTED_MEDIA_BYTES)
        .count();

    if exact != 1 {
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err(format!(
            "expected exactly one {EXPECTED_MEDIA_BYTES}-byte media artifact, found {exact}; all artifacts: {artifacts:?}"
        )
        .into());
    }

    let _ = running_arti.stop_and_wait()?;
    fs::remove_dir_all(&root)?;

    println!("PULQVA_YTDLP_TOR_MEDIA_OK");
    Ok(())
}

// Drain the pipe continuously, but retain at most 8 KiB in memory. No disk log.
fn bounded_diagnostic(mut input: impl std::io::Read) -> Vec<u8> {
    let mut retained = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                let keep = n.min(8192usize.saturating_sub(retained.len()));
                retained.extend_from_slice(&buffer[..keep]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    retained
}

fn report_diagnostic(rx: Option<std::sync::mpsc::Receiver<Vec<u8>>>) {
    let Some(rx) = rx else { return; };
    match rx.recv_timeout(Duration::from_secs(1)) {
        Ok(bytes) => {
            // Prefix every line and escape control characters; no Actions command injection.
            for line in String::from_utf8_lossy(&bytes).lines() {
                let safe: String = line.chars().flat_map(char::escape_default).collect();
                eprintln!("PULQVA_PUBLIC_FIXTURE_STDERR: {safe}");
            }
        }
        Err(_) => eprintln!("PULQVA_PUBLIC_FIXTURE_STDERR: unavailable within diagnostic deadline"),
    }
}

#[cfg(test)]
mod diagnostic_tests {
    #[test]
    fn diagnostic_capture_is_bounded_but_drains_the_entire_pipe() {
        let mut input = std::io::Cursor::new(vec![b'x'; 65_536]);
        let retained = super::bounded_diagnostic(&mut input);
        assert_eq!(retained, vec![b'x'; 8192]);
        assert_eq!(input.position(), 65_536);
    }
}

fn collect_regular_files(root: &Path) -> Result<Vec<(PathBuf, u64)>, Box<dyn Error>> {
    let mut out = Vec::new();

    if !root.exists() {
        return Ok(out);
    }

    collect_regular_files_inner(root, &mut out)?;
    Ok(out)
}

fn collect_regular_files_inner(
    root: &Path,
    out: &mut Vec<(PathBuf, u64)>,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;

        if metadata.file_type().is_symlink() {
            return Err(format!("unexpected symlink in isolated output root: {}", path.display()).into());
        }

        if metadata.is_dir() {
            collect_regular_files_inner(&path, out)?;
        } else if metadata.is_file() {
            out.push((path, metadata.len()));
        }
    }

    Ok(())
}
