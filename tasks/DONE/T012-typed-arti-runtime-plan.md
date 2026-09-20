# T012 — Add a typed Arti runtime plan

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Added pure-data `ArtiRuntimePlan` to `pulqva-privacy`.

The plan owns one canonical set of executable/config/cache/state paths plus one typed
`TorSocksEndpoint`, then derives both `ArtiLaunchSpec` and `ArtiConfigSpec`.

This prevents cache/state drift between launch metadata and rendered configuration while preserving
deterministic launch arguments and byte-stable configuration.

No file I/O, process spawn, socket creation, or Tor bootstrap was introduced.

## Verification

PR #11 verified head:
`10238c0d101c0f3fc377f931cab3b78d6bef3a36`

Checks:

- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.
