//! Test-only execution boundary for single-process, no-descendant fixtures.
//! Not an application launcher or a process-tree sandbox.
use super::deno_cache::DenoWorkspace;
use std::{process::{Child, Command, ExitStatus, Stdio}, thread, time::{Duration, Instant}};

#[derive(Debug)]
pub(super) enum Outcome { Exited(ExitStatus), TimedOut }

struct Trial {
    child: Child,
    workspace: Option<DenoWorkspace>,
    reaped: bool,
    stop_attempted: bool,
}
impl Trial {
    fn stop(&mut self) -> bool {
        self.stop_attempted = true;
        let _ = self.child.kill();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => { self.reaped = true; return true; }
                _ if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
                _ => return false,
            }
        }
    }
}
impl Drop for Trial {
    fn drop(&mut self) {
        if !self.reaped && (self.stop_attempted || !self.stop()) {
            // Never delete a workspace whose process termination is unknown.
            // Exceptional retention leaks the directory handles too, intentionally.
            if let Some(workspace) = self.workspace.take() { std::mem::forget(workspace); }
        }
    }
}

pub(super) fn run(workspace: DenoWorkspace, command: Command, budget: Duration)
    -> Result<Outcome, &'static str>
{
    run_with_fixture_output(workspace, command, budget, false)
}

fn run_with_fixture_output(workspace: DenoWorkspace, command: Command,
    budget: Duration, fixture_output: bool) -> Result<Outcome, &'static str>
{
    run_observed_inner(workspace, command, budget, fixture_output, &mut |_| Ok(()))
}

pub(super) fn run_observed(workspace: DenoWorkspace, command: Command, budget: Duration,
    observer: &mut dyn FnMut(&std::path::Path) -> Result<(), &'static str>) -> Result<Outcome, &'static str>
{
    run_observed_inner(workspace, command, budget, false, observer)
}

fn run_observed_inner(workspace: DenoWorkspace, mut command: Command,
    budget: Duration, fixture_output: bool,
    observer: &mut dyn FnMut(&std::path::Path) -> Result<(), &'static str>) -> Result<Outcome, &'static str>
{
    if budget.is_zero() || budget > Duration::from_secs(120) { return Err("trial-budget-invalid"); }
    workspace.apply(&mut command)?;
    let working_directory = command.get_current_dir().ok_or("trial-cwd-missing")?.to_owned();
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    // Only fixed local Rust test fixtures opt in; default trial output stays null.
    // No inherited environment dump, arbitrary program diagnostics or user data.
    if fixture_output { command.stdout(Stdio::inherit()).stderr(Stdio::inherit()); }
    let deadline = Instant::now() + budget;
    let child = command.spawn().map_err(|error| {
        // Test-only numeric diagnostics, without environment or path contents.
        eprintln!("PULQVA_TRIAL_SPAWN_ERROR kind={:?} os_code={:?}", error.kind(), error.raw_os_error());
        "trial-spawn-failed"
    })?;
    let mut trial = Trial { child, workspace: Some(workspace), reaped: false, stop_attempted: false };
    let outcome = loop {
        match trial.child.try_wait() {
            Ok(Some(status)) => { trial.reaped = true; break Outcome::Exited(status); }
            Err(_) => return Err("trial-wait-failed"),
            Ok(None) if Instant::now() >= deadline => {
                if !trial.stop() { return Err("trial-termination-unconfirmed-workspace-retained"); }
                break Outcome::TimedOut;
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
        }
    };
    observer(&working_directory)?;
    trial.workspace.as_mut().unwrap().cleanup()?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf, sync::atomic::{AtomicU64, Ordering}};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    fn parent() -> PathBuf {
        let path = std::env::temp_dir().join(format!("pulqva-trial-{}-{}", std::process::id(), SEQUENCE.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); fs::canonicalize(path).unwrap()
    }
    fn fixture(name: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", name, "--ignored", "--nocapture", "--test-threads=1"]); command
    }
    fn write_owned_data() {
        let cwd = std::env::current_dir().unwrap();
        for (key, child) in [("HOME", "home"), ("TMP", "tmp"), ("DENO_DIR", "cache")] {
            let path = PathBuf::from(std::env::var_os(key).unwrap());
            // Windows current_dir may omit the extended-length prefix retained
            // by canonical workspace paths in the environment. Compare identity,
            // not spelling; both paths must resolve to the intended directory.
            assert!(same_file::is_same_file(&path, cwd.join(child)).unwrap(),
                "fixture directory identity mismatch for {key}");
            assert!(path.is_dir());
            fs::write(path.join("fixture"), b"owned").unwrap();
        }
    }
    #[test]
    #[ignore = "spawned by lifetime tests"]
    fn success_fixture() { write_owned_data(); }
    #[test]
    #[ignore = "spawned by lifetime tests"]
    fn failure_fixture() { write_owned_data(); panic!("intentional fixture failure"); }
    #[test]
    #[ignore = "spawned by lifetime tests"]
    fn timeout_fixture() { write_owned_data(); thread::sleep(Duration::from_secs(30)); }

    #[test]
    fn cleanup_follows_success_nonzero_exit_and_timeout() {
        for (name, expected) in [("success_fixture", 0), ("failure_fixture", 1), ("timeout_fixture", 2)] {
            let parent = parent(); fs::write(parent.join("neighbor"), b"keep").unwrap();
            let workspace = DenoWorkspace::create(&parent).unwrap();
            let command = fixture(&format!("deno_trial_process::tests::{name}"));
            let budget = if expected == 2 { Duration::from_secs(2) } else { Duration::from_secs(15) };
            let outcome = run_with_fixture_output(workspace, command, budget, true)
                .unwrap_or_else(|error| panic!("fixture={name} runner_error={error}"));
            match outcome {
                Outcome::Exited(status) => assert_eq!(if status.success() { 0 } else { 1 }, expected,
                    "fixture={name} raw_status={status:?} code={:?}", status.code()),
                Outcome::TimedOut => assert_eq!(expected, 2),
            }
            assert_eq!(fs::read_dir(&parent).unwrap().count(), 1);
            assert_eq!(fs::read(parent.join("neighbor")).unwrap(), b"keep");
            fs::remove_file(parent.join("neighbor")).unwrap(); fs::remove_dir(parent).unwrap();
        }
    }
    #[test]
    fn spawn_failure_cleans_owned_workspace() {
        let parent = parent(); let workspace = DenoWorkspace::create(&parent).unwrap();
        assert!(matches!(run(workspace, Command::new(parent.join("missing-executable")), Duration::from_secs(1)), Err("trial-spawn-failed")));
        assert_eq!(fs::read_dir(&parent).unwrap().count(), 0); fs::remove_dir(parent).unwrap();
    }
}
