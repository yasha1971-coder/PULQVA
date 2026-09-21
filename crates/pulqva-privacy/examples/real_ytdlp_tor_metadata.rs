use pulqva_privacy::{
    ArtiRuntimePlan, TorReadinessError, TorSocksEndpoint, YtDlpLaunchPlan,
    YtDlpMediaRequestPlan, YtDlpMediaSourceUrl, launch_prepared_arti,
    launch_ytdlp_request, prepare_arti_runtime, verify_tor_readiness,
};
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

const METADATA_SOURCE: &str =
    "https://raw.githubusercontent.com/mediaelement/mediaelement-files/4d21a042353022326071acb0251ab75cd6bae114/big_buck_bunny.mp4";
const TOR_READY_TIMEOUT: Duration = Duration::from_secs(90);
const YTDLP_TIMEOUT: Duration = Duration::from_secs(45);

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
        "pulqva-real-ytdlp-tor-metadata-{}",
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
            println!("PULQVA_YTDLP_TOR_WINDOWS_FAIL_CLOSED_OK");
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
    let source = YtDlpMediaSourceUrl::parse(METADATA_SOURCE)?;
    let request = YtDlpMediaRequestPlan::new_metadata_only(launch, source, &output_root)?;

    if !request.is_metadata_only() {
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err("metadata proof constructed a non-metadata request".into());
    }

    let mut running_ytdlp = launch_ytdlp_request(request)?;
    let deadline = Instant::now() + YTDLP_TIMEOUT;

    let status = loop {
        if let Some(status) = running_ytdlp.try_wait()? {
            break status;
        }

        if Instant::now() >= deadline {
            let _ = running_ytdlp.stop_and_wait();
            let _ = running_arti.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            return Err("metadata-only yt-dlp request exceeded its bounded timeout".into());
        }

        thread::sleep(Duration::from_millis(100));
    };

    if !status.success() {
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err(format!("metadata-only yt-dlp request failed: {status}").into());
    }

    if output_contains_entries(&output_root)? {
        let _ = running_arti.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err("metadata-only yt-dlp request created download output".into());
    }

    let _ = running_arti.stop_and_wait()?;
    fs::remove_dir_all(&root)?;

    println!("PULQVA_YTDLP_TOR_METADATA_OK");
    Ok(())
}

fn output_contains_entries(root: &Path) -> Result<bool, Box<dyn Error>> {
    if !root.exists() {
        return Ok(false);
    }

    Ok(fs::read_dir(root)?.next().transpose()?.is_some())
}
