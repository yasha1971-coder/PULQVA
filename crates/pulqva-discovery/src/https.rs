//! Endpoint-private HTTPS executor. All production requests use a freshly verified
//! owned Tor sidecar. Local controlled tests are not live Tor/E2E evidence.
use crate::{CommonsSearch, CommonsSearchPlan, CommonsTransport, DiscoveryError, NetworkFailure, ReadinessFailure};
use pulqva_privacy::{
    ReadyTorTransport, RunningArti, RunningLittleTor, TorReadinessError, TorReadinessStage,
    verify_little_tor_readiness, verify_tor_readiness,
};
use std::{future::{Future, poll_fn}, net::IpAddr, sync::{Arc, atomic::{AtomicBool, Ordering}},
          task::Poll, time::Duration};

const API: &str = "https://commons.wikimedia.org/w/api.php";
const POLL: Duration = Duration::from_millis(50);

/// Cancellation is sticky, including for subsequent requests on the same search.
#[derive(Clone, Default)]
pub struct DiscoveryCancellation(Arc<AtomicBool>);
impl DiscoveryCancellation {
    pub fn cancel(&self) { self.0.store(true, Ordering::Release); }
    fn cancelled(&self) -> bool { self.0.load(Ordering::Acquire) }
}

/// Synchronous coordinator adapter: run on a backend blocking worker, not an
/// async runtime thread. There is no public client/URL/proxy/trust-store override.
enum OwnedTorChild<'a> {
    Arti(&'a mut RunningArti),
    LittleTor(&'a mut RunningLittleTor),
}

impl OwnedTorChild<'_> {
    fn is_alive(&mut self) -> bool {
        match self {
            Self::Arti(child) => matches!(child.try_wait(), Ok(None)),
            Self::LittleTor(child) => matches!(child.try_wait(), Ok(None)),
        }
    }
}

pub struct CommonsHttpsTransport<'a> {
    child: OwnedTorChild<'a>,
    ready: ReadyTorTransport,
    cancellation: DiscoveryCancellation,
}
impl<'a> CommonsHttpsTransport<'a> {
    /// Verify the route on THIS Arti child, rather than combining an unrelated
    /// process with an old readiness token.
    pub fn new(arti: &'a mut RunningArti) -> Result<Self, DiscoveryError> {
        if tokio::runtime::Handle::try_current().is_ok() { return Err(DiscoveryError::Transport); }
        let ready = verify_tor_readiness(arti, Duration::from_secs(90))
            .map_err(readiness_failure)?;
        Ok(Self {
            child: OwnedTorChild::Arti(arti),
            ready,
            cancellation: DiscoveryCancellation::default(),
        })
    }

    /// Windows little-t Tor path. The same readiness verifier and capability are
    /// required before the HTTPS adapter can exist.
    pub fn new_little_tor(tor: &'a mut RunningLittleTor) -> Result<Self, DiscoveryError> {
        if tokio::runtime::Handle::try_current().is_ok() { return Err(DiscoveryError::Transport); }
        let ready = verify_little_tor_readiness(tor, Duration::from_secs(90))
            .map_err(readiness_failure)?;
        Ok(Self {
            child: OwnedTorChild::LittleTor(tor),
            ready,
            cancellation: DiscoveryCancellation::default(),
        })
    }

    pub fn cancellation(&self) -> DiscoveryCancellation { self.cancellation.clone() }
    pub fn into_search(self) -> CommonsSearch<Self> {
        let ready = self.ready;
        CommonsSearch::new(self, ready)
    }
}
impl CommonsTransport for CommonsHttpsTransport<'_> {
    fn fetch(&mut self, plan: &CommonsSearchPlan) -> Result<Vec<u8>, DiscoveryError> {
        if plan.proxy_url() != self.ready.proxy_url() || plan.max_redirects() != 0 {
            return Err(DiscoveryError::InvalidRequest);
        }
        let cancel = self.cancellation.clone();
        let child = &mut self.child;
        execute(plan.url(), plan.proxy_url(), plan.user_agent(), plan.timeout(),
                plan.max_response_bytes(), trusted_roots(), &cancel,
                || child.is_alive())
    }
}

// Classify without formatting or retaining a possibly URL-bearing source chain.
fn readiness_failure(error: TorReadinessError) -> DiscoveryError {
    DiscoveryError::Readiness(match error {
        TorReadinessError::Timeout(timeout) => match timeout.stage {
            TorReadinessStage::Listener => ReadinessFailure::ListenerTimeout,
            TorReadinessStage::Negotiation => ReadinessFailure::NegotiationTimeout,
            TorReadinessStage::Destination => ReadinessFailure::DestinationTimeout,
        },
        TorReadinessError::BootstrapActivation(_) => ReadinessFailure::BootstrapActivation,
        TorReadinessError::ChildExited(_) => ReadinessFailure::ChildExited,
        TorReadinessError::Protocol(_) => ReadinessFailure::Protocol,
        TorReadinessError::Io { .. } => ReadinessFailure::Io,
    })
}
fn network_failure(error: reqwest::Error) -> DiscoveryError {
    DiscoveryError::Network(if error.is_timeout() { NetworkFailure::Timeout }
        else if error.is_connect() { NetworkFailure::ConnectOrTls }
        else if error.is_body() { NetworkFailure::Body }
        else { NetworkFailure::Other })
}

fn trusted_roots() -> rustls::RootCertStore {
    rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned())
}

