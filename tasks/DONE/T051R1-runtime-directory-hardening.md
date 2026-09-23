# T051R1 — Harden runtime directory verification and desktop test execution

Parent: T051  
Status: DONE  
Date: 2026-09-23

## Result

Recovered useful hardening from stale parallel PR #52 onto current main without reopening T051.

The runtime directory boundary now canonicalizes prepared directories and verifies they remain beneath
the verified app-owned runtime root. Existing symlink and non-directory hazards fail closed.

The desktop shell workflow now runs desktop boundary tests on both Windows and Linux in addition to
compilation.

The existing T051 separation remains unchanged: sidecar executable destinations are under
`runtime/bin/*`, while Tor state/config/cache remain under `runtime/arti/*`.

No sidecar binary materialization, bundle activation, external network path, or privacy weakening was
introduced.

## Verification

PR #53 implementation head:
`ca8755cf59da63cdf1a25e5ddf54891e368b525d`

All 11 required workflows passed.
