use pulqva_privacy::{
    ArtiRuntimePlan, TorSocksEndpoint, launch_prepared_arti, prepare_arti_runtime,
    verify_tor_readiness,
};
use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
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

    let root = env::temp_dir().join(format!("pulqva-real-arti-ready-{}", std::process::id()));
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
    let mut running = launch_prepared_arti(prepared)?;

    let ready = match verify_tor_readiness(&mut running, Duration::from_secs(150)) {
        Ok(ready) => ready,
        Err(error) => {
            let _ = running.stop_and_wait();
            let _ = fs::remove_dir_all(&root);
            return Err(Box::new(error));
        }
    };

    if ready.proxy_url() != "socks5h://127.0.0.1:19050" {
        let _ = running.stop_and_wait();
        let _ = fs::remove_dir_all(&root);
        return Err("ready transport exposed an unexpected proxy URL".into());
    }

    let _ = running.stop_and_wait()?;
    fs::remove_dir_all(&root)?;

    println!("PULQVA_TOR_READY_OK");
    Ok(())
}
