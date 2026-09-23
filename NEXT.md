# NEXT

## Current verified state

T053 is complete.

PULQVA now resolves the T052 logical sidecar resource identifiers from the desktop application's own
Tauri package resource directory:

- the real command obtains the resource root only from injected `AppHandle` via
  `app.path().resource_dir()`;
- frontend supplies no resource/filesystem path;
- logical sidecar source identifiers remain backend-owned relative paths;
- empty/traversing package-resource roots fail closed;
- non-normal logical resource paths fail closed;
- resolved source paths must remain beneath the package resource root;
- source and runtime destination must differ;
- T052 sidecar kind/version/SHA-256 identity metadata is preserved;
- runtime destinations remain direct children of `runtime/bin`;
- no source file existence is assumed yet;
- no file copy/write/chmod/process launch/network access or bundle-resource activation is introduced.

Verified PR head:
`fc0f1664e72b37f4d67de24d495d0f2a9c26c0bb`

All 11 required workflows passed for that exact head.

## Next atomic task

**T054 — Validate packaged sidecar source artifacts before materialization**

Add a local-only typed validation boundary over the T053 resolved source paths before any executable
bytes can be copied into `runtime/bin`.

Required boundary:

- input is the T053 resolved sidecar materialization plan;
- each resolved source must exist as a real regular file;
- symlinks and non-files fail closed;
- canonical source paths must remain beneath the verified package resource root;
- yt-dlp and FFmpeg content must match their already pinned SHA-256 identities before becoming valid
  materialization inputs;
- Arti retains the existing pinned version identity and must pass the same path/file-type containment
  checks;
- validation returns typed verified source artifacts while preserving destination and identity data;
- no destination file is created or modified;
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

The backend can fail closed on missing, symlinked, escaped, malformed, or hash-mismatched packaged
sidecar inputs before any executable reaches the app-owned runtime directory.
