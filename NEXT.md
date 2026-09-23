# NEXT

## Current verified state

T050 is complete.

PULQVA now resolves the app-owned runtime root through the desktop application's OS data directory:

- `download_completed_file` receives Tauri's injected `AppHandle`;
- `app.path().app_data_dir()` resolves the OS-appropriate application data directory;
- resolution failure maps to typed `app-data-path-resolution-failed`;
- a backend-owned `runtime` child is passed through the verified T049 `AppRuntimeLayout`;
- Arti, yt-dlp, FFmpeg, Tor config/cache/state, and output paths continue to derive only from that layout;
- frontend supplies no filesystem/runtime path;
- completed-file output remains only sanitized data;
- no network access, runtime binary materialization, bundle resource, installer, or packaging work is introduced.

Verified PR head:
`aa27bdaeb20a2db4f0c55f5a530aa7078964233d`

All 11 required workflows passed for that exact head.

## Next atomic task

**T051 — Add app-owned runtime directory preparation boundary**

Prepare only the directory structure required by the verified runtime layout before any sidecar process launch.

Required boundary:

- input is a verified `AppRuntimeLayout`;
- create only the app-owned runtime root, Tor config/cache/state parent directories, and download output directory;
- do not create, copy, download, or modify Arti, yt-dlp, or FFmpeg executables;
- preparation is local-filesystem-only and performs no external network access;
- preparation is idempotent;
- any filesystem failure is typed and fail-closed;
- all created directories must remain beneath the verified app-owned runtime root;
- frontend supplies no filesystem path;
- no process launch, bundle activation, installer, or packaging work is added.

## Do not do yet

- no sidecar binary materialization;
- no packaging/release installers;
- no Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The backend can prepare the verified app-owned runtime directory tree deterministically before future
local sidecar materialization, without giving the frontend filesystem control.
