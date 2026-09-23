# T051 — Add app-owned runtime directory preparation boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Prepare the local directory tree required by the verified T049/T050 app-owned runtime layout without
materializing any sidecar executable.

## Acceptance criteria

- input is a verified `AppRuntimeLayout`;
- create the runtime root if missing;
- create only required Tor config/cache/state parent directories and download output directory;
- no Arti, yt-dlp, or FFmpeg executable is created, copied, downloaded, or modified;
- every created path remains beneath the verified runtime root;
- preparation is idempotent;
- filesystem failures map to typed fail-closed desktop errors;
- no process is launched;
- no external network access occurs;
- frontend provides no filesystem path or runtime location;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- sidecar binary materialization;
- Tauri bundle resources;
- packaging/release installers;
- external candidate search;
- AI provider integration.
