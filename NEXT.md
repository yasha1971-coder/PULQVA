# NEXT

## Current verified state

T050 is complete.

Merged T050 main:
`7868040fe425822906a8f50e634f6eee6af33ecc`

## Active atomic task

**T051 — Add app-owned runtime directory preparation boundary**

T051 prepares only the local directory tree needed before sidecar launch:

- input is the verified `AppRuntimeLayout`;
- runtime root, Tor config parent, Tor cache/state, and download output directories are created locally;
- preparation is idempotent and rejects symlink/non-directory traversal inside the owned runtime tree;
- filesystem failures map to typed `runtime-directory-preparation-failed`;
- directory preparation runs inside the existing blocking backend task before T047 can launch Arti;
- no Arti, yt-dlp, or FFmpeg executable is created, copied, downloaded, or modified;
- the executable layout collision found at T051 start is removed: sidecar executable paths now live under
  `runtime/bin/*`, while Tor state remains under `runtime/arti/*`;
- the `bin` directory itself is not created by T051;
- frontend still supplies no filesystem/runtime path;
- no external network access, bundle activation, installer, or packaging work is introduced.

Branch:
`task/T051-app-owned-runtime-directory-preparation-boundary`

## Next task

T051 remains next until its exact head is verified green and closed.

## Do not do yet

- no sidecar binary materialization;
- no packaging/release installers;
- no Tauri bundle resources;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Desktop/privacy/media/remux checks are green for the exact T051 head and the app-owned runtime
directory tree is prepared deterministically before any sidecar process launch without materializing binaries.
