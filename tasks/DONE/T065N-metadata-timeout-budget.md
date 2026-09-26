# T065N — Explicit metadata timeout budget

Status: DONE for explicit CI-fixture timeout policy and native validation.
Verified head: 623aa3c65d91cb6d930ac073a4ed09f4bec42d48, all 12 workflows green.
Metadata run 36234630845: Linux job 108384062681 passed the policy test and
live metadata request with socket=60/process=120/attempts=1, elapsed=2803 ms.
Windows job 108384062476 passed the policy test but its live result was
PULQVA_YTDLP_TOR_WINDOWS_FAIL_CLOSED_OK, not positive metadata retrieval.
This run did not exercise a read taking longer than 20 seconds; do not infer an
external outage was cured or a slow-network branch was positively reproduced.
Failure: head 07927a001182fde1bf12a8a8f51350f7e171076f, metadata run
36233341608 Linux job 108380519026: raw.githubusercontent.com read timeout=20.0.
This establishes socket read inactivity timeout, not GitHub throttling or exit cause.

Fixed public CI example now sets socket timeout 60 seconds and total process
deadline 120 seconds (formerly implicit 20 / explicit 45). One process attempt,
HTTP/extractor retries explicitly zero to avoid multiplied waiting. Typed route,
socks5h remote DNS, source URL, no-download check and Tor readiness are preserved.
Budget/elapsed markers distinguish network waiting from supervisor expiry.
Unit test verifies command prefix and unchanged typed suffix including -- boundary,
plus coherent bounded outer budget. This does not change the production launcher.

Media recovery at the parent passed native tests and Linux live retrieval on attempt
1 (run 36233341632, job 108380519966); retry branch was not exercised by live CI.
Windows media success remains fail-closed evidence unless positive marker confirmed.

Next inspect exact-head native metadata tests and live elapsed/result markers. A
successful run does not prove the upstream network cause fixed. No default Deno
activation, isolation-token injection or nonblocking release gate introduced.
