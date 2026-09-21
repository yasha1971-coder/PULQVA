# T016 — Prove real Arti deferred-bootstrap lifecycle

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

The actual pinned Arti 2.6.0 sidecar was launched through `PreparedArtiRuntime` and the controlled
child-process launcher on Windows and Linux.

The verified config kept `application.defer_bootstrap = true`. The lifecycle proof sent no SOCKS
request, DNS request, readiness probe, or user workload and then stopped/reaped the child
deterministically.

## Verification

PR #15 verified head:
`1326fe255304fa72899d876f0b842260cac3f6b6`

Checks:

- arti-lifecycle-check: success;
- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.
