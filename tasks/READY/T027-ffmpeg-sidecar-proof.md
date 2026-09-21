# T027 — Pin and prove the FFmpeg/ffprobe sidecar

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Pin a reproducible FFmpeg/ffprobe sidecar source for Windows and Linux and prove the exact local CLI
before allowing any media transformation or post-processing.

## Acceptance criteria

- one exact FFmpeg build/version source is documented and pinned;
- Windows and Linux binary artifacts are integrity-verified before execution;
- `ffmpeg -version` succeeds;
- `ffprobe -version` succeeds;
- no media URL is requested;
- no media transformation is performed;
- no network route is introduced into runtime code;
- existing privacy/media checks remain green.

## Out of scope

- actual FFmpeg transcoding/remuxing;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
