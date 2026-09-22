# T038 — Add the backend-only prepared download runtime boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only prepared download runtime boundary.

A verified T037 `DownloadPreflightSpec` is converted into a typed prepared runtime by reusing the
existing `prepare_arti_runtime` privacy-layer boundary. The prepared state retains
`PreparedArtiRuntime`, the explicit yt-dlp executable path, the typed `YtDlpMediaSourceUrl`, and the
download output root.

Preparation failures fail closed. This boundary materializes only the deterministic Arti
configuration. It does not create sidecar executables, Tor cache/state directories, media output, or
download output, and it starts no process or network activity.

Frontend IPC/output is unchanged, so no backend executable path, Tor directory, output path, SOCKS
endpoint, or media URL is exposed.

## Verification

PR #38 verified head:
`78f814e1c183d13a937d609e620537e0a8fcc173`

All 11 required workflows passed:

- continuity-guard;
- rust-check;
- desktop-shell-check;
- arti-sidecar-check;
- arti-materialization-check;
- arti-config-contract;
- arti-lifecycle-check;
- tor-readiness-check;
- ytdlp-sidecar-check;
- ytdlp-tor-metadata-check;
- ytdlp-tor-media-check.
