# T037 — Add the typed desktop download preflight specification

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a pure-data backend download preflight specification.

The preflight starts from a revalidated supported local candidate and reuses the T036 backend-only
media-source mapping. Arti executable, yt-dlp executable, Tor config/cache/state directories, download
output root, and SOCKS port are explicit backend inputs.

The resulting typed Rust data contains an `ArtiRuntimePlan`, yt-dlp executable path,
`YtDlpMediaSourceUrl`, and output root. Empty required paths and invalid SOCKS ports fail closed.

The frontend receives none of those backend paths or the media URL. T037 starts no process, performs
no Tor bootstrap, runs no yt-dlp/FFmpeg process, opens no external network connection, and creates no
filesystem output.

## Verification

PR #37 verified head:
`a82f326d82f117a6053f86bbde5d1eb7d23f29a2`

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
