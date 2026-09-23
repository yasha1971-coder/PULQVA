# NEXT

## Current verified state

T048 is complete.

Merged T048 main:
`b831df242a074df6c28935453799ca6acdab0af3`

## Active atomic task

**T049 — Add an app-owned desktop runtime layout boundary**

T049 centralizes backend runtime filesystem ownership:

- `AppRuntimeLayout` owns one backend-only runtime root;
- empty roots and roots containing parent-directory traversal fail closed;
- Arti, yt-dlp, FFmpeg, Tor config/cache/state, and download output paths are derived only from this root;
- derived child paths are non-empty relative normal-component paths, remain beneath the root, and contain no parent traversal;
- T048 command preparation now builds `CompletedFilePipelineInputs` from the typed layout;
- the existing data-only Download planning path also reuses the same layout;
- frontend still supplies no filesystem/runtime path;
- no process launch, external network access, installer, or bundle activation is introduced by the layout.

Branch:
`task/T049-app-owned-desktop-runtime-layout-boundary`

## Next task

T049 remains next until its exact head is verified green and closed.

## Do not do yet

- no runtime binary materialization;
- no packaging/release installers;
- no Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Desktop/privacy/media/remux checks are green for the exact T049 head and all command runtime paths flow
through one typed backend-owned root.
