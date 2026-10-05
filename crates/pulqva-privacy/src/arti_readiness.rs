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


const MAX_READINESS_OBSERVATIONS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorReadinessLoopbackFamily { Ipv4, Ipv6 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorReadinessOperation {
    ConnectLoopback,
    ConfigureReadTimeout,
    ConfigureWriteTimeout,
    WriteNegotiation,
    ReadNegotiation,
    ValidateNegotiation,
    WriteDestination,
    ReadDestination,
    ClassifyReply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorReadinessIoKind {
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    TimedOut,
    WouldBlock,
    NotConnected,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorSocksReplyClass {
    Succeeded,
    GeneralFailure,
    RulesetDenied,
    NetworkUnreachable,
    HostUnreachable,
    ConnectionRefused,
    TtlExpired,
    CommandUnsupported,
    AddressTypeUnsupported,
    Unassigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorReadinessObservation {
    family: TorReadinessLoopbackFamily,
    stage: TorReadinessStage,
    operation: TorReadinessOperation,
    io_kind: Option<TorReadinessIoKind>,
    socks_reply: Option<u8>,
    socks_reply_class: Option<TorSocksReplyClass>,
    duration_ms: u64,
}

impl TorReadinessObservation {
    pub fn family(self) -> TorReadinessLoopbackFamily { self.family }
    pub fn stage(self) -> TorReadinessStage { self.stage }
    pub fn operation(self) -> TorReadinessOperation { self.operation }
    pub fn io_kind(self) -> Option<TorReadinessIoKind> { self.io_kind }
    pub fn socks_reply(self) -> Option<u8> { self.socks_reply }
    pub fn socks_reply_class(self) -> Option<TorSocksReplyClass> { self.socks_reply_class }
    pub fn duration_ms(self) -> u64 { self.duration_ms }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TorReadinessTrace {
    observations: Vec<TorReadinessObservation>,
    omitted: u64,
}

impl TorReadinessTrace {
    pub fn observations(&self) -> &[TorReadinessObservation] { &self.observations }
    pub fn omitted(&self) -> u64 { self.omitted }

    fn push(&mut self, observation: TorReadinessObservation) {
        if self.observations.len() < MAX_READINESS_OBSERVATIONS {
            self.observations.push(observation);
        } else {
            self.omitted = self.omitted.saturating_add(1);
        }
    }
}

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
    let mut trace = None;
    verify_tor_readiness_inner(running, timeout, &mut trace)
}

/// Runs the exact same readiness verifier while retaining a bounded,
/// privacy-safe observation trace. The trace is diagnostic data only:
/// it does not grant readiness, alter retry policy, or create a fallback.
pub fn verify_tor_readiness_observed(
    running: &mut RunningArti,
    timeout: Duration,
) -> (Result<ReadyTorTransport, TorReadinessError>, TorReadinessTrace) {
    let mut owned = TorReadinessTrace::default();
    let result = {
        let mut trace = Some(&mut owned);
        verify_tor_readiness_inner(running, timeout, &mut trace)
    };
    (result, owned)
}

fn verify_tor_readiness_inner(
    running: &mut RunningArti,
    timeout: Duration,
    trace: &mut Option<&mut TorReadinessTrace>,
) -> Result<ReadyTorTransport, TorReadinessError> {
    let mut furthest = TorReadinessStage::Listener;
    if timeout.is_zero() {
        return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
    }
    running.activate_bootstrap().map_err(TorReadinessError::BootstrapActivation)?;
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }))?;
    loop {
        if let Some(status) = running.try_wait().map_err(|source| TorReadinessError::Io {
            operation: "query Arti child status", source,
        })? {
            return Err(TorReadinessError::ChildExited(status));
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
        }
        let remaining = deadline.saturating_duration_since(now);
        let attempt_timeout = remaining.min(ATTEMPT_SLICE);
        match socks_connect_probe_with_trace(running.endpoint(), attempt_timeout, trace) {
            Ok(verified_loopback) => {
                return Ok(certify_tor_ready(running.endpoint(), verified_loopback));
            }
            Err(SocksProbeError::Retryable(stage)) => {
                furthest = furthest.max(stage);
                let sleep_for = deadline.saturating_duration_since(Instant::now()).min(RETRY_DELAY);
                if sleep_for.is_zero() {
                    return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
                }
                thread::sleep(sleep_for);
            }
            Err(SocksProbeError::Protocol(message)) => return Err(TorReadinessError::Protocol(message)),
        }
    }
}

/// Verifies an already-running Tor SOCKS endpoint without manufacturing a
/// readiness capability from a raw port.
///
/// This path is for an externally packaged/owned Tor implementation (for
/// example the official Tor Expert Bundle on native Windows). It performs the
/// same SOCKS5 remote-domain destination probe as the Arti path. No direct
/// network route is introduced: the verifier itself connects only to loopback.
pub fn verify_existing_tor_readiness(
    endpoint: crate::TorSocksEndpoint,
    timeout: Duration,
) -> Result<ReadyTorTransport, TorReadinessError> {
    let mut furthest = TorReadinessStage::Listener;
    if timeout.is_zero() {
        return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
    }
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }))?;
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
        }
        let remaining = deadline.saturating_duration_since(now);
        let attempt_timeout = remaining.min(ATTEMPT_SLICE);
        match socks_connect_probe(endpoint, attempt_timeout) {
            Ok(verified_loopback) => return Ok(certify_tor_ready(endpoint, verified_loopback)),
            Err(SocksProbeError::Retryable(stage)) => {
                furthest = furthest.max(stage);
                let sleep_for = deadline.saturating_duration_since(Instant::now()).min(RETRY_DELAY);
                if sleep_for.is_zero() {
                    return Err(TorReadinessError::Timeout(TorReadinessTimeout { stage: furthest }));
                }
                thread::sleep(sleep_for);
            }
            Err(SocksProbeError::Protocol(message)) => return Err(TorReadinessError::Protocol(message)),
        }
    }
}

