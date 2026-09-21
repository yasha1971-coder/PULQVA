# NEXT

## Current verified state

T017 is complete.

PULQVA now exposes external-network routing only through a readiness-gated
`ReadyTorTransport` capability. Raw `TorSocksEndpoint` data cannot expose a public proxy URL.

Verified PR head:
`7009058db21c86a442ce466bc200f6b035e5919a`

## Active atomic task

**T018 — Add bounded Tor readiness verification**

The privacy layer now owns the only path allowed to mint `ReadyTorTransport`.

The verifier:

- accepts a running Arti child owned by the privacy layer;
- uses only the local loopback SOCKS endpoint;
- sends a SOCKS5 domain-name CONNECT request so DNS stays on the Tor side;
- uses no direct/clearnet fallback;
- is bounded by an explicit timeout;
- returns no ready capability on timeout, child exit, or protocol failure.

The Windows/Linux proof uses the actual pinned Arti 2.6.0 sidecar.

## Queued next task

**T019 — Pin and prove the yt-dlp standalone sidecar**

Source-verify the current official standalone yt-dlp release, pin it, and prove the exact binary
CLI on Windows and Linux before adding any media-network request.

## Do not do yet

- no user download request;
- no direct-network fallback;
- no FFmpeg integration;
- no AI provider;
- no UI.

## Success

Only a successfully verified Tor route can produce `ReadyTorTransport`, and the proof is bounded,
fail-closed, remote-DNS-safe, and green on Windows and Linux.
