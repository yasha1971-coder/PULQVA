# T063M — Bounded public-fixture metadata diagnostics

DONE within diagnostic scope at 5dccf46a61e54c3e98bd4b3c74d7a5ce7e93d69b.
All 10 triggered workflows passed. Metadata run 36191270234:
- Linux job 108256683386: bounded capture/full drain unit test passed;
  PULQVA_YTDLP_TOR_METADATA_OK observed.
- Windows job 108256683616: same unit test passed;
  PULQVA_YTDLP_TOR_WINDOWS_FAIL_CLOSED_OK observed (no positive metadata proof).

Diagnostics are opt-in, limited to fixed public fixture, retain 8KiB in memory,
drain pipe continuously, escape controls and prefix lines. Receipt wait <=1s;
output on child failure/timeout only. Production stderr remains suppressed.

Historical main run 36177458014 Linux job 108211426176 exited 1 with no stderr.
Failure did not reproduce; underlying cause is NOT claimed fixed. Actual live
error receipt was not exercised by this successful Linux run. No privacy gate
or positive Linux acceptance weakened. Closeout requires exact-head CI/merge.
