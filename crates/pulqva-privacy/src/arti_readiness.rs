use std::{
    error::Error,
    fmt,
    io,
    process::ExitStatus,
    thread,
    time::{Duration, Instant},
};

use crate::{
    ArtiProcessError, ReadyTorTransport, RunningArti,
    arti_ready::certify_tor_ready,
    tor_socks_probe::{
        ATTEMPT_SLICE, RETRY_DELAY, SocksProbeError, TorReadinessStage,
        TorReadinessTimeout, socks_connect_probe,
    },
};

const READINESS_HOST: &str = "example.com";
const READINESS_PORT: u16 = 443;

/// Explicitly activates bootstrap for a previously deferred Arti child, then
/// verifies that the Tor transport can establish an outbound TCP connection
/// through its local SOCKS endpoint within an explicit deadline.
///
/// The verifier itself opens only a loopback TCP connection. The external
/// destination is encoded as a SOCKS5 domain-name request, so hostname
/// resolution and the outbound connection stay on the Tor side. There is no
/// direct-network fallback.
pub fn verify_tor_readiness(
    running: &mut RunningArti,
    timeout: Duration,
) -> Result<ReadyTorTransport, TorReadinessError> {
    let mut furthest = TorReadinessStage::Listener;
    if timeout.is_zero() {
        return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
    }

    running
        .activate_bootstrap()
        .map_err(TorReadinessError::BootstrapActivation)?;

    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }))?;

    loop {
        if let Some(status) = running
            .try_wait()
            .map_err(|source| TorReadinessError::Io {
                operation: "query Arti child status",
                source,
            })?
        {
            return Err(TorReadinessError::ChildExited(status));
        }

        let now = Instant::now();
        if now >= deadline {
            return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
        }

        let remaining = deadline.saturating_duration_since(now);
        let attempt_timeout = remaining.min(ATTEMPT_SLICE);

        match socks_connect_probe(
            running.endpoint(),
            READINESS_HOST,
            READINESS_PORT,
            attempt_timeout,
        ) {
            Ok(verified_loopback) => {
                return Ok(certify_tor_ready(running.endpoint(), verified_loopback));
            }
            Err(SocksProbeError::Retryable(stage)) => {
                furthest = furthest.max(stage);
                let sleep_for = deadline
                    .saturating_duration_since(Instant::now())
                    .min(RETRY_DELAY);
                if sleep_for.is_zero() {
                    return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
                }
                thread::sleep(sleep_for);
            }
            Err(SocksProbeError::Protocol(message)) => {
                return Err(TorReadinessError::Protocol(message));
            }
        }
    }
}

#[derive(Debug)]
pub enum TorReadinessError {
    Timeout(TorReadinessTimeout),
    BootstrapActivation(ArtiProcessError),
    ChildExited(ExitStatus),
    Protocol(&'static str),
    Io {
        operation: &'static str,
        source: io::Error,
    },
}

impl fmt::Display for TorReadinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(timeout) => {
                write!(f, "Tor readiness verification timed out at {:?}", timeout.stage)
            }
            Self::BootstrapActivation(source) => {
                write!(f, "failed to activate Tor bootstrap: {source}")
            }
            Self::ChildExited(status) => {
                write!(f, "Arti child exited before Tor became ready: {status}")
            }
            Self::Protocol(message) => write!(f, "Tor SOCKS readiness protocol error: {message}"),
            Self::Io { operation, source } => write!(f, "{operation}: {source}"),
        }
    }
}

impl Error for TorReadinessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::BootstrapActivation(source) => Some(source),
            Self::Io { source, .. } => Some(source),
            Self::Timeout(_) | Self::ChildExited(_) | Self::Protocol(_) => None,
        }
    }
}

#[cfg(test)]
mod tests;
