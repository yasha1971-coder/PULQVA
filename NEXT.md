# NEXT

## Current verified state

T050 is complete.

PULQVA resolves one backend-owned runtime root beneath the desktop application's OS application-data
directory. Frontend input cannot supply any runtime, executable, Tor, output, or filesystem path.

Verified PR head:
`aa27bdaeb20a2db4f0c55f5a530aa7078964233d`

All 11 required workflows passed for that exact head.

## Active atomic task

**T051 — Add app-owned runtime directory preparation boundary**

T051 prepares only the local directory tree required before future sidecar materialization:

- input is the verified `AppRuntimeLayout`;
- runtime root is created if missing;
- only Tor config/cache/state directory structure and download output directory are created;
- existing symlink and non-directory hazards inside the runtime root fail closed before creation;
- prepared paths are canonicalized and verified beneath the runtime root;
- repeated preparation is idempotent;
- Arti, yt-dlp, and FFmpeg executables are not created, copied, downloaded, or modified;
- executable destinations now live under `runtime/bin/*`, avoiding collision with the `runtime/arti/*`
  Tor state/config directory tree;
- desktop boundary tests execute on both Windows and Linux;
- no process launch or external network access is added.

## Queued next task

**T052 — Add typed bundled-sidecar materialization specification**

Define the backend-only specification that maps pinned Arti, yt-dlp, and FFmpeg bundle resources to
their verified `runtime/bin/*` destinations, including platform-specific names and expected
identities. Keep T052 data-only: do not copy executable bytes or launch processes yet.

## Do not do yet

- no sidecar binary copy/materialization;
- no Tauri bundle-resource activation;
- no packaging/release installers;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The backend can deterministically and idempotently prepare the app-owned runtime directory tree
without creating executable files or allowing any path to escape the verified runtime root.
