# T011 — Add a typed deterministic Arti configuration renderer

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Added `ArtiConfigSpec` to `pulqva-privacy`.

The renderer uses a typed non-zero loopback Tor SOCKS endpoint plus explicit cache/state paths and
deterministically emits the verified Arti 2.6.0 configuration.

The canonical render matches `sidecars/arti/pulqva.toml` byte-for-byte.

No file I/O, process spawn, socket creation, or Tor bootstrap was introduced.

## Verification

PR #10 verified head:
`24535b1fa08f5ca50698b7758c77c159c86fb0f8`

Checks:

- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.
