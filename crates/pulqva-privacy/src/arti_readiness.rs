use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream},
    process::ExitStatus,
    thread,
    time::{Duration, Instant},
};

use crate::{
    ArtiProcessError, ReadyTorTransport, RunningArti,
    arti_ready::certify_tor_ready,
};

const READINESS_HOST: &str = "example.com";
const READINESS_PORT: u16 = 443;
const ATTEMPT_SLICE: Duration = Duration::from_secs(12);
const RETRY_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TorReadinessStage { Listener, Negotiation, Destination }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorReadinessTimeout { pub stage: TorReadinessStage }

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

        match socks_connect_probe(running.endpoint(), attempt_timeout) {
            Ok(verified_loopback) => {
                return Ok(certify_tor_ready(
                    running.endpoint(),
                    verified_loopback,
                ));
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

fn socks_connect_probe(
    endpoint: crate::TorSocksEndpoint,
    timeout: Duration,
) -> Result<IpAddr, SocksProbeError> {
    let addresses = [
        SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::LOCALHOST,
            endpoint.port(),
        )),
        SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::LOCALHOST,
            endpoint.port(),
            0,
            0,
        )),
    ];

    probe_loopbacks(addresses, |local| socks_connect_probe_at(local, timeout))
}

// Keep the pre-diagnostics IPv4/IPv6 attempt order for EVERY retryable failure.
// "Furthest" is only observed protocol progress, not a bootstrap/root-cause claim.
// The private seam lets tests verify action traces without public-network I/O.
fn probe_loopbacks(
    addresses: [SocketAddr; 2],
    mut probe: impl FnMut(SocketAddr) -> Result<(), SocksProbeError>,
) -> Result<IpAddr, SocksProbeError> {
    let mut furthest = TorReadinessStage::Listener;
    for local in addresses {
        match probe(local) {
            Ok(()) => return Ok(local.ip()),
            Err(SocksProbeError::Retryable(stage)) => furthest = furthest.max(stage),
            Err(error) => return Err(error),
        }
    }

    Err(SocksProbeError::Retryable(furthest))
}

fn socks_connect_probe_at(
    local: SocketAddr,
    timeout: Duration,
) -> Result<(), SocksProbeError> {
    let connect_timeout = timeout.min(Duration::from_secs(2));

    let mut stream = match TcpStream::connect_timeout(&local, connect_timeout) {
        Ok(stream) => stream,
        Err(source) if is_retryable_io(&source) => return Err(SocksProbeError::Retryable(TorReadinessStage::Listener)),
        Err(_) => return Err(SocksProbeError::Retryable(TorReadinessStage::Listener)),
    };

    stream
        .set_read_timeout(Some(timeout))
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Negotiation))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Negotiation))?;

    stream
        .write_all(&[0x05, 0x01, 0x00])
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Negotiation))?;

    let mut method = [0_u8; 2];
    stream
        .read_exact(&mut method)
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Negotiation))?;

    if method != [0x05, 0x00] {
        return Err(SocksProbeError::Protocol(
            "Tor SOCKS endpoint rejected no-auth SOCKS5 negotiation",
        ));
    }

    let host = READINESS_HOST.as_bytes();
    let host_len = u8::try_from(host.len()).map_err(|_| {
        SocksProbeError::Protocol("Tor readiness hostname is too long for SOCKS5")
    })?;

    let mut request = Vec::with_capacity(7 + host.len());
    request.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, host_len]);
    request.extend_from_slice(host);
    request.extend_from_slice(&READINESS_PORT.to_be_bytes());

    stream
        .write_all(&request)
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Destination))?;

    let mut response = [0_u8; 4];
    stream
        .read_exact(&mut response)
        .map_err(|_| SocksProbeError::Retryable(TorReadinessStage::Destination))?;

    if response[0] != 0x05 {
        return Err(SocksProbeError::Protocol(
            "Tor SOCKS endpoint returned an invalid SOCKS5 version",
        ));
    }

    if response[1] == 0x00 {
        return Ok(());
    }

    Err(SocksProbeError::Retryable(TorReadinessStage::Destination))
}

fn is_retryable_io(source: &io::Error) -> bool {
    matches!(
        source.kind(),
        io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::TimedOut
            | io::ErrorKind::WouldBlock
            | io::ErrorKind::NotConnected
    )
}

#[derive(Debug, PartialEq, Eq)]
enum SocksProbeError {
    Retryable(TorReadinessStage),
    Protocol(&'static str),
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
            Self::Timeout(timeout) => write!(f, "Tor readiness verification timed out at {:?}", timeout.stage),
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
