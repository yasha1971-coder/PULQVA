# T065R — Windows firewall timeout diagnostics

Status: DONE for diagnostic instrumentation and successful native verification.
Verified head: 810abe3e9778105ad1952833c67ad7b9eff7339e, all 12 workflows green.
deno-artifact-check run 36229536187: Windows 108369938904, Linux 108369939025.

Windows phase diagnostics show Install exit 0 (last marker 2162 ms), Verify exit 0
(1063 ms), Remove exit 0 (absence verified at 1255 ms). These are elapsed times
inside each PowerShell script, not total subprocess duration or performance bounds.
Restricted fixtures denied net/read/write/env/run and receipt cleanup_verified=true.
Five mocked diagnostic/failure/cleanup tests passed locally and on Windows.

Historical failure remains unexplained: run 36228028781, Windows 108365700285,
head 66ebf2348a94a9e0884e22e749e5f8d235e0ceee timed out during Install at 30 seconds.
No failing-substep markers existed then. A successful diagnostic run is not proof
that its root cause was fixed. Timeout, policy and cleanup requirements unchanged.

Diagnostics now expose bounded escaped partial output and phase markers on failure.
No runtime activation, general sandbox or DNS-service isolation claim. T065 real
materialization evidence remains valid. Final closeout CI/merge is pending; T066
environment/cache work is READY only after merge. Do not add further docs-only
closeouts if this final head is green.
