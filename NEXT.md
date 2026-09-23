# NEXT

## Verified implementation; closeout pending

T054 implementation is verified at `a1caa4eb310e472333eb4263223b28c0ff22327b`.
All 11 triggered workflows passed for that exact head, including Windows/Linux desktop tests.
The task record is DONE for that implementation; the closeout commit still requires its own CI.
`PROJECT_STATE.ci` deliberately remains `pending` until the new head is observed green.

PR: #56
Branch: `task/T054-validate-packaged-sidecar-source-artifacts`
Main observed before closeout: `3df2c2eefaf64c3398c1d04568c889ff1b3d576f`
The latest PR #56 checkpoint records the exact closeout SHA and observed CI state.

## What T054 implements

The backend validates resolved packaged sources before runtime-directory preparation or sidecar
launch. The package root must be a real directory. Source components reject symlinks and invalid
file types; canonical source paths must remain beneath the canonical resource root. Exactly one
Arti, yt-dlp, and FFmpeg entry is required. yt-dlp and FFmpeg inputs are hashed locally in bounded
buffers and compared to their pinned source digests. Typed results retain kind, canonical source,
destination, identity, and size. Validation does not create or modify runtime destinations.

## Continuity notes that must not be lost

- FFmpeg's pinned digest authenticates its platform archive, NOT the extracted executable. The
  packaged source is now the exact `.tar.xz` or `.zip` archive; its eventual destination is still
  `runtime/bin/ffmpeg(.exe)`. Never copy archive bytes directly to that executable destination.
- Arti currently retains version metadata and file/path validation only. It has no pinned content
  digest here, and no version command runs during validation. Do not describe this as binary
  authentication or use the stored version string as proof of executable contents.
- T054 returns paths, not immutable file snapshots. A later materializer must revalidate the bytes
  it actually copies; an earlier successful hash alone cannot authenticate a changed file.
- Existing runtime executables are not authenticated or replaced by T054. Materialization, secure
  publication, archive extraction, and release packaging remain separate work.
- Regression tests added by T054 cover fixture success, hash mismatch, and non-file rejection.
  Do not claim a real packaged three-sidecar installation was exercised by those fixture tests.

## Immediate next action

Inspect PR #56's current head and all triggered checks. Merge only when that exact closeout head
is green and unchanged. On failure, diagnose only the concrete failure. Do not begin T055 before
T054 closes and merges.

## Next atomic task after merge

**T055 — Atomically materialize the verified yt-dlp direct-binary artifact**

Implement one backend-only local publication boundary for yt-dlp, whose pinned digest describes
its direct executable bytes. Reuse the prepared runtime layout and T054 verified input; revalidate
source containment and the bytes being copied, publish only a fully verified staged file under
`runtime/bin`, and preserve existing files on failure. Reject archives and unsupported sidecar
kinds at this boundary. The detailed READY task defines the tests and scope.

T055 is queued, not started. Arti authentication and FFmpeg archive extraction are not silently
folded into this task. No frontend filesystem authority, network fallback, bundle activation,
installer, external search provider, or AI provider is introduced by this closeout.
