# T051 — Add app-owned runtime directory preparation boundary

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a local-filesystem-only app-owned runtime directory preparation boundary.

The verified `AppRuntimeLayout` now drives idempotent preparation of the runtime root, Tor config
parent, Tor cache/state, and download output directories before any sidecar process launch.

Preparation rejects symlink or non-directory traversal inside the owned runtime tree and maps
filesystem failures to typed `runtime-directory-preparation-failed`.

No Arti, yt-dlp, or FFmpeg executable is created, copied, downloaded, or modified. During T051 the
previous executable/state path collision was also removed: sidecar executable paths now derive under
`runtime/bin/*`, while Tor state remains under `runtime/arti/*`. T051 deliberately does not create
the `bin` directory.

Frontend code still cannot supply filesystem/runtime paths. No external network access, bundle
activation, installer, or packaging work was introduced.

## Verification

PR #51 verified head:
`e00fdc496468595e7d6b9897e18d7b0f44449ff0`

All 11 required workflows passed.
