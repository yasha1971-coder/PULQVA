# T040 — Add the backend-only Tor-gated media request boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only Tor-gated media request boundary.

A T039 Tor-ready runtime can be converted into a typed `YtDlpMediaRequestPlan` whose launch plan is
derived only from the retained verified `ReadyTorTransport`. The request reuses the retained typed
media source and output root.

The resulting backend state retains ownership of the running Arti child together with the typed media
request plan. Request-planning failure explicitly stops and waits for Arti before returning, and
cleanup failure is surfaced separately.

Frontend IPC/output is unchanged and exposes no executable path, filesystem path, SOCKS endpoint,
proxy URL, media URL, or argv.

No yt-dlp or FFmpeg process is started and no media download begins.

## Verification

PR #40 verified head:
`021994420b8e3ee393785e6dc4ef3685150efdd6`

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
