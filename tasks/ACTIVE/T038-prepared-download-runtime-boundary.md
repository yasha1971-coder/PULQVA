# T038 — Add the backend-only prepared download runtime boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Convert a verified T037 download preflight into a typed prepared backend runtime state without
starting any process or external network activity.

## Acceptance criteria

- input is a verified `DownloadPreflightSpec`;
- Arti runtime preparation reuses the existing `prepare_arti_runtime` typed privacy-layer boundary;
- prepared state retains the explicit yt-dlp executable path;
- prepared state retains the typed `YtDlpMediaSourceUrl`;
- prepared state retains the download output root;
- preparation errors fail closed;
- frontend receives no media URL, executable path, Tor directory, or output filesystem path;
- frontend may receive only inert preparation readiness/stage data;
- no Arti process is spawned;
- no Tor bootstrap or readiness probe occurs;
- no yt-dlp or FFmpeg process is spawned;
- no external network access occurs;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- Arti process launch;
- Tor readiness verification;
- yt-dlp execution;
- actual media download;
- FFmpeg execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
