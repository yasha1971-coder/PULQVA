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
