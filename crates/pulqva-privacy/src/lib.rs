//! Privacy capability types for PULQVA.
//!
//! This crate deliberately does not open sockets. It models only the privacy
//! capability future networked adapters are allowed to receive.

mod arti_config;
mod arti_launch;
mod arti_launcher;
mod arti_materialize;
mod arti_prepared;
mod arti_readiness;
mod arti_ready;
mod arti_runtime;
mod ytdlp_launch;

pub use arti_config::{ArtiConfigRenderError, ArtiConfigSpec};
pub use arti_launch::ArtiLaunchSpec;
pub use arti_launcher::{ArtiProcessError, RunningArti, launch_prepared_arti};
pub use arti_materialize::{ArtiConfigMaterializeError, materialize_arti_config};
pub use arti_prepared::{PreparedArtiRuntime, prepare_arti_runtime};
pub use arti_readiness::{TorReadinessError, verify_tor_readiness};
pub use arti_ready::ReadyTorTransport;
pub use arti_runtime::ArtiRuntimePlan;
pub use ytdlp_launch::YtDlpLaunchPlan;

use std::{error::Error, fmt, num::NonZeroU16};

const TOR_SOCKS_HOST: &str = "127.0.0.1";

/// A local Tor SOCKS endpoint.
///
/// There is intentionally no direct/clearnet route type here. Networked
/// adapters are expected to receive this capability after Tor bootstrap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorSocksEndpoint {
    port: NonZeroU16,
}

impl TorSocksEndpoint {
    /// Creates a Tor SOCKS endpoint on fixed loopback host 127.0.0.1.
    pub fn new(port: u16) -> Result<Self, TorSocksEndpointError> {
        let port = NonZeroU16::new(port).ok_or(TorSocksEndpointError::ZeroPort)?;
        Ok(Self { port })
    }

    /// Returns the local SOCKS port.
    pub fn port(self) -> u16 {
        self.port.get()
    }

    /// Returns the proxy URL network adapters must use.
    ///
    /// socks5h is intentional: hostname resolution stays on the proxy side.
    pub(crate) fn proxy_url(self) -> String {
        format!("socks5h://{TOR_SOCKS_HOST}:{}", self.port)
    }
}

/// Validation errors for TorSocksEndpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorSocksEndpointError {
    ZeroPort,
}

impl fmt::Display for TorSocksEndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroPort => f.write_str("Tor SOCKS port must be non-zero"),
        }
    }
}

impl Error for TorSocksEndpointError {}

#[cfg(test)]
mod tests {
    use super::{TorSocksEndpoint, TorSocksEndpointError};

    #[test]
    fn rejects_zero_port() {
        assert_eq!(
            TorSocksEndpoint::new(0).unwrap_err(),
            TorSocksEndpointError::ZeroPort
        );
    }

    #[test]
    fn exposes_valid_port() {
        let endpoint = TorSocksEndpoint::new(19050).expect("non-zero port is valid");
        assert_eq!(endpoint.port(), 19050);
    }

}
