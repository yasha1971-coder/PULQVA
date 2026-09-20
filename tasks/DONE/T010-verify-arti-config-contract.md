# T010 — Verify the pinned Arti configuration contract

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Verified the exact Arti 2.6.0 configuration contract PULQVA will use before runtime launch code.

The repository now contains:

- a deterministic Arti config fixture;
- explicit cache and state directories;
- loopback-only SOCKS configuration;
- DNS listener disabled;
- deferred bootstrap;
- a source-level parser/resolver check using pinned Arti 2.6.0 APIs;
- Windows and Linux CI validation.

No Arti proxy process was started and no Tor bootstrap/network connection was attempted.

## Verification

PR #9 verified head:
`445331c69fc30ce580586d82669a1e3f4d3bbdb8`

Runs:

- arti-config-contract `35527538838`: success;
- rust-check `35527538803`: success;
- continuity-guard `35527538852`: success;
- arti-sidecar-check `35527538906`: success.