// Private, shared by the production adapter and controlled unit fixtures. Test
// roots never cross the public API and are never installed in the host trust store.
fn validate_route(url: &str, proxy: &str) -> Result<(), DiscoveryError> {
    let target = reqwest::Url::parse(url).map_err(|_| DiscoveryError::InvalidRequest)?;
    if target.scheme() != "https" || target.host_str() != Some("commons.wikimedia.org")
        || target.port_or_known_default() != Some(443) || target.path() != "/w/api.php"
        || !target.username().is_empty() || target.password().is_some() || target.fragment().is_some() {
        return Err(DiscoveryError::InvalidRequest);
    }
    let route = reqwest::Url::parse(proxy).map_err(|_| DiscoveryError::InvalidRequest)?;
    let loopback = route.host_str().and_then(|s| s.trim_matches(['[', ']']).parse::<IpAddr>().ok())
        .is_some_and(|ip| ip.is_loopback());
    if route.scheme() != "socks5h" || !loopback || route.port().is_none_or(|p| p == 0)
        || !route.username().is_empty() || route.password().is_some()
        || route.query().is_some() || route.fragment().is_some()
        || !matches!(route.path(), "" | "/") { return Err(DiscoveryError::InvalidRequest); }
    Ok(())
}

fn build_client(proxy: &str, budget: Duration, roots: rustls::RootCertStore)
    -> Result<reqwest::Client, DiscoveryError>
{
    let tls = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .with_safe_default_protocol_versions().map_err(|_| DiscoveryError::Transport)?
        .with_root_certificates(roots).with_no_client_auth();
    reqwest::Client::builder().no_proxy()
        .proxy(reqwest::Proxy::all(proxy).map_err(|_| DiscoveryError::Transport)?.no_proxy(None))
        .https_only(true).http1_only()
        .redirect(reqwest::redirect::Policy::none()).retry(reqwest::retry::never())
        .referer(false).no_gzip().no_brotli().no_deflate().no_zstd()
        .pool_max_idle_per_host(0).connect_timeout(budget.min(Duration::from_secs(15)))
        .timeout(budget).tls_backend_preconfigured(tls).tls_sslkeylogfile(false)
        .build().map_err(|_| DiscoveryError::Transport)
}

async fn receive(client: reqwest::Client, url: &str, agent: &str, cap: usize)
    -> Result<Vec<u8>, DiscoveryError>
{
    let mut response = client.get(url).header(reqwest::header::USER_AGENT, agent)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::ACCEPT_ENCODING, "identity")
        .send().await.map_err(network_failure)?;
    if response.status() != reqwest::StatusCode::OK { return Err(DiscoveryError::HttpStatus(response.status().as_u16())); }
    if response.headers().get(reqwest::header::CONTENT_ENCODING)
        .is_some_and(|v| v.as_bytes() != b"identity") { return Err(DiscoveryError::ContentEncoding); }
    let content_type = response.headers().get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok()).and_then(|v| v.split(';').next());
    if !content_type.is_some_and(|s| s.trim().eq_ignore_ascii_case("application/json")) {
        return Err(DiscoveryError::ContentType);
    }
    let declared = response.content_length();
    if declared.is_some_and(|size| size > cap as u64) { return Err(DiscoveryError::ResponseTooLarge); }
    let mut out = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network_failure)? {
        if chunk.len() > cap.saturating_sub(out.len()) { return Err(DiscoveryError::ResponseTooLarge); }
        out.extend_from_slice(&chunk);
    }
    if declared.is_some_and(|size| size != out.len() as u64) { return Err(DiscoveryError::InvalidResponse); }
    Ok(out)
}

// Drop the in-flight request on cancellation, child termination or timeout. No
// detached request task, fallback connector, unbounded retry or renewed deadline.
async fn supervise<F, L>(work: F, cancel: &DiscoveryCancellation, mut alive: L)
    -> Result<Vec<u8>, DiscoveryError>
where F: Future<Output = Result<Vec<u8>, DiscoveryError>>, L: FnMut() -> bool
{
    let mut work = Box::pin(work);
    let mut tick = Box::pin(tokio::time::sleep(POLL));
    poll_fn(|cx| {
        if cancel.cancelled() || !alive() { return Poll::Ready(Err(DiscoveryError::Transport)); }
        if let Poll::Ready(result) = work.as_mut().poll(cx) {
            return Poll::Ready(if cancel.cancelled() || !alive() { Err(DiscoveryError::Transport) } else { result });
        }
        if tick.as_mut().poll(cx).is_ready() {
            tick.as_mut().reset(tokio::time::Instant::now() + POLL);
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }).await
}

fn execute<L>(url: &str, proxy: &str, agent: &str, budget: Duration, cap: usize,
              roots: rustls::RootCertStore, cancel: &DiscoveryCancellation, mut alive: L)
    -> Result<Vec<u8>, DiscoveryError>
where L: FnMut() -> bool
{
    validate_route(url, proxy)?;
    if budget.is_zero() || cap == 0 || cap > crate::MAX_RESPONSE_BYTES
        || cancel.cancelled() || !alive() || tokio::runtime::Handle::try_current().is_ok() {
        return Err(DiscoveryError::Transport);
    }
    let deadline = std::time::Instant::now() + budget;
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()
        .map_err(|_| DiscoveryError::Transport)?;
    let outcome = runtime.block_on(async {
        tokio::time::timeout(budget, async {
            let client = build_client(proxy, budget, roots)?;
            supervise(receive(client, url, agent, cap), cancel,
                      || std::time::Instant::now() < deadline && alive()).await
        }).await.map_err(|_| DiscoveryError::Network(NetworkFailure::Timeout))?
    });
    if matches!(&outcome, Err(DiscoveryError::Transport)) && std::time::Instant::now() >= deadline {
        Err(DiscoveryError::Network(NetworkFailure::Timeout))
    } else { outcome }
}

#[cfg(test)]
mod tests;
