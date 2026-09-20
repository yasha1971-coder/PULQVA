# T009 — Add a typed Arti sidecar launch specification

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-20

## Result

Added a pure-data `ArtiLaunchSpec` in `pulqva-privacy`.

The specification carries:

- executable path;
- config-file path;
- cache directory;
- state directory;
- deterministic `proxy --config <path>` argument construction.

It has no process-spawn, socket, Tor-bootstrap or network behavior.

## Verification

PR #8 verified head:
`df0c0ad867ce10d868545b0014a5d6e8de577c58`

GitHub Actions:

- rust-check: success;
- continuity-guard: success;
- arti-sidecar-check: success.
