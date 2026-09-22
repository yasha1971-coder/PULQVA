# NEXT

## Current verified state

T040 is complete.

PULQVA now has a backend-only Tor-gated media request boundary:

- input is a T039 `TorReadyDownloadRuntime`;
- `YtDlpMediaRequestPlan::new_tor_gated` derives its launch plan only from verified `ReadyTorTransport`;
- the request uses the retained typed media source and output root;
- the resulting backend state retains ownership of the running Arti child with the typed media request plan;
- request-planning failures stop and wait for Arti before returning;
- cleanup failures are surfaced separately and fail closed;
- frontend IPC/output remains unchanged and exposes no executable path, output path, SOCKS/proxy data, media URL, or argv;
- no yt-dlp or FFmpeg process is started;
- no media download is started;
- no direct-network fallback exists.

Verified PR head:
`021994420b8e3ee393785e6dc4ef3685150efdd6`

All 11 required workflows passed for that exact head.

## Next atomic task

**T041 — Add the backend-only controlled yt-dlp media execution boundary**

Advance one verified T040 Tor-gated media request state into a controlled running yt-dlp child while
retaining ownership of the live Arti child.

Required boundary:

- input is a T040 Tor-gated media request runtime;
- yt-dlp launch reuses the existing controlled `launch_ytdlp_request` boundary;
- the running Arti child remains owned by the backend for the full yt-dlp child lifetime;
- yt-dlp launch failure stops and waits for Arti before returning failure;
- no shell execution;
- no direct-network fallback;
- frontend receives only inert running/stage data and no executable path, output path, proxy/SOCKS data, media URL, argv, or process identifiers;
- FFmpeg remains out of scope;
- completion/result handling remains out of scope.

## Do not do yet

- no external search provider;
- no AI provider;
- no FFmpeg execution from the desktop UI;
- no completion receipt/result surfacing;
- no packaging/release installers;
- no direct-network fallback.

## Success

A verified Tor-gated media request can launch yt-dlp through the existing controlled process boundary
while Arti remains backend-owned and no private runtime details cross into frontend code.
