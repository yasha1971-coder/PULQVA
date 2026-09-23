# NEXT

## Current verified state

T052 is complete.

PULQVA now has a typed, backend-owned, side-effect-free sidecar materialization plan:

- exactly Arti, yt-dlp, and FFmpeg are represented;
- logical packaged-resource identifiers are backend constants and never frontend input;
- pinned versions come from repository sidecar VERSION files;
- yt-dlp and FFmpeg identities include the repository-pinned SHA-256 values;
- runtime destinations derive only from `AppRuntimeLayout`;
- destinations are direct children of `runtime/bin`;
- Windows destinations use `.exe`, Linux destinations remain extensionless;
- source identifiers and runtime destinations are required to differ;
- planning creates no file/directory, copies no bytes, launches no process, builds no shell command, and performs no network access;
- Tauri bundle resources remain inactive.

Verified PR head:
`d21e589bac124426514f930f81b079cd5baf4050`

All 11 required workflows passed for that exact head.

## Next atomic task

**T053 — Resolve packaged sidecar source root from the Tauri resource directory**

Bind the T052 logical packaged-resource identifiers to one backend-owned OS package resource root
without copying bytes yet.

Required boundary:

- resolve the package resource directory only from Tauri's injected `AppHandle`;
- frontend supplies no resource/filesystem path;
- join T052 logical source identifiers beneath that resolved resource root;
- reject parent traversal and any source path that escapes the verified resource root;
- preserve the T052 pinned identity/version metadata;
- keep runtime destinations unchanged under `runtime/bin/*`;
- produce typed source filesystem paths only;
- no file copy/materialization;
- no chmod;
- no process launch;
- no external network access;
- no direct-network fallback.

## Do not do yet

- no sidecar binary copy/materialization;
- no installer/release packaging;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The backend can deterministically resolve all three packaged sidecar source paths from the desktop
application's own resource directory while retaining the T052 identities and app-owned runtime
destinations, without frontend filesystem control or side effects.
