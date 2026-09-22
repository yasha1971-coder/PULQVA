# T042 — Add the backend-only completed download result boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-22

## Result

Added a backend-only completed download result boundary.

A T041 `RunningMediaDownloadRuntime` now completes through the existing
`RunningYtDlp::complete_download` path, which waits for yt-dlp and validates the completed media
artifact before producing the existing typed `CompletedDownloadResult`.

The live Arti child is stopped and waited after yt-dlp completion on both success and failure paths.
A yt-dlp completion failure fails closed. An Arti cleanup failure after successful completion also
invalidates the boundary result and fails closed. If both completion and cleanup fail, both causes are
preserved in the returned error.

Frontend IPC/output is unchanged and receives no backend path, SOCKS/proxy data, source URL, argv,
process identifier, or raw completion internals. No FFmpeg process is started and no direct-network
fallback is introduced.

## Verification

PR #42 verified head:
`2202145197ec65089de3452d36c000957b6eb521`

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
