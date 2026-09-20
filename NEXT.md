# NEXT

## Current verified state

T011 is complete.

PULQVA now has a typed deterministic `ArtiConfigSpec`:

- SOCKS port comes from `TorSocksEndpoint`;
- cache and state paths are explicit typed inputs;
- `defer_bootstrap = true`;
- configuration watching disabled;
- DNS listener disabled;
- canonical rendering matches `sidecars/arti/pulqva.toml` byte-for-byte;
- TOML path characters are escaped deterministically;
- no file I/O, process spawn, sockets, or Tor bootstrap.

Verified PR head:
`24535b1fa08f5ca50698b7758c77c159c86fb0f8`

Verified checks:

- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Next atomic task

**T012 — Add a typed Arti runtime plan**

Create one pure-data `ArtiRuntimePlan` that composes the verified launch and configuration specs
from one set of inputs.

The plan must guarantee that:

- executable and config-file paths belong to one launch spec;
- cache/state paths used by launch metadata and rendered config cannot drift apart;
- the typed loopback SOCKS endpoint is shared with the config renderer;
- generated launch arguments remain deterministic;
- generated config remains byte-stable.

## Do not do yet

- no file writes;
- no process spawn;
- no Tor bootstrap/network;
- no readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

One typed, internally consistent runtime plan can deterministically produce both launch arguments
and Arti configuration while all existing checks remain green.
