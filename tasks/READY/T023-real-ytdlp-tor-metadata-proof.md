# T023 — Prove a real pinned yt-dlp metadata-only request through Tor

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Use the actual pinned yt-dlp standalone binary through the typed process boundary only after Tor
readiness, and prove a bounded metadata-only request with no direct-network fallback.

## Acceptance criteria

- actual pinned yt-dlp 2026.08.19 binary is used;
- execution starts from `YtDlpMediaRequestPlan`;
- `ReadyTorTransport` is required before the yt-dlp plan can exist;
- yt-dlp receives the verified `socks5h://` proxy argument;
- request is metadata-only / no media payload download;
- execution is bounded by an explicit timeout;
- Tor/readiness failure produces no direct retry;
- Windows and Linux behavior is recorded honestly;
- all existing checks remain green.

## Out of scope

- full media download;
- FFmpeg post-processing;
- AI;
- UI;
- any direct/clearnet fallback.
