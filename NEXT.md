# NEXT

## Current verified state

T013 is complete.

PULQVA can now materialize the deterministic Arti configuration to disk safely:

- config bytes come only from `ArtiRuntimePlan::render_config`;
- only the config parent directory is created;
- writes use a unique temporary sibling file;
- temporary contents are flushed and synced before replacement;
- final replacement uses same-filesystem rename semantics;
- handled failures clean up temporary files;
- Windows and Linux materialization tests pass;
- no Arti process is spawned;
- no sockets are opened;
- no Tor bootstrap occurs.

Verified PR head:
`e314b928365bcfa36cabbcccb3ea7c3982f2c2fb`

Verified checks:

- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Next atomic task

**T014 — Add a prepared Arti runtime capability**

Introduce a typed `PreparedArtiRuntime` capability that can only be produced after successful
configuration materialization.

The capability must:

- carry the immutable runtime plan needed by a future launcher;
- be constructible only through successful preparation/materialization;
- expose deterministic launch arguments and the typed Tor SOCKS endpoint;
- prevent future launch code from accepting an unprepared runtime plan by accident;
- introduce no process spawn and no network behavior.

## Do not do yet

- no Arti process spawn;
- no Tor bootstrap/network;
- no readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

A future launcher can be designed to require a `PreparedArtiRuntime` token, making successful
config materialization a type-level prerequisite while all existing checks remain green.
