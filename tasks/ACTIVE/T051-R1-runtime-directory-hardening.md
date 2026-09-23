# T051-R1 — Harden runtime directory verification and desktop test execution

Parent: T051  
Status: ACTIVE

## Goal

Port the useful hardening from stale parallel PR #52 onto current main without reopening unrelated
work.

## Acceptance criteria

- preserve T051 runtime/bin and Tor-state separation;
- canonicalize and verify prepared runtime directories beneath the verified runtime root;
- reject symlink and non-directory hazards fail-closed;
- preserve idempotent local-only directory preparation;
- create/copy/download/modify no sidecar executable;
- add execution of desktop boundary tests on both Windows and Linux in desktop-shell-check;
- no privacy invariant changes;
- no external network access is introduced;
- exact recovery head must be green before merge.

## Out of scope

- T052 implementation;
- sidecar materialization;
- bundle activation;
- packaging/installers.
