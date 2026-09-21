# T024 — Prove one bounded real media download through Tor

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Use the typed yt-dlp process path to download one fixed small sample media object through a verified
Tor transport into an isolated output root.

## Acceptance criteria

- actual pinned Arti and yt-dlp binaries are used;
- request starts from `YtDlpMediaRequestPlan`;
- `ReadyTorTransport` is required;
- yt-dlp receives only the verified Tor proxy route;
- source is one fixed small test media object;
- output root is isolated and explicit;
- execution has an explicit timeout;
- successful Linux proof writes a non-empty media file;
- no FFmpeg or post-processing is required;
- Tor/readiness failure produces no direct retry;
- Windows behavior is recorded honestly and remains fail-closed;
- all existing checks remain green.

## Out of scope

- arbitrary user URL execution;
- FFmpeg;
- AI;
- UI;
- any direct/clearnet fallback.
