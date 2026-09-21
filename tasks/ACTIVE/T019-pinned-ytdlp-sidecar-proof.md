# T019 — Pin and prove the yt-dlp standalone sidecar

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Source-verify the current official yt-dlp standalone release, pin the exact version and asset
digests, and prove the standalone CLI on Windows and Linux before any media request exists.

## Acceptance criteria

- current release is read from the official `yt-dlp/yt-dlp` GitHub release;
- repository pins exactly `2026.08.19`;
- official Linux `yt-dlp_linux` SHA-256 is pinned and checked;
- official Windows `yt-dlp.exe` SHA-256 is pinned and checked;
- version output exactly matches the pin;
- `--ignore-config --help` succeeds;
- no media URL is supplied;
- no external media content is requested by PULQVA;
- existing privacy checks remain green.

## Out of scope

- actual media download;
- product-runtime yt-dlp spawning;
- FFmpeg;
- AI;
- UI;
- direct-network fallback.
