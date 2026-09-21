# T018 — Add bounded Tor readiness verification

Parent: PRIVACY FOUNDATION  
Status: ACTIVE

## Goal

Implement the only privacy-layer path allowed to mint `ReadyTorTransport` after a bounded,
fail-closed Tor readiness proof.

## Acceptance criteria

- verifier accepts only a running Arti child owned by the privacy layer;
- deferred bootstrap is explicitly activated inside the privacy layer before probing readiness;
- successful verification is the only path that can mint `ReadyTorTransport`;
- failure returns no ready capability;
- readiness is bounded by an explicit timeout;
- the explicit bootstrap transition restarts the same pinned executable with the same deterministic arguments and no shell;
- the verifier connects only to the local loopback SOCKS endpoint;
- the external target is sent as a SOCKS5 domain name, never locally resolved;
- no direct/clearnet fallback exists;
- Windows and Linux CI use the actual pinned Arti 2.6.0 sidecar;
- all existing checks remain green.

## Out of scope

- yt-dlp/FFmpeg;
- AI provider;
- UI;
- direct-network fallback.
