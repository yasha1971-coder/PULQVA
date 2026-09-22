# T039 — Add the backend-only Tor-ready download runtime boundary

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Advance a prepared T038 download runtime into a typed Tor-ready backend capability using the existing
controlled Arti launcher and bounded readiness verifier, without starting media retrieval.

## Acceptance criteria

- input is a prepared T038 runtime;
- Arti process launch reuses the existing controlled `launch_prepared_arti` boundary;
- Tor readiness reuses the existing bounded `verify_tor_readiness` boundary;
- success retains a `ReadyTorTransport` together with the explicit yt-dlp executable path, typed media source, and output root;
- Arti launch failure fails closed;
- Tor readiness failure stops and waits for the Arti child before returning failure;
- frontend receives no executable path, filesystem path, SOCKS endpoint, proxy URL, or media URL;
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
