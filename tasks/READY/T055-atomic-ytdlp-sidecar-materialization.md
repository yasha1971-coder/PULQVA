# T055 — Atomically materialize the verified yt-dlp direct-binary artifact

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Publish one verified packaged yt-dlp executable into its app-owned runtime destination without
exposing partial bytes or overwriting an unrelated existing file. This is a backend-only local
materialization boundary, not a release or a live command-wiring task.

## Acceptance criteria

- Input is the prepared app-runtime-directory capability plus the T054 verified yt-dlp artifact;
  the destination must equal the layout's platform-specific yt-dlp destination under runtime/bin.
- Only the yt-dlp direct-binary kind is supported. Reject Arti, FFmpeg archives, missing digests,
  invalid identities, source/destination aliases, and paths outside the verified roots before copy.
- Recheck source and destination containment and reject symlink/non-directory hazards. Preserve
  the T054 distinction between archive and direct-executable digests.
- Copy locally in bounded buffers into a uniquely created staging file inside the validated bin
  directory. Check the bytes actually copied against the pinned direct-binary SHA-256; do not
  rely only on an earlier path-based validation. No shell, network request, or process launch.
- Publish only a complete verified staged file, using an atomic no-clobber strategy. Flush data
  before publication. On Unix, apply executable permissions only to the app-owned staged file.
- Existing regular destinations with the same verified content may be reused after validation.
  Reject differing or unsafe existing destinations without modifying them.
- Clean up only staging files created by this operation on failure. Never delete user files or
  report success for a partial destination. Return a typed backend-only materialized result.
- Deterministic Windows/Linux tests cover success, repeated use, hash mismatch, source changes
  after T054 validation, unsafe paths, unsupported artifact kinds, and existing-file preservation.

## Trust limits

Arti's version string is not a content digest. T054's FFmpeg digest authenticates an archive, not
its extracted executable. Neither can be fed into this direct yt-dlp publication boundary.
Do not call an untested path-race defense complete; retain any remaining limitation explicitly.

## Out of scope

Arti materialization/authentication, FFmpeg archive extraction, running the materialized executable,
changes to the live download command, Tauri bundle activation, installers, external providers,
and any direct-network fallback. Do not start this task until PR #56's closeout is green and merged.
