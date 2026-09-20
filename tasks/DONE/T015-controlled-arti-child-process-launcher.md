# T015 — Add a controlled Arti child-process launcher

Parent: PRIVACY FOUNDATION  
Status: DONE  
Date: 2026-09-21

## Result

Added the first process side effect behind `PreparedArtiRuntime`.

The launcher uses the explicit executable and deterministic argument vector directly through
`std::process::Command`, never through a shell. Successful launch returns typed `RunningArti`,
which retains the child handle/PID and provides deterministic stop/wait cleanup.

Windows/Linux tests use a controlled local fixture process and perform no SOCKS, DNS, or external
network request.

## Verification

PR #14 verified head:
`ab86bbadc91c09cedc77ac09547212ddd30e073d`

Checks:

- arti-materialization-check: success;
- rust-check: success;
- continuity-guard: success;
- arti-config-contract: success;
- arti-sidecar-check: success.
