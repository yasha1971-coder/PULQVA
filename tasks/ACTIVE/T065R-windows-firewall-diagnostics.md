# T065R — Windows firewall timeout diagnostics

Status: ACTIVE, diagnostic CI pending.
Failure: head 66ebf2348a94a9e0884e22e749e5f8d235e0ceee, run 36228028781,
Windows job 108365700285, Install exceeded 30 seconds before restricted fixtures.
Exact slow PowerShell substep and underlying cause were not observable.

Adds stderr phase/timing markers and bounded escaped partial-output reporting on
timeout; stdout evidence remains JSON. Timeout, firewall requirements and cleanup
policy are unchanged. Five mocked Python tests verify bounded logging, fail-closed
install/probe failures, cleanup attempts and no success after cleanup failure.
Local tests passed; native PowerShell diagnostics remain pending CI.

Next inspect exact-head Windows phase markers and cleanup. If red, diagnose the
specific substep; if green, report a successful diagnostic run without claiming the
historical timeout is fixed. Complete recovery before merge; no T066 implementation.
