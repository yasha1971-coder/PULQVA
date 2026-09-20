# T008 — Prove the pinned Arti sidecar builds on Windows and Linux

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

PULQVA proved the pinned official Arti sidecar can be built and its proxy CLI verified on both
release targets without starting Tor.

Pinned sidecar:

- package: `arti`;
- version: `2.6.0`;
- Rust: `1.91.0`.

Windows additionally enables Arti's `static-sqlite` feature so SQLite is linked into the binary
instead of requiring an external `sqlite3.lib`.

## Verification

PR #7 verified head:
`756b6c574c80a8cc99b22261cb546838ddebb85b`

GitHub Actions run:
`35519121511`

Jobs:

- Ubuntu 24.04: success;
- Windows: success.

Both jobs verified:

- binary version contains `2.6.0`;
- `arti help proxy` exits successfully.

No Tor bootstrap/runtime connection was attempted.
