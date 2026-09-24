# NEXT

## Current verified state

T057 is complete and merged on main at
`7efe4862c68a3ebe96f03d59a990098ce9c17625`.

Verified T057 closeout head:
`a075e293909cd020b008aa3feaf0b829855220b6`.

All 11 required workflows passed for that exact closeout head.

## Active atomic task

**T058 — Atomically materialize the verified Arti direct-binary artifact**

Phase: IMPLEMENT. T058 is NOT DONE.

This branch adds
`apps/pulqva-desktop/src-tauri/src/arti_materialize.rs`, a backend-only Arti direct-binary
materializer that deliberately mirrors the already verified T055 publication semantics while using
the T057 authenticated Arti identity.

Implemented behavior:

- only a T057-verified Arti artifact is accepted;
- expected version and platform SHA-256 come from backend-owned Arti metadata;
- package source containment, component types, canonical containment, receipt size, destination,
  and source/destination aliases are revalidated before publication;
- `runtime/bin` must be a real app-owned directory beneath the prepared runtime root;
- an unsafe or different existing Arti destination is preserved and rejected;
- a matching verified existing Arti destination may be reused;
- source bytes are copied in bounded buffers into an exclusive owned stage and SHA-256 checked
  against the T057 platform digest;
- Unix executable permissions are applied only to the owned stage;
- the complete stage is synchronized before no-clobber same-directory hard-link publication;
- cleanup removes only the operation's owned staging file after identity verification;
- competing publishers may converge on the same verified destination.

The module is compiled into the desktop backend but is intentionally not wired into the live
completed-file/Tor launch path yet.

## Trust limits retained

- concurrent replacement of already validated parent directories is not claimed fully solved;
- ordinary Windows CI is not adversarial reparse-point proof;
- filesystems without hard-link support fail closed; there is no overwrite fallback;
- file `sync_all` is not a universal directory-metadata durability guarantee;
- T058 does not prove that a real release bundle contains the Arti resource;
- T058 does not change Tor lifecycle/readiness or launch Arti;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable.

Branch:
`task/T058-atomic-arti-sidecar-materialization`

## Immediate next action

Inspect the exact T058 implementation head and all triggered CI. If a check fails, diagnose only that
concrete failure. If all required Windows/Linux/privacy checks are green, close T058 in a later
bounded phase.

Do not start T059.
