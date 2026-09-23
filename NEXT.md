# NEXT

## Current verified state

T051 is complete on main.

Merged main:
`2331ecc02df29cac08dd12836b76199cc120f3fc`

Post-merge main checks are green.

## Active recovery phase

**T051R1 — Harden runtime directory verification and execute desktop boundary tests**

An unexpected parallel PR #52 was found after T051 had already merged. It is based on the pre-T051
main and is therefore conflicting/stale as a merge vehicle, but it contains two useful hardening
improvements that are not present on current main:

- canonical verification that prepared runtime directories remain physically beneath the verified
  runtime root, with explicit symlink/non-directory rejection;
- execution of desktop boundary tests on both Windows and Linux in `desktop-shell-check`, not only
  compilation.

T051R1 ports only those useful changes onto current main. It does not restart T051 or alter the
privacy/network architecture.

Branch:
`recovery/T051R1-runtime-directory-hardening`

Superseded source PR:
`#52`

## Next task

**T052 — Add a typed local sidecar materialization plan boundary**

Do not start T052 until T051R1 is verified and merged.

## Do not do yet

- no sidecar binary copy/materialization;
- no Tauri bundle-resource activation;
- no packaging/release installers;
- no external search provider;
- no AI provider;
- no direct-network fallback.

## Success

The hardened T051 runtime-directory implementation and desktop tests pass on the exact recovery head
for both Windows and Linux, after which PR #52 can be closed as superseded and T052 can proceed.
