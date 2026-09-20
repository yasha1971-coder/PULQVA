# NEXT

## Current verified state

T014 is complete.

PULQVA now has a typed `PreparedArtiRuntime` capability:

- callers cannot construct it directly from raw fields;
- successful preparation materializes the verified Arti config first;
- failed materialization returns no prepared capability;
- the capability carries the immutable `ArtiRuntimePlan`;
- typed Tor SOCKS endpoint is preserved;
- deterministic launch arguments are exposed;
- Windows and Linux privacy runtime tests pass;
- no Arti process is spawned;
- no sockets are opened by PULQVA;
- no Tor bootstrap occurs.

Verified PR head:
`d76d23a2e3ac982d2a91721fc8216d7b96f7808f`

Verified checks:

- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Next atomic task

**T015 — Add a controlled Arti child-process launcher**

Introduce the first process side effect behind the prepared capability boundary.

The launcher must:

- accept only `PreparedArtiRuntime`, never a raw runtime plan;
- spawn only the explicit Arti executable with deterministic arguments;
- inherit no shell;
- capture process identity/handle in a typed `RunningArti` capability;
- provide deterministic shutdown/cleanup;
- keep `defer_bootstrap = true`;
- perform no SOCKS request, readiness probe, DNS request, or Tor bootstrap trigger.

## Do not do yet

- no external network request;
- no Tor bootstrap trigger;
- no SOCKS readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

A prepared runtime can be launched and cleanly stopped as a supervised child process without
triggering external network activity, with Windows/Linux tests and all existing checks green.
