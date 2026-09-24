# T057 — Establish packaged Arti executable content identity

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Replace the current Arti version-only trust boundary with an explicit backend-verifiable content
identity for the exact packaged Arti executable on each supported platform, without materializing or
launching it yet.

## Acceptance criteria

- define one immutable packaged Arti executable identity per supported Windows/Linux target;
- identity includes the existing pinned Arti version plus a SHA-256 digest of the exact executable
  bytes intended for packaging;
- runtime verification metadata is backend-owned and not frontend input;
- packaged-resource identifiers remain beneath the Tauri resource root;
- digest parsing fails closed on missing, duplicate, malformed, or wrong-platform entries;
- no claim that a version string authenticates executable bytes;
- no executable copy/materialization, chmod, process launch, shell command, or network access in the
  desktop runtime;
- CI proves the identity metadata corresponds to the Arti artifact production path used for each
  supported target;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

Arti runtime materialization, Tor launch changes, FFmpeg extraction/materialization, Tauri bundle
activation, installer/release packaging, external providers, and direct-network fallback.


## Current phase: IMPLEMENT / committed identity verification

The repository did not contain a content digest for the exact Arti executable bytes. This phase
changes the existing Windows/Linux Arti CI build into an explicit identity-candidate build recipe
and emits the exact SHA-256 plus byte size for each produced executable.

The candidate recipe remains pinned to:

- Arti 2.6.0;
- Rust 1.91.0;
- Cargo --locked;
- Windows static-sqlite feature;
- fixed runner-local install/target directory names;
- no Tor bootstrap or network use by the produced executable.

This phase does not yet define the committed immutable SHA256SUMS contract. T057 remains ACTIVE
until a later phase reads the exact CI receipts, commits one platform digest per supported target,
binds those digests into backend metadata, and re-runs CI to prove the same production recipe matches
the committed identities.


## Captured identities and binding

PREPARE head `ddd9be7815dc31bc88f2c405229da5e36b51e2ff` passed all 10 triggered
workflows. The Arti matrix run `35921760775` emitted:

- Linux: `d6bef8db24c6edbeb7c7ee7c91df53e8b37959eb0f838ac9b9d08ab4d716a543`,
  23,619,232 bytes.
- Windows: `437c9391d7c7d70b8d298c8e6edd5310cb6e0511204c872c3a6c84fbc79e5bba`,
  19,759,616 bytes.

This phase commits those digests, makes the same production-build workflow compare rebuilt bytes
against them, binds the current-platform Arti digest into backend materialization identity metadata,
and makes packaged-source validation require an Arti digest just like yt-dlp and the FFmpeg archive.

T057 remains ACTIVE until the exact implementation head proves the committed digests reproduce on
both supported CI targets and all required desktop/privacy checks are green.


## Recovery after first committed-identity rerun

Head `a5c0bc55b4cfc393d9cec3bb0179a1a227c1c4b8` passed every required workflow
except `arti-sidecar-check`. Linux reproduced its committed digest. Windows built successfully and
passed `arti --version` / `arti help proxy`, but failed only at the SHA-256 equality check.

The recovery adds an explicit `PULQVA_ARTI_IDENTITY_COMPARE` diagnostic line before that unchanged
fail-closed comparison. It does not update the committed Windows digest, weaken the equality check,
or alter runtime behavior.


## Immutable-source recovery phase

The Windows rebuild at `96a4e42b31728e1c3b7e5a879c2974c0d6532eb9` produced
`de62684554b7af8e783a79339f4bfefff7ac8dbbb4fc498eb54e5a300491bc6b` while the
earlier capture produced
`437c9391d7c7d70b8d298c8e6edd5310cb6e0511204c872c3a6c84fbc79e5bba`, with the same
19,759,616-byte size. A moving rebuild hash is therefore not an acceptable package identity.

This phase adds artifact capture only. The exact executable and receipt produced by each matrix job
are uploaded even if the existing fail-closed comparison fails. A later phase may retrieve the exact
Windows artifact from this run and make those exact bytes the immutable package source. The
committed SHA256SUMS is not changed here.


## Windows /Brepro recovery

The exact Windows artifact captured from run `35934005627` was inspected locally. Its PE/COFF
timestamp and all debug-directory timestamps equal the wall-clock build time
`2026-09-23T23:36:51Z`; it also carries an RSDS/PDB identity record. This is direct evidence that
the MSVC link output contains per-build data.

The Windows candidate build now passes `/Brepro` to the MSVC linker through Rust. Linux is
unchanged. The committed Windows digest and fail-closed comparison are intentionally not updated in
this phase. Exact candidate artifacts continue to be uploaded for later proof.


## Proven stable Windows identity

Two unchanged Windows `/Brepro` CI executions on head
`ad17896b52535122ef5766dcd4ae0960f2b19df7` were byte-identical:

- job `107436310670`: SHA-256
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`,
  19,760,128 bytes;
- rerun job `107449565233`: the same SHA-256 and the same 19,760,128-byte size.

The committed Windows `SHA256SUMS` identity is now updated to this proven stable value. The
`/Brepro` build recipe and fail-closed equality check remain mandatory. T057 remains ACTIVE until
one fresh CI attempt on the updated committed identity is fully green.
