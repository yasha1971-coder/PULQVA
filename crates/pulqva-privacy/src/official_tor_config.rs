//! Deterministic official Tor configuration. No filesystem, process or network I/O.
use std::{io, path::Path};
use crate::TorSocksEndpoint;

fn path_token(path: &Path) -> io::Result<String> {
    let value = path.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "non-UTF8 Tor path"))?;
    if !path.is_absolute() || value.chars().any(|c| c.is_control() || c == '"') {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsafe Tor path"));
    }
    Ok(format!("\"{}\"", value.replace('\\', "\\\\")))
}

/// Host-owned absolute paths only; no frontend-supplied configuration or URL.
/// Logging uses stdout, never an independently tokenized file path.
pub fn render_official_tor_config(
    data: &Path, geoip: &Path, geoip6: &Path, endpoint: TorSocksEndpoint,
) -> io::Result<String> {
    Ok(format!(
        "DataDirectory {}\nSocksPort 127.0.0.1:{}\nClientOnly 1\nSafeLogging 1\nGeoIPFile {}\nGeoIPv6File {}\nLog notice stdout\n",
        path_token(data)?, endpoint.port(), path_token(geoip)?, path_token(geoip6)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_option_injection_and_relative_resources() {
        let root = std::env::temp_dir();
        for bad in [Path::new("relative"), Path::new("bad\nSocksPort 0.0.0.0:1")] {
            assert!(render_official_tor_config(bad, &root, &root, TorSocksEndpoint::new(19050).unwrap()).is_err());
            assert!(render_official_tor_config(&root, bad, &root, TorSocksEndpoint::new(19050).unwrap()).is_err());
            assert!(render_official_tor_config(&root, &root, bad, TorSocksEndpoint::new(19050).unwrap()).is_err());
        }
    }
    #[test]
    fn does_not_create_file_logging_or_non_loopback_listener() {
        let root = std::env::temp_dir();
        let out = render_official_tor_config(&root, &root, &root, TorSocksEndpoint::new(19050).unwrap()).unwrap();
        assert!(out.contains("SocksPort 127.0.0.1:19050\n"));
        assert!(out.ends_with("Log notice stdout\n"));
        assert!(!out.contains("Log notice file"));
        assert!(out.contains("ClientOnly 1\nSafeLogging 1\n"));
    }
    #[test]
    fn quotes_spaced_unicode_host_paths_without_extra_directives() {
        let root = std::env::temp_dir().join("PULQVA тест with spaces");
        let out = render_official_tor_config(&root, &root, &root, TorSocksEndpoint::new(19050).unwrap()).unwrap();
        assert!(out.contains("PULQVA тест with spaces\"\n"));
        assert_eq!(out.lines().count(), 7);
    }
}
