# T024 — Prove one bounded real media download through Tor

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Use the typed yt-dlp process path to download one fixed small sample media object through a verified
Tor transport into an isolated output root.

## Acceptance criteria

- actual pinned Arti 2.6.0 and yt-dlp 2026.08.19 binaries are used;
- request starts from `YtDlpMediaRequestPlan`;
- `ReadyTorTransport` is required;
- yt-dlp receives only the verified Tor proxy route;
- source is pinned to `mediaelement/mediaelement-files` commit
  `4d21a042353022326071acb0251ab75cd6bae114`;
- expected object size is exactly 5,510,872 bytes;
- output root is isolated and explicit;
- execution has an explicit timeout;
- successful Linux proof finds exactly one regular media artifact with the pinned byte size;
- symlinks in the isolated output tree are rejected;
- no FFmpeg or post-processing is required;
- Tor/readiness failure produces no direct retry;
- Windows may use the existing bounded Tor-readiness fail-closed path;
- all existing checks remain green.

## Out of scope

- arbitrary user URL execution;
- FFmpeg;
- AI;
- UI;
- any direct/clearnet fallback.
