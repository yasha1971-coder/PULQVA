# T057 — Establish packaged Arti executable content identity

Parent: DESKTOP FOUNDATION  
Status: READY

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
