use std::{
    error::Error,
    ffi::OsString,
    fmt,
    fs,
    io,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::TorSocksEndpoint;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Owned little-t Tor client process.
///
/// The child is spawned directly without a shell. It is configured as a client
/// with one loopback SOCKS listener, an app/test-owned data directory, safe
/// logging and no direct-network fallback path in this API.
#[derive(Debug)]
pub struct RunningLittleTor {
    child: Child,
    endpoint: TorSocksEndpoint,
}

impl RunningLittleTor {
    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn endpoint(&self) -> TorSocksEndpoint {
        self.endpoint
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.child.try_wait()
    }

    pub fn stop_and_wait(mut self) -> Result<ExitStatus, LittleTorProcessError> {
        self.stop_child()
    }

    fn stop_child(&mut self) -> Result<ExitStatus, LittleTorProcessError> {
        if let Some(status) = self.child.try_wait().map_err(|source| LittleTorProcessError::Io {
            operation: "query Tor child status",
            source,
        })? {
            return Ok(status);
        }

        self.child.kill().map_err(|source| LittleTorProcessError::Io {
            operation: "terminate Tor child",
            source,
        })?;

        self.child.wait().map_err(|source| LittleTorProcessError::Io {
            operation: "wait for Tor child",
            source,
        })
    }
}

/// Launch the already verified little-t Tor executable.
///
/// This function does not download Tor and does not invoke a shell. The caller
/// supplies the verified executable path and owned runtime directories.
pub fn launch_little_tor(
    executable: impl Into<PathBuf>,
    data_dir: impl Into<PathBuf>,
    log_file: impl Into<PathBuf>,
    endpoint: TorSocksEndpoint,
) -> Result<RunningLittleTor, LittleTorProcessError> {
    let executable = executable.into();
    let data_dir = data_dir.into();
    let log_file = log_file.into();

    validate_path("Tor executable", &executable)?;
    validate_path("Tor data directory", &data_dir)?;
    validate_path("Tor log file", &log_file)?;

    fs::create_dir_all(&data_dir).map_err(|source| LittleTorProcessError::Io {
        operation: "create Tor data directory",
        source,
    })?;
    if let Some(parent) = log_file.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| LittleTorProcessError::Io {
                operation: "create Tor log directory",
                source,
            })?;
        }
    }

    let socks = format!("127.0.0.1:{}", endpoint.port());
    let mut log_spec = OsString::from("notice file ");
    log_spec.push(log_file.as_os_str());

    let args = vec![
        OsString::from("--ClientOnly"),
        OsString::from("1"),
        OsString::from("--SocksPort"),
        OsString::from(socks),
        OsString::from("--DataDirectory"),
        data_dir.as_os_str().to_os_string(),
        OsString::from("--AvoidDiskWrites"),
        OsString::from("1"),
        OsString::from("--SafeLogging"),
        OsString::from("1"),
        OsString::from("--Log"),
        log_spec,
    ];

    let mut command = Command::new(&executable);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let child = command.spawn().map_err(|source| LittleTorProcessError::Io {
        operation: "spawn verified Tor child",
        source,
    })?;

    Ok(RunningLittleTor { child, endpoint })
}

fn validate_path(name: &'static str, path: &Path) -> Result<(), LittleTorProcessError> {
    if path.as_os_str().is_empty() {
        return Err(LittleTorProcessError::InvalidPath(name));
    }
    Ok(())
}

#[derive(Debug)]
pub enum LittleTorProcessError {
    InvalidPath(&'static str),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for LittleTorProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(name) => write!(f, "{name} path must not be empty"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for LittleTorProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidPath(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_path;
    use std::path::Path;

    #[test]
    fn empty_runtime_paths_are_rejected() {
        assert!(validate_path("fixture", Path::new("")).is_err());
    }

    #[test]
    fn nonempty_runtime_paths_are_accepted() {
        assert!(validate_path("fixture", Path::new("runtime/tor")).is_ok());
    }
}
