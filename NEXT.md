# NEXT

## Current verified state

T012 is complete.

PULQVA now has a pure-data `ArtiRuntimePlan`:

- executable path is explicit;
- config-file path is explicit;
- one typed `TorSocksEndpoint` is shared with configuration rendering;
- cache/state paths are supplied once and derived into both launch and config specs;
- launch arguments are deterministic;
- rendered configuration remains byte-stable against the verified fixture;
- no file I/O, process spawn, sockets, or Tor bootstrap.

Verified PR head:
`10238c0d101c0f3fc377f931cab3b78d6bef3a36`

Verified checks:

- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Next atomic task

**T013 — Add atomic Arti config materialization**

Add the first controlled filesystem side effect: materialize the already-rendered Arti config to
the plan's explicit config path using an atomic replace strategy.

The implementation must:

- create only the required parent directory;
- write to a temporary sibling file;
- flush/sync before replacement;
- atomically replace the final config file where supported by the platform;
- never spawn Arti;
- never open sockets;
- never bootstrap Tor;
- preserve deterministic config bytes.

## Do not do yet

- no Arti process spawn;
- no Tor bootstrap/network;
- no readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

The verified config can be safely materialized to disk on Windows and Linux without introducing any
network or process behavior, with tests and all existing checks green.
