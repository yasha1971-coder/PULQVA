# NEXT

## Current verified state

T009 is complete.

PULQVA now has a pure-data Arti sidecar launch specification:

- executable path is explicit input data;
- config, cache and state paths are explicit input data;
- proxy subcommand arguments are deterministic;
- no process is spawned;
- no socket is opened;
- no Tor bootstrap occurs.

Verified PR head:
`df0c0ad867ce10d868545b0014a5d6e8de577c58`

Verified checks:

- rust-check: success;
- continuity-guard: success;
- arti-sidecar-check: success.

## Next atomic task

**T010 — Verify the pinned Arti configuration contract**

Confirm the exact Arti 2.6.0 configuration keys and CLI path needed for a self-contained proxy
configuration, then add a deterministic config fixture and CI parse/check proof without starting
the proxy.

The proof must cover:

- explicit cache directory;
- explicit state directory;
- loopback-only SOCKS listen configuration;
- config accepted by pinned Arti 2.6.0 on Windows and Linux;
- no Tor bootstrap and no network connection.

## Do not do yet

- no product process spawn;
- no Tor bootstrap/network;
- no SOCKS readiness probing;
- no dynamic port discovery;
- no yt-dlp;
- no HTTP;
- no AI provider;
- no UI.

## Success

The exact configuration contract is source-verified, represented by a deterministic fixture, and
validated by CI on Windows and Linux without starting Tor.
