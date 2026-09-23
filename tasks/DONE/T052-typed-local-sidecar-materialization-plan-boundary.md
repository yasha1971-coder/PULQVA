# T052 — Add a typed local sidecar materialization plan boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a pure-data backend-owned materialization plan for exactly three local sidecars: Arti, yt-dlp,
and FFmpeg.

Logical packaged-resource identifiers are backend constants. Runtime destinations derive only from
the verified `AppRuntimeLayout` and remain direct children of `runtime/bin`.

Pinned versions are loaded from the repository sidecar VERSION files. yt-dlp and FFmpeg plan entries
also retain their existing repository-pinned SHA-256 identities.

Windows executable destinations use `.exe`; Linux executable destinations remain extensionless.
Source identifiers and runtime destinations must differ.

The planning boundary creates no directories or files, copies no bytes, performs no chmod, launches
no process, constructs no shell command, and makes no network connection. Tauri bundle resources
remain inactive.

## Verification

PR #54 verified head:
`d21e589bac124426514f930f81b079cd5baf4050`

All 11 required workflows passed.
