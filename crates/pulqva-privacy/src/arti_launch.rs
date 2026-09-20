use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// Pure-data launch specification for the pinned Arti sidecar.
///
/// This type never starts a process and never opens a socket. It only carries
/// the data a later runtime launcher will need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtiLaunchSpec {
    executable: PathBuf,
    config_file: PathBuf,
    cache_dir: PathBuf,
    state_dir: PathBuf,
}

impl ArtiLaunchSpec {
    pub fn new(
        executable: impl Into<PathBuf>,
        config_file: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            executable: executable.into(),
            config_file: config_file.into(),
            cache_dir: cache_dir.into(),
            state_dir: state_dir.into(),
        }
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Deterministic Arti CLI arguments for proxy mode.
    ///
    /// Storage directories are explicit fields on the spec and will be
    /// rendered into configuration by a later task. T009 does not write files.
    pub fn arguments(&self) -> Vec<OsString> {
        vec![
            OsString::from("proxy"),
            OsString::from("--config"),
            self.config_file.as_os_str().to_os_string(),
        ]
    }

    pub fn subcommand(&self) -> &OsStr {
        OsStr::new("proxy")
    }
}

#[cfg(test)]
mod tests {
    use super::ArtiLaunchSpec;
    use std::{ffi::OsString, path::Path};

    fn spec() -> ArtiLaunchSpec {
        ArtiLaunchSpec::new(
            "runtime/arti",
            "runtime/arti.toml",
            "runtime/cache",
            "runtime/state",
        )
    }

    #[test]
    fn keeps_all_runtime_paths_explicit() {
        let spec = spec();

        assert_eq!(spec.executable(), Path::new("runtime/arti"));
        assert_eq!(spec.config_file(), Path::new("runtime/arti.toml"));
        assert_eq!(spec.cache_dir(), Path::new("runtime/cache"));
        assert_eq!(spec.state_dir(), Path::new("runtime/state"));
    }

    #[test]
    fn constructs_deterministic_proxy_arguments() {
        let spec = spec();

        assert_eq!(
            spec.arguments(),
            vec![
                OsString::from("proxy"),
                OsString::from("--config"),
                OsString::from("runtime/arti.toml"),
            ]
        );
    }

    #[test]
    fn exposes_proxy_subcommand_without_execution() {
        let spec = spec();
        assert_eq!(spec.subcommand(), "proxy");
    }
}
