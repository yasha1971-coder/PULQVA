use std::net::IpAddr;

use crate::TorSocksEndpoint;

/// Capability proving that the Tor transport has passed the privacy layer's
/// readiness verification.
///
/// There is intentionally no public constructor. External/network adapters can
/// receive this capability, but they cannot manufacture one from a raw port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadyTorTransport {
    endpoint: TorSocksEndpoint,
    verified_loopback: IpAddr,
}

impl ReadyTorTransport {
    /// Returns the verified local Tor SOCKS endpoint.
    pub fn endpoint(self) -> TorSocksEndpoint {
        self.endpoint
    }

    /// Returns the only proxy URL that future external-network adapters may use.
    ///
    /// The exact loopback address is the one that passed readiness verification.
    /// `socks5h` keeps remote hostname resolution on the Tor side.
    pub fn proxy_url(self) -> String {
        match self.verified_loopback {
            IpAddr::V4(address) => {
                format!("socks5h://{address}:{}", self.endpoint.port())
            }
            IpAddr::V6(address) => {
                format!("socks5h://[{address}]:{}", self.endpoint.port())
            }
        }
    }
}

/// Internal constructor reserved for the privacy-layer readiness verifier.
pub(crate) fn certify_tor_ready(
    endpoint: TorSocksEndpoint,
    verified_loopback: IpAddr,
) -> ReadyTorTransport {
    debug_assert!(verified_loopback.is_loopback());
    ReadyTorTransport {
        endpoint,
        verified_loopback,
    }
}

#[cfg(test)]
mod tests {
    use super::certify_tor_ready;
    use crate::TorSocksEndpoint;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn ready_transport_preserves_verified_endpoint() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        let ready = certify_tor_ready(endpoint, IpAddr::V4(Ipv4Addr::LOCALHOST));

        assert_eq!(ready.endpoint(), endpoint);
    }

    #[test]
    fn ready_transport_exposes_ipv4_remote_dns_proxy_url() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        let ready = certify_tor_ready(endpoint, IpAddr::V4(Ipv4Addr::LOCALHOST));

        assert_eq!(ready.proxy_url(), "socks5h://127.0.0.1:19050");
    }

    #[test]
    fn ready_transport_exposes_ipv6_remote_dns_proxy_url() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        let ready = certify_tor_ready(endpoint, IpAddr::V6(Ipv6Addr::LOCALHOST));

        assert_eq!(ready.proxy_url(), "socks5h://[::1]:19050");
    }
}
