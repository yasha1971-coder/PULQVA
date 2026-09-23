# NEXT

## Current verified state

T055 implementation is complete and verified.

Verified implementation head:
`05c37563301c0d3d953909f4d45b4642442c6d02`

All 11 required workflows passed for that exact head, including Windows and Linux desktop tests.

T055 now provides a backend-only no-clobber yt-dlp materialization boundary:

- accepts only the verified yt-dlp direct-binary artifact;
- binds to backend-pinned version and SHA-256 identity;
- rechecks package-source containment and runtime destination layout;
- rejects unsupported kinds, unsafe paths, symlink/file-type hazards, and hard-link aliases;
- preserves different existing destinations and reuses only matching verified content;
- rehashes the bytes actually copied into an exclusive staging file;
- synchronizes the complete staged file and publishes by same-directory hard link;
- has no overwrite fallback, shell command, process launch, frontend filesystem authority, or network
  operation.

The Windows repair at the verified head canonicalizes the trusted package resource root before
component inspection so canonical artifact paths are accepted without weakening final containment.

## Trust limits that remain

- parent-directory replacement races after validation are not claimed fully solved;
- ordinary Windows CI is not adversarial reparse-point proof;
- hard-link unsupported filesystems fail closed;
- file synchronization is not a universal directory-metadata durability guarantee;
- T055 did not install a real packaged yt-dlp resource;
- Arti still lacks pinned executable content authentication in this path;
- FFmpeg's pinned SHA-256 authenticates its archive, not an extracted executable.

## Closeout

Branch:
`task/T055-atomic-ytdlp-sidecar-materialization`

Draft PR:
#57

The implementation head is green, but this closeout commit must pass its own CI before PR #57 may
be marked ready/merged.

## Next atomic task after T055 merge

**T056 — Wire verified yt-dlp materialization into backend prelaunch preparation**

Bind the T055 materializer into the real completed-file backend flow after source validation and
runtime-directory preparation, before yt-dlp launch. T056 must not silently absorb Arti
materialization, FFmpeg extraction, packaging, installer work, external providers, or any
direct-network fallback.

Do not start T056 before PR #57 closeout is green and merged.
