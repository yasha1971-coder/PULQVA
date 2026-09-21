# T027 — Pin and prove the FFmpeg/ffprobe sidecar

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Pin a reproducible FFmpeg/ffprobe sidecar identity for Windows and Linux and prove the exact local
CLI before allowing any media transformation or post-processing.

## Acceptance criteria

- exact upstream FFmpeg source commit is pinned:
  `a5923073bfd8f25b7300d93af3f8e690174ebd30`;
- exact version marker is `n9.0.2-3-ga5923073bf`;
- dated binary build snapshot is pinned to
  `BtbN/FFmpeg-Builds` tag `autobuild-2026-09-20-13-11`;
- Linux and Windows static archive SHA-256 values are frozen in-repo;
- archives are integrity-verified before extraction or execution;
- `ffmpeg -version` succeeds and reports the pinned version marker;
- `ffprobe -version` succeeds and reports the pinned version marker;
- no media URL is requested;
- no media transformation is performed;
- no network route is introduced into runtime code;
- existing privacy/media checks remain green.

## Out of scope

- actual FFmpeg transcoding/remuxing;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
