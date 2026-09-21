# T030 — Prove one real local FFmpeg remux

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Proved one real bounded local stream-copy remux with the actual pinned FFmpeg/ffprobe sidecars.

The validated local MP4 enters through `CompletedDownloadResult`, execution starts only from
`FfmpegRemuxPlan`, FFmpeg is restricted to the `file` protocol, stream codecs are copied with no
re-encoding, and non-media hint/data tracks are dropped explicitly. Windows and Linux both pass the
same proof and pinned ffprobe validates the Matroska output.

## Verification

PR #29 verified head:
`378ef54cb5bf9cec834e40f826a89b5ffb0913fa`

All existing checks passed, including `ffmpeg-real-remux-check`.