fn socks_connect_probe(
    endpoint: crate::TorSocksEndpoint,
    timeout: Duration,
) -> Result<IpAddr, SocksProbeError> {
    let mut trace = None;
    socks_connect_probe_with_trace(endpoint, timeout, &mut trace)
}

fn socks_connect_probe_with_trace(
    endpoint: crate::TorSocksEndpoint,
    timeout: Duration,
    trace: &mut Option<&mut TorReadinessTrace>,
) -> Result<IpAddr, SocksProbeError> {
    let addresses = [
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, endpoint.port())),
        SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, endpoint.port(), 0, 0)),
    ];
    probe_loopbacks(addresses, |local| socks_connect_probe_at(local, timeout, trace))
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
    trace: &mut Option<&mut TorReadinessTrace>,
) -> Result<(), SocksProbeError> {
    let family = if local.is_ipv4() { TorReadinessLoopbackFamily::Ipv4 } else { TorReadinessLoopbackFamily::Ipv6 };
    let connect_timeout = timeout.min(Duration::from_secs(2));

    let started = Instant::now();
    let mut stream = match TcpStream::connect_timeout(&local, connect_timeout) {
        Ok(stream) => {
            record_observation(trace, family, TorReadinessStage::Listener,
                TorReadinessOperation::ConnectLoopback, None, None, started);
            stream
        }
        Err(source) => {
            record_observation(trace, family, TorReadinessStage::Listener,
                TorReadinessOperation::ConnectLoopback, Some(&source), None, started);
            return Err(SocksProbeError::Retryable(TorReadinessStage::Listener));
        }
    };

    let started = Instant::now();
    if let Err(source) = stream.set_read_timeout(Some(timeout)) {
        record_observation(trace, family, TorReadinessStage::Negotiation,
            TorReadinessOperation::ConfigureReadTimeout, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Negotiation));
    }
    record_observation(trace, family, TorReadinessStage::Negotiation,
        TorReadinessOperation::ConfigureReadTimeout, None, None, started);

    let started = Instant::now();
    if let Err(source) = stream.set_write_timeout(Some(timeout)) {
        record_observation(trace, family, TorReadinessStage::Negotiation,
            TorReadinessOperation::ConfigureWriteTimeout, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Negotiation));
    }
    record_observation(trace, family, TorReadinessStage::Negotiation,
        TorReadinessOperation::ConfigureWriteTimeout, None, None, started);

    let started = Instant::now();
    if let Err(source) = stream.write_all(&[0x05, 0x01, 0x00]) {
        record_observation(trace, family, TorReadinessStage::Negotiation,
            TorReadinessOperation::WriteNegotiation, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Negotiation));
    }
    record_observation(trace, family, TorReadinessStage::Negotiation,
        TorReadinessOperation::WriteNegotiation, None, None, started);

    let mut method = [0_u8; 2];
    let started = Instant::now();
    if let Err(source) = stream.read_exact(&mut method) {
        record_observation(trace, family, TorReadinessStage::Negotiation,
            TorReadinessOperation::ReadNegotiation, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Negotiation));
    }
    record_observation(trace, family, TorReadinessStage::Negotiation,
        TorReadinessOperation::ReadNegotiation, None, None, started);

    let started = Instant::now();
    record_observation(trace, family, TorReadinessStage::Negotiation,
        TorReadinessOperation::ValidateNegotiation, None, None, started);
    if method != [0x05, 0x00] {
        return Err(SocksProbeError::Protocol("Tor SOCKS endpoint rejected no-auth SOCKS5 negotiation"));
    }

    let request = readiness_request()?;
    let started = Instant::now();
    if let Err(source) = stream.write_all(&request) {
        record_observation(trace, family, TorReadinessStage::Destination,
            TorReadinessOperation::WriteDestination, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Destination));
    }
    record_observation(trace, family, TorReadinessStage::Destination,
        TorReadinessOperation::WriteDestination, None, None, started);

    let mut response = [0_u8; 4];
    let started = Instant::now();
    if let Err(source) = stream.read_exact(&mut response) {
        record_observation(trace, family, TorReadinessStage::Destination,
            TorReadinessOperation::ReadDestination, Some(&source), None, started);
        return Err(SocksProbeError::Retryable(TorReadinessStage::Destination));
    }
    record_observation(trace, family, TorReadinessStage::Destination,
        TorReadinessOperation::ReadDestination, None, None, started);

    if response[0] != 0x05 {
        return Err(SocksProbeError::Protocol("Tor SOCKS endpoint returned an invalid SOCKS5 version"));
    }

    let started = Instant::now();
    record_observation(trace, family, TorReadinessStage::Destination,
        TorReadinessOperation::ClassifyReply, None, Some(response[1]), started);
    if response[1] == 0x00 { return Ok(()); }
    Err(SocksProbeError::Retryable(TorReadinessStage::Destination))
}

