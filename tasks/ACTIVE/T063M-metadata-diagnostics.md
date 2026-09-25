# T063M — Bounded public-fixture metadata diagnostics

Status: ACTIVE, diagnostic phase.
Main cdd2299e metadata workflow 36177458014 failed Linux job 108211426176.
yt-dlp exited 1; stderr was discarded, so historical cause is unknown.

Outcome: opt-in stderr receipt for the fixed public metadata fixture, mirroring
the media example. Retain at most 8 KiB, drain the pipe on a worker, wait at most
one second for receipt, escape controls and prefix lines. Emit on failure only.
Production launcher remains unchanged and quiet. No disk log, extra network
request, direct fallback or relaxed success criterion.

Acceptance: example unit test proves bounded retention/full drain; Windows/Linux
CI builds and runs the fixture. Inspect any nonzero exit with its new diagnostic.
A successful rerun means not reproduced, not historical root cause repaired.
T064 stays blocked while exact-head recovery CI is incomplete.
