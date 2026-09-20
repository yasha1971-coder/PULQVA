use pulqva_privacy::{
    ArtiRuntimePlan, TorSocksEndpoint, launch_prepared_arti, prepare_arti_runtime,
};
use std::{
    fs,
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

fn fixture_executable() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_pulqva-child-fixture"))
}

fn plan(root: &Path) -> ArtiRuntimePlan {
    ArtiRuntimePlan::new(
        fixture_executable(),
        root.join("config/pulqva.toml"),
        root.join("cache"),
        root.join("state"),
        TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
    )
}

#[test]
fn prepared_runtime_launches_direct_child_and_stops_cleanly() {
    let root = test_root("child-launch");
    let prepared = prepare_arti_runtime(plan(&root)).expect("runtime preparation succeeds");
    let marker = prepared.plan().config_file().with_extension("fixture-ran");

    let running = launch_prepared_arti(prepared).expect("direct child launch succeeds");
    assert!(running.id() > 0);

    let deadline = Instant::now() + Duration::from_secs(5);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(25));
    }

    assert_eq!(
        fs::read(&marker).expect("fixture marker proves direct child received deterministic args"),
        b"prepared-only direct child launch"
    );

    let _status = running.stop_and_wait().expect("child shutdown succeeds");

    fs::remove_dir_all(root).expect("test tree cleanup succeeds");
}

#[test]
fn launch_uses_no_shell_and_only_explicit_executable() {
    let root = test_root("explicit-exe");
    let prepared = prepare_arti_runtime(plan(&root)).expect("runtime preparation succeeds");

    assert_eq!(prepared.plan().executable(), fixture_executable());

    let running = launch_prepared_arti(prepared).expect("direct child launch succeeds");
    let _status = running.stop_and_wait().expect("child shutdown succeeds");

    fs::remove_dir_all(root).expect("test tree cleanup succeeds");
}
