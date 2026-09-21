# T023 — Prove a real pinned yt-dlp metadata-only request through Tor

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Use the actual pinned yt-dlp standalone binary through the typed process boundary only after Tor
readiness, and prove a bounded metadata-only request with no direct-network fallback.

## Acceptance criteria

- actual pinned yt-dlp 2026.08.19 binary is used;
- actual pinned Arti 2.6.0 binary is used;
- execution starts from `YtDlpMediaRequestPlan`;
- metadata-only mode explicitly adds `--skip-download --dump-single-json --no-playlist`;
- `ReadyTorTransport` is required before the yt-dlp plan can exist;
- yt-dlp receives only the verified `socks5h://` route;
- execution is bounded by an explicit timeout;
- successful proof creates no download output;
- Tor/readiness failure produces no direct retry;
- Linux requires successful metadata extraction;
- Windows accepts successful metadata extraction or the bounded Tor-readiness fail-closed path;
- all existing checks remain green.

## Out of scope

- full media download;
- FFmpeg post-processing;
- AI;
- UI;
- any direct/clearnet fallback.
