# T049 — Add an app-owned desktop runtime layout boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added one typed backend-owned desktop runtime layout.

`AppRuntimeLayout` owns a single runtime root. Empty roots and roots containing parent-directory
traversal fail closed. Arti, yt-dlp, FFmpeg, Tor config/cache/state, and download output paths are all
derived from that root.

Derived child paths are required to be non-empty relative normal-component paths, remain beneath the
runtime root, and contain no parent traversal.

T048 completed-file command preparation now builds runtime inputs through this layout, and the existing
data-only Download planning path reuses the same layout. Frontend code cannot supply or override the
runtime root or any derived runtime path.

No process launch, external network access, installer, or Tauri bundle activation was introduced.

## Verification

PR #49 verified head:
`44955ca8cb015bbb5dde061bb1ae041b22bd952d`

All 11 required workflows passed.
