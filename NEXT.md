# NEXT

## Current verified state

T049 is complete.

PULQVA now has one typed app-owned desktop runtime layout boundary:

- `AppRuntimeLayout` owns the backend runtime root;
- empty roots and roots containing parent-directory traversal fail closed;
- Arti, yt-dlp, FFmpeg, Tor config/cache/state, and download output paths are derived only from that root;
- derived child paths must be non-empty relative normal-component paths, stay beneath the root, and contain no parent traversal;
- T048 completed-file command preparation consumes the typed layout;
- the data-only Download planning path also reuses the same layout;
- frontend supplies no filesystem/runtime path;
- no process launch, external network access, installer, or bundle activation is introduced by the layout.

Verified PR head:
`44955ca8cb015bbb5dde061bb1ae041b22bd952d`

All 11 required workflows passed for that exact head.

## Next atomic task

**T050 — Resolve the app-owned runtime root from the desktop application data directory**

Replace the remaining fixed relative runtime root with one backend-resolved OS application-data root.

Required boundary:

- derive the root through Tauri desktop path resolution, never frontend input;
- create a PULQVA-owned runtime child beneath the resolved application data directory;
- feed that root into the verified T049 `AppRuntimeLayout`;
- path resolution itself performs no external network access;
- path-resolution failure is typed and fail-closed;
- no user-supplied filesystem path, executable path, proxy/SOCKS value, media URL, argv, or process identifier is accepted;
- command output remains only the sanitized completed-file view;
- no runtime binary materialization, packaging, or installer work yet.

## Do not do yet

- no runtime binary materialization;
- no packaging/release installers;
- no Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The desktop command resolves its runtime root from an OS-appropriate app-owned data location and still
feeds all runtime paths through the verified T049 layout without exposing filesystem control to the frontend.
