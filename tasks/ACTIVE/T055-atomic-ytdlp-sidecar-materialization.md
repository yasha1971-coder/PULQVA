# T055 — Atomically materialize the verified yt-dlp direct-binary artifact

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

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

## Current phase: PREPARE

The separate `t055_publication_contract` integration test target exercises a test-only prototype
of staging and no-clobber publication. It does not implement the production capability boundary.
Its CI result is pending; no local Rust test execution was possible in this session.

## Next IMPLEMENT checklist

1. Preserve the existing desktop API. Add an internal materializer consuming the prepared runtime
   capability and one T054 verified yt-dlp artifact. Resolve expected version/digest from the pinned
   backend metadata, not unchecked caller-provided strings. Test fixtures may inject identities only
   behind a test-only boundary.
2. Revalidate source/root and destination/bin containment and file types before mutation. Reject
   source/destination canonical aliases and hard-link aliases. Use platform-appropriate file
   identity checks; canonical paths alone do not detect hard links. Reject empty sources, invalid
   digests, and unsupported kinds. Do not reuse Arti or FFmpeg archive verification as a direct
   executable digest.
3. Inspect existing destinations before creating staging. Reuse only a verified matching regular
   executable; reject unsafe/different files without chmod, deletion, or truncation. Revalidate
   if another publisher wins the final no-clobber race. Test concurrent success and conflict.
4. Create a unique owned staging file with create_new. Stream bounded buffers, hash actual copied
   bytes, reject read/write/digest failures, and synchronize the complete staged file. Set Unix
   executable permissions through the owned staging file, never through a user destination.
5. Publish by a no-clobber primitive. Proposed baseline: same-directory hard_link after verified
   staging, then remove only the owned staging name. Unsupported hard-link filesystems fail closed.
   Keep cleanup ownership explicit; do not mistake publication success followed by cleanup failure
   for permission to delete the final file. Document directory-durability and path-race limitations.
6. Return a typed backend-only result. Add tests against the production boundary, not just this
   prototype: supported success/reuse, unsupported kinds, malformed identity, source mutation,
   symlink/reparse and alias hazards, changed/different destination preservation, and failures.
   Test-only primitive success is necessary evidence, not sufficient evidence of task completion.

## API references reviewed for this plan

- https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new
- https://doc.rust-lang.org/std/fs/fn.hard_link.html
- https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all
- https://doc.rust-lang.org/std/fs/

The production and test build baseline remains Rust 1.91.0; a current documentation page does not
establish compatibility or successful execution. CI must verify the pinned baseline.

## Out of scope

Arti materialization/authentication, FFmpeg archive extraction, running the materialized executable,
changes to the live download command, Tauri bundle activation, installers, external providers,
and any direct-network fallback. PR #56 is already merged; no other task is started here.
