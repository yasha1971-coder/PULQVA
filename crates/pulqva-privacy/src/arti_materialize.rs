use std::{
    error::Error,
    ffi::OsString,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{ArtiConfigRenderError, ArtiRuntimePlan};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
const TEMP_CREATE_ATTEMPTS: usize = 16;

/// Materializes the already-validated Arti configuration using a temporary
/// sibling file followed by an atomic rename/replace on the same filesystem.
///
/// This function performs filesystem I/O only. It never spawns Arti, opens a
/// socket, or bootstraps Tor.
pub fn materialize_arti_config(
    plan: &ArtiRuntimePlan,
) -> Result<(), ArtiConfigMaterializeError> {
    let rendered = plan
        .render_config()
        .map_err(ArtiConfigMaterializeError::Render)?;

    let destination = plan.config_file();
    let file_name = destination
        .file_name()
        .ok_or(ArtiConfigMaterializeError::MissingConfigFileName)?;

    let parent = destination.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent).map_err(|source| ArtiConfigMaterializeError::Io {
            operation: "create config parent directory",
            source,
        })?;
    }

    let (temp_path, mut temp_file) = create_temp_sibling(parent, file_name)?;

    let write_result = (|| -> io::Result<()> {
        temp_file.write_all(rendered.as_bytes())?;
        temp_file.flush()?;
        temp_file.sync_all()?;
        Ok(())
    })();

    if let Err(source) = write_result {
        drop(temp_file);
        let _ = fs::remove_file(&temp_path);
        return Err(ArtiConfigMaterializeError::Io {
            operation: "write and sync temporary config",
            source,
        });
    }

    drop(temp_file);

    if let Err(source) = fs::rename(&temp_path, destination) {
        let _ = fs::remove_file(&temp_path);
        return Err(ArtiConfigMaterializeError::Io {
            operation: "atomically replace config",
            source,
        });
    }

    Ok(())
}

fn create_temp_sibling(
    parent: &Path,
    file_name: &std::ffi::OsStr,
) -> Result<(PathBuf, std::fs::File), ArtiConfigMaterializeError> {
    for _ in 0..TEMP_CREATE_ATTEMPTS {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut temp_name = OsString::from(".");
        temp_name.push(file_name);
        temp_name.push(format!(
            ".pulqva-tmp-{}-{sequence}",
            std::process::id()
        ));

        let temp_path = parent.join(temp_name);

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(ArtiConfigMaterializeError::Io {
                    operation: "create temporary config",
                    source,
                });
            }
        }
    }

    Err(ArtiConfigMaterializeError::TemporaryNameExhausted)
}

#[derive(Debug)]
pub enum ArtiConfigMaterializeError {
    Render(ArtiConfigRenderError),
    MissingConfigFileName,
    TemporaryNameExhausted,
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for ArtiConfigMaterializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Render(source) => write!(f, "failed to render Arti config: {source}"),
            Self::MissingConfigFileName => {
                f.write_str("Arti config path must include a file name")
            }
            Self::TemporaryNameExhausted => {
                f.write_str("could not allocate a unique temporary Arti config file")
            }
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for ArtiConfigMaterializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Render(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::MissingConfigFileName | Self::TemporaryNameExhausted => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::materialize_arti_config;
    use crate::{ArtiRuntimePlan, TorSocksEndpoint};
    use std::{
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
    fn materializes_exact_rendered_bytes_and_only_config_parent() {
        let root = test_root("materialize");
        let plan = plan(&root);
        let expected = plan.render_config().expect("test paths are UTF-8");

        materialize_arti_config(&plan).expect("config materialization succeeds");

        assert_eq!(
            fs::read(plan.config_file()).expect("config file exists"),
            expected.as_bytes()
        );
        assert!(plan.config_file().parent().expect("parent exists").is_dir());
        assert!(!plan.cache_dir().exists());
        assert!(!plan.state_dir().exists());
        assert!(!plan.executable().exists());

        fs::remove_dir_all(root).expect("test tree cleanup succeeds");
    }

    #[test]
    fn atomically_replaces_existing_config_without_leaving_temp_files() {
        let root = test_root("replace");
        let plan = plan(&root);
        let parent = plan.config_file().parent().expect("config has parent");
        fs::create_dir_all(parent).expect("parent creation succeeds");
        fs::write(plan.config_file(), b"stale config").expect("seed config write succeeds");

        materialize_arti_config(&plan).expect("replacement succeeds");

        let expected = plan.render_config().expect("test paths are UTF-8");
        assert_eq!(
            fs::read(plan.config_file()).expect("replacement exists"),
            expected.as_bytes()
        );

        let leftovers: Vec<_> = fs::read_dir(parent)
            .expect("config parent is readable")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .filter(|name| name.to_string_lossy().contains(".pulqva-tmp-"))
            .collect();

        assert!(leftovers.is_empty(), "temporary files were left behind");

        fs::remove_dir_all(root).expect("test tree cleanup succeeds");
    }
}
