# T068 — First live Tor-backed request -> choice -> file

Status: ACTIVE. A and B0 verified; B library implementation LAUNCH, native CI pending.
PR73 / feat/T068-live-tor-e2e. Verified parent c6ce549: 13 associated workflows green.

## Parent outcome — NOT complete
Natural-language request -> at least two real externally obtained candidates ->
explicit selection -> existing Tor-gated retrieval -> actual verified local file.
Use the same T067 coordinator, not a parallel demonstration or canned locator.

## Acceptance retained
- No account/API key or frontend external network. Tor default, remote DNS, fail closed.
- Bounded total time/output size, owned output, objectively verified file bytes/digest.
- Explicit selection belongs to the presented response; errors never yield a success receipt.
- Positive native Windows/Linux results distinguished from negative fail-closed evidence.
- Stable small public fixture first; YouTube/AI/UI/packaging remain separate gates.

## Current B implementation
Exact Cargo-generated B0 lock and manifest adopted with the library implementation.
Former lib.rs is moved byte-for-byte to commons.rs; all existing exports preserved.
CommonsHttpsTransport verifies the owned Arti child and feeds CommonsSearch via
into_search; same existing typed coordinator. Endpoint/proxy/trust are not public
caller overrides. Explicit socks5h remote DNS, validated TLS/root bundle, streaming
256-KiB cap, zero redirect/retry, no ambient proxy/cookie/referer/keylog/decompression.
Request deadline and sticky cancellation supervise liveness before/during completion.
Synchronous backend-worker API; existing bootstrap remains a separate 90s gate.

Controlled native suite covers domain-forwarding/refusal, TLS positive/untrusted/
wrong-name, redirects/status/encoding/type, chunked/oversize/truncation/stall, local
supervision and poisoned proxy/cert/keylog environment. Fixture certificates and
public test key are cfg(test) ONLY. Source generation is not test execution.
Copy-generators, duplicate candidate workflows and duplicate example clients retired.

## Evidence boundaries
No local Rust/Cargo execution. First direct locked Windows/Linux compilation/tests
PENDING. Local byte/hash checks are not TLS proof. TLS fixtures, when green, will
prove the implementation against a controlled local service, not Commons/Tor E2E.
Actual Arti loss during network I/O and C's real choices/selected file still require
recorded platform evidence. Declared API SHA-1/size are not authenticated file receipts.
T066 race/retention, descendant confinement and SDK-content limitations remain open.

## ONE next action
Observe new exact-head native discovery-contract-check and existing checks; diagnose
red. Then converge on actual same-coordinator live evidence, not more graph generation.
Current entry review and scope: decisions/ADR-0008-commons-https-executor.md.
