# T041 — Add the backend-only controlled yt-dlp media execution boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only controlled yt-dlp media execution boundary.

A verified T040 Tor-gated media request runtime can launch yt-dlp only through the existing typed
`launch_ytdlp_request` process boundary. The resulting backend state owns both the live Arti child
and the running yt-dlp child.

yt-dlp launch failure explicitly stops and waits for Arti before returning. Coordinated shutdown
attempts both child cleanups and surfaces either or both failures fail-closed.

No shell command string is constructed. Frontend IPC/output remains unchanged and exposes no
executable path, filesystem path, SOCKS/proxy information, media URL, argv, or process identifiers.
FFmpeg and completion/result handling remain out of scope.

## Verification

PR #41 verified head:
`e9aa2b87dc6b8e88807a35ab464628ef5f5b2487`

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
