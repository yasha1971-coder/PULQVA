# NEXT

## Current verified state

T038 is complete.

Verified T038 implementation head:
`78f814e1c183d13a937d609e620537e0a8fcc173`

Merged T038 main:
`6595561b1d982945d01bfa574069a24de391ae18`

## Active atomic task

**T039 — Add the backend-only Tor-ready download runtime boundary**

T039 advances a prepared T038 runtime to a Tor-ready backend capability:

- input is a prepared `PreparedDownloadRuntime`;
- Arti launch reuses `launch_prepared_arti`;
- Tor readiness reuses bounded `verify_tor_readiness`;
- success retains the running Arti child, `ReadyTorTransport`, yt-dlp executable path, typed media source, and output root;
- Arti launch failures fail closed;
- readiness failures explicitly stop and wait for the Arti child before returning;
- cleanup failure is surfaced separately and fail-closed;
- frontend IPC/output is unchanged and exposes no paths, SOCKS endpoint, proxy URL, or media URL;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback exists.

Branch:
`task/T039-tor-ready-download-runtime-boundary`

## Next task

T039 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no yt-dlp execution from the desktop UI;
- no FFmpeg execution from the desktop UI;
- no actual media download;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T039 head.
