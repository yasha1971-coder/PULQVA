# NEXT

## Current verified state

T038 is complete.

PULQVA now has a backend-only prepared download runtime boundary:

- input is a verified T037 `DownloadPreflightSpec`;
- Arti configuration preparation reuses `prepare_arti_runtime`;
- prepared state retains `PreparedArtiRuntime`, the explicit yt-dlp executable path, typed `YtDlpMediaSourceUrl`, and output root;
- preparation failures fail closed;
- only the deterministic Arti configuration is materialized;
- frontend IPC/output remains unchanged and exposes no backend paths or media URL;
- no Arti, yt-dlp, or FFmpeg process is spawned;
- no Tor bootstrap/readiness probe or external network access occurs.

Verified PR head:
`78f814e1c183d13a937d609e620537e0a8fcc173`

All 11 required workflows passed for that exact head.

## Next atomic task

**T039 — Add the backend-only Tor-ready download runtime boundary**

Advance one prepared T038 runtime to a Tor-ready backend capability using only the existing controlled
Arti launcher and readiness verifier.

Required boundary:

- input is a prepared T038 runtime;
- launch only the prepared Arti runtime through the existing controlled launcher;
- verify Tor readiness through the existing bounded readiness boundary;
- retain yt-dlp executable, typed media source, and output root only after Tor readiness succeeds;
- failures stop and clean up the Arti child and fail closed;
- frontend receives only inert readiness/stage data and no paths, SOCKS endpoint, or media URL;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback is introduced.

## Do not do yet

- no external search provider;
- no AI provider;
- no yt-dlp execution from the desktop UI;
- no FFmpeg execution from the desktop UI;
- no actual media download;
- no packaging/release installers;
- no direct-network fallback.

## Success

A prepared download runtime can become a typed Tor-ready backend capability with bounded failure and
cleanup, while media execution remains out of scope.
