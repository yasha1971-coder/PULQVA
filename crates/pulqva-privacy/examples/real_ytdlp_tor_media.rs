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
    // One shared budget across all attempts, never renewed by a retry.
    let deadline = Instant::now() + YTDLP_TIMEOUT;
    let mut attempt = 0;
    loop {
        if Instant::now() >= deadline || running_arti.try_wait()?.is_some() {
            let _ = running_arti.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            return Err("media deadline expired or Tor process stopped".into());
        }
        attempt += 1;
        let mut command = std::process::Command::new(request.executable());
        command.args(request.arguments())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());
        // Fixed public CI fixture only: capture for classification, print only if opted in.
        // Every attempt uses exactly the same typed socks5h argv and ready transport.
        let mut running_ytdlp = command.spawn()?;
        let diagnostic_rx = running_ytdlp.stderr.take().map(|stderr| {
            let (tx, rx) = std::sync::mpsc::channel();
            thread::spawn(move || { let _ = tx.send(bounded_diagnostic(stderr)); });
            rx
        });
        let status = loop {
            if let Some(status) = running_ytdlp.try_wait()? { break status; }
            if Instant::now() >= deadline {
                let _ = running_ytdlp.kill();
                let _ = running_ytdlp.wait();
                if diagnostics { report_diagnostic(diagnostic_rx); }
                let _ = running_arti.stop_and_wait();
                let _ = fs::remove_dir_all(&root);
                return Err("real yt-dlp media request exceeded its shared bounded timeout".into());
            }
            thread::sleep(Duration::from_millis(100));
        };
        if status.success() {
            eprintln!("PULQVA_MEDIA_ATTEMPT result=success attempt={attempt}");
            break;
        }
        let captured = diagnostic_rx.and_then(|rx| rx.recv_timeout(Duration::from_secs(1)).ok());
        if diagnostics {
            if let Some(bytes) = &captured { print_diagnostic(bytes); }
        }
        // Retry only the observed pre-download proxy failure, with no output to reuse.
        let empty = collect_regular_files(&output_root)?.is_empty();
        if let Some(delay) = retry_delay(attempt, deadline.saturating_duration_since(Instant::now()),
                                        captured.as_deref(), empty) {
            eprintln!("PULQVA_MEDIA_ATTEMPT result=socks-general-failure attempt={attempt} retry_delay_ms={}", delay.as_millis());
            thread::sleep(delay);
            continue;
        }
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err(format!("real yt-dlp media request failed after {attempt} attempt(s): {status}").into());
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
            print_diagnostic(&bytes);
        }
        Err(_) => eprintln!("PULQVA_PUBLIC_FIXTURE_STDERR: unavailable within diagnostic deadline"),
    }
}

fn print_diagnostic(bytes: &[u8]) {
    for line in String::from_utf8_lossy(bytes).lines() {
        let safe: String = line.chars().flat_map(char::escape_default).collect();
        eprintln!("PULQVA_PUBLIC_FIXTURE_STDERR: {safe}");
    }
}

// Retry is a bounded recovery policy, not a diagnosis that SOCKS REP 1 is transient.
fn retry_delay(attempt: u32, remaining: Duration, diagnostic: Option<&[u8]>, output_empty: bool)
    -> Option<Duration>
{
    if !output_empty || !(1..=2).contains(&attempt) { return None; }
    let text = std::str::from_utf8(diagnostic?).ok()?;
    if !text.contains("Unable to download webpage")
        || !text.contains("Socks5Error(1, 'general SOCKS server failure')") {
        return None;
    }
    let delay = Duration::from_secs(2u64.pow(attempt));
    // Preserve time for a meaningful new request; all processing still shares the deadline.
    (remaining > delay + Duration::from_secs(5)).then_some(delay)
}

#[cfg(test)]
mod diagnostic_tests {
    const SOCKS: &[u8] = b"Unable to download webpage: Socks5Error(1, 'general SOCKS server failure')";
    #[test]
    fn socks_recovery_is_capped_and_uses_one_budget() {
        use std::time::Duration;
        assert_eq!(super::retry_delay(1, Duration::from_secs(100), Some(SOCKS), true), Some(Duration::from_secs(2)));
        assert_eq!(super::retry_delay(2, Duration::from_secs(100), Some(SOCKS), true), Some(Duration::from_secs(4)));
        for attempt in [0, 3, 4, u32::MAX] {
            assert_eq!(super::retry_delay(attempt, Duration::from_secs(180), Some(SOCKS), true), None);
        }
        assert_eq!(super::retry_delay(1, Duration::from_secs(7), Some(SOCKS), true), None);
        assert_eq!(super::retry_delay(2, Duration::ZERO, Some(SOCKS), true), None);
    }
    #[test]
    fn permanent_unknown_or_partial_output_failures_do_not_retry() {
        use std::time::Duration;
        for bytes in [None, Some(&b"HTTP Error 403"[..]), Some(&b"Socks5Error(2, ruleset denied)"[..]), Some(&b"bad output"[..])] {
            assert_eq!(super::retry_delay(1, Duration::from_secs(180), bytes, true), None);
        }
        assert_eq!(super::retry_delay(1, Duration::from_secs(180), Some(SOCKS), false), None);
    }
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
