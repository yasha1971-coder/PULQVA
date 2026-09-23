# NEXT

## Current verified state

T051 and recovery hardening T051R1 are complete.

Merged main:
`34eae0f3075c9c8d3116eeac2def0d689a3a68b5`

## Active atomic task

**T052 — Add a typed local sidecar materialization plan boundary**

T052 defines a pure-data backend-owned plan for the three runtime sidecars without copying bytes:

- exactly Arti, yt-dlp, and FFmpeg are represented;
- logical bundle-resource identifiers are backend constants and are never frontend input;
- pinned versions are loaded from the repository sidecar VERSION files;
- yt-dlp and FFmpeg source provenance carries the already pinned SHA-256 from repository manifests;
- runtime destinations derive only from `AppRuntimeLayout` and are direct children of `runtime/bin`;
- Windows destinations use `.exe`; Linux destinations remain extensionless;
- supported plan targets are Windows x86_64 and Linux x86_64;
- source resource identifiers and runtime destinations are required to differ;
- planning is side-effect free: no directory/file creation, copying, download, chmod, process launch, shell command, or network access;
- Tauri bundle resources remain inactive.

Branch:
`task/T052-typed-local-sidecar-materialization-plan-boundary`

## Next task

T052 remains next until its exact head is verified green and closed.

## Do not do yet

- no sidecar binary copy/materialization;
- no Tauri bundle-resource activation;
- no packaging/release installers;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Desktop/privacy/media/remux checks are green on the exact T052 head and the backend can produce one
deterministic typed sidecar plan whose destinations are confined to the app-owned runtime tree.
