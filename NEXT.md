# NEXT

## Current verified state

T010 is complete.

PULQVA now has a source-verified Arti 2.6.0 configuration contract and deterministic fixture:

- `application.defer_bootstrap = true`;
- configuration watching disabled;
- SOCKS listener configured as a numeric loopback-only Arti listener;
- DNS listener disabled;
- cache and state directories explicit;
- fixture parsed and resolved through Arti 2.6.0 APIs on Windows and Linux;
- no Arti proxy process was started;
- no Tor bootstrap or network connection occurred.

Verified PR head:
`445331c69fc30ce580586d82669a1e3f4d3bbdb8`

Verified runs:

- arti-config-contract: `35527538838` — success;
- rust-check: `35527538803` — success;
- continuity-guard: `35527538852` — success;
- arti-sidecar-check: `35527538906` — success.

## Next atomic task

**T011 — Add a typed deterministic Arti configuration renderer**

Add a pure-data `ArtiConfigSpec` in `pulqva-privacy` that renders the validated Arti 2.6.0
configuration contract from explicit inputs.

The renderer must carry:

- SOCKS port as typed non-zero loopback endpoint data;
- explicit cache directory;
- explicit state directory;
- `defer_bootstrap = true`;
- DNS listener disabled;
- configuration watching disabled.

The rendered canonical fixture for port 19050 and the repository test paths must match
`sidecars/arti/pulqva.toml` byte-for-byte.

## Do not do yet

- no file writes;
- no process spawn;
- no Tor bootstrap/network;
- no SOCKS readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

The configuration is generated deterministically from typed privacy inputs, exact-fixture parity is
covered by tests, and all existing checks remain green.
