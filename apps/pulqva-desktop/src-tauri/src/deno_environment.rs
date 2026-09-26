//! Environment policy only. Callers must supply owned directories and retain
//! their lifetime through child exit. This does not authorize a runtime launch.
use std::{path::{Component, Path, PathBuf}, process::Command};

#[derive(Debug)]
pub(crate) struct DenoEnvironment {
    root: PathBuf,
    system_root: Option<PathBuf>,
}

fn absolute_normal(path: &Path) -> bool {
    path.is_absolute() && !path.components().any(|c| matches!(c, Component::ParentDir | Component::CurDir))
}

impl DenoEnvironment {
    // system_root must come from the trusted native platform boundary, never
    // from an arbitrary inherited environment map. Ownership is a separate gate.
    pub(crate) fn new(root: PathBuf, system_root: Option<PathBuf>) -> Result<Self, &'static str> {
        if !absolute_normal(&root) || system_root.as_ref().is_some_and(|p| !absolute_normal(p)) {
            return Err("environment paths must be absolute and normal");
        }
        if cfg!(windows) != system_root.is_some() {
            return Err("native system directory required only on Windows");
        }
        Ok(Self { root, system_root })
    }

    pub(crate) fn apply(&self, command: &mut Command) {
        command.env_clear();
        command.current_dir(&self.root);
        for key in ["HOME", "USERPROFILE"] {
            command.env(key, self.root.join("home"));
        }
        for key in ["TMP", "TEMP", "TMPDIR"] {
            command.env(key, self.root.join("tmp"));
        }
        command.env("DENO_DIR", self.root.join("cache"));
        for key in ["DENO_NO_UPDATE_CHECK", "DENO_NO_PROMPT", "NO_COLOR"] {
            command.env(key, "1");
        }
        if let Some(root) = &self.system_root {
            command.env("SystemRoot", root).env("WINDIR", root);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeMap, ffi::OsString};

    fn policy() -> DenoEnvironment {
        DenoEnvironment::new(std::env::current_dir().unwrap().join("owned-fixture"),
            if cfg!(windows) { Some(PathBuf::from(r"C:\Windows")) } else { None }).unwrap()
    }

    #[test]
    fn poisoned_command_environment_is_replaced_by_exact_policy() {
        let policy = policy();
        let mut command = Command::new("not-executed");
        for key in ["PATH", "LD_PRELOAD", "PYTHONPATH", "NODE_OPTIONS", "DENO_FLAGS",
            "DENO_DIR", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY", "SSL_CERT_FILE"] {
            command.env(key, "poison");
        }
        policy.apply(&mut command);
        let actual: BTreeMap<OsString, OsString> = command.get_envs()
            .map(|(k,v)| (k.into(), v.expect("no removals after clear").into())).collect();
        let mut expected = BTreeMap::new();
        for (key, child) in [("HOME", "home"), ("USERPROFILE", "home"), ("TMP", "tmp"),
            ("TEMP", "tmp"), ("TMPDIR", "tmp"), ("DENO_DIR", "cache")] {
            expected.insert(OsString::from(key), policy.root.join(child).into_os_string());
        }
        for key in ["DENO_NO_UPDATE_CHECK", "DENO_NO_PROMPT", "NO_COLOR"] {
            expected.insert(key.into(), "1".into());
        }
        if let Some(root) = &policy.system_root {
            for key in ["SystemRoot", "WINDIR"] { expected.insert(key.into(), root.as_os_str().into()); }
        }
        assert_eq!(actual, expected);
        assert_eq!(command.get_current_dir(), Some(policy.root.as_path()));
        // Reapplication must not retain additions made between applications.
        command.env("DENO_FLAGS", "poison");
        policy.apply(&mut command);
        assert!(!command.get_envs().any(|(key,_)| key == "DENO_FLAGS"));
    }

    #[test]
    fn rejects_relative_traversing_and_wrong_platform_paths() {
        assert!(DenoEnvironment::new("relative".into(), None).is_err());
        let root = std::env::current_dir().unwrap();
        assert!(DenoEnvironment::new(root.join(".."), None).is_err());
        let wrong = if cfg!(windows) { None } else { Some(root.clone()) };
        assert!(DenoEnvironment::new(root, wrong).is_err());
    }

    const POISON_KEYS: &[&str] = &[
        "PULQVA_ENV_POISON", "DENO_FLAGS", "DENO_DIR", "NODE_OPTIONS",
        "PYTHONPATH", "HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY",
    ];

    fn helper(name: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args(["--exact", name, "--ignored", "--nocapture", "--test-threads=1"]);
        command
    }

    fn bounded_success(mut command: Command) {
        use std::time::{Duration, Instant};
        let mut child = command.spawn().expect("spawn environment fixture");
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            match child.try_wait() {
                Ok(Some(status)) => { assert!(status.success(), "environment fixture failed"); return; }
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                other => {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("environment fixture timeout/wait failure: {other:?}");
                }
            }
        }
    }

    #[test]
    fn native_child_does_not_inherit_poisoned_parent_environment() {
        // Poison a separate process, never the multithreaded test runner.
        let mut command = helper("deno_environment::tests::poisoned_parent_fixture");
        for key in POISON_KEYS { command.env(key, "pulqva-test-poison"); }
        bounded_success(command);
    }

    #[test]
    #[ignore = "invoked only by native_child_does_not_inherit_poisoned_parent_environment"]
    fn poisoned_parent_fixture() {
        for key in POISON_KEYS {
            assert_eq!(std::env::var(key).unwrap(), "pulqva-test-poison");
        }
        // CI fixture input only: production native-directory discovery remains
        // a separate gate. The target root already exists and is not modified.
        let system_root = if cfg!(windows) {
            Some(PathBuf::from(std::env::var_os("SystemRoot").expect("Windows CI SystemRoot")))
        } else { None };
        let policy = DenoEnvironment::new(std::env::current_dir().unwrap(), system_root).unwrap();
        let mut command = helper("deno_environment::tests::clean_child_fixture");
        policy.apply(&mut command);
        bounded_success(command);
    }

    #[test]
    #[ignore = "invoked only by poisoned_parent_fixture"]
    fn clean_child_fixture() {
        let root = std::env::current_dir().unwrap();
        for key in POISON_KEYS.iter().filter(|key| **key != "DENO_DIR") {
            assert!(std::env::var_os(key).is_none(), "unexpected inherited key: {key}");
        }
        for (key, child) in [("DENO_DIR", "cache"), ("HOME", "home"),
            ("USERPROFILE", "home"), ("TMP", "tmp"), ("TEMP", "tmp"), ("TMPDIR", "tmp")] {
            assert_eq!(std::env::var_os(key).unwrap(), root.join(child).into_os_string());
        }
        for key in ["DENO_NO_UPDATE_CHECK", "DENO_NO_PROMPT", "NO_COLOR"] {
            assert_eq!(std::env::var(key).unwrap(), "1");
        }
        let allowed = ["HOME", "USERPROFILE", "TMP", "TEMP", "TMPDIR", "DENO_DIR",
            "DENO_NO_UPDATE_CHECK", "DENO_NO_PROMPT", "NO_COLOR", "SYSTEMROOT", "WINDIR"];
        for (key, _) in std::env::vars_os() {
            let key = key.to_string_lossy().to_ascii_uppercase();
            assert!(allowed.contains(&key.as_str()), "unexpected environment key: {key}");
        }
        if cfg!(windows) {
            assert_eq!(std::env::var_os("SystemRoot"), std::env::var_os("WINDIR"));
            assert!(std::env::var_os("SystemRoot").is_some());
        }
        println!("PULQVA_DENO_CHILD_ENVIRONMENT_OK");
    }
}
