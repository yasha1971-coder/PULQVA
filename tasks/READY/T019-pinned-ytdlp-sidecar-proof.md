# T019 — Pin and prove the yt-dlp standalone sidecar

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Source-verify the current official yt-dlp standalone release, pin the exact version, and prove the
standalone binary CLI on Windows and Linux before any media download behavior is added.

## Acceptance criteria

- release/version is verified from the official yt-dlp project;
- repository pins one exact version;
- Windows standalone binary is verified;
- Linux standalone binary is verified;
- version output matches the pin;
- basic help/CLI invocation succeeds;
- no media URL is requested;
- no external content download is attempted by PULQVA;
- existing privacy checks remain green.

## Out of scope

- media downloads;
- FFmpeg;
- AI;
- UI;
- any direct-network fallback.
