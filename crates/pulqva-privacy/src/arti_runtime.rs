use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    ArtiConfigRenderError, ArtiConfigSpec, ArtiLaunchSpec, TorSocksEndpoint,
};

/// Pure-data, internally consistent plan for a future Arti runtime.
///
/// The plan owns one canonical set of paths and one SOCKS endpoint, then derives
/// both the launch specification and rendered configuration from those values.
/// It performs no file I/O, process spawning, socket creation, or Tor bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtiRuntimePlan {
    executable: PathBuf,
    config_file: PathBuf,
    cache_dir: PathBuf,
    state_dir: PathBuf,
    socks: TorSocksEndpoint,
}

impl ArtiRuntimePlan {
    pub fn new(
        executable: impl Into<PathBuf>,
        config_file: impl Into<PathBuf>,
        cache_dir: impl Into<PathBuf>,
        state_dir: impl Into<PathBuf>,
        socks: TorSocksEndpoint,
    ) -> Self {
        Self {
            executable: executable.into(),
            config_file: config_file.into(),
            cache_dir: cache_dir.into(),
            state_dir: state_dir.into(),
            socks,
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

    pub fn socks_endpoint(&self) -> TorSocksEndpoint {
        self.socks
    }

    pub fn launch_spec(&self) -> ArtiLaunchSpec {
        ArtiLaunchSpec::new(
            self.executable.clone(),
            self.config_file.clone(),
            self.cache_dir.clone(),
            self.state_dir.clone(),
        )
    }

    pub fn config_spec(&self) -> ArtiConfigSpec {
        ArtiConfigSpec::new(
            self.socks,
            self.cache_dir.clone(),
            self.state_dir.clone(),
        )
    }

    pub fn launch_arguments(&self) -> Vec<OsString> {
        self.launch_spec().arguments()
    }

    pub fn render_config(&self) -> Result<String, ArtiConfigRenderError> {
        self.config_spec().render()
    }

    pub(crate) fn render_bootstrap_config(&self) -> Result<String, ArtiConfigRenderError> {
        self.config_spec().render_for_bootstrap()
    }
}

#[cfg(test)]
mod tests {
    use super::ArtiRuntimePlan;
    use crate::TorSocksEndpoint;
    use std::{ffi::OsString, path::Path};

    fn canonical_plan() -> ArtiRuntimePlan {
        ArtiRuntimePlan::new(
            "runtime/arti",
            "runtime/arti.toml",
            "runtime/cache",
            "runtime/state",
            TorSocksEndpoint::new(19050).expect("canonical SOCKS port is non-zero"),
        )
    }

    #[test]
    fn one_plan_owns_all_runtime_inputs() {
        let plan = canonical_plan();

        assert_eq!(plan.executable(), Path::new("runtime/arti"));
        assert_eq!(plan.config_file(), Path::new("runtime/arti.toml"));
        assert_eq!(plan.cache_dir(), Path::new("runtime/cache"));
        assert_eq!(plan.state_dir(), Path::new("runtime/state"));
        assert_eq!(plan.socks_endpoint().port(), 19050);
    }

    #[test]
    fn launch_and_config_specs_share_the_same_paths() {
        let plan = canonical_plan();
        let launch = plan.launch_spec();
        let config = plan.config_spec();

        assert_eq!(launch.cache_dir(), config.cache_dir());
        assert_eq!(launch.state_dir(), config.state_dir());
        assert_eq!(config.socks_endpoint(), plan.socks_endpoint());
    }

    #[test]
    fn launch_arguments_are_deterministic() {
        let plan = canonical_plan();

        assert_eq!(
            plan.launch_arguments(),
            vec![
                OsString::from("proxy"),
                OsString::from("--config"),
                OsString::from("runtime/arti.toml"),
            ]
        );
    }

    #[test]
    fn canonical_config_is_byte_stable() {
        let plan = canonical_plan();
        let rendered = plan.render_config().expect("canonical paths are UTF-8");
        let fixture = include_str!("../../../sidecars/arti/pulqva.toml");

        assert_eq!(rendered.as_bytes(), fixture.as_bytes());
    }
}
