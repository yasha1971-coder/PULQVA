# NEXT

## Current verified state

T051 is complete and its recovery hardening T051R1 is also complete.

PULQVA now has the verified T051 runtime-directory boundary plus additional hardening:

- prepared runtime directories are canonicalized and verified to remain physically beneath the verified runtime root;
- existing symlink and non-directory hazards fail closed;
- executable destinations remain separated under `runtime/bin/*`;
- Tor state/config/cache remain under `runtime/arti/*`;
- directory preparation remains idempotent and local-filesystem-only;
- `desktop-shell-check` now executes desktop boundary tests on both Windows and Linux, not only compilation;
- no sidecar executable is created, copied, downloaded, or modified;
- no privacy/network invariant was weakened.

Verified recovery head:
`ca8755cf59da63cdf1a25e5ddf54891e368b525d`

All 11 required workflows passed for that exact head.

The stale parallel PR #52 was closed as superseded.

## Next atomic task

**T052 — Add a typed local sidecar materialization plan boundary**

Define, without copying binaries yet, the exact backend-owned mapping from verified packaged sidecar
sources to the T051 `runtime/bin/*` destinations.

## Do not do yet

- no sidecar binary copy/materialization;
- no Tauri bundle-resource activation;
- no packaging/release installers;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The backend can produce one deterministic typed sidecar materialization plan for Arti, yt-dlp, and
FFmpeg, with all runtime destinations confined to the app-owned runtime tree and no frontend control.
