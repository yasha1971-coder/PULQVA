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

## Recovery after first committed-identity verification

Implementation head `a5c0bc55b4cfc393d9cec3bb0179a1a227c1c4b8` produced 10/11 green
workflows. Only `arti-sidecar-check` failed:

- Linux job `107393088967` reproduced the committed digest successfully.
- Windows job `107393089339` built Arti and passed the local CLI proof, then failed only at the
  fail-closed SHA comparison.
- The previous workflow compared before printing the actual rebuilt Windows digest, so the mismatch
  could not yet be classified as a stable recipe difference versus nondeterministic output.

This recovery changes diagnostics only: the workflow now prints actual SHA-256, expected SHA-256,
platform, and byte size immediately before the same fail-closed comparison. The comparison is not
weakened and SHA256SUMS is unchanged.

## Immediate next action

Inspect the exact diagnostic-recovery head. If Windows fails again, recover the printed actual and
expected SHA values and diagnose only that concrete difference. If it matches and all required
checks are green, close T057 in a later closeout phase.

Do not start T058.

## Trust limits retained

- the committed digests authenticate only executables produced by the pinned recipe and proven by
  matching CI; they do not make a Tauri release package exist;
- T057 does not yet materialize Arti into runtime/bin;
- FFmpeg's pinned digest still authenticates its archive, not an extracted executable;
- existing T055 parent-directory/reparse/durability limitations remain retained.


## Recovery: immutable artifact capture

The exact-head diagnostic proved that the Windows `cargo install` output is not byte-reproducible
across GitHub Actions runs: identical version/features/toolchain and identical byte size produced a
different SHA-256. Therefore T057 must not chase a moving digest.

Tor Project's current Arti compiling guide states that official binaries are not published, so there
is no upstream signed/prebuilt Windows executable to pin instead.

This recovery phase keeps the existing fail-closed comparison intact and adds an `upload-artifact`
capture for the exact built executable plus its receipt, even when the SHA comparison fails. The
purpose is to retrieve one concrete CI-produced executable and make those exact bytes the future
package source; later CI can verify the stored/package source bytes against their committed SHA
without requiring a fresh Windows rebuild to be byte-identical.

No runtime behavior, digest allowlist, or privacy rule changes in this phase.


## Windows reproducibility recovery: /Brepro

The captured Windows artifact from run `35934005627` was downloaded and inspected directly.

Exact captured executable:

- `arti.exe` size: 19,759,616 bytes;
- receipt SHA-256:
  `e40620ee8332f90967bf13dec2338a2307aadd0f8579fede83ba9aed71a798c5`;
- PE/COFF timestamp: `2026-09-23T23:36:51Z`, exactly the build time;
- all three PE debug-directory entries also carried that same build-time timestamp;
- the executable contains an RSDS/PDB identity record.

That proves at least one concrete source of Windows byte drift: build-time data emitted by the MSVC
link step. This recovery changes only the Windows candidate build by passing MSVC linker option
`/Brepro` through Rust (`RUSTFLAGS="-C link-arg=/Brepro"`). Microsoft tooling identifies
`/Brepro` images by using reproducible-build identity data rather than a wall-clock PE timestamp.

The committed Windows SHA256SUMS entry remains unchanged. The existing equality check remains
fail-closed, so this head is expected to remain red until a stable `/Brepro` candidate is proven.
Artifact capture remains enabled so the exact candidate can be recovered.

## Immediate next action after this head runs

Recover the Windows artifact and its printed actual SHA. Then rerun this exact same failed Windows
job once, without changing code. If the two `/Brepro` runs produce the same exact SHA-256, update
the committed Windows identity to that proven stable value in a later T057 phase. If they differ,
diagnose the remaining nondeterminism rather than moving the allowlist.
