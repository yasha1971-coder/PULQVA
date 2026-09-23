# NEXT

## Current verified state

T056 is complete and merged on main at
`de146d549471cb80a564ff224b7091d94f0cf022`.

Verified T056 closeout head:
`0f0e58f4c813be016c940bc910debc71a28354ec`.

All 11 required workflows passed for that exact closeout head.

## Active atomic task

**T057 — Establish packaged Arti executable content identity**

Phase: PREPARE / identity capture. T057 is NOT DONE.

The existing Arti sidecar CI previously proved only that `arti 2.6.0` builds and exposes the proxy
CLI. It did not establish a digest for the exact executable bytes intended for packaging.

This phase updates the same Windows/Linux production-build path to emit an explicit identity receipt
for each candidate executable:

- platform;
- Arti version;
- executable name;
- SHA-256 of the exact built executable bytes;
- byte size;
- Rust 1.91.0;
- Cargo --locked;
- target-specific feature arguments.

The build still does not bootstrap Tor. The desktop runtime receives no new network, shell,
materialization, chmod, or process behavior from this phase.

Branch:
`task/T057-establish-packaged-arti-executable-content-identity`

## Why T057 is still open

No digest is invented or inferred. The exact Windows/Linux SHA-256 values must come from the new CI
receipts. A later T057 phase must:

1. read the exact successful Windows/Linux identity receipts from this branch head;
2. commit one immutable digest per supported platform under `sidecars/arti`;
3. make the Arti CI fail closed if a rebuilt candidate differs from the committed identity;
4. bind the committed Arti digest into the backend sidecar identity metadata;
5. prove missing/duplicate/malformed/wrong-platform identities fail closed;
6. run the full Windows/Linux/privacy CI again.

Only after that exact head is green can T057 close.

## Trust limits retained

- a version string alone is not executable authentication;
- this phase does not claim cross-machine reproducibility until a later rebuild matches the captured
  identity;
- no real Tauri bundle resource is populated yet;
- no Arti runtime materialization or Tor launch behavior changes here;
- FFmpeg's pinned SHA-256 still authenticates an archive, not an extracted executable.

## Immediate next action

Inspect the exact T057 PREPARE head. If the Arti Windows/Linux jobs succeed, recover the two emitted
`PULQVA_ARTI_IDENTITY` receipts and continue T057 on this same branch. If a job fails, diagnose only
that concrete failure.

Do not start T058.
