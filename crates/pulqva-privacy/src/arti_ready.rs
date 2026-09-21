use crate::TorSocksEndpoint;

/// Capability proving that the Tor transport has passed the privacy layer's
/// readiness verification.
///
/// There is intentionally no public constructor. External/network adapters can
/// receive this capability, but they cannot manufacture one from a raw port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadyTorTransport {
    endpoint: TorSocksEndpoint,
}

impl ReadyTorTransport {
    /// Returns the verified local Tor SOCKS endpoint.
    pub fn endpoint(self) -> TorSocksEndpoint {
        self.endpoint
    }

    /// Returns the only proxy URL that future external-network adapters may use.
    ///
    /// `socks5h` keeps remote hostname resolution on the Tor side.
    pub fn proxy_url(self) -> String {
        self.endpoint.proxy_url()
    }
}

/// Internal constructor reserved for the privacy-layer readiness verifier.
///
/// T017 deliberately does not implement readiness verification itself.
pub(crate) fn certify_tor_ready(endpoint: TorSocksEndpoint) -> ReadyTorTransport {
    ReadyTorTransport { endpoint }
}

#[cfg(test)]
mod tests {
    use super::certify_tor_ready;
    use crate::TorSocksEndpoint;

    #[test]
    fn ready_transport_preserves_verified_endpoint() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        let ready = certify_tor_ready(endpoint);

        assert_eq!(ready.endpoint(), endpoint);
    }

    #[test]
    fn ready_transport_exposes_remote_dns_proxy_url() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        let ready = certify_tor_ready(endpoint);

        assert_eq!(ready.proxy_url(), "socks5h://127.0.0.1:19050");
    }
}
