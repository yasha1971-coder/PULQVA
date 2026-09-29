# T068-C1 — safe diagnostic recovery, 2026-09-27

Problem: at 82bb76a the only failing workflow is public Commons discovery.
Linux reaches search; Windows does not finish readiness. Generic Transport and
SearchFailed erase the distinction needed for attribution. Neither log proves 403.

Decision: retain a Copy, bounded, data-only error category in the existing search
adapter; preserve coordinator behavior and expose it to the fixed-public harness.
Readiness variants map exhaustively from the existing TorReadinessError. Reqwest
classifies timeout, connect-or-TLS, body, other using its pinned public predicates.
Non-200 becomes HttpStatus(u16); encoding/type refusal is distinct from parser
validation. Parser behavior, provider query and candidate validation are unchanged.
No raw errors, URLs, messages, headers or API payloads become diagnostic fields.
No logging in the library, new diagnostic network request or extra dependency.

Minimal evidence vocabulary, not a complete tracing platform: API errors/warnings
still use RemoteRejected and setup/cancellation/liveness may use Transport.
ConnectOrTls deliberately does not claim to know a circuit or certificate cause.
Underlying source-chain text is discarded rather than merely formatted without the
outer URL: nested errors can contain data too. Do not claim source-chain retention.

Tests added to existing deterministic native suite: error preservation over three
coordinator calls, stale-choice clearing, successful reset, validation before fetch,
readiness category/redaction and observed 302/403/429. Existing negative tests remain.
Rust tests are pending until exact-head native results are read.

The user's testing-pyramid proposal is sound as an evidence distinction: hermetic
correctness and public operational availability are separate. This diagnostic
recovery does not reclassify live failures, change branch protection, create a
schedule, or mark C1/whole E2E complete from local fixtures. Operational failures
and inconclusive attribution must remain visible, not laundered into green tests.
Chutney/Shadow and expanded telemetry are not prerequisites for this small repair.

Primary sources actually retrieved during this phase:
- https://github.com/seanmonstar/reqwest/blob/v0.13.5/src/error.rs
  Connector reads lines 20-160 and 157-206, blob
  63f66033c9728ef6836aec02b321b6fd3f988ecc: URL-bearing errors, is_timeout,
  is_connect and is_body. Exact docs.rs Error URL cache-missed, not claimed read.
- https://www.mediawiki.org/wiki/API:Errors_and_warnings
  API errors/warnings may accompany HTTP 200; keep transport status and parser
  rejection separate. The retrieved document is not evidence of current Commons
  reachability through Tor.

No privacy invariant or successful-E2E definition changes. Next: observe categories
and choose a targeted repair, not blind retries, provider replacement or new graph.
