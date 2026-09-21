# T022 — Add a controlled yt-dlp child-process launcher

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Introduce the first yt-dlp process side effect behind `YtDlpMediaRequestPlan`, while keeping the
proof network-free.

## Acceptance criteria

- launcher accepts only `YtDlpMediaRequestPlan`;
- executable and argv come only from the typed request plan;
- process is spawned directly, never through a shell;
- typed running-process capability owns the child handle;
- deterministic stop/wait cleanup exists;
- Windows and Linux tests use a local fixture executable that records argv and performs no network;
- fixture proves Tor proxy/base arguments and request arguments are passed unchanged;
- no real media URL is contacted;
- no direct-network fallback exists;
- all existing checks remain green.

## Out of scope

- real yt-dlp media download;
- FFmpeg;
- AI;
- UI.
