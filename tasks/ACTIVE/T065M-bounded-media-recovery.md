# T065M — Bounded Tor media fixture recovery

Status: ACTIVE, implementation pending native CI.
Observed failure: d5ade9b91d7dbeb6559a8556b7ba3a53186a5f33,
run 36232028938, Linux job 108376857791: yt-dlp generic pre-download request
returned ProxyError/Socks5Error(1, general SOCKS server failure). Windows media
passed; both Deno/firewall jobs passed. Do not conflate these separate failures.

The fixture previously aborted on the first exit-1 despite remaining time in its
180-second download budget. It now permits at most three total process attempts
only for the observed pre-download SOCKS REP 1 signature, only when no regular
output exists. Backoff 2/4 seconds, one unchanged 180-second deadline, same typed
argv, same ready Tor socks5h endpoint. Tor process must still be alive before each
attempt. No direct fallback, relay forcing, URL switching or weakened success gate.
Unknown/permanent/partial-output failures terminate immediately. Exhaustion fails CI.
Logs expose attempt counts even after eventual success; no green-by-skipping.

Tests cover count/budget bounds and nonretryable/partial-output errors. Captured
stderr stays bounded at 8KiB and public-fixture diagnostics remain opt-in. This is
CI-fixture recovery, not a claim that production already retries downloads.

## Primary-source research, 2026-09-26
- https://www.rfc-editor.org/rfc/rfc1928.html section 6: REP 1 is general SOCKS
  server failure; it does not identify root cause or guarantee transience.
- https://github.com/yt-dlp/yt-dlp#extractor-options documents extractor retries for
  known extractor errors; blanket increasing that option is not evidence this
  observed generic proxy failure will be retried.
- https://spec.torproject.org/path-spec/attaching-streams-to-circuits.html explains
  stream attachment and its timeout independently from application readiness.

Engineering inference: bounded application-level retry can recover a temporary
connection failure while preserving the privacy route, but cannot repair an
external Tor relay/destination outage. The underlying network cause remains unknown.
Acceptance: native tests and actual same-fixture Tor retrieval; preserve failures
and attempt counts. Do not claim a first-attempt success proves the retry branch.

Parent verification: 07927a001182fde1bf12a8a8f51350f7e171076f passed media unit tests (3) and Linux live retrieval at attempt=1, run 36233341632 job 108380519966. Live retry not exercised; closeout deferred during T065N metadata recovery.
