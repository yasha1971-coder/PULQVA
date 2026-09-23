# T050 — Resolve app-owned runtime root from desktop application data

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Replaced the fixed relative runtime root with backend-only operating-system application-data resolution.

The async completed-file command receives Tauri's injected `AppHandle` and resolves the application
data directory through `app.path().app_data_dir()`. Resolution failures map to typed
`app-data-path-resolution-failed`.

A backend-owned `runtime` child beneath the resolved application data directory is validated through
the existing T049 `AppRuntimeLayout`. All Arti, yt-dlp, FFmpeg, Tor config/cache/state, and download
output paths continue to derive from that layout.

Frontend input remains query text plus candidate locator only. No filesystem/runtime path, executable
path, proxy/SOCKS value, media URL, argv, or process identifier is accepted from frontend input.

No external network access, runtime binary materialization, Tauri bundle resource, installer, or
packaging work was introduced.

## Verification

PR #50 verified head:
`aa27bdaeb20a2db4f0c55f5a530aa7078964233d`

All 11 required workflows passed.
