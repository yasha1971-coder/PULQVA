# NEXT

## Verified base

T054 is merged through PR #56. Verified closeout head:
`e25ebb09fc35ea6937999e3b81c26f2361417377`.
Main observed at startup: `dff37d8f4c2448e7c0ace03a02e28444e488ec6c`.
The main-head Actions success query returned 11 successful runs; this observation is not the CI
result for any newer T055 commit. There were no open PRs at startup.

## Active task and bounded phase

**T055 — Atomically materialize the verified yt-dlp direct-binary artifact**

Phase: PREPARE / executable publication-strategy proof. T055 is NOT DONE.
Branch: `task/T055-atomic-ytdlp-sidecar-materialization`.
The draft PR and its latest checkpoint are the authority for this branch's exact head and CI runs.

This phase adds an isolated Windows/Linux test target:
`apps/pulqva-desktop/src-tauri/tests/t055_publication_contract.rs`.
It exercises a test-only staging algorithm and filesystem publication primitives, not the shipped
materializer. No production source, live download command, dependency, or frozen kernel was changed.

The tests exercise bounded copy/hash of the actual stream, complete-file publication by hard link,
no-clobber preservation, exclusive staging creation, cleanup on hash/read failure, rejection of a
source changed after an earlier receipt, competing publishers, and Unix-only symlink/permission
cases. They have NOT been executed locally: this session has no Rust toolchain. Only observed CI
may establish whether they pass. A green run of these tests is NOT completion of T055.

## Implementation decision and limits

Use exclusive staging creation, hash the bytes written, synchronize the completed staging file,
and publish without replacing any destination. `std::fs::hard_link` is the proposed no-clobber
publication primitive; reject unsupported filesystems rather than falling back to overwrite/rename.
See `tasks/ACTIVE/T055-atomic-ytdlp-sidecar-materialization.md` for the implementation checklist.

The test helper intentionally assumes its temporary directory is owned and not concurrently
replaced. It is NOT a production containment or path-race defense. Production work must bind the
prepared runtime capability and T054 artifact, validate identities and aliases, and address or
explicitly retain concurrent-directory-replacement limitations. Do not promote the test helper
unchanged and claim full safety. File synchronization is not a claim of crash-durable directory
publication on every filesystem. Unix-only tests do not constitute Windows reparse-point proof.

## Immediate next action

Inspect the draft PR's exact head and CI. Diagnose a concrete failure first. If the primitive proof
is green, continue the IMPLEMENT phase of T055 on this SAME branch: add the typed backend boundary
and production-boundary tests. Keep the PR draft and T055 ACTIVE until all acceptance criteria are
implemented and the exact implementation head is verified. Do not create a parallel T055 branch,
merge a test-only draft, or advance to another task.

## Continuity notes that must not be lost

- FFmpeg's pinned digest authenticates the platform archive, NOT an extracted executable. Never
  copy archive bytes to `runtime/bin/ffmpeg(.exe)`.
- Arti currently has version metadata plus path/file-type checks, not a pinned executable digest.
  The version string is not binary authentication.
- T054 returns paths, not immutable snapshots. Revalidate containment and the actual bytes copied.
- Existing runtime executables must not be overwritten or trusted merely because they exist.
- Preserve the frozen kernel, Tor-first fail-closed behavior, and backend-only filesystem authority.
- This phase neither materializes nor runs any real sidecar. No bundle activation, installer,
  external search, AI provider, or direct-network fallback is introduced.
