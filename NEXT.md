# NEXT

## Verified T057 identity capture

T057 PREPARE head:
`ddd9be7815dc31bc88f2c405229da5e36b51e2ff`

All 10 workflows triggered for that capture-only head passed. The Arti sidecar matrix run
`35921760775` produced exact executable receipts:

- Linux x86_64 / `arti`:
  `d6bef8db24c6edbeb7c7ee7c91df53e8b37959eb0f838ac9b9d08ab4d716a543`,
  23,619,232 bytes, job `107387185462`.
- Windows x86_64 / `arti.exe` with `static-sqlite`:
  `437c9391d7c7d70b8d298c8e6edd5310cb6e0511204c872c3a6c84fbc79e5bba`,
  19,759,616 bytes, job `107387185755`.

## Active atomic task

**T057 — Establish packaged Arti executable content identity**

Phase: IMPLEMENT / committed identity verification. T057 is NOT DONE.

This branch now:

- commits exactly one Arti SHA-256 per supported platform in `sidecars/arti/SHA256SUMS`;
- documents the capture run/jobs and build recipe in `sidecars/arti/README.md`;
- makes the same Windows/Linux Arti CI recipe rebuild the candidate and fail closed unless the
  computed SHA-256 equals the committed platform digest;
- binds the current platform's committed Arti digest into backend `BundledSidecarIdentity`;
- uses a strict Arti parser that rejects missing, duplicate, malformed, and wrong-platform asset
  bindings;
- makes packaged-source validation require and verify Arti content identity rather than accepting a
  version string alone;
- adds desktop tests for strict identity parsing and missing Arti digest rejection.

No runtime Arti copy/materialization, chmod, Tor launch change, shell execution, frontend path,
bundle activation, installer, external provider, or direct-network fallback is introduced.

## Immediate next action

Inspect the exact T057 implementation head and all triggered checks. The decisive proof is that both
Arti matrix jobs rebuild the same exact bytes and match the committed SHA256SUMS. If any check fails,
diagnose only that concrete failure. If all required Windows/Linux desktop/privacy checks are green,
close T057 in a later closeout phase.

Do not start T058.

## Trust limits retained

- the committed digests authenticate only executables produced by the pinned recipe and proven by
  matching CI; they do not make a Tauri release package exist;
- T057 does not yet materialize Arti into runtime/bin;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable;
- existing T055 parent-directory/reparse/durability limitations remain retained.
