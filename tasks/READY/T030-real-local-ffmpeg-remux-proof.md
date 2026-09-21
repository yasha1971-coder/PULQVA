# T030 — Prove one real local FFmpeg remux

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Run the actual pinned FFmpeg sidecar through the typed local process boundary on one validated local
media artifact and prove a bounded stream-copy remux.

## Acceptance criteria

- actual pinned FFmpeg/ffprobe sidecar snapshot is used;
- FFmpeg execution starts only from `FfmpegRemuxPlan`;
- input comes from a validated `CompletedDownloadResult`;
- input is local only;
- protocol whitelist remains `file`;
- remux uses stream copy and performs no re-encoding;
- execution is bounded by an explicit timeout;
- successful output is a non-empty regular file distinct from input;
- ffprobe validates the local remuxed output;
- no network input or direct-network fallback exists;
- Windows and Linux behavior is recorded honestly;
- all existing checks remain green.

## Out of scope

- transcoding/re-encoding;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
