# NEXT

## Current verified state

T049 is complete.

Merged T049 main:
`ab295eb156fda55762a8673819cbe18e90fa57a0`

## Active atomic task

**T050 — Resolve the app-owned runtime root from the desktop application data directory**

T050 replaces the fixed relative runtime root with backend-only OS path resolution:

- `download_completed_file` receives Tauri's injected `AppHandle`, not a frontend filesystem value;
- `app.path().app_data_dir()` resolves the OS-appropriate app data directory;
- path-resolution failure maps to typed `app-data-path-resolution-failed`;
- a `runtime` child beneath that resolved app data directory is passed through the verified T049
  `AppRuntimeLayout::new`;
- all Arti, yt-dlp, FFmpeg, Tor config/cache/state, and output paths continue to derive from that layout;
- the data-only Download planning command no longer needs a runtime filesystem root at all;
- frontend still supplies only query text plus candidate locator for completed-file execution;
- command output remains only sanitized completed-file data;
- no network access, runtime binary materialization, bundle resource, installer, or packaging work is introduced.

Branch:
`task/T050-resolve-app-owned-runtime-root`

## Next task

T050 remains next until its exact head is verified green and closed.

## Do not do yet

- no runtime binary materialization;
- no packaging/release installers;
- no Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Desktop checks are green for the exact T050 head and completed-file execution resolves its runtime
root from Tauri's application data directory without exposing filesystem control to the frontend.
