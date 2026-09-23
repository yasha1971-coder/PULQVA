# NEXT

## Current verified state

T056 implementation is complete and verified.

Verified implementation head:
`c06539668b8d9dc94ea8e61138b5ab144a56623f`

All 11 required workflows passed for that exact head, including Windows and Linux desktop tests.

T056 now wires the T055 verified yt-dlp materializer into the real completed-file backend prelaunch
path:

- package sources are validated first;
- exactly one verified yt-dlp artifact is required;
- runtime directories are prepared before publication;
- the real T055 materializer runs before any yt-dlp process launch;
- the materialized path must equal both the app-owned runtime layout destination and the backend
  completed-file yt-dlp input;
- missing/duplicate artifacts and path mismatches fail closed before process launch.

No frontend filesystem authority, shell command, new network operation, direct-network fallback,
Arti materialization, FFmpeg extraction, bundle activation, installer, external provider, or AI
provider was added.

## Closeout

Branch:
`task/T056-wire-ytdlp-materialization-into-backend-prelaunch`

PR:
#58

The implementation head is green, but this closeout commit must pass its own CI before PR #58 may
be merged.

## Next atomic task after T056 merge

**T057 — Establish packaged Arti executable content identity**

Arti is now the next prelaunch dependency gap. Its current metadata proves a pinned version and
path/file type only; it does not authenticate executable bytes. T057 must create a platform-specific
SHA-256 identity contract for the exact packaged Arti executables before any Arti materialization is
attempted.

Do not start T057 before PR #58 closeout is green and merged.

## Trust limits retained

- T055/T056 do not make a real release package exist; package-resource population remains separate.
- Arti version metadata is not executable authentication until T057 or later establishes content
  identity.
- FFmpeg's pinned SHA-256 authenticates its archive, not an extracted executable.
- Existing parent-directory race/reparse/durability limitations from T055 remain explicitly retained.
