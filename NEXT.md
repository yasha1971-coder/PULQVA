# NEXT

## Current verified state

T053 is complete.

Verified T053 closeout head:
`17959d1943b574e47ff28ab59b39f144fdd52e84`

All 11 required workflows passed for that exact head. T053 is merged on main at
`3df2c2eefaf64c3398c1d04568c889ff1b3d576f`.

## Active atomic task

**T054 — Validate packaged sidecar source artifacts before materialization**

T054 adds a local-only fail-closed verification capability over the T053 resolved package sources:

- package resource root must exist as a real directory and not a symlink;
- every path component below the resource root is inspected without following symlinks;
- source leaf must be a real regular file;
- canonical source paths must remain beneath the canonical package resource root;
- yt-dlp and FFmpeg sources must match their repository-pinned SHA-256 identities;
- Arti retains its pinned version identity and receives the same path/file-type containment checks;
- verified artifacts retain kind, canonical source, runtime destination, identity, and byte size;
- the real completed-file backend validates package sources before runtime-directory preparation or
  any sidecar process launch;
- no runtime destination is created or modified by validation;
- no chmod, process launch, shell execution, or network access is introduced.

T054 also corrects one identity mismatch discovered at task start: the FFmpeg pinned SHA-256 belongs
to the pinned platform archive, not the extracted `ffmpeg` executable. Therefore the FFmpeg logical
package source now names the exact pinned archive while the runtime destination remains
`runtime/bin/ffmpeg(.exe)`. Future materialization must extract the verified archive rather than
pretend its archive digest authenticates an extracted binary.

Branch:
`task/T054-validate-packaged-sidecar-source-artifacts`

## Next task

T054 remains next until its exact head is verified green and closed.

## Do not do yet

- no sidecar binary copy/extraction/materialization;
- no Tauri bundle-resource activation;
- no installer/release packaging;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

Windows/Linux desktop boundary tests and all existing privacy/media checks are green on the exact
T054 head, and missing, symlinked, escaped, non-file, or hash-mismatched packaged sidecar sources fail
closed before any runtime executable destination is touched.
