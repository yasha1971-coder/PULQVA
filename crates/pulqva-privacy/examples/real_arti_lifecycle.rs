use pulqva_privacy::{
    ArtiRuntimePlan, TorSocksEndpoint, launch_prepared_arti, prepare_arti_runtime,
};
use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    thread,
    time::Duration,
};

fn main() -> Result<(), Box<dyn Error>> {
    let arti = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("expected path to pinned arti executable")?;

    if !arti.is_file() {
        return Err(format!("Arti executable does not exist: {}", arti.display()).into());
    }

    let root = env::temp_dir().join(format!("pulqva-real-arti-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }

    let plan = ArtiRuntimePlan::new(
        arti,
        root.join("config/pulqva.toml"),
        root.join("cache"),
        root.join("state"),
        TorSocksEndpoint::new(19050)?,
    );

    let prepared = prepare_arti_runtime(plan)?;
    let config = fs::read_to_string(prepared.plan().config_file())?;
    if !config.contains("defer_bootstrap = true") {
        return Err("prepared config does not defer bootstrap".into());
    }

    let mut running = launch_prepared_arti(prepared)?;

    thread::sleep(Duration::from_millis(750));

    if let Some(status) = running.try_wait()? {
        let _ = fs::remove_dir_all(&root);
        return Err(format!("real Arti child exited before controlled shutdown: {status}").into());
    }

    let _ = running.stop_and_wait()?;
    fs::remove_dir_all(&root)?;

    println!("PULQVA_REAL_ARTI_LIFECYCLE_OK");
    Ok(())
}
