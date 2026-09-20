use std::ffi::OsString;

use crate::{
    ArtiConfigMaterializeError, ArtiRuntimePlan, TorSocksEndpoint,
    materialize_arti_config,
};

/// Capability proving that an Arti runtime plan has had its configuration
/// successfully materialized.
///
/// Callers cannot construct this type directly. Future launch code can require
/// this capability rather than accepting an unprepared `ArtiRuntimePlan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedArtiRuntime {
    plan: ArtiRuntimePlan,
}

impl PreparedArtiRuntime {
    pub fn plan(&self) -> &ArtiRuntimePlan {
        &self.plan
    }

    pub fn socks_endpoint(&self) -> TorSocksEndpoint {
        self.plan.socks_endpoint()
    }

    pub fn launch_arguments(&self) -> Vec<OsString> {
        self.plan.launch_arguments()
    }
}

/// Materializes the plan's deterministic configuration and returns a prepared
/// runtime capability only after successful completion.
pub fn prepare_arti_runtime(
    plan: ArtiRuntimePlan,
) -> Result<PreparedArtiRuntime, ArtiConfigMaterializeError> {
    materialize_arti_config(&plan)?;
    Ok(PreparedArtiRuntime { plan })
}

#[cfg(test)]
mod tests {
    use super::prepare_arti_runtime;
    use crate::{ArtiConfigMaterializeError, ArtiRuntimePlan, TorSocksEndpoint};
    use std::{
        ffi::OsString,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_root(label: &str) -> PathBuf {
        let sequence = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pulqva-{label}-{}-{sequence}",
            std::process::id()
        ))
    }

    fn plan(root: &Path) -> ArtiRuntimePlan {
        ArtiRuntimePlan::new(
            root.join("bin/arti"),
            root.join("config/pulqva.toml"),
            root.join("cache"),
            root.join("state"),
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
        )
    }

    #[test]
    fn preparation_materializes_exact_config_before_returning_capability() {
        let root = test_root("prepare");
        let plan = plan(&root);
        let expected = plan.render_config().expect("test paths are UTF-8");

        let prepared = prepare_arti_runtime(plan).expect("preparation succeeds");

        assert_eq!(
            fs::read(prepared.plan().config_file()).expect("config file exists"),
            expected.as_bytes()
        );
        assert_eq!(prepared.socks_endpoint().port(), 19050);
        assert_eq!(
            prepared.launch_arguments(),
            vec![
                OsString::from("proxy"),
                OsString::from("--config"),
                prepared.plan().config_file().as_os_str().to_os_string(),
            ]
        );

        fs::remove_dir_all(root).expect("test tree cleanup succeeds");
    }

    #[test]
    fn failed_materialization_returns_no_prepared_capability() {
        let root = test_root("prepare-fail");
        let plan = ArtiRuntimePlan::new(
            root.join("bin/arti"),
            PathBuf::new(),
            root.join("cache"),
            root.join("state"),
            TorSocksEndpoint::new(19050).expect("test SOCKS port is non-zero"),
        );

        let error = prepare_arti_runtime(plan).expect_err("preparation must fail");
        assert!(matches!(
            error,
            ArtiConfigMaterializeError::MissingConfigFileName
        ));
        assert!(!root.exists());
    }
}
