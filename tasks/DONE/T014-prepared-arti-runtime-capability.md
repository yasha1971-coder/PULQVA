# T014 — Add a prepared Arti runtime capability

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added `PreparedArtiRuntime`, a capability that is returned only after successful Arti config
materialization.

The capability carries the immutable `ArtiRuntimePlan`, exposes the typed Tor SOCKS endpoint and
deterministic launch arguments, and cannot be directly constructed from raw fields by callers.

The Windows newline mismatch discovered by the expanded cross-platform test was fixed by pinning
`sidecars/arti/pulqva.toml` to LF in `.gitattributes`.

## Verification

PR #13 verified head:
`d76d23a2e3ac982d2a91721fc8216d7b96f7808f`

Checks:

- arti-materialization-check: success on Windows and Linux;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

No Arti process spawn, socket creation, or Tor bootstrap was introduced.
