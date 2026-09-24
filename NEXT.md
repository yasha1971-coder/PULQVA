# NEXT

## Current verified state

T058 implementation is complete and verified.

Verified implementation head:
`0748bfaaef65fb9b3b9ffce4657d02ab220143c5`

All 11 required workflows passed for that exact head.

T058 now provides a backend-only no-clobber Arti materialization boundary using the authenticated T057
platform identity:

- only a verified Arti direct-binary artifact is accepted;
- backend-owned version + platform SHA-256 identity are required;
- source containment/type/receipt size and destination layout are revalidated;
- source/destination aliases and unsafe paths are rejected;
- matching verified existing destinations may be reused;
- different or unsafe existing destinations are preserved and rejected;
- copied bytes are hashed again from the actual staging stream;
- Unix executable permission is applied only to the owned stage;
- the complete stage is synchronized before no-clobber hard-link publication;
- owned staging cleanup is identity-checked;
- no shell, network, Tor bootstrap, process launch, or frontend filesystem authority is introduced.

## Closeout

Branch:
`task/T058-atomic-arti-sidecar-materialization`

Draft PR:
#60

The implementation head is green, but this closeout commit must pass its own CI before PR #60 may
be marked ready and merged.

## Next atomic task after T058 merge

**T059 — Wire verified Arti materialization into backend prelaunch preparation**

Bind T058 into the real completed-file backend path before Arti/Tor runtime preparation or launch,
while preserving the already verified T056 yt-dlp materialization ordering. T059 must not absorb
FFmpeg extraction, bundle activation, installer work, external providers, AI providers, or any
direct-network fallback.

Do not start T059 before PR #60 closeout is green and merged.

## Trust limits retained

- T058 does not prove a real release package contains the Arti resource;
- T058 is not adversarial Windows reparse-point proof;
- unsupported hard-link filesystems fail closed;
- file synchronization is not a universal directory-metadata durability guarantee;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable.
