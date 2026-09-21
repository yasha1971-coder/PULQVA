# T018 — Add bounded Tor readiness verification

Parent: PRIVACY FOUNDATION  
Status: READY

## Goal

Implement the privacy-layer verification step that is allowed to mint `ReadyTorTransport` only
after the real Tor transport is actually ready.

## Acceptance criteria

- verifier accepts only a running/prepared Tor runtime owned by the privacy layer;
- successful verification is the only path that can mint `ReadyTorTransport`;
- failure returns no ready capability;
- bootstrap/readiness is bounded by an explicit timeout;
- no clearnet fallback exists;
- remote DNS remains on the Tor side;
- verification is isolated to the privacy layer;
- Windows and Linux CI coverage;
- all existing checks remain green.

## Out of scope

- yt-dlp/FFmpeg;
- AI provider;
- UI;
- any direct-network fallback.
