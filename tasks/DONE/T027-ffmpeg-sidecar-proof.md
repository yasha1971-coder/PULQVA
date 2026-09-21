# T027 — Pin and prove the FFmpeg/ffprobe sidecar

Parent: MEDIA FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Pinned the exact FFmpeg source identity and one dated Windows/Linux static build snapshot.

The Linux and Windows archives are SHA-256 verified before extraction/execution. CI proves local
`ffmpeg -version` and `ffprobe -version` availability with the pinned version marker. No media
URL, transformation, or runtime network route was added.

## Verification

PR #26 verified head:
`84cd0dca0e6610cabec9f42a1f9905065856fe25`

All existing checks passed, including `ffmpeg-sidecar-check`.
