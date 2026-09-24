# NEXT

## Current verified state

T057 implementation is complete and verified.

Verified implementation head:
`233b4cc291066404c3b0119b7e9664981cfc6b10`

All 11 required workflows passed for that exact head.

T057 now establishes authenticated Arti executable identities for both supported platforms:

- Linux x86_64 / `arti`:
  `d6bef8db24c6edbeb7c7ee7c91df53e8b37959eb0f838ac9b9d08ab4d716a543`,
  23,619,232 bytes.
- Windows x86_64 / `arti.exe`:
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`,
  19,760,128 bytes.

The Windows identity is backed by a reproducible `/Brepro` recipe. Two unchanged proof runs
produced byte-identical output, and the fresh committed-identity run matched actual to expected
exactly in job `107480733952`.

Backend packaged-source validation now requires and verifies Arti SHA-256 content identity; version
metadata alone is no longer accepted as executable authentication.

## Closeout

Branch:
`task/T057-establish-packaged-arti-executable-content-identity`

Draft PR:
#59

The verified implementation head is green. This closeout commit must pass its own CI before PR #59
may be marked ready and merged.

## Next atomic task after T057 merge

**T058 — Atomically materialize the verified Arti direct-binary artifact**

Apply the T055-style local no-clobber publication boundary to the now-authenticated Arti executable.
T058 must remain backend-local and must not yet wire Arti materialization into live Tor launch.

Do not start T058 before PR #59 closeout is green and merged.

## Trust limits retained

- authenticated package identity does not itself populate a real Tauri release bundle;
- T057 does not materialize or launch Arti;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable;
- existing parent-directory/reparse/durability limits remain explicit.
