# NEXT

## CURRENT: T065N explicit metadata timeout budget pending CI

PR70 feat/T065-deno-materialization, parent 07927a001182fde1bf12a8a8f51350f7e171076f.
Metadata run 36233341608 Linux 108380519026 failed read timeout=20.0; other 11
workflows passed. Last all-green remains 810abe3e9778105ad1952833c67ad7b9eff7339e.

Fix in public metadata fixture: socket 60s, outer process 120s; one attempt, explicit
zero HTTP/extractor retries, unchanged typed Tor route and no-download assertions.
Unit test checks exact policy, preserved route/source and coherent limits. Local
Rust unavailable; native tests/live request pending. Exact head/runs in PR70.

ONE next action: inspect metadata CI budget/elapsed/result and all exact-head checks.
Diagnose red; if green close T065M/N recovery together, no repeated docs-only cycle.
Media parent proof passed at attempt=1, so live retry behavior was not exercised.
No GitHub throttling/exit-node diagnosis established. T066 not started.
