# NEXT

## Current verified state

T039 is complete.

Verified T039 implementation head:
`0821cfc0bb70b9f43388fecdaf50e253d72a22ad`

Merged T039 main:
`9052815f2003d8c31a540b80961f1e3833e95f8f`

## Active atomic task

**T040 — Add the backend-only Tor-gated media request boundary**

T040 converts a Tor-ready T039 runtime into a typed yt-dlp media request state:

- input is a `TorReadyDownloadRuntime`;
- `YtDlpMediaRequestPlan::new_tor_gated` constructs its `YtDlpLaunchPlan` only from verified `ReadyTorTransport`;
- the request uses the retained typed media source and output root;
- the resulting backend state retains ownership of the running Arti child together with the typed request plan;
- plan-construction failure explicitly stops and waits for Arti before returning;
- cleanup failure is surfaced separately and fail-closed;
- frontend IPC/output remains unchanged and exposes no executable path, output path, SOCKS/proxy information, media URL, or argv;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback exists.

Branch:
`task/T040-tor-gated-media-request-boundary`

## Next task

T040 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no yt-dlp execution from the desktop UI;
- no FFmpeg execution from the desktop UI;
- no actual media download;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T040 head.
