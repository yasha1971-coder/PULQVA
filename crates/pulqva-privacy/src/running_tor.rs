use std::{
    error::Error,
    fmt,
    io,
    process::ExitStatus,
    time::Duration,
};

use crate::{
    ArtiProcessError, LittleTorProcessError, LittleTorReadinessError, ReadyTorTransport,
    RunningArti, RunningLittleTor, TorReadinessError, verify_little_tor_readiness,
    verify_tor_readiness,
};

/// Owns exactly one PULQVA-launched Tor engine.
///
/// This type abstracts the engine while preserving ownership of the child process.
/// Network adapters never receive this pre-readiness value.
#[derive(Debug)]
pub enum RunningTorTransport {
    Arti(RunningArti),
    LittleT(RunningLittleTor),
}

impl From<RunningArti> for RunningTorTransport {
    fn from(value: RunningArti) -> Self { Self::Arti(value) }
}

impl From<RunningLittleTor> for RunningTorTransport {
    fn from(value: RunningLittleTor) -> Self { Self::LittleT(value) }
}

impl RunningTorTransport {
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        match self {
            Self::Arti(running) => running.try_wait(),
            Self::LittleT(running) => running.try_wait(),
        }
    }

    pub fn stop_and_wait(self) -> Result<ExitStatus, TorTransportProcessError> {
        match self {
            Self::Arti(running) => running.stop_and_wait().map_err(TorTransportProcessError::Arti),
            Self::LittleT(running) => running.stop_and_wait().map_err(TorTransportProcessError::LittleT),
        }
    }
}

/// Owns the exact child process whose SOCKS endpoint passed readiness.
///
/// There is intentionally no public constructor. The only creation path is
/// establish_ready_running_tor, which binds readiness to this owned process.
#[derive(Debug)]
pub struct ReadyRunningTorTransport {
    running: RunningTorTransport,
    ready: ReadyTorTransport,
}

impl ReadyRunningTorTransport {
    pub fn ready_transport(&self) -> ReadyTorTransport { self.ready }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, io::Error> {
        self.running.try_wait()
    }

    pub fn stop_and_wait(self) -> Result<ExitStatus, TorTransportProcessError> {
        self.running.stop_and_wait()
    }
}

/// Verifies the selected Tor engine and binds the readiness token to that owned child.
/// Failure explicitly stops/reaps the child before returning. No alternate route exists.
pub fn establish_ready_running_tor(
    mut running: RunningTorTransport,
    timeout: Duration,
) -> Result<ReadyRunningTorTransport, TorTransportReadinessError> {
    let ready = match &mut running {
        RunningTorTransport::Arti(arti) => match verify_tor_readiness(arti, timeout) {
            Ok(ready) => ready,
            Err(readiness) => {
                return match running.stop_and_wait() {
                    Ok(_) => Err(TorTransportReadinessError::Arti(readiness)),
                    Err(cleanup) => Err(TorTransportReadinessError::Cleanup {
                        readiness: format!("Arti readiness failed: {readiness}"),
                        cleanup,
                    }),
                };
            }
        },
        RunningTorTransport::LittleT(tor) => match verify_little_tor_readiness(tor, timeout) {
            Ok(ready) => ready,
            Err(readiness) => {
                return match running.stop_and_wait() {
                    Ok(_) => Err(TorTransportReadinessError::LittleT(readiness)),
                    Err(cleanup) => Err(TorTransportReadinessError::Cleanup {
                        readiness: format!("little-t Tor readiness failed: {readiness}"),
                        cleanup,
                    }),
                };
            }
        },
    };

    Ok(ReadyRunningTorTransport { running, ready })
}

#[derive(Debug)]
pub enum TorTransportProcessError {
    Arti(ArtiProcessError),
    LittleT(LittleTorProcessError),
}

impl fmt::Display for TorTransportProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arti(source) => write!(f, "Arti process: {source}"),
            Self::LittleT(source) => write!(f, "little-t Tor process: {source}"),
        }
    }
}

impl Error for TorTransportProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Arti(source) => Some(source),
            Self::LittleT(source) => Some(source),
        }
    }
}

#[derive(Debug)]
pub enum TorTransportReadinessError {
    Arti(TorReadinessError),
    LittleT(LittleTorReadinessError),
    Cleanup {
        readiness: String,
        cleanup: TorTransportProcessError,
    },
}

impl fmt::Display for TorTransportReadinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arti(source) => write!(f, "Arti readiness: {source}"),
            Self::LittleT(source) => write!(f, "little-t Tor readiness: {source}"),
            Self::Cleanup { readiness, cleanup } => write!(f, "{readiness}; cleanup failed: {cleanup}"),
        }
    }
}

impl Error for TorTransportReadinessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Arti(source) => Some(source),
            Self::LittleT(source) => Some(source),
            Self::Cleanup { cleanup, .. } => Some(cleanup),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{LittleTorRuntimePlan, TorSocksEndpoint, launch_little_tor, prepare_little_tor_runtime};
    use std::{path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "pulqva-ready-running-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn failed_little_t_launch_never_creates_ready_running_capability() {
        let root = root("missing");
        let plan = LittleTorRuntimePlan::new(
            root.join("missing-tor"),
            root.join("config/torrc"),
            root.join("data"),
            TorSocksEndpoint::new(19050).unwrap(),
        );
        let prepared = prepare_little_tor_runtime(plan).unwrap();
        assert!(launch_little_tor(prepared).is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
