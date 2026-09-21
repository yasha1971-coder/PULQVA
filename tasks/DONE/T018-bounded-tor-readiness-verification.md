# T018 — Add bounded Tor readiness verification

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added the only privacy-layer path allowed to mint `ReadyTorTransport`.

The verifier explicitly activates Tor bootstrap from the deferred state, probes only the local
SOCKS listener, sends the external target as a SOCKS5 domain name, and has no direct-network
fallback.

Linux CI proved successful readiness with the actual pinned Arti 2.6.0 sidecar. On the
GitHub-hosted Windows runner, the bounded timeout path proved fail-closed behavior: no ready
capability was minted and the supervised child was stopped.

## Verification

PR #17 verified head:
`490e6467ac2f56d69dcfdcb436775e216ebf3859`

Checks:

- tor-readiness-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-lifecycle-check: success;
- arti-materialization-check: success;
- arti-config-contract: success;
- arti-sidecar-check: success.
