//! Prepared official Tor runtime; package authentication is the host materializer's gate.
use std::{fs::{self, OpenOptions}, io::{self, Write}, path::{Path, PathBuf}};
use crate::{TorSocksEndpoint, render_official_tor_config};

#[derive(Debug)]
pub struct PreparedOfficialTorRuntime {
    pub(crate) executable: PathBuf,
    pub(crate) config: PathBuf,
    pub(crate) endpoint: TorSocksEndpoint,
}

/// Materialize only a new host-owned config. This capability proves config
/// preparation, not publisher authentication: callers must first authenticate
/// all packaged resources in the same way as the existing Arti materializer.
pub fn prepare_official_tor_runtime(
    executable: &Path, config: &Path, data: &Path, geoip: &Path, geoip6: &Path,
    endpoint: TorSocksEndpoint,
) -> io::Result<PreparedOfficialTorRuntime> {
    let bytes = render_official_tor_config(data, geoip, geoip6, endpoint)?;
    for path in [executable, geoip, geoip6] {
        if !path.is_absolute() || !fs::symlink_metadata(path)?.is_file() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid verified Tor resource"));
        }
    }
    if !config.is_absolute() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "relative Tor config"));
    }
    fs::create_dir_all(data)?;
    let mut output = OpenOptions::new().write(true).create_new(true).open(config)?;
    output.write_all(bytes.as_bytes())?;
    output.sync_all()?;
    Ok(PreparedOfficialTorRuntime {
        executable: executable.to_path_buf(), config: config.to_path_buf(), endpoint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_resource_cannot_yield_prepared_capability_or_config() {
        let root = std::env::temp_dir().join(format!("pulqva-official-missing-{}",std::process::id()));
        let result = prepare_official_tor_runtime(&root.join("absent.exe"), &root.join("torrc"),
            &root.join("data"), &root.join("geoip"), &root.join("geoip6"), TorSocksEndpoint::new(19050).unwrap());
        assert!(result.is_err());
        assert!(!root.exists());
    }
    #[test]
    fn existing_config_is_not_overwritten() {
        let root = std::env::temp_dir().join(format!("pulqva-official-new-{}",std::process::id()));
        fs::create_dir(&root).unwrap();
        let fixture = root.join("resource");
        fs::write(&fixture,b"fixture, not executed").unwrap();
        let config = root.join("torrc");
        fs::write(&config,b"preserved").unwrap();
        let result = prepare_official_tor_runtime(&fixture,&config,&root.join("data"),&fixture,&fixture,TorSocksEndpoint::new(19050).unwrap());
        assert!(result.is_err());
        assert_eq!(fs::read(&config).unwrap(), b"preserved");
        fs::remove_dir_all(root).unwrap();
    }
}
