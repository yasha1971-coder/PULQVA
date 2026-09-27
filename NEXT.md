# NEXT

## CURRENT: T068-A externally reviewed; query-envelope correction awaiting native CI

Branch: feat/T068-live-tor-e2e. PR #73 remains draft and must not be merged yet.
Verified PR head c0b68dfbec23967c0f12009716f1a51286a2762c: all 11 associated
workflows passed. Native discovery run 36299874386, Windows 108565433372 and
Linux 108565433599 passed; Linux log contains 10 core + 7 discovery tests and one
compile-fail doctest. Actual integration checkout was c6338de1dabe93897e5cf3860031953c98c8c534.

Current correction aligns server-side discovery with the existing local envelope:
quoted exact WebM MIME filter, <=8192 KiB filesize hint, one image revision.
Local MIME/size/URL checks are unchanged. Two regression tests added. This avoids
spending the first ten result slots on media the local parser cannot use; it does
not establish live availability. Current-world findings and remaining gates are in
decisions/T068-review-2026-09-27.md. No dependency, runtime, kernel or Tor change.

ONE next action: observe the new exact PR head's native discovery CI. Diagnose red;
if green, proceed directly to bounded T068-B implementation in the same branch:
a production endpoint-specific HTTPS executor with verified Tor SOCKS routing,
remote DNS, TLS validation, streaming byte limits, one total deadline, no redirects,
no ambient proxy/config and no implicit retries. Use a maintained HTTP/TLS library,
not hand-written TLS or an assumed host curl installation. Record exact compatible
versions and Cargo-generated lock before accepting the dependency change.

Then C binds real discovery results to the SAME coordinator/selection/retrieval
path and actual-file verification. No more optional discovery features before B.
A static token, a request plan, synthetic JSON or Windows timeout is not live E2E.
No next-task launch or partial merge during this review phase. Rust tests are not
available locally (no Cargo; container DNS unavailable); new checks are pending CI.
