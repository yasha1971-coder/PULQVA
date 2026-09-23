# T055 — Atomically materialize the verified yt-dlp direct-binary artifact

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-23

## Result

Added a backend-only local publication boundary for the pinned yt-dlp direct executable.

The materializer:

- accepts only a T054-verified yt-dlp artifact plus the prepared app-runtime capability;
- revalidates backend-pinned version and SHA-256 identity;
- revalidates source/resource-root containment, file type, receipt size, and destination layout;
- rejects unsupported kinds, malformed identities, escaped paths, symlink hazards, and hard-link
  source/destination aliases;
- preserves differing or unsafe existing destinations without modification;
- reuses an existing matching verified executable;
- copies in bounded buffers into an exclusively created staging file and hashes the actual bytes
  copied;
- applies Unix executable permissions only to the owned staging file;
- synchronizes the complete staging file before publication;
- publishes with a no-clobber same-directory hard link and no overwrite fallback;
- checks staging ownership before cleanup;
- supports competing publishers converging on one verified destination;
- performs no shell execution, network access, process launch, or frontend filesystem operation.

The first IMPLEMENT head exposed a Windows lexical-vs-canonical resource-root mismatch. The repaired
head canonicalizes the trusted resource root before component inspection while retaining final
canonical containment and fail-closed symlink checks.

## Verification

Verified implementation head:
`05c37563301c0d3d953909f4d45b4642442c6d02`

All 11 required workflows passed for that exact head, including Windows and Linux desktop tests.

## Explicit trust limits retained

- parent-directory replacement races after validation are not claimed fully solved;
- ordinary Windows CI is not adversarial reparse-point proof;
- unsupported hard-link filesystems fail closed;
- file synchronization is not a universal directory-metadata durability guarantee;
- no real release package resource was installed by this task;
- Arti and FFmpeg remain outside this direct-binary materialization boundary.
