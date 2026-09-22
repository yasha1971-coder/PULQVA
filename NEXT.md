# NEXT

## Current verified state

T039 is complete.

PULQVA now has a backend-only Tor-ready download runtime boundary:

- input is a prepared T038 runtime;
- Arti launch reuses the existing controlled `launch_prepared_arti` boundary;
- Tor readiness reuses the bounded `verify_tor_readiness` boundary;
- success retains the running Arti child, a `ReadyTorTransport`, yt-dlp executable path, typed media source, and output root;
- Arti launch failures fail closed;
- Tor readiness failures explicitly stop and wait for the Arti child;
- cleanup failures are surfaced separately and fail closed;
- frontend IPC/output remains unchanged and exposes no backend path, SOCKS endpoint, proxy URL, or media URL;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback exists.

Verified PR head:
`0821cfc0bb70b9f43388fecdaf50e253d72a22ad`

All 11 required workflows passed for that exact head.

## Next atomic task

**T040 — Add the backend-only Tor-gated media request boundary**

Convert one T039 Tor-ready runtime into a typed yt-dlp media request plan without starting yt-dlp.

Required boundary:

- input is a T039 `TorReadyDownloadRuntime`;
- construct `YtDlpLaunchPlan` only from its verified `ReadyTorTransport`;
- construct `YtDlpMediaRequestPlan` from the retained typed media source and output root;
- retain ownership of the running Arti child so the Tor capability stays live;
- frontend receives no executable path, output path, proxy URL, SOCKS endpoint, media URL, or argv;
- plan-construction failures stop and clean up Arti and fail closed;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback exists.

## Do not do yet

- no external search provider;
- no AI provider;
- no yt-dlp execution from the desktop UI;
- no FFmpeg execution from the desktop UI;
- no actual media download;
- no packaging/release installers;
- no direct-network fallback.

## Success

A Tor-ready runtime can be converted into a typed, Tor-gated yt-dlp media request plan while keeping
the Arti child owned by the backend and without executing media retrieval.
