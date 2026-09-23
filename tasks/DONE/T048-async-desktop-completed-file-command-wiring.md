# T048 — Add async desktop completed-file command wiring

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added async Tauri command wiring for the verified T047 completed-file pipeline.

Frontend input is limited to natural-language query text plus candidate locator. Both are revalidated
before any runtime work. Unsupported candidates fail before a blocking task starts.

All runtime inputs remain backend-owned: Arti, yt-dlp, FFmpeg, Tor config/cache/state, download output
root, SOCKS port, and readiness timeout are constructed internally. No executable path, filesystem
root, proxy/SOCKS value, media URL, argv, or process identifier is accepted from frontend input.

The `download_completed_file` command is async and moves the existing synchronous T047 pipeline to
`tauri::async_runtime::spawn_blocking`, keeping blocking process work off the UI thread. Join failure
maps to typed `completed-file-task-join-failed`. Successful output is only the sanitized
`CompletedFileView`.

No duplicate direct network/process implementation and no direct-network fallback were introduced.

## Verification

PR #48 verified head:
`12f4306a027de390dd136c13e302be2ada6c7d1e`

All 11 required workflows passed.
