# NEXT

## Current verified state

T042 is complete.

Verified T042 implementation head:
`2202145197ec65089de3452d36c000957b6eb521`

Merged T042 main:
`5c7de5186e7b8152310de5ddc40805c7bd4311f3`

## Active atomic task

**T043 — Add the backend-only FFmpeg remux planning boundary**

T043 converts a verified completed download into a deterministic local remux plan:

- input is a verified `CompletedDownloadResult`;
- FFmpeg executable path is an explicit backend input;
- remux output is derived from the validated artifact as a sibling `pulqva-remux-*.mp4` path without UTF-8 filename assumptions;
- the typed plan reuses `FfmpegRemuxPlan` with explicit `FfmpegRemuxContainer::Mp4`;
- existing typed plan validation rejects missing executable/output, traversal, and output-equals-input cases fail-closed;
- plan arguments retain the existing local-only `file` protocol whitelist;
- frontend IPC/output remains unchanged and exposes no executable path, input/output filesystem path, argv, source URL, or process identifier;
- no FFmpeg process is spawned;
- no external network access occurs.

Branch:
`task/T043-ffmpeg-remux-planning-boundary`

## Next task

T043 remains the next task until its exact head is verified green and closed.

## Do not do yet

- no FFmpeg execution from the desktop UI;
- no completed-result surfacing to frontend;
- no external search provider;
- no AI provider;
- no packaging/release installers;
- no direct-network fallback.

## Success

Windows/Linux desktop checks and all existing privacy/media checks are green for the exact T043 head.
