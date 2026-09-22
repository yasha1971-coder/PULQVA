# NEXT

## Current verified state

T042 is complete.

PULQVA now has a backend-only completed download result boundary:

- input is a T041 `RunningMediaDownloadRuntime`;
- yt-dlp completion reuses `RunningYtDlp::complete_download`;
- successful completion yields the existing typed `CompletedDownloadResult`;
- Arti is stopped and waited after yt-dlp completion on both success and failure paths;
- yt-dlp completion failures fail closed;
- Arti cleanup failure after successful download invalidates the boundary result and fails closed;
- combined completion and cleanup failure preserves both causes;
- frontend IPC/output remains unchanged and exposes no backend paths, SOCKS/proxy data, media URL, argv, process identifiers, or raw completion internals;
- no FFmpeg process is started;
- no direct-network fallback exists.

Verified PR head:
`2202145197ec65089de3452d36c000957b6eb521`

All 11 required workflows passed for that exact head.

## Next atomic task

**T043 — Add the backend-only FFmpeg remux planning boundary**

Convert one verified T042 `CompletedDownloadResult` into the existing typed local FFmpeg remux plan
without starting FFmpeg.

Required boundary:

- input is a verified `CompletedDownloadResult`;
- FFmpeg executable path is explicit backend input;
- output path is derived inside the backend from the validated completed artifact;
- `FfmpegRemuxPlan` is built through the existing typed privacy-layer API;
- the plan is local-file-only and inherits the existing `file` protocol whitelist;
- invalid executable/output/path traversal/equal-input cases fail closed;
- frontend receives no executable path, input/output filesystem path, argv, source URL, or process identifier;
- no FFmpeg process is started;
- no external network access occurs.

## Do not do yet

- no FFmpeg execution from the desktop UI;
- no completed-result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

A completed validated download can be converted into a deterministic typed local remux plan without
starting FFmpeg or exposing backend runtime details to frontend code.
