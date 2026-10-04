# T069-G2B — passive readiness diagnostics before another live generation

Date: 2026-10-04. Scope: existing Tor readiness/media acceptance path and deterministic
evidence capture. User outcome: diagnose the pre-download readiness blocker without
changing routing, retry/time budgets, destinations, success predicates, model or download policy.

## Current primary sources and decision

Released standards/platform sources reviewed on 2026-10-04:
- RFC 1928, SOCKS5 request/reply format and REP codes:
  https://www.rfc-editor.org/rfc/rfc1928.html
- Tor SOCKS extensions, including Tor-specific extended error behavior:
  https://spec.torproject.org/socks-extensions.html
- Rust `TcpStream` timeout behavior; Unix may surface `WouldBlock` while Windows may
  surface `TimedOut` for configured read/write timeouts:
  https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html
- Rust `Instant` monotonic elapsed-time semantics:
  https://doc.rust-lang.org/stable/std/time/struct.Instant.html

Decision: retain the pinned Tor/runtime/download components and all current budgets.
Add only passive bounded in-memory observations to the existing readiness verifier:
fixed loopback-family, stage, operation, I/O-kind, SOCKS REP classification and monotonic
duration. No host, port, path or raw OS error text is retained. No new probe, socket,
retry, destination, DNS route, clearnet fallback or success condition is introduced.

## Deterministic acceptance boundary

The existing pre-diagnostics dispatcher remains the behavioral oracle. Six independent
synthetic probes are predeclared and executed as fresh processes against one measured
native libtest binary:

- RD-WIRE: fixed remote-domain CONNECT request bytes.
- RD-REPLY: all 256 REP values map to a bounded classification; REP 0 alone is success.
- RD-TRACE: all 25 dispatcher outcome pairs preserve the pre-existing action/result trace.
- RD-CAUSES: timeout-like OS I/O kinds remain distinct observations, not policy changes.
- RD-CAP: trace storage is capped and omissions are counted.
- RD-PRIVATE: retained observations contain no hostname, port, path or raw error text.

The G2A matrix codec records the immutable plan, runtime/evaluator/fixture identities,
fresh-process isolation, monotonic probe durations and all outcomes. This is a native
contract gate only. It does not prove live Tor health, a root cause, model isolation,
Commons success or a downloaded file.

## Publication and post-test rule

The source generation is acceptable only after the exact-head native CI artifact is
read, its checkout/provenance is verified, and every predeclared probe is present.
No live run is authorized merely because this deterministic matrix passes. Existing
automatic repository workflows may execute because their triggers are unchanged; their
status is not relabelled as compliant G2B evidence.

Post-test review is PENDING until the native gate completes. If the gate is green, retain
the passive capture unless the observed evidence contradicts the fixed invariants. If it
fails, diagnose one concrete failure before any correction. Historical DestinationTimeout,
Commons ConnectOrTls and the Windows SDK pin mismatch remain separate open observations.
