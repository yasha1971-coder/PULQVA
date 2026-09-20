# NEXT

## Current verified state

T015 is complete.

PULQVA now has a controlled child-process launcher behind the prepared-runtime capability:

- launcher accepts `PreparedArtiRuntime`, not a raw runtime plan;
- executable path and argument vector come only from the prepared runtime;
- child is spawned directly with `std::process::Command`, never through a shell;
- `RunningArti` retains the child handle and PID;
- deterministic stop/wait cleanup exists;
- Windows and Linux tests use a local network-free fixture process;
- no SOCKS request, DNS request, or external network request is made by the test path;
- no Tor bootstrap is intentionally triggered.

Verified PR head:
`ab86bbadc91c09cedc77ac09547212ddd30e073d`

Verified checks:

- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.

## Next atomic task

**T016 — Prove the real pinned Arti sidecar can start and stop with deferred bootstrap**

Use the actual pinned Arti 2.6.0 sidecar in a bounded Windows/Linux lifecycle proof.

The proof must:

- use the existing prepared-runtime path and real Arti executable;
- use the verified config with `defer_bootstrap = true`;
- start the real child directly, without a shell;
- perform no SOCKS request, DNS request, or readiness probe;
- send no user workload that can trigger bootstrap;
- stop and reap the child deterministically;
- preserve fail-closed privacy invariants.

## Do not do yet

- no Tor bootstrap trigger;
- no external request through Tor;
- no SOCKS readiness probing;
- no dynamic port discovery;
- no yt-dlp/FFmpeg;
- no AI;
- no UI.

## Success

The real pinned Arti sidecar can be started and stopped under PULQVA control on Windows and Linux
without any client request that could trigger bootstrap, and all existing checks remain green.
