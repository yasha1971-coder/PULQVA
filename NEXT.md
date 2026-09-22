# NEXT

## Current verified state

T041 is complete.

Verified T041 implementation head:
`e9aa2b87dc6b8e88807a35ab464628ef5f5b2487`

Merged T041 main:
`b7885f2d62f72d02f4063dcd394e87508d18e6e4`

## Active atomic task

**T042 — Add the backend-only completed download result boundary**

T042 completes a running T041 media runtime through the existing typed completion path:

- input is a `RunningMediaDownloadRuntime`;
- yt-dlp completion reuses `RunningYtDlp::complete_download`;
- successful completion yields the existing `CompletedDownloadResult`;
- Arti is stopped and waited after yt-dlp completion on both success and failure paths;
- yt-dlp completion failure is surfaced fail-closed;
- Arti cleanup failure after a successful download invalidates the boundary result and fails closed;
- combined completion and Arti cleanup failure preserves both causes;
- frontend IPC/output remains unchanged and exposes no backend paths, SOCKS/proxy data, media URL, argv, process identifiers, or raw completion internals;
- no FFmpeg process is started;
- no direct-network fallback exists.

Branch:
`task/T042-completed-download-result-boundary`

## Next task

T042 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no external search provider;
- no AI provider;
- no FFmpeg execution from the desktop UI;
- no completed-result surfacing to frontend yet;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T042 head.
