# NEXT

## Current verified state

T037 is complete.

PULQVA now has a typed backend download preflight specification:

- input starts from a revalidated supported `SearchCandidate`;
- media-source resolution reuses the backend-only T036 `YtDlpMediaSourceUrl` boundary;
- Arti executable, yt-dlp executable, Tor config/cache/state directories, output root, and SOCKS port are explicit backend inputs;
- the resulting typed Rust data contains an `ArtiRuntimePlan`, yt-dlp executable path, typed media source, and output root;
- empty required paths and invalid SOCKS ports fail closed;
- frontend receives no media URL, executable path, Tor directory, or output filesystem path;
- no process spawn, Tor bootstrap, yt-dlp/FFmpeg execution, external network, or filesystem output occurs.

Verified PR head:
`a82f326d82f117a6053f86bbde5d1eb7d23f29a2`

All 11 required workflows passed for that exact head.

## Next atomic task

**T038 — Add the backend-only prepared download runtime boundary**

Convert the verified T037 preflight into a prepared backend runtime state using the existing privacy
layer, while keeping process execution and external networking out of scope.

Required boundary:

- input is a verified T037 `DownloadPreflightSpec`;
- Arti configuration materialization uses the existing typed `prepare_arti_runtime` path;
- prepared state retains the explicit yt-dlp executable, typed media source, and output root;
- frontend receives only inert readiness/stage data and no paths or media URL;
- preparation failures fail closed;
- no Arti process is spawned;
- no Tor bootstrap/readiness probe occurs;
- no yt-dlp or FFmpeg process is spawned;
- no external network access occurs.

## Do not do yet

- no external search provider;
- no AI provider;
- no actual media download from the desktop UI;
- no Arti process launch from the desktop UI;
- no yt-dlp/FFmpeg execution from the desktop UI;
- no packaging/release installers;
- no direct-network fallback.

## Success

A verified preflight can be converted into a typed prepared runtime state without exposing backend
paths or source URLs to frontend code and without starting any process or network activity.
