//! Fixed local CI script only; no external traffic, imports or child creation.
use super::{deno_cache::DenoWorkspace, deno_source::materialize_deno,
    deno_trial_process::{run_observed, Outcome}, deno_zip::DenoTarget};
use std::{fs, path::PathBuf, process::Command, time::Duration};

#[test]
#[ignore = "requires pinned official Deno archive prepared by CI"]
fn pinned_runtime_uses_owned_environment_and_cleans_after_exit() {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => DenoTarget::LinuxX64,
        ("windows", "x86_64") => DenoTarget::WindowsX64,
        _ => panic!("unsupported trial target"),
    };
    let source = fs::canonicalize(PathBuf::from(std::env::var_os("PULQVA_DENO_FIXTURE").expect("archive"))).unwrap();
    let resources = source.parent().unwrap();
    let root = resources.join("rust-runtime-trial"); fs::create_dir(&root).unwrap();
    let root = fs::canonicalize(root).unwrap();
    let mut stage = materialize_deno(resources, &source, &root, target).unwrap();
    let executable = stage.path().to_owned();
    // Windows denies execution of a file still held open for write (ERROR_SHARING_VIOLATION),
    // and Linux reports ETXTBSY. Seal the authenticated stage before spawn while
    // retaining its owned identity/lifetime for cleanup.
    stage.seal_for_execution().unwrap();
    let script = root.join("fixture.js");
    // No permission grants. Query state rather than attempt network/child access.
    // Exit42/43 proves the entire fixed script ran, unlike a generic exit1.
    fs::write(&script, r#"
for (const name of ['net','read','write','env','run','ffi','sys']) {
  const permission = await Deno.permissions.query({name});
  if (permission.state === 'granted') throw new Error('unexpected grant: ' + name);
}
if ([1,2,3].reduce((a,b)=>a+b,0) !== 6) throw new Error('compute');
const code = Number(Deno.args[0]);
if (code !== 42 && code !== 43) throw new Error('fixture exit code');
Deno.exit(code);
"#).unwrap();
    let baseline = fs::read_dir(&root).unwrap().count();
    for expected in [42, 43] {
        let workspace = DenoWorkspace::create(&root).unwrap();
        let mut command = Command::new(&executable);
        command.args(["run", "--ext=js", "--no-code-cache", "--no-prompt", "--no-remote",
            "--no-lock", "--node-modules-dir=none", "--no-config", "--no-npm", "--cached-only"])
            .arg(&script).arg(expected.to_string());
        for key in ["DENO_DIR", "HOME", "TMP", "NODE_OPTIONS", "HTTP_PROXY", "HTTPS_PROXY"] {
            command.env(key, "poison-not-a-path");
        }
        let mut observed = None;
        let result = run_observed(workspace, command, Duration::from_secs(20), &mut |path| {
            for name in ["home", "tmp", "cache"] {
                if !path.join(name).is_dir() { return Err("owned-runtime-directory-missing"); }
            }
            let cache_entries = fs::read_dir(path.join("cache")).map_err(|_| "cache-read")?.count();
            println!("PULQVA_DENO_CACHE_OBSERVED fixture_exit={expected} immediate_entries={cache_entries}");
            observed = Some(path.to_owned());
            Ok(())
        }).unwrap();
        match result {
            Outcome::Exited(status) => assert_eq!(status.code(), Some(expected), "Deno fixture status: {status:?}"),
            Outcome::TimedOut => panic!("Deno fixture timed out"),
        }
        assert!(!observed.expect("post-exit observation").exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), baseline);
        assert!(script.is_file() && executable.is_file());
    }
    drop(stage); assert!(!executable.exists());
    fs::remove_file(script).unwrap(); fs::remove_dir(root).unwrap();
    println!("PULQVA_DENO_RUNTIME_WORKSPACE_OK version=2.9.7 cases=2 cleanup=ok");
}
