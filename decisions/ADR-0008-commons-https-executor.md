# ADR-0008 — Endpoint-private Commons HTTPS executor

Date: 2026-09-27. Status: implemented for native verification, NOT live-E2E accepted.

Use the already verified reqwest 0.13.5 / Rustls 0.23.43 / Tokio 1.53.1 /
webpki-roots 1.0.9 graph. No regenerated lock, version upgrade or new provider.
Reuse ReadyTorTransport verification on the borrowed RunningArti child, followed by
CommonsSearch/CandidateSearch/T067. New HTTP capability is endpoint-specific and
private; no caller URL/client/proxy/trust override. Core has no HTTP dependency.

The transport builds a fresh async client inside a short-lived current-thread
runtime for each synchronous backend-worker request. No detached request task.
Disable system proxy selection first, then install only the verified socks5h route
with exclusions disabled. Proxy authority must be a literal loopback IP. Keep HTTPS
certificate/name checks with pinned webpki roots, HTTP/1, no redirects/implicit
retries/referer/cookies/decompression/SSLKEYLOGFILE. No CA files from ambient env.

Accept only HTTP 200 application/json with absent/identity content encoding, reject
oversized declared length and enforce the actual 256-KiB streaming cap. Reject
truncated framing. Outer request timeout is supplemented by monotonic deadline and
child/cancellation checks before request polling and after a result becomes ready.
A 50-ms wake timer permits observing cancellation/child termination during a stalled
future. This is cooperative scheduling, not an OS-hard egress or realtime guarantee.
Bootstrap has the existing separate 90-second readiness budget and does not yet
accept the request cancellation handle. Run the adapter on a blocking backend worker;
reject nested Tokio runtime use instead of panicking on block_on.

## Targeted current-world verification
Exact-version docs.rs URLs initially returned cache misses; do not count those as
successful review. Pinned reqwest v0.13.5 src/async_impl/client.rs was read from
upstream GitHub; prior native B0 validates this builder API combination. A web search
subsequently returned Tokio 1.53.1 timeout docs and source, including cooperative
cancellation and the caveat that timeout polling alone does not bound non-yielding
work. Hence the explicit monotonic guard and no detached JoinHandle timeout pattern.

Sources checked:
- https://github.com/seanmonstar/reqwest/blob/v0.13.5/src/async_impl/client.rs
- https://docs.rs/tokio/latest/tokio/time/fn.timeout.html (served 1.53.1)
- https://docs.rs/tokio/latest/src/tokio/time/timeout.rs.html
- https://docs.rs/reqwest/latest/reqwest/blocking/struct.ClientBuilder.html
  (preconfigured TLS version-coupling warning; async APIs validated by pinned B0)

No exhaustive security/industry review or automated root-bundle update is claimed.
Keep root-bundle/version maintenance a deliberate release process, not runtime fetch.

## Verification and known limits
Controlled tests use local SOCKS plus real Rustls handshake with test-only credentials;
no fixture connects outward. Production roots are never replaced outside cfg(test).
These tests are pending first locked native Windows/Linux CI, not locally executed.
They cover bytes/status/framing, remote-host forwarding, refusal, certificate/name
rejection, redirects, cancellation/liveness callback and environment poisoning.
Actual Arti stop during real transfer, real Commons availability and selected-file
E2E still need separate evidence. Constructor readiness and liveness are not OS
sandboxing; inherited T066/Arti ownership/confinement/retention limits remain.
The synchronous error surface currently maps transport/deadline/cancel to the existing
DiscoveryError::Transport; it does not leak URLs or TLS/server details into user logs.
