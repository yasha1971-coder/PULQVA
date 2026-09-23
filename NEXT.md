# NEXT

## Current verified state

T051 is complete.

PULQVA now has an app-owned runtime directory preparation boundary:

- input is the verified `AppRuntimeLayout`;
- the runtime root, Tor config parent, Tor cache/state, and download output directories are prepared locally;
- preparation is idempotent;
- symlink/non-directory traversal inside the owned runtime tree fails closed;
- filesystem failures map to typed `runtime-directory-preparation-failed`;
- preparation runs in the backend blocking task before any sidecar launch;
- Arti, yt-dlp, and FFmpeg executables are not created, copied, downloaded, or modified;
- executable paths are separated under `runtime/bin/*`;
- Tor state remains under `runtime/arti/*`;
- T051 deliberately does not create `runtime/bin`;
- frontend supplies no filesystem/runtime path;
- no external network access, bundle activation, installer, or packaging work is introduced.

Verified PR head:
`e00fdc496468595e7d6b9897e18d7b0f44449ff0`

All 11 required workflows passed for that exact head.

## Next atomic task

**T052 — Add a typed local sidecar materialization plan boundary**

Define, without copying binaries yet, the exact backend-owned mapping from verified packaged sidecar
sources to the T051 `runtime/bin/*` destinations.

Required boundary:

- plan covers Arti, yt-dlp, and FFmpeg only;
- each destination comes only from the verified `AppRuntimeLayout`;
- each source is backend-owned packaged-resource metadata, never frontend input;
- source and destination must be distinct;
- destination must remain beneath `runtime/bin`;
- no shell command is constructed;
- no process launch occurs;
- no external network access occurs;
- no binary copy/materialization occurs yet;
- no direct-network fallback is introduced.

## Do not do yet

- no sidecar binary copy/materialization;
- no packaging/release installers;
- no active Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The backend can produce one deterministic typed sidecar materialization plan for Arti, yt-dlp, and
FFmpeg, with all runtime destinations confined to the app-owned runtime tree and no frontend control.
