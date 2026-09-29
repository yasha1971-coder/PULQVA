use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpStream},
    time::Duration,
};

use crate::TorSocksEndpoint;

pub(crate) const ATTEMPT_SLICE: Duration = Duration::from_secs(12);
pub(crate) const RETRY_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TorReadinessStage {
    Listener,
    Negotiation,
    Destination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorReadinessTimeout {
    pub stage: TorReadinessStage,
}

pub(crate) enum SocksProbeError {
    Retryable(TorReadinessStage),
    Protocol(&'static str),
}

/// Probes only the configured local SOCKS endpoint.
///
/// The external destination is encoded as a SOCKS5 domain-name request so hostname
/// resolution and the outbound connection remain on the proxy side. This helper
/// never opens a direct connection to the destination.
pub(crate) fn socks_connect_probe(
    endpoint: TorSocksEndpoint,
    destination_host: &str,
    destination_port: u16,
    timeout: Duration,
) -> Result<IpAddr, SocksProbeError> {
    let addresses = [
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, endpoint.port())),
        SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, endpoint.port(), 0, 0)),
    ];

    let mut furthest = TorReadinessStage::Listener;
    for local in addresses {
        match socks_connect_probe_at(local, destination_host, destination_port, timeout) {
            Ok(()) => return Ok(local.ip()),
            Err(SocksProbeError::Retryable(stage)) => furthest = furthest.max(stage),
            Err(error) => return Err(error),
        }
    }

    Err(SocksProbeError::Retryable(furthest))
}

fn socks_connect_probe_at(
    local: SocketAddr,
    destination_host: &str,
    destination_port: u16,
    timeout: Duration,
) -> Result<(), SocksProbeError> {
    let connect_timeout = timeout.min(Duration::from_secs(2));

    let mut stream = match TcpStream::connect_timeout(&local, connect_timeout) {
        Ok(stream) => stream,
        Err(source) if is_retryable_io(&source) => {
            return Err(SocksProbeError::Retryable(TorReadinessStage::Listener));
        }
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

    let host = destination_host.as_bytes();
    let host_len = u8::try_from(host.len()).map_err(|_| {
        SocksProbeError::Protocol("Tor readiness hostname is too long for SOCKS5")
    })?;

    let mut request = Vec::with_capacity(7 + host.len());
    request.extend_from_slice(&[0x05, 0x01, 0x00, 0x03, host_len]);
    request.extend_from_slice(host);
    request.extend_from_slice(&destination_port.to_be_bytes());

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

#[cfg(test)]
mod tests {
    use super::{socks_connect_probe, TorReadinessStage};
    use crate::TorSocksEndpoint;
    use std::time::Duration;

    #[test]
    fn absent_listener_reports_listener_stage() {
        let endpoint = TorSocksEndpoint::new(9).expect("non-zero port");
        let error = socks_connect_probe(endpoint, "example.com", 443, Duration::from_millis(1))
            .expect_err("discard port must not provide Tor SOCKS");
        match error {
            super::SocksProbeError::Retryable(stage) => {
                assert_eq!(stage, TorReadinessStage::Listener);
            }
            super::SocksProbeError::Protocol(message) => panic!("unexpected protocol error: {message}"),
        }
    }
}
