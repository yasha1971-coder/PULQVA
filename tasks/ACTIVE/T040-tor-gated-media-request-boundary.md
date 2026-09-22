# T040 — Add the backend-only Tor-gated media request boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Convert a T039 Tor-ready download runtime into a typed yt-dlp media request plan while retaining the
running Arti child and without starting yt-dlp or any media retrieval.

## Acceptance criteria

- input is a T039 `TorReadyDownloadRuntime`;
- `YtDlpLaunchPlan` is constructed only from the retained verified `ReadyTorTransport`;
- `YtDlpMediaRequestPlan` uses the retained typed media source and output root;
- the running Arti child remains owned by the resulting backend state;
- plan construction failure stops and waits for Arti before returning failure;
- frontend receives no executable path, filesystem path, SOCKS endpoint, proxy URL, media URL, or argv;
- frontend may receive only inert readiness/stage data;
- no yt-dlp process is spawned;
- no FFmpeg process is spawned;
- no media download is started;
- no direct-network fallback exists;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- yt-dlp execution;
- actual media download;
- FFmpeg execution;
- external candidate search;
- AI provider integration;
- packaging/release installers.
