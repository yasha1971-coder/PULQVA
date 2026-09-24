# T058 — Atomically materialize the verified Arti direct-binary artifact

Parent: DESKTOP FOUNDATION  
Status: DONE  
Date: 2026-09-24

## Result

Added a backend-only atomic materialization boundary for the T057-authenticated Arti executable.

The materializer:

- accepts only a verified `BundledSidecarKind::Arti` artifact;
- binds to backend-owned Arti version plus platform SHA-256 identity;
- revalidates package-source containment, source type, receipt size, destination layout, and
  source/destination aliases before mutation;
- creates/verifies `runtime/bin` only beneath the prepared app-owned runtime root;
- preserves different or unsafe existing destinations without modification;
- reuses only an existing matching verified executable;
- copies source bytes in bounded buffers into an exclusive staging file and hashes the bytes actually
  copied;
- applies Unix executable permission only to the owned staging file;
- synchronizes the complete stage before no-clobber same-directory hard-link publication;
- identity-checks the owned stage before cleanup;
- supports competing publishers converging on one verified destination;
- performs no shell execution, network access, Tor bootstrap, process launch, or frontend filesystem
  operation.

## Verification

Verified implementation head:
`0748bfaaef65fb9b3b9ffce4657d02ab220143c5`

All 11 required workflows passed for that exact head, including Windows/Linux desktop checks and all
existing privacy/media checks.

## Trust limits retained

- concurrent replacement of already validated parent directories is not claimed fully solved;
- ordinary Windows CI is not adversarial reparse-point proof;
- filesystems without hard-link support fail closed;
- file synchronization is not a universal directory-metadata durability guarantee;
- this task does not populate a real Tauri bundle resource;
- this task does not wire Arti materialization into live Tor launch;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable.