fn readiness_request() -> Result<Vec<u8>, SocksProbeError> {
    let host = READINESS_HOST.as_bytes();
    let host_len = u8::try_from(host.len()).map_err(|_| {
        SocksProbeError::Protocol("Tor readiness hostname is too long for SOCKS5")
    })?;
    let mut request = Vec::with_capacity(7 + host.len());
    request.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, host_len]);
    request.extend_from_slice(host);
    request.extend_from_slice(&READINESS_PORT.to_be_bytes());
    Ok(request)
}

fn classify_io_kind(source: &io::Error) -> TorReadinessIoKind {
    match source.kind() {
        io::ErrorKind::ConnectionRefused => TorReadinessIoKind::ConnectionRefused,
        io::ErrorKind::ConnectionReset => TorReadinessIoKind::ConnectionReset,
        io::ErrorKind::ConnectionAborted => TorReadinessIoKind::ConnectionAborted,
        io::ErrorKind::TimedOut => TorReadinessIoKind::TimedOut,
        io::ErrorKind::WouldBlock => TorReadinessIoKind::WouldBlock,
        io::ErrorKind::NotConnected => TorReadinessIoKind::NotConnected,
        _ => TorReadinessIoKind::Other,
    }
}

fn classify_socks_reply(reply: u8) -> TorSocksReplyClass {
    match reply {
        0x00 => TorSocksReplyClass::Succeeded,
        0x01 => TorSocksReplyClass::GeneralFailure,
        0x02 => TorSocksReplyClass::RulesetDenied,
        0x03 => TorSocksReplyClass::NetworkUnreachable,
        0x04 => TorSocksReplyClass::HostUnreachable,
        0x05 => TorSocksReplyClass::ConnectionRefused,
        0x06 => TorSocksReplyClass::TtlExpired,
        0x07 => TorSocksReplyClass::CommandUnsupported,
        0x08 => TorSocksReplyClass::AddressTypeUnsupported,
        _ => TorSocksReplyClass::Unassigned,
    }
}

fn record_observation(
    trace: &mut Option<&mut TorReadinessTrace>,
    family: TorReadinessLoopbackFamily,
    stage: TorReadinessStage,
    operation: TorReadinessOperation,
    source: Option<&io::Error>,
    socks_reply: Option<u8>,
    started: Instant,
) {
    let Some(trace) = trace.as_deref_mut() else { return; };
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    trace.push(TorReadinessObservation {
        family, stage, operation,
        io_kind: source.map(classify_io_kind),
        socks_reply,
        socks_reply_class: socks_reply.map(classify_socks_reply),
        duration_ms,
    });
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
