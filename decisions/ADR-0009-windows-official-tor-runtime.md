# ADR-0009 — Windows official Tor runtime: canonical identity

Status: Accepted, identity reconciliation only
Date: 2026-10-03

The Windows decision was already accepted on 2026-09-29 as ADR-0007 on main.
That number independently identifies Commons discovery on the active T069 branch.
Do not overwrite either decision or infer policy from a bare number.

The complete original Windows decision is preserved byte-for-byte at
`recovery/legacy/ADR-0007-windows-official-tor-runtime.md` (Git blob
12352f634fda51b3fe12769fef6178f8b0ba6372), imported from main commit
3c7d42b6fbb06974d59bf99cb0d9f90ad4ca992c. Its canonical identity is now ADR-0009.
It supersedes ADR-0004 for the Windows production sidecar only. Linux's Arti
choice is not changed. ADR-0007 on this branch continues to mean Commons.

This imports a decision, not a runtime implementation or a new test result.
The original pins, limitations and acceptance requirements remain unchanged.
`decisions/INDEX.json` records the fully qualified historical alias. On future
integration of main, resolve its old filename to this identity before merging.
